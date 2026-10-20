use crate::services::{FcBoundary, FcHandle};
use crate::types::{FcOverlayView, FcStatus};

/// Fills `out` with the path geometry to draw this frame. On Ok the pointers in `out` are non-null, are
/// valid for `segment_count` and `marker_count` elements, borrow buffers the handle owns, and stay valid
/// until the next fc_overlay or fc_destroy call on the same handle.
#[unsafe(no_mangle)]
pub extern "C" fn fc_overlay(handle: *const FcHandle, out: *mut FcOverlayView) -> FcStatus {
    FcBoundary::guard(|| {
        // SAFETY: the handle came from fc_create and out is writable for the call
        let (Some(handle), Some(out)) = (unsafe { FcBoundary::handle(handle) }, unsafe { FcBoundary::output(out) })
        else {
            return FcStatus::InvalidArgument;
        };
        match handle.refresh_overlay() {
            Some(view) => {
                *out = view;
                FcStatus::Ok
            }
            None => FcStatus::Failed,
        }
    })
}
