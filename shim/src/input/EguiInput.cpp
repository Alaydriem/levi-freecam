#include "input/EguiInput.h"

#include "bridge/RustBridge.h"
#include "frame/FrameDriver.h"
#include "mod/FreecamMod.h"
#include "render/InFlightCall.h"
#include "render/OverlayHook.h"

#include <windowsx.h>

#include "ll/api/io/Logger.h"

#include <atomic>
#include <chrono>
#include <mutex>
#include <thread>

namespace levi_freecam {

namespace {

constexpr uint32_t                  EVENT_MOVE        = 0;
constexpr uint32_t                  EVENT_BUTTON      = 1;
constexpr uint32_t                  EVENT_WHEEL       = 2;
constexpr uint32_t                  EVENT_KEY         = 3;
constexpr uint32_t                  EVENT_GONE        = 4;
constexpr uint32_t                  EVENT_FOCUS_LOST  = 5;
constexpr uint32_t                  BUTTON_LEFT       = 0;
constexpr uint32_t                  BUTTON_RIGHT      = 1;
constexpr uint32_t                  BUTTON_MIDDLE     = 2;
constexpr uint8_t                   CURSOR_HAND       = 1;
constexpr float                     NOTCH             = static_cast<float>(WHEEL_DELTA);
constexpr std::chrono::milliseconds DRAIN_TIMEOUT{2000};
constexpr std::chrono::milliseconds DRAIN_POLL{1};

// guards install and uninstall; the WndProc itself reads only the atomics
std::mutex           gMutex;
HWND                 gWindow     = nullptr;
bool                 gSubclassed = false;
std::atomic<WNDPROC> gOriginal{nullptr};
std::atomic<bool>    gForwarding{false};
std::atomic<uint8_t> gCursor{0};
// threads inside forward(), the only part that calls Rust. The drain waits on this one alone, so a modal loop in
// the game's own WndProc (a window drag) does not hold up shutdown
std::atomic<int>     gRustCalls{0};
std::atomic<bool>    gThreadLogged{false};
HCURSOR              gArrow = nullptr;
HCURSOR              gHand  = nullptr;
// this thread's own calls inside forward(), so an uninstall run from inside one does not wait for itself
thread_local int tDepth = 0;
// window thread only: TrackMouseEvent was asked for WM_MOUSELEAVE and that message has not come yet
bool gTrackingLeave = false;

ll::io::Logger& logger() { return FreecamMod::getInstance().getSelf().getLogger(); }

void send(FcUiEvent const& event) {
    const FcHandle* handle = RustBridge::handle();
    if (handle) fc_ui_event(handle, &event);
}

// window messages give client-area pixels and the UI lays out in back-buffer pixels; they differ when the game
// renders at another size than its client area (another fullscreen mode, or the frames before ResizeBuffers)
void toBackBuffer(HWND hwnd, float& x, float& y) {
    float width  = 0.0f;
    float height = 0.0f;
    OverlayHook::backBufferSize(width, height);
    RECT client{};
    if (width <= 0.0f || height <= 0.0f || !GetClientRect(hwnd, &client)) return;
    if (client.right <= 0 || client.bottom <= 0) return;
    x *= width / static_cast<float>(client.right);
    y *= height / static_cast<float>(client.bottom);
}

void sendPointer(HWND hwnd, uint32_t kind, LPARAM lParam, uint32_t button, bool pressed) {
    FcUiEvent event{};
    event.kind    = kind;
    event.x       = static_cast<float>(GET_X_LPARAM(lParam));
    event.y       = static_cast<float>(GET_Y_LPARAM(lParam));
    toBackBuffer(hwnd, event.x, event.y);
    event.button  = button;
    event.pressed = pressed ? 1 : 0;
    send(event);
}

void sendWheel(float dx, float dy) {
    FcUiEvent event{};
    event.kind = EVENT_WHEEL;
    event.x    = dx;
    event.y    = dy;
    send(event);
}

void sendKey(WPARAM wParam, bool pressed) {
    FcUiEvent event{};
    event.kind    = EVENT_KEY;
    event.vk      = static_cast<uint32_t>(wParam);
    event.pressed = pressed ? 1 : 0;
    send(event);
}

void sendKind(uint32_t kind) {
    FcUiEvent event{};
    event.kind = kind;
    send(event);
}

// WM_MOUSELEAVE comes only after TrackMouseEvent, and once per request
void trackLeave(HWND hwnd) {
    if (gTrackingLeave) return;
    TRACKMOUSEEVENT track{};
    track.cbSize    = sizeof(track);
    track.dwFlags   = TME_LEAVE;
    track.hwndTrack = hwnd;
    gTrackingLeave  = TrackMouseEvent(&track) != FALSE;
}

bool forwardMouse(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    switch (msg) {
    case WM_MOUSEMOVE:
        trackLeave(hwnd);
        sendPointer(hwnd, EVENT_MOVE, lParam, 0, false);
        return true;
    case WM_LBUTTONDOWN:
    case WM_LBUTTONDBLCLK: sendPointer(hwnd, EVENT_BUTTON, lParam, BUTTON_LEFT, true); return true;
    case WM_LBUTTONUP: sendPointer(hwnd, EVENT_BUTTON, lParam, BUTTON_LEFT, false); return true;
    case WM_RBUTTONDOWN:
    case WM_RBUTTONDBLCLK: sendPointer(hwnd, EVENT_BUTTON, lParam, BUTTON_RIGHT, true); return true;
    case WM_RBUTTONUP: sendPointer(hwnd, EVENT_BUTTON, lParam, BUTTON_RIGHT, false); return true;
    case WM_MBUTTONDOWN:
    case WM_MBUTTONDBLCLK: sendPointer(hwnd, EVENT_BUTTON, lParam, BUTTON_MIDDLE, true); return true;
    case WM_MBUTTONUP: sendPointer(hwnd, EVENT_BUTTON, lParam, BUTTON_MIDDLE, false); return true;
    case WM_MOUSEWHEEL: sendWheel(0.0f, static_cast<float>(GET_WHEEL_DELTA_WPARAM(wParam)) / NOTCH); return true;
    case WM_MOUSEHWHEEL: sendWheel(-static_cast<float>(GET_WHEEL_DELTA_WPARAM(wParam)) / NOTCH, 0.0f); return true;
    default: return false;
    }
}

void forwardMessage(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    if (forwardMouse(hwnd, msg, wParam, lParam)) return;
    switch (msg) {
    case WM_MOUSELEAVE: sendKind(EVENT_GONE); break;
    case WM_KILLFOCUS: sendKind(EVENT_FOCUS_LOST); break;
    case WM_KEYDOWN:
    case WM_SYSKEYDOWN: sendKey(wParam, true); break;
    case WM_KEYUP:
    case WM_SYSKEYUP: sendKey(wParam, false); break;
    default: break;
    }
}

// counted for the drain; forwarding is checked again inside the count, so no call starts after uninstall saw 0
void forward(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    InFlightCall guard(gRustCalls);
    ++tDepth;
    if (gForwarding.load()) forwardMessage(hwnd, msg, wParam, lParam);
    --tDepth;
}

void noteWindowThread() {
    if (gThreadLogged.load(std::memory_order_relaxed) || gThreadLogged.exchange(true)) return;
    logger().info("ui input: the game window's WndProc runs on thread {}", GetCurrentThreadId());
}

LRESULT passOn(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    WNDPROC original = gOriginal.load();
    if (!original) return DefWindowProcW(hwnd, msg, wParam, lParam);
    return CallWindowProcW(original, hwnd, msg, wParam, lParam);
}

// during a look drag the cursor is hidden and pointer moves stay away from the UI; buttons, keys and focus still go to it
LRESULT routeDrag(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    if (msg == WM_SETCURSOR && LOWORD(lParam) == HTCLIENT) {
        SetCursor(nullptr);
        return TRUE;
    }
    if (msg != WM_MOUSEMOVE && msg != WM_MOUSELEAVE) forward(hwnd, msg, wParam, lParam);
    return passOn(hwnd, msg, wParam, lParam);
}

LRESULT route(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    noteWindowThread();
    if (msg == WM_MOUSELEAVE) gTrackingLeave = false;
    if (!gForwarding.load() || !FrameDriver::cursorMode()) return passOn(hwnd, msg, wParam, lParam);
    if (FrameDriver::lookDrag()) return routeDrag(hwnd, msg, wParam, lParam);
    if (msg == WM_SETCURSOR && LOWORD(lParam) == HTCLIENT) {
        SetCursor(gCursor.load(std::memory_order_relaxed) == CURSOR_HAND ? gHand : gArrow);
        return TRUE;
    }
    forward(hwnd, msg, wParam, lParam);
    return passOn(hwnd, msg, wParam, lParam);
}

LRESULT CALLBACK onWndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) { return route(hwnd, msg, wParam, lParam); }

bool waitForDrain() {
    auto deadline = std::chrono::steady_clock::now() + DRAIN_TIMEOUT;
    while (gRustCalls.load() > tDepth) {
        if (std::chrono::steady_clock::now() > deadline) return false;
        std::this_thread::sleep_for(DRAIN_POLL);
    }
    return true;
}

// another subclass installed after ours would lose its place if ours were restored under it
bool restoreOriginal() {
    if (!IsWindow(gWindow)) return true;
    auto current = reinterpret_cast<WNDPROC>(GetWindowLongPtrW(gWindow, GWLP_WNDPROC));
    if (current != &onWndProc) return false;
    SetWindowLongPtrW(gWindow, GWLP_WNDPROC, reinterpret_cast<LONG_PTR>(gOriginal.load()));
    return true;
}

} // namespace

