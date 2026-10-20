use freecam_core::structs::UiEvent;

use crate::types::FcStatus;

/// One input event for the UI. `x` and `y` are physical pixels from the top left of the game window. The fields read
/// depend on `kind`:
/// - 0 pointer moved: `x`, `y`
/// - 1 pointer button: `x`, `y`, `button` (0 left, 1 right, 2 middle), `pressed`
/// - 2 wheel, in notches (one notch is 1.0, WHEEL_DELTA): `x` is horizontal, positive scrolls left
///   (-WM_MOUSEHWHEEL delta / 120); `y` is vertical, positive scrolls up (WM_MOUSEWHEEL delta / 120)
/// - 3 key: `vk` (a Windows virtual-key code), `pressed`
/// - 4 pointer gone
/// - 5 focus lost
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FcUiEvent {
    pub kind: u32,
    pub x: f32,
    pub y: f32,
    pub button: u32,
    pub pressed: u8,
    pub vk: u32,
}

impl FcUiEvent {
    /// The highest `button` value: 0 left, 1 right, 2 middle.
    pub const MAX_BUTTON: u32 = 2;
}

// an unknown kind, or a button above 2, gives InvalidArgument
impl TryFrom<&FcUiEvent> for UiEvent {
    type Error = FcStatus;

    fn try_from(value: &FcUiEvent) -> Result<Self, Self::Error> {
        let pressed = value.pressed != 0;
        match value.kind {
            0 => Ok(Self::PointerMoved { x: value.x, y: value.y }),
            1 if value.button <= FcUiEvent::MAX_BUTTON => {
                Ok(Self::PointerButton { x: value.x, y: value.y, button: value.button as u8, pressed })
            }
            2 => Ok(Self::Wheel { dx: value.x, dy: value.y }),
            3 => Ok(Self::Key { vk: value.vk, pressed }),
            4 => Ok(Self::PointerGone),
            5 => Ok(Self::FocusLost),
            _ => Err(FcStatus::InvalidArgument),
        }
    }
}
