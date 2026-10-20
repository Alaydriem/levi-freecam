#include "camera/RenderOverride.h"

#include <mc/client/renderer/game/LevelRendererCamera.h>
#include <mc/deps/core/math/Vec3.h>
#include <mc/deps/minecraft_renderer/objects/ViewRenderData.h>
#include <mc/deps/renderer/Camera.h>

#include <glm/glm.hpp>
#include <glm/gtc/matrix_transform.hpp>

#include <cmath>
#include <mutex>

namespace levi_freecam {

namespace {

constexpr float DEG = 3.14159265358979f / 180.0f;

struct OverrideState {
    std::mutex mutex;
    bool       active = false;
    FcPose     pose{};
    bool       hasRendered = false;
    FcVec3     rendered{};
};

OverrideState& state() {
    static OverrideState instance;
    return instance;
}

// forward from yaw and pitch in degrees; pitch positive looks down
glm::vec3 forwardOf(float yawDeg, float pitchDeg) {
    float yaw   = yawDeg * DEG;
    float pitch = pitchDeg * DEG;
    return {-std::sin(yaw) * std::cos(pitch), -std::sin(pitch), std::cos(yaw) * std::cos(pitch)};
}

std::optional<FcVec3> activePosition() {
    auto&           s = state();
    std::lock_guard lock(s.mutex);
    if (!s.active) return std::nullopt;
    s.hasRendered = true;
    s.rendered    = s.pose.position;
    return s.pose.position;
}

} // namespace

void RenderOverride::set(bool active, const FcPose& pose) {
    auto&           s = state();
    std::lock_guard lock(s.mutex);
    s.active = active;
    s.pose   = pose;
}

void RenderOverride::applyRotation(mce::Camera& camera) {
    FcPose pose{};
    {
        auto&           s = state();
        std::lock_guard lock(s.mutex);
        if (!s.active) return;
        pose = s.pose;
    }
    glm::vec3 fwd   = forwardOf(pose.yaw, pose.pitch);
    glm::vec3 right = glm::normalize(glm::cross(fwd, glm::vec3(0, 1, 0)));
    glm::vec3 up    = glm::cross(right, fwd);

    glm::mat4& view = *camera.viewMatrixStack->stack->back()._m;
    view            = glm::lookAt(glm::vec3(0.0f), fwd, glm::vec3(0, 1, 0));
    camera.viewMatrixStack->_isDirty = true;
    camera.mInverseViewMatrix        = glm::inverse(view);
    camera.mForward                  = fwd;
    camera.mRight                    = right;
    camera.mUp                       = up;
    camera.updateViewMatrixDependencies();
}

void RenderOverride::applyPosition(ViewRenderData& data) {
    auto position = activePosition();
    if (!position) return;
    glm::vec3 pos(position->x, position->y, position->z);
    data.mCameraPos       = pos;
    data.mCameraTargetPos = pos;
}

void RenderOverride::applyPosition(LevelRendererCamera& camera) {
    auto position = activePosition();
    if (!position) return;
    camera.mCameraPos       = Vec3(position->x, position->y, position->z);
    camera.mCameraTargetPos = Vec3(position->x, position->y, position->z);
}

std::optional<FcVec3> RenderOverride::takeRenderedPosition() {
    auto&           s = state();
    std::lock_guard lock(s.mutex);
    if (!s.hasRendered) return std::nullopt;
    s.hasRendered = false;
    return s.rendered;
}

void RenderOverride::reset() {
    auto&           s = state();
    std::lock_guard lock(s.mutex);
    s.active      = false;
    s.hasRendered = false;
}

} // namespace levi_freecam
