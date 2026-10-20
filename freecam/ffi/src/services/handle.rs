use std::sync::Mutex;

use freecam_core::services::FreecamController;

use crate::services::HandleState;
use crate::types::FcOverlayView;

// the opaque handle C++ holds
pub struct FcHandle {
    state: Mutex<HandleState>,
}

impl FcHandle {
    pub fn new(controller: FreecamController) -> Self {
        Self { state: Mutex::new(HandleState::new(controller)) }
    }

    // None when a panic poisoned the lock
    pub fn with_controller<R>(&self, body: impl FnOnce(&mut FreecamController) -> R) -> Option<R> {
        self.with_state(|state| body(state.controller_mut()))
    }

    // None when a panic poisoned the lock
    pub fn refresh_overlay(&self) -> Option<FcOverlayView> {
        self.with_state(HandleState::refresh_overlay)
    }

    fn with_state<R>(&self, body: impl FnOnce(&mut HandleState) -> R) -> Option<R> {
        match self.state.lock() {
            Ok(mut state) => Some(body(&mut state)),
            Err(_) => {
                log::error!("freecam controller lock is poisoned");
                None
            }
        }
    }
}
