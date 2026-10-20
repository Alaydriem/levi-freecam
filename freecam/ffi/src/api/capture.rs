use crate::services::{FcBoundary, FcHandle};
use crate::types::{FcStatus, FcStr};

/// Answers the capture that an fc_frame asked for with `capture_active`, as soon as the frame is copied. Call it from
/// the game thread once for every request, with that frame's `capture_sequence`: `ok` is 1 when the frame was copied
/// for the PNG writer, 0 when the capture failed or was refused (a write still pending). The mod's drawing comes back
/// after this call; ok = 0 shows the failed toast. An answer more than 10 presents after the request, or with a wrong
/// sequence, changes nothing: the request has timed out with the failed toast. After ok = 1, answer the same sequence
/// once more with fc_capture_written.
#[unsafe(no_mangle)]
pub extern "C" fn fc_capture_taken(handle: *mut FcHandle, sequence: u32, ok: u8) -> FcStatus {
    FcBoundary::guard(|| {
        // SAFETY: the handle came from fc_create and is not destroyed
        let Some(handle) = (unsafe { FcBoundary::handle(handle) }) else { return FcStatus::InvalidArgument };
        match handle.with_controller(|controller| controller.on_capture_taken(sequence, ok != 0)) {
            Some(()) => FcStatus::Ok,
            None => FcStatus::Failed,
        }
    })
}

/// Answers a capture that fc_capture_taken reported with ok = 1, once its PNG is written. Call it from the game thread
/// once for that sequence: `ok` is 1 when the file was saved, 0 when the write failed. `name_ptr` and `name_len` are
/// the UTF-8 file name without the folder, not null-terminated and borrowed for the call; they are ignored when `ok`
/// is 0 and may be null with a length of 0. There is no frame timeout, and no new screenshot starts until this answer
/// arrives. An answer with no taken capture waiting for it changes nothing. A null handle, or a name that is not UTF-8,
/// gives FC_STATUS_INVALID_ARGUMENT and reaches nothing.
#[unsafe(no_mangle)]
pub extern "C" fn fc_capture_written(
    handle: *mut FcHandle,
    sequence: u32,
    ok: u8,
    name_ptr: *const u8,
    name_len: usize,
) -> FcStatus {
    FcBoundary::guard(|| {
        // SAFETY: the handle came from fc_create and is not destroyed
        let Some(handle) = (unsafe { FcBoundary::handle(handle) }) else { return FcStatus::InvalidArgument };
        // SAFETY: the shim passes name_len readable bytes it owns for the call, or a null pointer with a length of 0
        let Some(name) = (unsafe { FcBoundary::text(&FcStr { ptr: name_ptr, len: name_len }) }) else {
            return FcStatus::InvalidArgument;
        };
        match handle.with_controller(|controller| controller.on_capture_written(sequence, ok != 0, &name)) {
            Some(()) => FcStatus::Ok,
            None => FcStatus::Failed,
        }
    })
}

/// Writes the screenshot folder from `screenshot.folder` in config into `out`, as UTF-8 that is not expanded:
/// environment variables such as `%USERPROFILE%` are still in it, and the shim expands them and creates the folder.
/// A blank value in config gives the default `%USERPROFILE%\Pictures\Screenshots`. On FC_STATUS_OK `out` borrows bytes
/// the handle owns; they are not null-terminated and stay valid and unchanged until fc_destroy.
#[unsafe(no_mangle)]
pub extern "C" fn fc_screenshot_folder(handle: *const FcHandle, out: *mut FcStr) -> FcStatus {
    FcBoundary::guard(|| {
        // SAFETY: the handle came from fc_create; out is writable for the call
        let (Some(handle), Some(out)) = (unsafe { FcBoundary::handle(handle) }, unsafe { FcBoundary::output(out) })
        else {
            return FcStatus::InvalidArgument;
        };
        let folder = handle.screenshot_folder();
        *out = FcStr { ptr: folder.as_ptr(), len: folder.len() };
        FcStatus::Ok
    })
}
