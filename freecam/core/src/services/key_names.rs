const VK_ZERO: u32 = 0x30;
const VK_NINE: u32 = 0x39;
const VK_A: u32 = 0x41;
const VK_Z: u32 = 0x5A;
const VK_F1: u32 = 0x70;
const VK_F24: u32 = 0x87;

// display names for Windows virtual-key codes
pub struct KeyNames;

impl KeyNames {
    // allocates, so call it at config load and never per frame
    pub fn name(vk: u32) -> String {
        if let Some(name) = Self::fixed(vk) {
            return name.to_owned();
        }
        match vk {
            VK_ZERO..=VK_NINE | VK_A..=VK_Z => char::from_u32(vk).map_or_else(|| Self::hex(vk), |c| c.to_string()),
            VK_F1..=VK_F24 => format!("F{}", vk - VK_F1 + 1),
            _ => Self::hex(vk),
        }
    }

    fn fixed(vk: u32) -> Option<&'static str> {
        Some(match vk {
            0x08 => "Backspace",
            0x09 => "Tab",
            0x0D => "Enter",
            0x10 => "Shift",
            0x11 => "Ctrl",
            0x12 => "Alt",
            0x1B => "Esc",
            0x20 => "Space",
            0x25 => "Left",
            0x26 => "Up",
            0x27 => "Right",
            0x28 => "Down",
            0x2E => "Del",
            0xA0 => "Left Shift",
            0xA1 => "Right Shift",
            0xA2 => "Left Ctrl",
            0xA3 => "Right Ctrl",
            0xA4 => "Left Alt",
            0xA5 => "Right Alt",
            _ => return None,
        })
    }

    fn hex(vk: u32) -> String {
        format!("Key 0x{vk:02X}")
    }
}
