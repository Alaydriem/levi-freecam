// one input event for the UI; coordinates are physical pixels from the top left of the game window
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UiEvent {
    PointerMoved { x: f32, y: f32 },
    // button 0 is left, 1 is right, 2 is middle
    PointerButton { x: f32, y: f32, button: u8, pressed: bool },
    // wheel notches, one notch is 1.0 (WHEEL_DELTA). Positive dy scrolls up (WM_MOUSEWHEEL delta / 120); positive
    // dx scrolls left, the content moves right (-WM_MOUSEHWHEEL delta / 120)
    Wheel { dx: f32, dy: f32 },
    // a Windows virtual-key code
    Key { vk: u32, pressed: bool },
    PointerGone,
    FocusLost,
}
