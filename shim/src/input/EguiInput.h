#pragma once

#include <windows.h>

#include <cstdint>

namespace levi_freecam {

// a WndProc subclass on the overlay's game window. While cursor mode is on it hands mouse, key and focus messages
// to Rust with fc_ui_event and sets the cursor shape on WM_SETCURSOR. While a look drag holds the mouse it hides the
// cursor and keeps pointer moves from the UI, so the hover stays where the drag started. Every message still reaches the game's own
// WndProc; the existing input listeners keep freecam keys and keyboard flight working
class EguiInput {
public:
    // subclasses `window` once; render thread, when the overlay binds to the game window
    static void install(HWND window);

    // stops forwarding, restores the original WndProc and waits until no thread is inside a call into Rust. False
    // when a thread stayed inside one past the wait: the Rust handle must then be left alive. Safe to call more
    // than once
    static bool uninstall();

    // the cursor shape of the last UI frame: 0 arrow, 1 hand. Render thread; WM_SETCURSOR reads it
    static void setCursor(uint8_t shape);
};

} // namespace levi_freecam
