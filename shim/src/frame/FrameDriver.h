#pragma once

class ClientInstance;

namespace levi_freecam {

// the per-frame round trip: read the game, ask Rust, apply what Rust returns
class FrameDriver {
public:
    static void setClient(ClientInstance* client);

    // reads the status, calls fc_frame, then applies the camera, gui, override and input-block parts of the result
    static void beforeSetupCamera();

    // once per client tick: with no local player or no active world, runs a frame with in_world 0, because
    // setupCamera may not run there; applies only the gui, override and input-block parts, never a camera command
    static void tick(ClientInstance& client);

    // true while this frame's result blocks the player's own look input
    static bool blocksInput();

    static void reset();
};

} // namespace levi_freecam
