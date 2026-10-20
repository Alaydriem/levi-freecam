#pragma once

#include "levi_freecam.h"

class ClientInstance;

namespace mce {
class Camera;
}

namespace levi_freecam {

// reads the game state Rust needs for one frame; read-only, it never changes the game
class FrameStatusReader {
public:
    // client may be null before the first client tick
    static FcFrameStatus read(ClientInstance* client);

    // the status for a tick with no world: only the time and focus, so in_world is 0
    static FcFrameStatus readOutsideWorld();

    // derives the field of view and aspect from the camera's projection matrix after setupCamera; the next read() passes them on
    static void observeCamera(mce::Camera const& camera);

    // logs the matrix-derived field of view and aspect beside the projection inputs read into status
    static void logProjectionInputs(FcFrameStatus const& status);
};

} // namespace levi_freecam
