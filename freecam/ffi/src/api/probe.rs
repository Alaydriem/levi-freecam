use crate::services::{FcBoundary, FcHandle};
use crate::types::FcStatus;

/// Marks a flight key-down for the key-to-move telemetry. `time_seconds` is on the clock of
/// `FcFrameStatus::time_seconds`. Call it from the thread that runs the key event; it does nothing while
/// telemetry is off.
#[unsafe(no_mangle)]
pub extern "C" fn fc_probe_key(handle: *const FcHandle, time_seconds: f64) -> FcStatus {
    FcBoundary::guard(|| {
        // SAFETY: the handle came from fc_create
        let Some(handle) = (unsafe { FcBoundary::handle(handle) }) else { return FcStatus::InvalidArgument };
        match handle.with_controller(|controller| controller.on_key_probe(time_seconds)) {
            Some(()) => FcStatus::Ok,
            None => FcStatus::Failed,
        }
    })
}
