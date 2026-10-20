use crate::config::KeyConfig;
use crate::structs::{HeldKeys, InputAction, KeyOutcome};

const VK_ESCAPE: u32 = 0x1B;
// F1 shows the HUD that freecam hides, and with it the game's name tags
const VK_F1: u32 = 0x70;
const BUTTON_WHEEL: u8 = 4;

pub struct InputTracker {
    keys: KeyConfig,
    held: HeldKeys,
    toggle_down: bool,
    smooth_down: bool,
    add_down: bool,
    cursor_down: bool,
    delete_down: bool,
    look_x: f32,
    look_y: f32,
    scroll: i32,
}

impl InputTracker {
    pub fn new(keys: KeyConfig) -> Self {
        Self {
            keys,
            held: HeldKeys::default(),
            toggle_down: false,
            smooth_down: false,
            add_down: false,
            cursor_down: false,
            delete_down: false,
            look_x: 0.0,
            look_y: 0.0,
            scroll: 0,
        }
    }

    pub fn on_key(&mut self, code: u32, down: bool, active: bool, menu_open: bool) -> KeyOutcome {
        self.track_held(code, down);
        if code == self.keys.delete {
            return self.on_delete(down, active, menu_open);
        }
        let is_repeat = self.track_chord_key(code, down);
        if !down {
            return KeyOutcome::default();
        }
        let action = if is_repeat { None } else { self.chord_action(code, active, menu_open) };
        if let Some(action) = action {
            return KeyOutcome { consume: true, action: Some(action) };
        }
        KeyOutcome { consume: active && !menu_open && code != VK_ESCAPE && code != VK_F1, action: None }
    }

    // Del needs no modifier and acts only while freecam is active
    fn on_delete(&mut self, down: bool, active: bool, menu_open: bool) -> KeyOutcome {
        let repeat = down && self.delete_down;
        self.delete_down = down;
        if !down || !active || menu_open {
            return KeyOutcome::default();
        }
        let action = if repeat { None } else { Some(InputAction::DeletePoint) };
        KeyOutcome { consume: true, action }
    }

    pub fn on_mouse_move(&mut self, dx: i32, dy: i32, active: bool, menu_open: bool) {
        if active && !menu_open {
            self.look_x += dx as f32;
            self.look_y += dy as f32;
        }
    }

    pub fn on_mouse_button(&mut self, button: u8, data: i8, active: bool, menu_open: bool) -> bool {
        if !active || menu_open {
            return false;
        }
        if button == BUTTON_WHEEL {
            self.scroll += i32::from(data.signum());
            return true;
        }
        // button data 0 is a release, which must reach the game so a held click ends
        data != 0
    }

    pub fn take_look(&mut self) -> (f32, f32) {
        let look = (self.look_x, self.look_y);
        self.look_x = 0.0;
        self.look_y = 0.0;
        look
    }

    pub fn take_scroll(&mut self) -> i32 {
        std::mem::take(&mut self.scroll)
    }

    pub fn held(&self) -> HeldKeys {
        self.held
    }

    pub fn clear_held(&mut self) {
        self.held = HeldKeys::default();
        self.toggle_down = false;
        self.smooth_down = false;
        self.add_down = false;
        self.cursor_down = false;
        self.delete_down = false;
    }

    fn track_held(&mut self, code: u32, down: bool) {
        let keys = &self.keys;
        if code == keys.modifier {
            self.held.fast = down;
        }
        if code == keys.forward {
            self.held.forward = down;
        }
        if code == keys.back {
            self.held.back = down;
        }
        if code == keys.left {
            self.held.left = down;
        }
        if code == keys.right {
            self.held.right = down;
        }
        if code == keys.up {
            self.held.up = down;
        }
        if code == keys.down {
            self.held.down = down;
        }
    }

    // records the chord key state and returns true when a down repeats a key already down
    fn track_chord_key(&mut self, code: u32, down: bool) -> bool {
        let slot = if code == self.keys.toggle {
            &mut self.toggle_down
        } else if code == self.keys.smooth_toggle {
            &mut self.smooth_down
        } else if code == self.keys.add_point {
            &mut self.add_down
        } else if code == self.keys.cursor {
            &mut self.cursor_down
        } else {
            return false;
        };
        let repeat = down && *slot;
        *slot = down;
        repeat
    }

    fn chord_action(&self, code: u32, active: bool, menu_open: bool) -> Option<InputAction> {
        if menu_open || !self.held.fast {
            return None;
        }
        if code == self.keys.toggle {
            return Some(InputAction::ToggleFreecam);
        }
        if active && code == self.keys.smooth_toggle {
            return Some(InputAction::ToggleSmooth);
        }
        if active && code == self.keys.add_point {
            return Some(InputAction::AddPoint);
        }
        if active && code == self.keys.cursor {
            return Some(InputAction::ToggleCursor);
        }
        None
    }
}
