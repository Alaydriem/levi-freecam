use freecam_core::structs::Vec3;

use crate::services::{FcBoundary, FcHandle};
use crate::types::{FcMarkerEntry, FcStatus};

/// Answers the ray that the last fc_frame asked for in `ray`. Call it from the game thread after that fc_frame and
/// before the second next one; a later answer, or an answer with no ray waiting, changes nothing. `hit` is 1 when the
/// ray hit a block at (x, y, z), 0 when it hit nothing (the position is then ignored).
#[unsafe(no_mangle)]
pub extern "C" fn fc_ray_result(handle: *mut FcHandle, hit: u8, x: f32, y: f32, z: f32) -> FcStatus {
    FcBoundary::guard(|| {
        // SAFETY: the handle came from fc_create and is not destroyed
        let Some(handle) = (unsafe { FcBoundary::handle(handle) }) else { return FcStatus::InvalidArgument };
        let hit = (hit != 0).then(|| Vec3::new(x, y, z));
        match handle.with_controller(|controller| controller.on_ray_result(hit)) {
            Some(()) => FcStatus::Ok,
            None => FcStatus::Failed,
        }
    })
}

/// Writes a remove for every marker still in the world into `out` and `count`, ignoring `markers.maxPerSend`. Call it
/// from the game thread before fc_destroy, while the ClientNetworkHandler is still captured, and deliver every entry.
/// Afterwards core holds no live marker. On FC_STATUS_OK `*out` points to `*count` entries the handle owns (null when
/// `*count` is 0). They stay valid until the next fc_markers_shutdown or fc_destroy on the same handle; fc_frame does
/// not change them.
#[unsafe(no_mangle)]
pub extern "C" fn fc_markers_shutdown(
    handle: *mut FcHandle,
    out: *mut *const FcMarkerEntry,
    count: *mut usize,
) -> FcStatus {
    FcBoundary::guard(|| {
        // SAFETY: the handle came from fc_create; out and count are writable for the call
        let (Some(handle), Some(out), Some(count)) =
            (unsafe { FcBoundary::handle(handle) }, unsafe { FcBoundary::output(out) }, unsafe {
                FcBoundary::output(count)
            })
        else {
            return FcStatus::InvalidArgument;
        };
        match handle.shutdown_markers() {
            Some((entries, len)) => {
                *out = entries;
                *count = len;
                FcStatus::Ok
            }
            None => FcStatus::Failed,
        }
    })
}
