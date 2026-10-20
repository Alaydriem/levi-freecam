use freecam_core::structs::FrameStatus;

use crate::services::{FcBoundary, FcHandle};
use crate::types::{FcFrameOutput, FcFrameStatus, FcStatus};

// writes what the shim does this frame into out
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
        match handle.with_controller(|controller| controller.frame(&status)) {
            Some(output) => {
                *out = FcFrameOutput::from(&output);
                FcStatus::Ok
            }
            None => FcStatus::Failed,
        }
    })
}
