#include "mod/FreecamMod.h"

#include "bridge/RustBridge.h"
#include "camera/CameraInstructions.h"
#include "camera/RenderOverride.h"
#include "frame/FrameDriver.h"
#include "hooks/ClientUpdateHook.h"
#include "hooks/GrabMouseHook.h"
#include "hooks/PacketHandlerHooks.h"
#include "hooks/RenderHooks.h"
#include "hooks/TurnDeltaHook.h"
#include "hooks/UIRenderHook.h"
#include "input/EguiInput.h"
#include "input/InputListeners.h"
#include "input/MouseGrab.h"
#include "network/ClientPacketHandler.h"
#include "render/FrameCapture.h"
#include "render/OverlayHook.h"
#include "render/PngWriter.h"
#include "render/ScreenshotPath.h"
#include "world/ShapeDelivery.h"

#include "ll/api/mod/RegisterHelper.h"
#include "ll/api/service/GamingStatus.h"
#include "ll/api/service/TargetedBedrock.h"

#include <mc/client/game/ClientInstance.h>
#include <mc/client/player/LocalPlayer.h>

#include <string>
#include <string_view>
#include <utility>

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
    configureScreenshots();
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
    GrabMouseHook::hook();
    if (!PngWriter::start()) getSelf().getLogger().warn("screenshot: the PNG writer thread could not start; screenshots fail");
    OverlayHook::arm();
    UIRenderHook::hook();
    InputListeners::registerListeners();
    getSelf().getLogger().info("levi-freecam loaded");
    return true;
}

bool FreecamMod::disable() {
    // before any unhook, while the packet handler is still captured and Rust is still alive
    removeMarkers();
    InputListeners::unregisterListeners();
    // the game window gets its own WndProc back, and the game may grab the mouse again, before the overlay stops
    bool inputDrained = EguiInput::uninstall();
    GrabMouseHook::unhook();
    if (FrameDriver::cursorMode()) MouseGrab::restoreOnDisable(OverlayHook::gameWindow());
    // unhook first so no game, render or packet call can enter Rust, then stop Rust
    UIRenderHook::unhook();
    bool overlayDrained = OverlayHook::shutdown();
    // the render thread is out of the capture code only when the overlay drained; a worker still writing is left to
    // finish on its own
    if (overlayDrained && !PngWriter::shutdown()) {
        getSelf().getLogger().warn("screenshot: the PNG writer was still writing at shutdown and is left to finish");
    }
    // a Present that bound the window during the first uninstall may have subclassed it again
    inputDrained = EguiInput::uninstall() && inputDrained;
    TurnDeltaHook::unhook();
    LevelCameraHook::unhook();
    ViewRenderObjectHook::unhook();
    SetupCameraHook::unhook();
    CameraInstructionHook::unhook();
    CameraPresetsHook::unhook();
    ChunkPublisherHook::unhook();
    ClientUpdateHook::unhook();
    FrameDriver::reset();
    FrameCapture::reset();
    RenderOverride::reset();
    CameraInstructions::reset();
    ShapeDelivery::reset();
    ClientPacketHandler::reset();
    // a thread still inside the overlay draw may be in fc_ui_frame, or one inside the WndProc in fc_ui_event;
    // leaking the handle at exit is safe
    if (!overlayDrained || !inputDrained) {
        getSelf().getLogger().warn("levi-freecam leaves its Rust core alive because the overlay or the UI input did not stop in time");
        return true;
    }
    RustBridge::destroy();
    return true;
}

// the folder is expanded once at load; it is created, if missing, before each write
void FreecamMod::configureScreenshots() {
    FcStr folder{};
    if (fc_screenshot_folder(RustBridge::handle(), &folder) != FC_STATUS_OK || !folder.ptr) {
        getSelf().getLogger().warn("screenshot: the folder could not be read from config; screenshots fail");
        return;
    }
    std::string_view configured(reinterpret_cast<char const*>(folder.ptr), folder.len);
    std::wstring     expanded = ScreenshotPath::expand(configured);
    if (expanded.empty()) {
        getSelf().getLogger().warn("screenshot: the folder {} could not be expanded; screenshots fail", configured);
        return;
    }
    getSelf().getLogger().info("screenshot: files go to {}", ScreenshotPath::utf8(expanded));
    PngWriter::configure(std::move(expanded));
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
        getSelf().getLogger().info("markers: the game is stopping; {} shapes go with the process", count);
        return;
    }
    auto client    = ll::service::getClientInstance();
    bool worldLive = client && client->getLocalPlayer() != nullptr;
    if (worldLive && ShapeDelivery::deliver(entries, count)) {
        getSelf().getLogger().info("markers: removed {} shapes at shutdown", count);
        return;
    }
    getSelf().getLogger().warn(
        "markers: {} shapes not removed at shutdown (no world, no captured handler, or another thread)",
        count
    );
}

} // namespace levi_freecam

LL_REGISTER_MOD(levi_freecam::FreecamMod, levi_freecam::FreecamMod::getInstance());
