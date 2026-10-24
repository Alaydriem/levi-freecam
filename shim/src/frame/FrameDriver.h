#pragma once

class ClientInstance;

namespace levi_freecam {

// the per-frame round trip: read the game, ask Rust, apply what Rust returns
class FrameDriver {
public:
    static void setClient(ClientInstance* client);

    // reads the status, calls fc_frame, then delivers the markers, answers the ray and applies the camera and the
    // override
    static void beforeSetupCamera();

    // once per client tick: with no local player or no active world, runs a frame with in_world 0, because
    // setupCamera may not run there; applies the override, never a camera command
    static void tick(ClientInstance& client);

    // the last frame's camera_owned, read on any thread; true while freecam controls the camera
    static bool cameraOwned();

    static void reset();
};

} // namespace levi_freecam
