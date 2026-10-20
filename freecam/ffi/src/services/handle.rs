use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, TryLockError};
use std::time::Instant;

use freecam_core::config::FreecamConfig;
use freecam_core::services::FreecamController;
use freecam_core::structs::{FrameStatus, UiEvent};

use crate::services::{HandleState, UiEventQueue, UiHandoff, UiState};
use crate::types::{FcFrameOutput, FcMarkerEntry, FcUiFrame, FcUiFrameInput};

// the opaque handle C++ holds.
// - state: the controller, game thread (and the input exports)
// - ui: the egui runtime, render thread only
// - handoff: snapshot, actions and timings between the two, locked only to copy in or out; the game thread uses
//   try_lock on it and never waits
// - events: lock-free, bounded by ui.eventQueueCapacity, evicts the oldest event
// - pointer_over_ui: the last UI frame's wants_pointer, written by the render thread, read on a mouse button
// Lock order: state before handoff, ui before handoff; handoff is always the last lock taken
pub struct FcHandle {
    state: Mutex<HandleState>,
    ui: Mutex<UiState>,
    handoff: Mutex<UiHandoff>,
    events: UiEventQueue,
    pointer_over_ui: AtomicBool,
    measure: bool,
    // the sanitized `screenshot.folder`, unexpanded; lent to the shim until the handle is dropped
    screenshot_folder: String,
    // set once the first poisoned lock was logged, so a panic gives one log line and not one per call
    poison_logged: AtomicBool,
}

impl FcHandle {
    pub fn new(config: FreecamConfig) -> Self {
        let ui_config = config.ui.clone().sanitized();
        let measure = config.telemetry;
        let screenshot_folder = config.screenshot.clone().sanitized().folder;
        let controller = FreecamController::new(config);
        let ui = UiState::new(&ui_config, measure, controller.legend().clone());
        Self {
            state: Mutex::new(HandleState::new(controller)),
            ui: Mutex::new(ui),
            handoff: Mutex::new(UiHandoff::new()),
            events: UiEventQueue::new(ui_config.event_queue_capacity),
            pointer_over_ui: AtomicBool::new(false),
            measure,
            screenshot_folder,
            poison_logged: AtomicBool::new(false),
        }
    }

    // None when a panic poisoned the lock
    pub fn with_controller<R>(&self, body: impl FnOnce(&mut FreecamController) -> R) -> Option<R> {
        self.with_state(|state| body(state.controller_mut()))
    }

    // a mouse button or wheel event with the last UI frame's pointer ownership; true when the game must not see it.
    // None when a panic poisoned the state lock
    pub fn mouse_button(&self, button: u8, data: i8) -> Option<bool> {
        let over_ui = self.pointer_over_ui.load(Ordering::Relaxed);
        self.with_controller(|controller| {
            controller.set_pointer_over_ui(over_ui);
            controller.on_mouse_button(button, data)
        })
    }

    // game thread: applies the UI's actions, runs the core frame, then publishes the snapshot and the UI timings. Each
    // hand-off is a try_lock held for a copy only; when it is busy that side waits for the next frame. The marker view
    // in the result borrows the state's buffer until the next call. None when a panic poisoned the state lock
    pub fn frame(&self, status: &FrameStatus) -> Option<FcFrameOutput> {
        self.with_state(|state| {
            let controller = state.controller_mut();
            if let Some(mut handoff) = self.try_handoff() {
                handoff.apply_actions(controller);
            }
            let output = controller.frame(status);
            if let Some(mut handoff) = self.try_handoff() {
                handoff.publish(controller);
            }
            let mut out = FcFrameOutput::from(&output);
            (out.markers, out.marker_count) = state.publish_markers();
            out
        })
    }

    // the screenshot folder from config; the bytes live as long as the handle
    pub fn screenshot_folder(&self) -> &str {
        &self.screenshot_folder
    }

    // a remove for every live marker; the view borrows the state's shutdown buffer until the next call. None when a
    // panic poisoned the state lock
    pub fn shutdown_markers(&self) -> Option<(*const FcMarkerEntry, usize)> {
        self.with_state(HandleState::shutdown_markers)
    }

    // window thread: queues one event for the next UI frame; never blocks
    pub fn push_ui_event(&self, event: UiEvent) {
        self.events.push(event);
    }

    // render thread: runs one UI frame. cpu_ms covers the whole call, lock waits included. None when a panic poisoned a
    // lock
    pub fn ui_frame(&self, input: &FcUiFrameInput) -> Option<FcUiFrame> {
        let started = self.measure.then(Instant::now);
        let mut ui = self.lock(&self.ui, "UI")?;
        let snapshot = self.lock(&self.handoff, "UI hand-off")?.snapshot();
        ui.take_events(&self.events);
        let frame = ui.run(input, &snapshot);
        self.pointer_over_ui.store(frame.wants_pointer != 0, Ordering::Relaxed);
        let cpu_ms = started.map_or(0.0, |started| started.elapsed().as_secs_f32() * 1000.0);
        let timing = ui.timing(cpu_ms, input.gpu_ms, self.events.dropped());
        self.lock(&self.handoff, "UI hand-off")?.push_frame(ui.actions(), timing);
        Some(frame)
    }

    // None when a panic poisoned the lock
    pub fn reset_ui_textures(&self) -> Option<()> {
        self.lock(&self.ui, "UI")?.reset_textures();
        Some(())
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
            log::error!("{name} lock is poisoned by an earlier panic; calls that need it now fail");
        }
    }
}
