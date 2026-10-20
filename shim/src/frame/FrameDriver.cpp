#include "frame/FrameDriver.h"

#include "bridge/RustBridge.h"
#include "camera/CameraInstructions.h"
#include "camera/RenderOverride.h"
#include "frame/FrameStatusReader.h"
#include "hud/HudControl.h"

#include <mc/client/game/ClientInstance.h>
#include <mc/client/player/LocalPlayer.h>

#include <atomic>

namespace levi_freecam {

namespace {

constexpr uint8_t CAMERA_SET   = 1;
constexpr uint8_t CAMERA_CLEAR = 2;

std::atomic<ClientInstance*> g_client{nullptr};
std::atomic<bool>            g_blockInput{false};
// the last frame's show_overlay, kept only to log the projection inputs when the overlay appears
std::atomic<bool>            g_showOverlay{false};

void applyCamera(FcFrameOutput const& out, LocalPlayer const* player) {
    if (out.camera == CAMERA_SET) CameraInstructions::sendSet(player, out.pose);
    if (out.camera == CAMERA_CLEAR) CameraInstructions::sendClear(player);
}

// asks Rust for this frame; a failed call gives the zeroed output, which does nothing
FcFrameOutput runFrame(const FcHandle* handle, FcFrameStatus const& status) {
    FcFrameOutput out{};
    if (fc_frame(handle, &status, &out) != FC_STATUS_OK) out = FcFrameOutput{};
    return out;
}

// returns true on the frame the overlay appears
bool applyRest(ClientInstance* client, FcFrameOutput const& out) {
    if (client) HudControl::apply(*client, out.gui);
    RenderOverride::set(out.has_override != 0, out.override_pose);
    g_blockInput.store(out.block_input != 0, std::memory_order_relaxed);
    bool showOverlay = out.show_overlay != 0;
    bool wasShown    = g_showOverlay.exchange(showOverlay, std::memory_order_relaxed);
    return showOverlay && !wasShown;
}

} // namespace

void FrameDriver::setClient(ClientInstance* client) { g_client.store(client, std::memory_order_release); }

void FrameDriver::beforeSetupCamera() {
    const FcHandle* handle = RustBridge::handle();
    if (!handle) return;
    ClientInstance* client = g_client.load(std::memory_order_acquire);
    LocalPlayer*    player = client ? client->getLocalPlayer() : nullptr;
    CameraInstructions::observeLocalPlayer(player);
    FcFrameStatus status = FrameStatusReader::read(client);
    FcFrameOutput out    = runFrame(handle, status);
    applyCamera(out, player);
    if (applyRest(client, out)) FrameStatusReader::logProjectionInputs(status);
}

void FrameDriver::tick(ClientInstance& client) {
    LocalPlayer* player = client.getLocalPlayer();
    if (player && client.isWorldActive()) return;
    const FcHandle* handle = RustBridge::handle();
    if (!handle) return;
    CameraInstructions::observeLocalPlayer(player);
    FcFrameOutput out = runFrame(handle, FrameStatusReader::readOutsideWorld());
    applyRest(&client, out);
}

bool FrameDriver::blocksInput() { return g_blockInput.load(std::memory_order_relaxed); }

void FrameDriver::reset() {
    g_blockInput.store(false, std::memory_order_relaxed);
    g_showOverlay.store(false, std::memory_order_relaxed);
    g_client.store(nullptr, std::memory_order_release);
}

} // namespace levi_freecam
