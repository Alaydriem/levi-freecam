#include "world/LookRay.h"
#include "bridge/ModLog.h"

#include "mod/FreecamMod.h"
#include "network/ClientPacketHandler.h"

#include <windows.h>

#include <mc/client/player/LocalPlayer.h>
#include <mc/deps/core/math/Vec3.h>
#include <mc/world/level/BlockSource.h>
#include <mc/world/level/ShapeType.h>
#include <mc/world/phys/HitDetection.h>
#include <mc/world/phys/HitResult.h>
#include <mc/world/phys/HitResultType.h>

#include <atomic>
#include <cmath>
#include <functional>

namespace levi_freecam {

namespace {

// a voxel walk crosses at most one block face per axis for each block of length, plus the start and end blocks
constexpr int STEPS_PER_BLOCK = 3;
constexpr int EXTRA_STEPS     = 3;

bool finite(FcVec3 const& value) { return std::isfinite(value.x) && std::isfinite(value.y) && std::isfinite(value.z); }

FcVec3 toFc(Vec3 const& value) { return FcVec3{value.x, value.y, value.z}; }

std::atomic<bool> g_threadMismatchLogged{false};

// the level is read only from the thread that handles the server's packets; any other thread is logged once and
// gets a miss
bool onHandlerThread() {
    uint32_t handlerThread = ClientPacketHandler::captureThread();
    uint32_t current       = static_cast<uint32_t>(GetCurrentThreadId());
    if (handlerThread == current) return true;
    if (!g_threadMismatchLogged.exchange(true)) {
        ModLog::warn(
            "look ray: thread {} differs from the packet handler thread {}; every ray is a miss",
            current,
            handlerThread
        );
    }
    return false;
}

// every block is a candidate; built once, so a cast does not construct a std::function
std::function<bool(BlockSource const&, Block const&, bool)> const& everyBlock() {
    static std::function<bool(BlockSource const&, Block const&, bool)> const check =
        [](BlockSource const&, Block const&, bool) { return true; };
    return check;
}

} // namespace

std::optional<FcVec3> LookRay::cast(LocalPlayer& player, FcVec3 origin, FcVec3 direction, float maxDistance) {
    if (!finite(origin) || !finite(direction) || !std::isfinite(maxDistance) || maxDistance <= 0.0f) return std::nullopt;
    if (!onHandlerThread()) return std::nullopt;
    Vec3 from(origin.x, origin.y, origin.z);
    Vec3 to(
        origin.x + direction.x * maxDistance,
        origin.y + direction.y * maxDistance,
        origin.z + direction.z * maxDistance
    );
    int          steps  = static_cast<int>(std::ceil(maxDistance)) * STEPS_PER_BLOCK + EXTRA_STEPS;
    BlockSource& region = player.getDimensionBlockSource();
    HitResult block = region.clip(from, to, false, ShapeType::Outline, steps, false, false, nullptr, everyBlock(), false);
    bool hitBlock   = block.mType == HitResultType::Tile;
    Vec3 end        = hitBlock ? block.mPos : to;
    // with no source the actor search box spans from..end, so it follows the camera ray, and it stops at the block
    // hit, so an actor behind a wall is not picked; the local player is never a target
    auto notSelf = [&player](Actor& candidate, Actor* source, Actor* owner) {
        return &candidate != &player && HitDetection::isValidHitDefault(candidate, source, owner);
    };
    HitResult actor = HitDetection::getClosestHitResult(region, nullptr, nullptr, from, end, notSelf);
    if (actor.mType == HitResultType::Entity) return toFc(actor.mPos);
    if (hitBlock) return toFc(block.mPos);
    return std::nullopt;
}

} // namespace levi_freecam
