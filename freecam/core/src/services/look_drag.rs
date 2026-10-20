use crate::structs::MouseCommand;

const BUTTON_RIGHT: u8 = 2;

// the right-button look drag in bar mode: the button goes down away from the UI, the mouse is grabbed and turns the
// camera like mouse-look until the button comes up, then the cursor comes back where it was. The drag lives only
// inside bar mode
pub struct LookDrag {
    // the right button is down and went down where a drag may start
    held: bool,
    // the mouse is grabbed for the drag
    active: bool,
}

impl LookDrag {
    pub fn new() -> Self {
        Self { held: false, active: false }
    }

    // a mouse button event; a right press starts a drag only when `can_start`, a right release always ends one
    pub fn on_button(&mut self, button: u8, data: i8, can_start: bool) {
        if button != BUTTON_RIGHT {
            return;
        }
        if data == 0 {
            self.held = false;
        } else if can_start {
            self.held = true;
        }
    }

    // true while the mouse is grabbed for the drag and its movement turns the camera
    pub fn is_active(&self) -> bool {
        self.active
    }

    // once per frame, after bar mode is known for the frame: the command that starts or ends the drag. Leaving bar mode
    // drops the drag with no command, because the bar mode change gives the mouse its own command
    pub fn update(&mut self, cursor_mode: bool) -> MouseCommand {
        if !cursor_mode {
            self.held = false;
            self.active = false;
            return MouseCommand::None;
        }
        match (self.held, self.active) {
            (true, false) => {
                self.active = true;
                MouseCommand::DragGrab
            }
            (false, true) => {
                self.active = false;
                MouseCommand::DragRelease
            }
            _ => MouseCommand::None,
        }
    }
}

impl Default for LookDrag {
    fn default() -> Self {
        Self::new()
    }
}
