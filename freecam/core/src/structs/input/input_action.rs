#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    ToggleFreecam,
    ToggleSmooth,
    AddPoint,
    UndoPoint,
    TogglePlayback,
}
