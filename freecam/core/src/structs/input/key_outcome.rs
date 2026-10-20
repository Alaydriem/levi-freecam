use super::input_action::InputAction;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KeyOutcome {
    // true when the game must not see the event
    pub consume: bool,
    pub action: Option<InputAction>,
}
