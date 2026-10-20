use freecam_core::services::FreecamController;
use freecam_core::structs::{BoundedQueue, UiAction, UiSnapshot, UiTiming, UiTimingSummary};

// what the game thread and the render thread pass each other: the snapshot one way, the actions and the frame
// timings the other. Each side locks it only to copy in or out, never across its own work. `actions` holds
// ACTION_CAPACITY and evicts the oldest
pub struct UiHandoff {
    snapshot: UiSnapshot,
    actions: BoundedQueue<UiAction>,
    timing: UiTimingSummary,
}

impl UiHandoff {
    pub const ACTION_CAPACITY: usize = 16;

    pub fn new() -> Self {
        Self {
            snapshot: UiSnapshot::default(),
            actions: BoundedQueue::new(Self::ACTION_CAPACITY),
            timing: UiTimingSummary::new(),
        }
    }

    // render thread, before the UI frame
    pub fn snapshot(&self) -> UiSnapshot {
        self.snapshot
    }

    // render thread, after the UI frame
    pub fn push_frame(&mut self, actions: &[UiAction], timing: UiTiming) {
        for action in actions {
            self.actions.push(*action);
        }
        self.timing.add(timing);
    }

    // game thread, before the core frame
    pub fn apply_actions(&mut self, controller: &mut FreecamController) {
        while let Some(action) = self.actions.pop() {
            controller.apply_ui_action(action);
        }
    }

    // game thread, after the core frame: stores the new snapshot and hands core every UI frame timed since the last
    // call
    pub fn publish(&mut self, controller: &mut FreecamController) {
        self.snapshot = controller.ui_snapshot();
        let timing = std::mem::take(&mut self.timing);
        controller.record_ui_summary(&timing);
    }
}

impl Default for UiHandoff {
    fn default() -> Self {
        Self::new()
    }
}
