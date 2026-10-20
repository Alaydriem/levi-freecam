/// How a [`super::TextButton`] looks and whether it takes clicks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    /// Waiting for a confirming click: drawn with an accent outline. The theme has no danger color.
    Armed,
    /// Dimmed, and it takes no clicks.
    Disabled,
}
