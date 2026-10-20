#include "render/OverlayHook.h"

#include <windows.h>

// these names clash with LeviLamina's own definitions
#define D3D12_FEATURE_DATA_D3D12_OPTIONS D3D12_FEATURE_DATA_D3D12_OPTIONS_LEGACY
#define D3D12_FEATURE_DATA_ARCHITECTURE  D3D12_FEATURE_DATA_ARCHITECTURE_LEGACY
#define D3D12_RAYTRACING_GEOMETRY_DESC   D3D12_RAYTRACING_GEOMETRY_DESC_LEGACY

#include <d3d11.h>
#include <d3d11on12.h>
#include <d3d12.h>
#include <dxgi1_4.h>
#include <wrl/client.h>

#undef D3D12_FEATURE_DATA_D3D12_OPTIONS
#undef D3D12_FEATURE_DATA_ARCHITECTURE
#undef D3D12_RAYTRACING_GEOMETRY_DESC

#include "bridge/RustBridge.h"
#include "mod/FreecamMod.h"
#include "render/InFlightCall.h"
#include "render/OverlayDraw.h"

#include <backends/imgui_impl_dx11.h>
#include <backends/imgui_impl_win32.h>
#include <imgui.h>

#include "ll/api/io/Logger.h"
#include "ll/api/memory/Hook.h"

#include <array>
#include <atomic>
#include <chrono>
#include <cstdint>
#include <mutex>
#include <thread>

using Microsoft::WRL::ComPtr;

