use serde::{Deserialize, Serialize};

// Windows virtual-key codes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct KeyConfig {
    pub toggle: u32,
    pub smooth_toggle: u32,
    pub add_point: u32,
    pub undo_point: u32,
    pub play: u32,
    // both the chord key and the fast key
    pub modifier: u32,
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
            undo_point: 0x5A,
            play: 0x0D,
            modifier: 0x11,
            forward: 0x57,
            back: 0x53,
            left: 0x41,
            right: 0x44,
            up: 0x20,
            down: 0x10,
        }
    }
}
