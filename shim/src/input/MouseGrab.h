#pragma once

#include <windows.h>

#include <cstdint>

class ClientInstance;

namespace levi_freecam {

// lets go of the mouse or takes it back for the game when Rust says so, and tells the grab hook when the game must
// not take it back on its own
class MouseGrab {
public:
    // command: 0 none, 1 MinecraftGame::releaseMouse(), 2 MinecraftGame::grabMouse(), 3 keep the cursor position and
    // grab for a look drag, 4 release after a look drag and put the cursor back at the kept position. Game thread
    static void apply(ClientInstance& client, uint8_t command);

    // true while cursor mode is on and no look drag holds the mouse: GrabMouseHook then skips the game's own grab
    static bool blocksGrab();

    // on disable with cursor mode still on: grabs the mouse once if the game is in play (in a world, with a local
    // player and no menu) and `gameWindow` is the foreground window, so the player is not left with a free cursor
    // once freecam is gone. An unfocused game, as at game close, is left alone
    static void restoreOnDisable(HWND gameWindow);
};

} // namespace levi_freecam
