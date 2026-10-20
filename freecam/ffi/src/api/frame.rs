use freecam_core::structs::FrameStatus;

use crate::services::{FcBoundary, FcHandle};
use crate::types::{FcFrameOutput, FcFrameStatus, FcStatus};

/// Writes what the shim does this frame into `out`. Call it from the game thread. It never waits on the UI: it takes
/// the UI actions before the core frame and publishes the UI snapshot after it, each through a hand-off that is held
/// only for a copy. When the render thread holds the hand-off at that moment, that swap moves to the next frame.
///
/// On FC_STATUS_OK, `out->markers` points to `out->marker_count` entries the handle owns. They stay valid until the
/// next fc_frame or fc_destroy on the same handle; the other exports do not change them. Deliver them in order, then
/// report the time it took in the next frame's `last_shape_send_us`. Skip the batch while no ClientNetworkHandler is
/// captured: core sends nothing while `handler_generation` is 0.
#[unsafe(no_mangle)]
pub extern "C" fn fc_frame(handle: *const FcHandle, status: *const FcFrameStatus, out: *mut FcFrameOutput) -> FcStatus {
    FcBoundary::guard(|| {
        // SAFETY: the handle came from fc_create; status is readable and out is writable for the call
        let (Some(handle), Some(status), Some(out)) =
            (unsafe { FcBoundary::handle(handle) }, unsafe { FcBoundary::input(status) }, unsafe {
                FcBoundary::output(out)
            })
        else {
            return FcStatus::InvalidArgument;
        };
        let status = FrameStatus::from(status);
        match handle.frame(&status) {
            Some(output) => {
                *out = output;
                FcStatus::Ok
            }
            None => FcStatus::Failed,
        }
    })
}
