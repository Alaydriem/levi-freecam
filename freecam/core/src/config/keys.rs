use log::warn;
use serde::{Deserialize, Serialize};

const VK_TAB: u32 = 0x09;
const DEFAULT_CURSOR_KEY: u32 = 0x45;

// Windows virtual-key codes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct KeyConfig {
    pub toggle: u32,
    pub smooth_toggle: u32,
    pub add_point: u32,
    // both the chord key and the fast key
    pub modifier: u32,
    // chord key that shows or hides the mouse cursor
    pub cursor: u32,
    // pressed on its own, without the modifier
    pub delete: u32,
    pub forward: u32,
    pub back: u32,
    pub left: u32,
    pub right: u32,
    pub up: u32,
    pub down: u32,
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            toggle: 0x50,
            smooth_toggle: 0xDB,
            add_point: 0x42,
            modifier: 0x11,
            cursor: DEFAULT_CURSOR_KEY,
            delete: 0x2E,
            forward: 0x57,
            back: 0x53,
            left: 0x41,
            right: 0x44,
            up: 0x20,
            down: 0x10,
        }
    }
}

impl KeyConfig {
    // returns a copy with every key moved to a usable code
    pub fn sanitized(self) -> Self {
        if self.cursor != VK_TAB {
            return self;
        }
        warn!("keys.cursor is Tab, which the game uses; using 0x{DEFAULT_CURSOR_KEY:02X}");
        Self { cursor: DEFAULT_CURSOR_KEY, ..self }
    }
}
