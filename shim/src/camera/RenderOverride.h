#pragma once

#include "levi_freecam.h"

#include <optional>

class LevelRendererCamera;
struct ViewRenderData;

namespace mce {
class Camera;
}

namespace levi_freecam {

// this frame's renderer override pose, applied by the render hooks; also records the position it drew from
class RenderOverride {
public:
    // inactive clears the override
    static void set(bool active, const FcPose& pose);

    // rotation only: the game draws relative to the camera, so the view keeps its zero translation
    static void applyRotation(mce::Camera& camera);

    static void applyPosition(ViewRenderData& data);

    static void applyPosition(LevelRendererCamera& camera);

    // the position last drawn from since the previous call, then forgets it
    static std::optional<FcVec3> takeRenderedPosition();

    static void reset();
};

} // namespace levi_freecam
