use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};

use amm_client::link::{ApiTable, ManagerLink};
use amm_core::errors::LinkError;
use freecam_core::services::FreecamController;
use freecam_core::structs::{FrameStatus, ServerInstruction};

use crate::services::{
    FreecamDescriptor, FreecamFrameHandler, HandleState, LinkSession, RenderClock, ScreenshotPath, ScreenshotRelay,
    UiHandoff,
};
use crate::types::{FcFrameOutput, FcMarkerEntry};

// the opaque handle C++ holds.
// - state: the controller and the mod manager link, game thread only
// - handoff: snapshot, legend, actions and timings between the game thread and the manager's render-thread frame
//   handler, locked only to copy in or out; the game thread uses try_lock on it and never waits
// - clock: what the frame handler saw last: frames drawn, screen size, pointer over the UI
// Lock order: state before handoff; handoff is always the last lock taken
pub struct FcHandle {
    state: Mutex<HandleState>,
    handoff: Arc<Mutex<UiHandoff>>,
    clock: Arc<RenderClock>,
    // set once the first poisoned lock was logged, so a panic gives one log line and not one per call
    poison_logged: AtomicBool,
}

impl FcHandle {
    // registers with the mod manager behind `table` and reads the config it keeps in `config_dir`; game thread, from
    // the mod's enable, never from load
    pub fn create(table: ApiTable, config_dir: &Path) -> Result<FcHandle, LinkError> {
        let handoff = Arc::new(Mutex::new(UiHandoff::new()));
        let clock = Arc::new(RenderClock::new());
        let handler = FreecamFrameHandler::new(Arc::clone(&handoff), Arc::clone(&clock));
        let link = ManagerLink::new(table, FreecamDescriptor::DESCRIPTOR, config_dir, handler)?;
        let session = LinkSession::new(link);
        let config = session.config();
        clock.set_measure(config.telemetry);
        let folder = config.screenshot.clone().sanitized().folder;
        let relay = ScreenshotRelay::new(ScreenshotPath::new(&folder, |name| std::env::var(name).ok()));
        let mut controller = FreecamController::new(config);
        controller.set_can_screenshot(session.can_screenshot());
        curia::info!("freecam registered with the mod manager", { screenshots: session.can_screenshot() });
        Ok(Self {
            state: Mutex::new(HandleState::new(controller, session, relay)),
            handoff,
            clock,
            poison_logged: AtomicBool::new(false),
        })
    }

    // None when a panic poisoned the lock
    pub fn with_controller<R>(&self, body: impl FnOnce(&mut FreecamController) -> R) -> Option<R> {
        self.with_state(|state| body(state.controller_mut()))
    }

    // a camera instruction reached the client; it is the server's only while no other mod holds the Camera lease.
    // None when a panic poisoned the state lock
    pub fn server_instruction(&self, kind: ServerInstruction) -> Option<()> {
        self.with_state(|state| state.server_instruction(kind))
    }

    // game thread: drains the manager's events into core, answers a start with the flight leases, runs the core frame,
    // then applies the cursor and gives the leases back once freecam is off. The UI hand-off is a try_lock held for a
    // copy only; when it is busy that side waits for the next frame. The marker view in the result borrows the
    // state's buffer until the next call. None when a panic poisoned the state lock
    pub fn frame(&self, status: &FrameStatus) -> Option<FcFrameOutput> {
        let mut status = *status;
        status.present_count = self.clock.frames();
        if !(status.aspect.is_finite() && status.aspect > 0.0)
            && let Some(aspect) = self.clock.aspect()
        {
            status.aspect = aspect;
        }
        let over_ui = self.clock.pointer_over_ui();
        self.with_state(|state| {
            state.controller_mut().set_pointer_over_ui(over_ui);
            state.apply_events();
            state.start_if_asked();
            if let Some(mut handoff) = self.try_handoff() {
                handoff.apply_actions(state.controller_mut());
            }
            let mut status = status;
            state.complete_status(&mut status);
            let output = state.controller_mut().frame(&status);
            if let Some(mut handoff) = self.try_handoff() {
                handoff.publish(state.controller_mut());
            }
            state.apply_output(&output);
            let mut out = FcFrameOutput::from(&output);
            (out.markers, out.marker_count) = state.publish_markers();
            out
        })
    }

    // a remove for every live marker; the view borrows the state's shutdown buffer until the next call. None when a
    // panic poisoned the state lock
    pub fn shutdown_markers(&self) -> Option<(*const FcMarkerEntry, usize)> {
        self.with_state(HandleState::shutdown_markers)
    }

    fn with_state<R>(&self, body: impl FnOnce(&mut HandleState) -> R) -> Option<R> {
        let mut state = self.lock(&self.state, "freecam controller")?;
        Some(body(&mut state))
    }

    fn try_handoff(&self) -> Option<MutexGuard<'_, UiHandoff>> {
        match self.handoff.try_lock() {
            Ok(guard) => Some(guard),
            Err(TryLockError::WouldBlock) => None,
            Err(TryLockError::Poisoned(_)) => {
                self.log_poison("UI hand-off");
                None
            }
        }
    }

    fn lock<'a, T>(&self, mutex: &'a Mutex<T>, name: &str) -> Option<MutexGuard<'a, T>> {
        match mutex.lock() {
            Ok(guard) => Some(guard),
            Err(_) => {
                self.log_poison(name);
                None
            }
        }
    }

    fn log_poison(&self, name: &str) {
        if !self.poison_logged.swap(true, Ordering::Relaxed) {
            curia::error!("lock is poisoned by an earlier panic; calls that need it now fail", { lock: name });
        }
    }
}