namespace levi_freecam {

namespace {

using PresentFn       = HRESULT(STDMETHODCALLTYPE*)(IDXGISwapChain*, UINT, UINT);
using Present1Fn      = HRESULT(STDMETHODCALLTYPE*)(IDXGISwapChain1*, UINT, UINT, DXGI_PRESENT_PARAMETERS const*);
using ResizeBuffersFn = HRESULT(STDMETHODCALLTYPE*)(IDXGISwapChain*, UINT, UINT, UINT, DXGI_FORMAT, UINT);
using ResizeBuffers1Fn =
    HRESULT(STDMETHODCALLTYPE*)(IDXGISwapChain3*, UINT, UINT, UINT, DXGI_FORMAT, UINT, UINT const*, IUnknown* const*);
using ExecuteCommandListsFn = void(STDMETHODCALLTYPE*)(ID3D12CommandQueue*, UINT, ID3D12CommandList* const*);

constexpr int                       PRESENT_INDEX               = 8;
constexpr int                       RESIZE_BUFFERS_INDEX        = 13;
constexpr int                       PRESENT1_INDEX              = 22;
constexpr int                       RESIZE_BUFFERS1_INDEX       = 39;
constexpr int                       EXECUTE_COMMAND_LISTS_INDEX = 10;
constexpr UINT                      MAX_BACK_BUFFERS            = DXGI_MAX_SWAP_CHAIN_BUFFERS;
constexpr int                       PROBE_SIZE                  = 8;
constexpr wchar_t                   PROBE_CLASS[]               = L"LeviFreecamOverlayProbe";
constexpr std::chrono::milliseconds DRAIN_TIMEOUT{2000};
constexpr std::chrono::milliseconds DRAIN_POLL{1};
constexpr int                       QUEUE_WAIT_LOG_FRAMES = 600;

PresentFn             gOriginalPresent        = nullptr;
Present1Fn            gOriginalPresent1       = nullptr;
ResizeBuffersFn       gOriginalResizeBuffers  = nullptr;
ResizeBuffers1Fn      gOriginalResizeBuffers1 = nullptr;
ExecuteCommandListsFn gOriginalExecute        = nullptr;

ll::memory::FuncPtr gPresentTarget        = nullptr;
ll::memory::FuncPtr gPresentDetour        = nullptr;
ll::memory::FuncPtr gPresent1Target       = nullptr;
ll::memory::FuncPtr gPresent1Detour       = nullptr;
ll::memory::FuncPtr gResizeBuffersTarget  = nullptr;
ll::memory::FuncPtr gResizeBuffersDetour  = nullptr;
ll::memory::FuncPtr gResizeBuffers1Target = nullptr;
ll::memory::FuncPtr gResizeBuffers1Detour = nullptr;
ll::memory::FuncPtr gExecuteTarget        = nullptr;
ll::memory::FuncPtr gExecuteDetour        = nullptr;

std::mutex        gInstallMutex;
std::atomic<bool> gAttempted{false};
std::atomic<bool> gInstalled{false};
std::atomic<bool> gShuttingDown{false};
std::atomic<int>  gInFlight{0};
// set when shutdown timed out and left the device alive; guarded by gInstallMutex
bool gLeaked = false;

// the first direct queue the game submits to; D3D11On12 draws through it. Holds one reference
std::atomic<ID3D12CommandQueue*> gQueue{nullptr};

// Present1 may reach Present inside DXGI; only the outermost call on a thread draws
thread_local int tPresentDepth = 0;

// render state below is touched only inside Present and ResizeBuffers of the bound window's chain,
// which the game never runs at the same time
ImGuiContext*                                                gImGui        = nullptr;
bool                                                         gDeviceFailed = false;
ComPtr<ID3D11Device>                                         gDevice;
ComPtr<ID3D11DeviceContext>                                  gContext;
ComPtr<ID3D11On12Device>                                     gDevice11On12;
IDXGISwapChain*                                              gTargetChain = nullptr;
UINT                                                         gTargetCount = 0;
std::array<ComPtr<ID3D11Resource>, MAX_BACK_BUFFERS>         gWrapped;
std::array<ComPtr<ID3D11RenderTargetView>, MAX_BACK_BUFFERS> gTargets;

// the window the overlay draws for, kept across a device rebuild; the device pointer is identity only,
// kept alive by gDevice or gDevice11On12, and set only while the ImGui context exists
bool  gBound       = false;
HWND  gBoundWindow = nullptr;
void* gBoundDevice = nullptr;

int  gQueueWaitFrames = 0;
bool gQueueWaitLogged = false;

// the bound chain's back-buffer width in the high 32 bits and height in the low 32 bits, so a reader on
// another thread never sees half of a resize; 0 until a chain is bound
std::atomic<uint64_t> gBackBufferSize{0};

ll::io::Logger& logger() { return FreecamMod::getInstance().getSelf().getLogger(); }

// info lines are buffered; a hang inside Present would lose them, so the device lines are written out at once
void logDeviceStep(char const* step) {
    logger().info("overlay: {}", step);
    logger().flush();
}

constexpr int BACK_BUFFER_WIDTH_SHIFT = 32;

void recordBackBufferSize(DXGI_SWAP_CHAIN_DESC const& desc) {
    uint64_t packed = (static_cast<uint64_t>(desc.BufferDesc.Width) << BACK_BUFFER_WIDTH_SHIFT) | desc.BufferDesc.Height;
    gBackBufferSize.store(packed, std::memory_order_relaxed);
}

// the arguments to ResizeBuffers can be 0, meaning the window size; only the chain knows the size it chose
void recordBackBufferSize(IDXGISwapChain* chain) {
    DXGI_SWAP_CHAIN_DESC desc{};
    if (SUCCEEDED(chain->GetDesc(&desc))) recordBackBufferSize(desc);
}

void releaseQueue() {
    if (ID3D12CommandQueue* queue = gQueue.exchange(nullptr)) queue->Release();
}

// every reference to a back buffer must be gone before ResizeBuffers, or it fails
void releaseTargets() {
    for (UINT i = 0; i < MAX_BACK_BUFFERS; ++i) {
        gTargets[i].Reset();
        gWrapped[i].Reset();
    }
    gTargetCount = 0;
    gTargetChain = nullptr;
    if (gDevice11On12 && gContext) {
        gContext->ClearState();
        gContext->Flush();
    }
}

bool createWrappedTargets(IDXGISwapChain* chain, UINT bufferCount) {
    UINT count = bufferCount < MAX_BACK_BUFFERS ? bufferCount : MAX_BACK_BUFFERS;
    for (UINT i = 0; i < count; ++i) {
        ComPtr<ID3D12Resource> buffer;
        if (FAILED(chain->GetBuffer(i, IID_PPV_ARGS(&buffer)))) return false;
        D3D11_RESOURCE_FLAGS flags{};
        flags.BindFlags = D3D11_BIND_RENDER_TARGET;
        if (FAILED(gDevice11On12->CreateWrappedResource(
                buffer.Get(),
                &flags,
                D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_PRESENT,
                IID_PPV_ARGS(&gWrapped[i])
            ))) {
            return false;
        }
        if (FAILED(gDevice->CreateRenderTargetView(gWrapped[i].Get(), nullptr, &gTargets[i]))) return false;
    }
    gTargetCount = count;
    return true;
}

// a D3D11 flip chain always shows its current back buffer as buffer 0
bool createNativeTarget(IDXGISwapChain* chain) {
    ComPtr<ID3D11Texture2D> buffer;
    if (FAILED(chain->GetBuffer(0, IID_PPV_ARGS(&buffer)))) return false;
    if (FAILED(gDevice->CreateRenderTargetView(buffer.Get(), nullptr, &gTargets[0]))) return false;
    gTargetCount = 1;
    return true;
}

bool createTargets(IDXGISwapChain* chain) {
    DXGI_SWAP_CHAIN_DESC desc{};
    if (FAILED(chain->GetDesc(&desc))) return false;
    bool created = gDevice11On12 ? createWrappedTargets(chain, desc.BufferCount) : createNativeTarget(chain);
    if (!created) {
        releaseTargets();
        return false;
    }
    gTargetChain = chain;
    recordBackBufferSize(desc);
    return true;
}

void noteQueueWait() {
    if (gQueueWaitLogged || ++gQueueWaitFrames < QUEUE_WAIT_LOG_FRAMES) return;
    gQueueWaitLogged = true;
    logger().warn("overlay: no D3D12 command queue captured for the game's device yet; the overlay waits for one");
}

// the queue must belong to the swap chain's device; a foreign one is dropped so the next submit can be captured
bool createDevice11On12(IDXGISwapChain* chain) {
    ID3D12CommandQueue* queue = gQueue.load();
    if (!queue) {
        noteQueueWait();
        return false;
    }
    ComPtr<ID3D12Device> chainDevice;
    ComPtr<ID3D12Device> queueDevice;
    if (FAILED(chain->GetDevice(IID_PPV_ARGS(&chainDevice)))) {
        gDeviceFailed = true;
        logger().error("overlay: the swap chain has neither a D3D11 nor a D3D12 device");
        return false;
    }
    if (FAILED(queue->GetDevice(IID_PPV_ARGS(&queueDevice))) || queueDevice != chainDevice) {
        if (gQueue.compare_exchange_strong(queue, nullptr)) queue->Release();
        noteQueueWait();
        return false;
    }
    IUnknown* queues[] = {queue};
    logDeviceStep("creating device (D3D12 through D3D11On12)");
    if (FAILED(D3D11On12CreateDevice(
            chainDevice.Get(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            nullptr,
            0,
            queues,
            1,
            0,
            &gDevice,
            &gContext,
            nullptr
        ))
        || FAILED(gDevice.As(&gDevice11On12))) {
        gDeviceFailed = true;
        logger().error("overlay: could not create a D3D11On12 device");
        gContext.Reset();
        gDevice.Reset();
        return false;
    }
    return true;
}

// freecam's own context: whatever context was current before is current again on return
bool createImGui(HWND window) {
    ImGuiContext* previous = ImGui::GetCurrentContext();
    gImGui                 = ImGui::CreateContext();
    ImGui::SetCurrentContext(gImGui);
    ImGuiIO& io     = ImGui::GetIO();
    io.IniFilename  = nullptr;
    io.LogFilename  = nullptr;
    io.ConfigFlags |= ImGuiConfigFlags_NoMouse | ImGuiConfigFlags_NoMouseCursorChange | ImGuiConfigFlags_NoKeyboard;
    io.Fonts->AddFontDefault();
    bool platform = ImGui_ImplWin32_Init(window);
    bool renderer = platform && ImGui_ImplDX11_Init(gDevice.Get(), gContext.Get());
    if (!renderer) {
        if (platform) ImGui_ImplWin32_Shutdown();
        ImGui::DestroyContext(gImGui);
        gImGui = nullptr;
    }
    ImGui::SetCurrentContext(previous);
    return renderer;
}

void destroyImGui() {
    if (!gImGui) return;
    ImGuiContext* previous = ImGui::GetCurrentContext();
    ImGui::SetCurrentContext(gImGui);
    ImGui_ImplDX11_Shutdown();
    ImGui_ImplWin32_Shutdown();
    ImGui::DestroyContext(gImGui);
    ImGui::SetCurrentContext(previous == gImGui ? nullptr : previous);
    gImGui = nullptr;
}

// drops the ImGui device objects, the targets, the D3D11On12 wrappers, the device and the queue
void releaseDevice() {
    destroyImGui();
    releaseTargets();
    gDevice11On12.Reset();
    gContext.Reset();
    gDevice.Reset();
    releaseQueue();
    gDeviceFailed    = false;
    gBoundDevice     = nullptr;
    gQueueWaitFrames = 0;
    gQueueWaitLogged = false;
}

void releaseBinding() {
    releaseDevice();
    gBound       = false;
    gBoundWindow = nullptr;
    gBackBufferSize.store(0, std::memory_order_relaxed);
}

bool ensureDevice(IDXGISwapChain* chain, HWND window, void* device) {
    if (gImGui) return true;
    if (gDeviceFailed) return false;
    if (!gDevice) {
        if (SUCCEEDED(chain->GetDevice(IID_PPV_ARGS(&gDevice)))) {
            logDeviceStep("creating device (D3D11)");
            gDevice->GetImmediateContext(&gContext);
        } else if (!createDevice11On12(chain)) {
            return false;
        }
    }
    if (!createImGui(window)) {
        gDeviceFailed = true;
        logger().error("overlay: could not start ImGui on the game window");
        return false;
    }
    gBound       = true;
    gBoundWindow = window;
    gBoundDevice = device;
    logDeviceStep(gDevice11On12 ? "device ready (D3D12 through D3D11On12)" : "device ready (D3D11)");
    return true;
}

// the device pointer is returned for identity only; the reference taken here is released on return
void* chainDevice(IDXGISwapChain* chain) {
    ComPtr<ID3D11Device> device11;
    if (SUCCEEDED(chain->GetDevice(IID_PPV_ARGS(&device11)))) return device11.Get();
    ComPtr<ID3D12Device> device12;
    if (SUCCEEDED(chain->GetDevice(IID_PPV_ARGS(&device12)))) return device12.Get();
    return nullptr;
}

// the bound chain passes at once; another chain on another window is skipped; a new device on the bound window
// tears everything down and builds it again
bool acceptChain(IDXGISwapChain* chain) {
    if (chain == gTargetChain) return true;
    DXGI_SWAP_CHAIN_DESC desc{};
    if (FAILED(chain->GetDesc(&desc))) return false;
    if (gBound && desc.OutputWindow != gBoundWindow) return false;
    void* device = chainDevice(chain);
    if (gImGui && device != gBoundDevice) {
        logger().info("overlay: the game window has a new device; the overlay device is built again");
        releaseDevice();
    }
    if (!ensureDevice(chain, desc.OutputWindow, device)) return false;
    releaseTargets();
    return createTargets(chain);
}

void drawWrapped(UINT index) {
    ID3D11Resource* wrapped[] = {gWrapped[index].Get()};
    gDevice11On12->AcquireWrappedResources(wrapped, 1);
    ID3D11RenderTargetView* target = gTargets[index].Get();
    gContext->OMSetRenderTargets(1, &target, nullptr);
    ImGui_ImplDX11_RenderDrawData(ImGui::GetDrawData());
    gContext->OMSetRenderTargets(0, nullptr, nullptr);
    gDevice11On12->ReleaseWrappedResources(wrapped, 1);
    gContext->Flush();
}

// the D3D11 context is the game's own: put its render targets back after drawing
void drawNative() {
    std::array<ID3D11RenderTargetView*, D3D11_SIMULTANEOUS_RENDER_TARGET_COUNT> saved{};
    ID3D11DepthStencilView*                                                     savedDepth = nullptr;
    gContext->OMGetRenderTargets(static_cast<UINT>(saved.size()), saved.data(), &savedDepth);
    ID3D11RenderTargetView* target = gTargets[0].Get();
    gContext->OMSetRenderTargets(1, &target, nullptr);
    ImGui_ImplDX11_RenderDrawData(ImGui::GetDrawData());
    gContext->OMSetRenderTargets(static_cast<UINT>(saved.size()), saved.data(), savedDepth);
    for (ID3D11RenderTargetView* view : saved) {
        if (view) view->Release();
    }
    if (savedDepth) savedDepth->Release();
}

bool currentBackBuffer(IDXGISwapChain* chain, UINT& index) {
    index = 0;
    if (!gDevice11On12) return true;
    ComPtr<IDXGISwapChain3> chain3;
    if (FAILED(chain->QueryInterface(IID_PPV_ARGS(&chain3)))) return false;
    index = chain3->GetCurrentBackBufferIndex();
    return true;
}

// ImGui coordinates are back-buffer pixels, the same pixels Rust projects the overlay into
void useBackBufferSize() {
    float width  = 0.0f;
    float height = 0.0f;
    OverlayHook::backBufferSize(width, height);
    if (width > 0.0f && height > 0.0f) ImGui::GetIO().DisplaySize = ImVec2{width, height};
}

// a hidden overlay costs no ImGui frame and no D3D draw; the view borrows Rust's buffers until the next fc_overlay or
// fc_destroy, so it is used only inside this Present
bool fetchVisibleOverlay(FcOverlayView& view) {
    if (OverlayHook::shuttingDown()) return false;
    const FcHandle* handle = RustBridge::handle();
    if (!handle) return false;
    return fc_overlay(handle, &view) == FC_STATUS_OK && view.visible != 0;
}

// the device and targets are built at the first Present, before the overlay is ever shown, so no D3D device is
// created mid-game; acceptChain is one pointer compare for the bound chain
void renderFrame(IDXGISwapChain* chain) {
    if (!acceptChain(chain)) return;
    FcOverlayView view{};
    if (!fetchVisibleOverlay(view)) return;
    UINT index = 0;
    if (!currentBackBuffer(chain, index) || index >= gTargetCount || !gTargets[index]) return;
    ImGuiContext* previous = ImGui::GetCurrentContext();
    ImGui::SetCurrentContext(gImGui);
    ImGui_ImplDX11_NewFrame();
    ImGui_ImplWin32_NewFrame();
    useBackBufferSize();
    ImGui::NewFrame();
    OverlayDraw::draw(*ImGui::GetForegroundDrawList(), view);
    ImGui::Render();
    if (gDevice11On12) {
        drawWrapped(index);
    } else {
        drawNative();
    }
    ImGui::SetCurrentContext(previous);
}

void drawOverlay(IDXGISwapChain* chain, UINT flags) {
    if (tPresentDepth != 1 || (flags & DXGI_PRESENT_TEST) != 0 || gShuttingDown.load()) return;
    renderFrame(chain);
}

bool isBoundChain(IDXGISwapChain* chain) { return chain != nullptr && chain == gTargetChain; }

// only the bound chain's targets are ours to drop; other chains resize untouched
void releaseTargetsFor(IDXGISwapChain* chain) {
    if (isBoundChain(chain)) releaseTargets();
}

HRESULT STDMETHODCALLTYPE onPresent(IDXGISwapChain* chain, UINT sync, UINT flags) {
    InFlightCall guard(gInFlight);
    ++tPresentDepth;
    drawOverlay(chain, flags);
    HRESULT result = gOriginalPresent(chain, sync, flags);
    --tPresentDepth;
    return result;
}

HRESULT STDMETHODCALLTYPE onPresent1(IDXGISwapChain1* chain, UINT sync, UINT flags, DXGI_PRESENT_PARAMETERS const* params) {
    InFlightCall guard(gInFlight);
    ++tPresentDepth;
    drawOverlay(chain, flags);
    HRESULT result = gOriginalPresent1(chain, sync, flags, params);
    --tPresentDepth;
    return result;
}

HRESULT STDMETHODCALLTYPE
onResizeBuffers(IDXGISwapChain* chain, UINT count, UINT width, UINT height, DXGI_FORMAT format, UINT flags) {
    InFlightCall guard(gInFlight);
    bool bound = isBoundChain(chain);
    releaseTargetsFor(chain);
    HRESULT result = gOriginalResizeBuffers(chain, count, width, height, format, flags);
    if (bound && SUCCEEDED(result)) recordBackBufferSize(chain);
    return result;
}

HRESULT STDMETHODCALLTYPE onResizeBuffers1(
    IDXGISwapChain3* chain,
    UINT             count,
    UINT             width,
    UINT             height,
    DXGI_FORMAT      format,
    UINT             flags,
    UINT const*      nodeMasks,
    IUnknown* const* presentQueues
) {
    InFlightCall guard(gInFlight);
    bool bound = isBoundChain(chain);
    releaseTargetsFor(chain);
    HRESULT result = gOriginalResizeBuffers1(chain, count, width, height, format, flags, nodeMasks, presentQueues);
    if (bound && SUCCEEDED(result)) recordBackBufferSize(chain);
    return result;
}

void STDMETHODCALLTYPE onExecuteCommandLists(ID3D12CommandQueue* queue, UINT count, ID3D12CommandList* const* lists) {
    InFlightCall guard(gInFlight);
    if (!gShuttingDown.load() && !gQueue.load() && queue->GetDesc().Type == D3D12_COMMAND_LIST_TYPE_DIRECT) {
        ID3D12CommandQueue* expected = nullptr;
        queue->AddRef();
        if (!gQueue.compare_exchange_strong(expected, queue)) queue->Release();
    }
    gOriginalExecute(queue, count, lists);
}

template <class Fn>
bool hookEntry(
    char const*          name,
    void*                entry,
    Fn                   detour,
    ll::memory::FuncPtr& target,
    ll::memory::FuncPtr& detourOut,
    Fn&                  original
) {
    target     = ll::memory::toFuncPtr(reinterpret_cast<Fn>(entry));
    detourOut  = ll::memory::toFuncPtr(detour);
    int status = ll::memory::hook(target, detourOut, reinterpret_cast<ll::memory::FuncPtr*>(&original), ll::memory::HookPriority::Normal);
    if (status == 0) return true;
    logger().warn("overlay: hooking {} failed with status {}", name, status);
    target    = nullptr;
    detourOut = nullptr;
    return false;
}

void unhookEntry(ll::memory::FuncPtr& target, ll::memory::FuncPtr& detour) {
    if (target && detour) ll::memory::unhook(target, detour);
    target = nullptr;
    detour = nullptr;
}

// the swap chain vtable comes from a throwaway chain on a hidden window of our own, never the game's window.
// The four entries are one set: cached targets are safe only while both resize hooks are in place
bool hookSwapChain(HWND probe) {
    D3D_FEATURE_LEVEL    level = D3D_FEATURE_LEVEL_11_0;
    DXGI_SWAP_CHAIN_DESC desc{};
    desc.BufferCount       = 1;
    desc.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
    desc.BufferUsage       = DXGI_USAGE_RENDER_TARGET_OUTPUT;
    desc.OutputWindow      = probe;
    desc.SampleDesc.Count  = 1;
    desc.Windowed          = TRUE;
    desc.SwapEffect        = DXGI_SWAP_EFFECT_DISCARD;
    ComPtr<ID3D11Device>    device;
    ComPtr<IDXGISwapChain>  chain;
    ComPtr<IDXGISwapChain3> chain3;
    if (FAILED(D3D11CreateDeviceAndSwapChain(nullptr, D3D_DRIVER_TYPE_HARDWARE, nullptr, 0, &level, 1, D3D11_SDK_VERSION, &desc, &chain, &device, nullptr, nullptr))
        || FAILED(chain.As(&chain3))) {
        logger().warn("overlay: could not make the probe swap chain");
        return false;
    }
    void** vtable = *reinterpret_cast<void***>(chain3.Get());
    return hookEntry("Present", vtable[PRESENT_INDEX], &onPresent, gPresentTarget, gPresentDetour, gOriginalPresent)
        && hookEntry("Present1", vtable[PRESENT1_INDEX], &onPresent1, gPresent1Target, gPresent1Detour, gOriginalPresent1)
        && hookEntry(
            "ResizeBuffers",
            vtable[RESIZE_BUFFERS_INDEX],
            &onResizeBuffers,
            gResizeBuffersTarget,
            gResizeBuffersDetour,
            gOriginalResizeBuffers
        )
        && hookEntry(
            "ResizeBuffers1",
            vtable[RESIZE_BUFFERS1_INDEX],
            &onResizeBuffers1,
            gResizeBuffers1Target,
            gResizeBuffers1Detour,
            gOriginalResizeBuffers1
        );
}

// optional: without it the D3D11 path still draws, and the D3D12 path cannot
void hookCommandQueue() {
    ComPtr<ID3D12Device>       device;
    ComPtr<ID3D12CommandQueue> queue;
    D3D12_COMMAND_QUEUE_DESC   desc{};
    desc.Type = D3D12_COMMAND_LIST_TYPE_DIRECT;
    if (FAILED(D3D12CreateDevice(nullptr, D3D_FEATURE_LEVEL_11_0, IID_PPV_ARGS(&device)))
        || FAILED(device->CreateCommandQueue(&desc, IID_PPV_ARGS(&queue)))) {
        logger().warn("overlay: could not make the probe D3D12 queue; the overlay can draw only on a D3D11 game");
        return;
    }
    void** vtable = *reinterpret_cast<void***>(queue.Get());
    if (!hookEntry("ExecuteCommandLists", vtable[EXECUTE_COMMAND_LISTS_INDEX], &onExecuteCommandLists, gExecuteTarget, gExecuteDetour, gOriginalExecute)) {
        logger().warn("overlay: without the queue hook the overlay can draw only on a D3D11 game");
    }
}

bool installHooks() {
    WNDCLASSEXW wc{};
    wc.cbSize        = sizeof(wc);
    wc.lpfnWndProc   = DefWindowProcW;
    wc.hInstance     = GetModuleHandleW(nullptr);
    wc.lpszClassName = PROBE_CLASS;
    RegisterClassExW(&wc);
    HWND probe = CreateWindowW(PROBE_CLASS, L"", WS_OVERLAPPEDWINDOW, 0, 0, PROBE_SIZE, PROBE_SIZE, nullptr, nullptr, wc.hInstance, nullptr);
    bool hooked = probe && hookSwapChain(probe);
    if (hooked) hookCommandQueue();
    if (probe) DestroyWindow(probe);
    UnregisterClassW(PROBE_CLASS, wc.hInstance);
    return hooked;
}

void unhookAll() {
    unhookEntry(gPresentTarget, gPresentDetour);
    unhookEntry(gPresent1Target, gPresent1Detour);
    unhookEntry(gResizeBuffersTarget, gResizeBuffersDetour);
    unhookEntry(gResizeBuffers1Target, gResizeBuffers1Detour);
    unhookEntry(gExecuteTarget, gExecuteDetour);
}

bool waitForDrain() {
    auto deadline = std::chrono::steady_clock::now() + DRAIN_TIMEOUT;
    while (gInFlight.load() > 0) {
        if (std::chrono::steady_clock::now() > deadline) return false;
        std::this_thread::sleep_for(DRAIN_POLL);
    }
    return true;
}

} // namespace

void OverlayHook::backBufferSize(float& width, float& height) {
    uint64_t packed = gBackBufferSize.load(std::memory_order_relaxed);
    width           = static_cast<float>(packed >> BACK_BUFFER_WIDTH_SHIFT);
    height          = static_cast<float>(packed & UINT32_MAX);
}

void OverlayHook::arm() {
    std::scoped_lock lock(gInstallMutex);
    if (!gInstalled.load() && !gLeaked) gAttempted.store(false);
}

bool OverlayHook::init() {
    if (gAttempted.load()) return gInstalled.load();
    std::scoped_lock lock(gInstallMutex);
    if (gAttempted.load()) return gInstalled.load();
    gAttempted.store(true);
    gShuttingDown.store(false);
    if (!installHooks()) {
        unhookAll();
        logger().warn("overlay: could not hook the swap chain; freecam runs without its overlay");
        return false;
    }
    gInstalled.store(true);
    logger().info("overlay: swap chain hooks installed");
    return true;
}

bool OverlayHook::shuttingDown() { return gShuttingDown.load(); }

bool OverlayHook::shutdown() {
    std::scoped_lock lock(gInstallMutex);
    gAttempted.store(true);
    if (!gInstalled.load()) return !gLeaked;
    gShuttingDown.store(true);
    unhookAll();
    gInstalled.store(false);
    if (!waitForDrain()) {
        // freeing under a thread still inside a hook would crash; leaking at exit is safe
        gLeaked = true;
        logger().warn("overlay: a thread is still inside a swap chain hook; the overlay device is left alive");
        return false;
    }
    releaseBinding();
    return true;
}

} // namespace levi_freecam
