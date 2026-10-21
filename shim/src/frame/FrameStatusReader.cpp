#include "frame/FrameStatusReader.h"

#include "camera/CameraInstructions.h"
#include "camera/RenderOverride.h"

#include <mc/client/game/ClientInstance.h>
#include <mc/client/options/IOptionRegistry.h>
#include <mc/client/player/LocalPlayer.h>
#include <mc/deps/core/math/Vec2.h>
#include <mc/deps/core/math/Vec3.h>
#include <mc/deps/core/string/HashedString.h>
#include <mc/deps/ecs/gamerefs_entity/EntityContext.h>
#include <mc/deps/ecs/gamerefs_entity/GameRefsEntity.h>
#include <mc/deps/minecraft_camera/CameraRegistry.h>
#include <mc/deps/minecraft_camera/components/ActiveCameraComponent.h>
#include <mc/deps/minecraft_camera/components/CameraComponent.h>
#include <mc/deps/renderer/Camera.h>
#include <mc/world/level/Level.h>
#include <mc/world/level/dimension/DimensionType.h>

#include <glm/glm.hpp>
#include <glm/gtc/quaternion.hpp>

#include <windows.h>

#include <atomic>
#include <cmath>
#include <string_view>

namespace levi_freecam {

namespace {

constexpr std::string_view FREE_CAMERA = "minecraft:free";

constexpr std::string_view PLAYER_CAMERAS[] = {
    "minecraft:first_person",
    "minecraft:third_person",
    "minecraft:third_person_front",
};

constexpr float DEGREES_PER_RADIAN = 57.2957795f;

std::atomic<float> gCameraFov{0.0f};
std::atomic<float> gCameraAspect{0.0f};

double nowSeconds() {
    static const double frequency = [] {
        LARGE_INTEGER value;
        QueryPerformanceFrequency(&value);
        return static_cast<double>(value.QuadPart);
    }();
    LARGE_INTEGER now;
    QueryPerformanceCounter(&now);
    return static_cast<double>(now.QuadPart) / frequency;
}

bool gameWindowFocused() {
    HWND window = GetForegroundWindow();
    if (!window) return false;
    DWORD process = 0;
    GetWindowThreadProcessId(window, &process);
    return process == GetCurrentProcessId();
}

FcVec3 toFc(glm::vec3 const& value) { return FcVec3{value.x, value.y, value.z}; }

bool isPlayerCamera(std::string_view name) {
    for (auto camera : PLAYER_CAMERAS) {
        if (camera == name) return true;
    }
    return false;
}

// read-only walk of the camera entities: is a player camera active, and where is the game's free camera
void readCameras(ClientInstance& client, FcFrameStatus& status) {
    auto registry = client.getCameraRegistry();
    if (!registry) return;
    for (auto& owner : *registry->mCameraEntities) {
        if (!owner) continue;
        EntityContext& ctx = *owner;
        auto           cam = ctx.tryGetComponent<MinecraftCamera::CameraComponent>();
        if (!cam) continue;
        std::string_view name = cam->mId->getString();
        if (ctx.hasComponent<MinecraftCamera::ActiveCameraComponent>() && isPlayerCamera(name)) {
            status.player_camera_active = 1;
        }
        if (name == FREE_CAMERA) {
            glm::quat orientation           = cam->mOrientation;
            glm::vec3 position              = cam->mPosition;
            status.has_game_camera_forward  = 1;
            status.game_camera_forward      = toFc(orientation * glm::vec3(0, 0, -1));
            status.has_game_camera_position = 1;
            status.game_camera_position     = toFc(position);
        }
    }
}

void readPlayer(ClientInstance& client, LocalPlayer& player, FcFrameStatus& status) {
    status.in_world   = client.isWorldActive() ? 1 : 0;
    status.ready      = CameraInstructions::ready(player.getLevel()) ? 1 : 0;
    Vec3 head         = player.getHeadPos();
    Vec2 rotation     = player.getRotation();
    status.start_pose = FcPose{FcVec3{head.x, head.y, head.z}, rotation.y, rotation.x};
    status.health     = player.getHealth();
    status.dimension  = static_cast<int32_t>(player.getDimensionId());
}

// a camera with no aspect yet gives 0, and Rust falls back to the screen the mod manager draws on
void readProjection(FcFrameStatus& status) {
    status.fov_y_degrees = gCameraFov.load(std::memory_order_relaxed);
    status.aspect        = gCameraAspect.load(std::memory_order_relaxed);
}

} // namespace

FcFrameStatus FrameStatusReader::readOutsideWorld() {
    FcFrameStatus status{};
    status.time_seconds = nowSeconds();
    status.focused      = gameWindowFocused() ? 1 : 0;
    return status;
}

FcFrameStatus FrameStatusReader::read(ClientInstance* client) {
    FcFrameStatus status = readOutsideWorld();
    if (auto rendered = RenderOverride::takeRenderedPosition()) {
        status.has_rendered_position = 1;
        status.rendered_position     = *rendered;
    }
    readProjection(status);
    if (!client) return status;
    status.menu_open = client->isShowingMenu() ? 1 : 0;
    if (LocalPlayer* player = client->getLocalPlayer()) readPlayer(*client, *player, status);
    readCameras(*client, status);
    return status;
}

// the camera's mFov and mAspectRatio are 0 here, so both come from the projection matrix:
// P[1][1] = 1 / tan(fov_y / 2) and P[0][0] = P[1][1] / aspect. A degenerate matrix gives 0 for both, which
// core rejects, and the aspect then falls back to the screen size
void FrameStatusReader::observeCamera(mce::Camera const& camera) {
    glm::mat4 projection = camera.getProjectionMatrix();
    float     focalY     = projection[1][1];
    float     focalX     = projection[0][0];
    bool      usable     = std::isfinite(focalY) && std::isfinite(focalX) && focalY > 0.0f && focalX > 0.0f;
    gCameraFov.store(usable ? 2.0f * std::atan(1.0f / focalY) * DEGREES_PER_RADIAN : 0.0f, std::memory_order_relaxed);
    gCameraAspect.store(usable ? focalY / focalX : 0.0f, std::memory_order_relaxed);
}

} // namespace levi_freecam