void EguiInput::install(HWND window) {
    std::scoped_lock lock(gMutex);
    if (!window) return;
    if (gSubclassed) {
        // the WndProc is still in place from before (see uninstall); only forwarding stops and starts
        if (window == gWindow) gForwarding.store(true);
        return;
    }
    gArrow = LoadCursorW(nullptr, IDC_ARROW);
    gHand  = LoadCursorW(nullptr, IDC_HAND);
    // a message that arrives between the swap and the store below still finds the original
    gOriginal.store(reinterpret_cast<WNDPROC>(GetWindowLongPtrW(window, GWLP_WNDPROC)));
    LONG_PTR previous = SetWindowLongPtrW(window, GWLP_WNDPROC, reinterpret_cast<LONG_PTR>(&onWndProc));
    if (previous == 0) {
        logger().warn("ui input: could not subclass the game window ({}); the UI gets no mouse input", GetLastError());
        gOriginal.store(nullptr);
        return;
    }
    gOriginal.store(reinterpret_cast<WNDPROC>(previous));
    gWindow     = window;
    gSubclassed = true;
    gForwarding.store(true);
    logger().info("ui input: game window subclassed");
}

bool EguiInput::uninstall() {
    std::scoped_lock lock(gMutex);
    gForwarding.store(false);
    if (gSubclassed) {
        if (restoreOriginal()) {
            gSubclassed = false;
            gWindow     = nullptr;
        } else {
            logger().warn("ui input: another WndProc sits above freecam's; freecam's stays in place and only passes messages on");
        }
    }
    if (waitForDrain()) return true;
    logger().warn("ui input: a thread is still inside a call into Rust after the wait");
    return false;
}

void EguiInput::setCursor(uint8_t shape) { gCursor.store(shape, std::memory_order_relaxed); }

} // namespace levi_freecam
