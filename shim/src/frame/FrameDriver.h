#pragma once

class ClientInstance;

namespace levi_freecam {

// the per-frame round trip: read the game, ask Rust, apply what Rust returns
class FrameDriver {
public:
    static void setClient(ClientInstance* client);

    // reads the status, calls fc_frame, then applies the camera, gui, override, input-block and mouse parts of the
    // result
    static void beforeSetupCamera();

    // once per client tick: with no local player or no active world, runs a frame with in_world 0, because
    // setupCamera may not run there; applies the gui, override, input-block and mouse parts, never a camera command
    static void tick(ClientInstance& client);

    // true while this frame's result blocks the player's own look input
    static bool blocksInput();

    // the last frame's cursor_mode, read on any thread
    static bool cursorMode();

    // the last frame's look_drag, read on any thread; true while a right-button look drag holds the mouse in bar mode
    static bool lookDrag();

    // the last frame's ui_visible, read on any thread; the overlay runs a UI frame only while it is true
    static bool uiVisible();

    // the last frame's hide_ui, read on any thread; while true a screenshot is under way and the overlay draws
    // nothing
    static bool uiHidden();

    // the last frame's camera_owned, read on any thread; true while freecam controls the camera
    static bool cameraOwned();

    static void reset();
};

} // namespace levi_freecam
