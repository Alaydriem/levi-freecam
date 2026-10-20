#pragma once

#include "frame/FrameDriver.h"

#include <ll/api/memory/Hook.h>
#include <mc/client/player/LocalPlayer.h>
#include <mc/deps/core/math/Vec2.h>

namespace levi_freecam {

// mouse look reaches the player here; skipped while Rust blocks player input
LL_TYPE_INSTANCE_HOOK(
    TurnDeltaHook,
    ll::memory::HookPriority::Normal,
    LocalPlayer,
    &LocalPlayer::_applyTurnDelta,
    void,
    Vec2 const& turnOffset
) {
    if (FrameDriver::blocksInput()) return;
    origin(turnOffset);
}

} // namespace levi_freecam
