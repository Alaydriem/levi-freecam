#include "mod/FreecamMod.h"

#include "bridge/RustBridge.h"
#include "camera/CameraInstructions.h"
#include "camera/RenderOverride.h"
#include "frame/FrameDriver.h"
#include "hooks/ClientUpdateHook.h"
#include "hooks/PacketHandlerHooks.h"
#include "hooks/RenderHooks.h"
#include "hooks/TurnDeltaHook.h"
#include "hooks/UIRenderHook.h"
#include "input/InputListeners.h"
#include "render/OverlayHook.h"

#include "ll/api/mod/RegisterHelper.h"

namespace levi_freecam {

FreecamMod& FreecamMod::getInstance() {
    static FreecamMod instance;
    return instance;
}

bool FreecamMod::load() {
    auto& self       = getSelf();
    auto  configFile = self.getConfigDir() / "config.json";
    if (!RustBridge::create(configFile)) {
        self.getLogger().error("levi-freecam could not start its Rust core");
        return false;
    }
    return true;
}

bool FreecamMod::enable() {
    ClientUpdateHook::hook();
    ChunkPublisherHook::hook();
    CameraPresetsHook::hook();
    CameraInstructionHook::hook();
    SetupCameraHook::hook();
    ViewRenderObjectHook::hook();
    LevelCameraHook::hook();
    TurnDeltaHook::hook();
    OverlayHook::arm();
    UIRenderHook::hook();
    InputListeners::registerListeners();
    getSelf().getLogger().info("levi-freecam loaded");
    return true;
}

bool FreecamMod::disable() {
    InputListeners::unregisterListeners();
    // unhook first so no game, render or packet call can enter Rust, then stop Rust
    UIRenderHook::unhook();
    bool overlayDrained = OverlayHook::shutdown();
    TurnDeltaHook::unhook();
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
    // a thread still inside the overlay draw may be in fc_overlay; leaking the handle at exit is safe
    if (!overlayDrained) {
        getSelf().getLogger().warn("levi-freecam leaves its Rust core alive because the overlay did not stop in time");
        return true;
    }
    RustBridge::destroy();
    return true;
}

} // namespace levi_freecam

LL_REGISTER_MOD(levi_freecam::FreecamMod, levi_freecam::FreecamMod::getInstance());
