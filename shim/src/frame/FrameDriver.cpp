#include "frame/FrameDriver.h"

#include "bridge/RustBridge.h"
#include "camera/CameraInstructions.h"
#include "camera/RenderOverride.h"
#include "frame/FrameStatusReader.h"
#include "hud/HudControl.h"
#include "input/MouseGrab.h"
#include "mod/FreecamMod.h"
#include "network/ClientPacketHandler.h"
#include "render/FrameCapture.h"
#include "render/OverlayHook.h"
#include "world/LookRay.h"
#include "world/ShapeDelivery.h"

#include <windows.h>

#include <mc/client/game/ClientInstance.h>
#include <mc/client/player/LocalPlayer.h>

#include <atomic>
#include <optional>

namespace levi_freecam {

namespace {

constexpr uint8_t CAMERA_SET   = 1;
constexpr uint8_t CAMERA_CLEAR = 2;

std::atomic<ClientInstance*> g_client{nullptr};
std::atomic<bool>            g_blockInput{false};
std::atomic<bool>            g_uiVisible{false};
std::atomic<bool>            g_uiHidden{false};
std::atomic<bool>            g_cursorMode{false};
std::atomic<bool>            g_lookDrag{false};
std::atomic<bool>            g_cameraOwned{false};
std::atomic<float>           g_lastShapeSendUs{0.0f};

void applyCamera(FcFrameOutput const& out, LocalPlayer const* player) {
    if (out.camera == CAMERA_SET) CameraInstructions::sendSet(player, out.pose);
    if (out.camera == CAMERA_CLEAR) CameraInstructions::sendClear(player);
}

// one line per call site with the id of the thread that runs it, for the live check of which thread runs what
std::atomic<bool> g_setupCameraThreadLogged{false};
std::atomic<bool> g_tickThreadLogged{false};
std::atomic<bool> g_updateThreadLogged{false};

void noteThread(std::atomic<bool>& logged, char const* site) {
    if (logged.exchange(true)) return;
    FreecamMod::getInstance().getSelf().getLogger().info("frame: {} runs on thread {}", site, GetCurrentThreadId());
}

double microsecondsSince(LARGE_INTEGER const& start) {
    static const double frequency = [] {
        LARGE_INTEGER value;
        QueryPerformanceFrequency(&value);
        return static_cast<double>(value.QuadPart);
    }();
    LARGE_INTEGER now;
    QueryPerformanceCounter(&now);
    return static_cast<double>(now.QuadPart - start.QuadPart) * 1'000'000.0 / frequency;
}

// the core sees the handler only on the thread that may deliver to it, and the time the last batch took
void addDeliveryStatus(FcFrameStatus& status) {
    status.last_shape_send_us = g_lastShapeSendUs.load(std::memory_order_relaxed);
    status.handler_generation = ShapeDelivery::generation();
}

// asks Rust for this frame; a failed call gives the zeroed output, which does nothing. The screenshot answers the
// render thread and the PNG worker left go in first, and a capture this frame asks for goes to the next Present
FcFrameOutput runFrame(const FcHandle* handle, FcFrameStatus status) {
    addDeliveryStatus(status);
    FrameCapture::deliver(RustBridge::mutableHandle());
    FcFrameOutput out{};
    if (fc_frame(handle, &status, &out) != FC_STATUS_OK) out = FcFrameOutput{};
    if (out.capture_active != 0) FrameCapture::request(out.capture_sequence, OverlayHook::presentCount());
    return out;
}

// delivers this frame's marker batch and keeps the time it took for the next status; 0 when nothing went out
void deliverMarkers(FcFrameOutput const& out) {
    float elapsed = 0.0f;
    if (out.marker_count > 0) {
        LARGE_INTEGER start;
        QueryPerformanceCounter(&start);
        if (ShapeDelivery::deliver(out.markers, out.marker_count)) elapsed = static_cast<float>(microsecondsSince(start));
    }
    g_lastShapeSendUs.store(elapsed, std::memory_order_relaxed);
}

// answers the ray this frame asked for; with no local player the answer is a miss
void answerRay(FcFrameOutput const& out, LocalPlayer* player) {
    if (out.ray.active == 0) return;
    FcHandle* handle = RustBridge::mutableHandle();
    if (!handle) return;
    std::optional<FcVec3> hit;
    if (player) hit = LookRay::cast(*player, out.ray.origin, out.ray.direction, out.ray.max_distance);
    if (hit) fc_ray_result(handle, 1, hit->x, hit->y, hit->z);
    else fc_ray_result(handle, 0, 0.0f, 0.0f, 0.0f);
}

void applyRest(ClientInstance* client, FcFrameOutput const& out) {
    if (client) HudControl::apply(*client, out.gui);
    RenderOverride::set(out.has_override != 0, out.override_pose);
    g_blockInput.store(out.block_input != 0, std::memory_order_relaxed);
    g_uiVisible.store(out.ui_visible != 0, std::memory_order_relaxed);
    g_uiHidden.store(out.hide_ui != 0, std::memory_order_relaxed);
    // stored before the mouse command, so the grab hook lets freecam's own grab through once cursor mode ends, and the
    // drag's grab through while a look drag starts
    g_cursorMode.store(out.cursor_mode != 0, std::memory_order_relaxed);
    g_lookDrag.store(out.look_drag != 0, std::memory_order_relaxed);
    g_cameraOwned.store(out.camera_owned != 0, std::memory_order_relaxed);
    if (client) MouseGrab::apply(*client, out.mouse);
}

} // namespace

void FrameDriver::setClient(ClientInstance* client) { g_client.store(client, std::memory_order_release); }

void FrameDriver::beforeSetupCamera() {
    const FcHandle* handle = RustBridge::handle();
    if (!handle) return;
    ClientInstance* client = g_client.load(std::memory_order_acquire);
    LocalPlayer*    player = client ? client->getLocalPlayer() : nullptr;
    noteThread(g_setupCameraThreadLogged, "fc_frame (setupCamera)");
    ClientPacketHandler::observeLocalPlayer(player);
    FcFrameStatus status = FrameStatusReader::read(client);
    FcFrameOutput out    = runFrame(handle, status);
    deliverMarkers(out);
    answerRay(out, player);
    applyCamera(out, player);
    applyRest(client, out);
}

void FrameDriver::tick(ClientInstance& client) {
    noteThread(g_updateThreadLogged, "ClientInstance::update");
    LocalPlayer* player = client.getLocalPlayer();
    if (player && client.isWorldActive()) return;
    const FcHandle* handle = RustBridge::handle();
    if (!handle) return;
    noteThread(g_tickThreadLogged, "fc_frame (client update)");
    ClientPacketHandler::observeLocalPlayer(player);
    FcFrameOutput out = runFrame(handle, FrameStatusReader::readOutsideWorld());
    deliverMarkers(out);
    // no world is active here, so there is nothing to cast through
    answerRay(out, nullptr);
    applyRest(&client, out);
}

bool FrameDriver::blocksInput() { return g_blockInput.load(std::memory_order_relaxed); }

bool FrameDriver::cursorMode() { return g_cursorMode.load(std::memory_order_relaxed); }

bool FrameDriver::lookDrag() { return g_lookDrag.load(std::memory_order_relaxed); }

bool FrameDriver::uiVisible() { return g_uiVisible.load(std::memory_order_relaxed); }

bool FrameDriver::uiHidden() { return g_uiHidden.load(std::memory_order_relaxed); }

bool FrameDriver::cameraOwned() { return g_cameraOwned.load(std::memory_order_relaxed); }

void FrameDriver::reset() {
    g_blockInput.store(false, std::memory_order_relaxed);
    g_uiVisible.store(false, std::memory_order_relaxed);
    g_uiHidden.store(false, std::memory_order_relaxed);
    g_cursorMode.store(false, std::memory_order_relaxed);
    g_lookDrag.store(false, std::memory_order_relaxed);
    g_cameraOwned.store(false, std::memory_order_relaxed);
    g_lastShapeSendUs.store(0.0f, std::memory_order_relaxed);
    g_client.store(nullptr, std::memory_order_release);
}

} // namespace levi_freecam
