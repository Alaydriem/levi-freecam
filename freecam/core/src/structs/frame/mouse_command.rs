// what the shim does with the mouse this frame
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MouseCommand {
    #[default]
    None,
    // give the cursor back to the user
    Release,
    // lock the cursor to the game again
    Grab,
    // lock the cursor to the game for a look drag in bar mode, and keep where the cursor is
    DragGrab,
    // give the cursor back after a look drag, at the place it was when the drag started
    DragRelease,
}
