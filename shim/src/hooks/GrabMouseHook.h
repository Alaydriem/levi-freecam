#pragma once

#include "input/MouseGrab.h"

#include <ll/api/memory/Hook.h>
#include <mc/client/game/MinecraftGame.h>

namespace levi_freecam {

// the game grabs the mouse again on its own while the world shows with no screen; in cursor mode that grab is
// skipped so the cursor stays free. Freecam's own grab runs after cursor mode has ended, or while a look drag holds
// the mouse
LL_TYPE_INSTANCE_HOOK(
    GrabMouseHook,
    ll::memory::HookPriority::Normal,
    MinecraftGame,
    &MinecraftGame::$grabMouse,
    void
) {
    if (MouseGrab::blocksGrab()) return;
    origin();
}

} // namespace levi_freecam
