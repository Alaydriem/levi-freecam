#include "mod/FreecamMod.h"

#include "bridge/ModLog.h"
#include "bridge/RustBridge.h"
#include "camera/CameraInstructions.h"
#include "camera/RenderOverride.h"
#include "frame/FrameDriver.h"
#include "hooks/ClientUpdateHook.h"
#include "hooks/PacketHandlerHooks.h"
#include "hooks/RenderHooks.h"
#include "network/ClientPacketHandler.h"
#include "telemetry/LatencyProbe.h"
#include "world/ShapeDelivery.h"

#include "ll/api/mod/RegisterHelper.h"
#include "ll/api/service/GamingStatus.h"
#include "ll/api/service/TargetedBedrock.h"

#include <mc/client/game/ClientInstance.h>
#include <mc/client/player/LocalPlayer.h>

namespace levi_freecam {

FreecamMod& FreecamMod::getInstance() {
    static FreecamMod instance;
    return instance;
}

bool FreecamMod::load() {
    mConfigDir = std::filesystem::absolute(getSelf().getConfigDir());
    return true;
}

bool FreecamMod::enable() {
    if (!RustBridge::create(mConfigDir)) {
        // no Rust sink exists without a registration, so this one line uses the LeviLamina logger
        getSelf().getLogger().error("alaydriem-mod-manager not found or registration failed; freecam stays off");
        return true;
    }
    ClientUpdateHook::hook();
    ChunkPublisherHook::hook();
    CameraPresetsHook::hook();
    CameraInstructionHook::hook();
    SetupCameraHook::hook();
    ViewRenderObjectHook::hook();
    LevelCameraHook::hook();
    LatencyProbe::install();
    mHooked = true;
    ModLog::info("levi-freecam loaded");
    return true;
}

bool FreecamMod::disable() {
    if (!mHooked) return true;
    // before any unhook, while the packet handler is still captured and Rust is still alive
    removeMarkers();
    // unhook first so no game, render, packet or key call can enter Rust, then stop Rust
    LatencyProbe::uninstall();
    LevelCameraHook::unhook();
    ViewRenderObjectHook::unhook();
    SetupCameraHook::unhook();
    CameraInstructionHook::unhook();
    CameraPresetsHook::unhook();
    ChunkPublisherHook::unhook();
    ClientUpdateHook::unhook();
    FrameDriver::reset();
    RenderOverride::reset();
    CameraInstructions::reset();
    ShapeDelivery::reset();
    ClientPacketHandler::reset();
    mHooked = false;
    RustBridge::destroy();
    return true;
}

// the handler is used only while the game still has a client with a local player, and never while the game is
// stopping: client shapes end with the process, and the handler may already be freed
void FreecamMod::removeMarkers() {
    FcHandle* handle = RustBridge::mutableHandle();
    if (!handle) return;
    const FcMarkerEntry* entries = nullptr;
    uintptr_t            count   = 0;
    if (fc_markers_shutdown(handle, &entries, &count) != FC_STATUS_OK || count == 0) return;
    if (ll::getGamingStatus() == ll::GamingStatus::Stopping) {
        ModLog::info("markers: the game is stopping; {} shapes go with the process", count);
        return;
    }
    auto client    = ll::service::getClientInstance();
    bool worldLive = client && client->getLocalPlayer() != nullptr;
    if (worldLive && ShapeDelivery::deliver(entries, count)) {
        ModLog::info("markers: removed {} shapes at shutdown", count);
        return;
    }
    ModLog::warn("markers: {} shapes not removed at shutdown (no world, no captured handler, or another thread)", count);
}

} // namespace levi_freecam

LL_REGISTER_MOD(levi_freecam::FreecamMod, levi_freecam::FreecamMod::getInstance());
