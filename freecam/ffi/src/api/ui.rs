use freecam_core::structs::UiEvent;

use crate::services::{FcBoundary, FcHandle};
use crate::types::{FcStatus, FcUiEvent, FcUiFrame, FcUiFrameInput};

/// Queues one UI input event for the next fc_ui_frame. Call it from the window thread, only while cursor mode is on.
/// It never blocks: the queue is lock-free and holds `ui.eventQueueCapacity` events; when it is full the oldest event
/// is dropped and counted in telemetry. An unknown `kind`, or a `button` above 2, gives FC_STATUS_INVALID_ARGUMENT.
#[unsafe(no_mangle)]
pub extern "C" fn fc_ui_event(handle: *const FcHandle, event: *const FcUiEvent) -> FcStatus {
    FcBoundary::guard(|| {
        // SAFETY: the handle came from fc_create; event is readable for the call
        let (Some(handle), Some(event)) = (unsafe { FcBoundary::handle(handle) }, unsafe { FcBoundary::input(event) })
        else {
            return FcStatus::InvalidArgument;
        };
        let event = match UiEvent::try_from(event) {
            Ok(event) => event,
            Err(status) => return status,
        };
        handle.push_ui_event(event);
        FcStatus::Ok
    })
}

/// Runs one UI frame and fills `out`. Call it from the render thread, once per Present, while the last fc_frame said
/// `ui_visible`. It drains the queued events and draws the snapshot the last fc_frame published.
///
/// On FC_STATUS_OK every pointer in `out` is valid for its count, borrows buffers the handle owns, and stays valid
/// until the next fc_ui_frame or fc_destroy on the same handle. fc_frame, fc_ui_event and fc_ui_reset_textures do not
/// change them. A pointer with a count of 0 must not be read.
///
/// Apply `textures` before drawing and release `freed` after. When one frame changes more than one texture, the order
/// between different texture ids is not fixed. A full delta for an id the renderer already holds replaces that
/// texture. When `skipped` is 1 the frame went over `ui.maxVertices`: it has no vertices, indices or draws, but its
/// texture changes are present and must still be applied.
#[unsafe(no_mangle)]
pub extern "C" fn fc_ui_frame(handle: *const FcHandle, input: *const FcUiFrameInput, out: *mut FcUiFrame) -> FcStatus {
    FcBoundary::guard(|| {
        // SAFETY: the handle came from fc_create; input is readable and out is writable for the call
        let (Some(handle), Some(input), Some(out)) =
            (unsafe { FcBoundary::handle(handle) }, unsafe { FcBoundary::input(input) }, unsafe {
                FcBoundary::output(out)
            })
        else {
            return FcStatus::InvalidArgument;
        };
        match handle.ui_frame(input) {
            Some(frame) => {
                *out = frame;
                FcStatus::Ok
            }
            None => FcStatus::Failed,
        }
    })
}

/// Makes the next fc_ui_frame send every texture again in full, the font atlas included, with the same ids. Call it
/// from the render thread only after the renderer has released its GPU textures (device loss, or a swap-chain
/// recreate that dropped them). The renderer then treats each full delta as create-or-replace. The reset restarts
/// the UI's state: hover animations restart, and a mouse button held during the reset loses its press, so that click
/// does not complete.
#[unsafe(no_mangle)]
pub extern "C" fn fc_ui_reset_textures(handle: *const FcHandle) -> FcStatus {
    FcBoundary::guard(|| {
        // SAFETY: the handle came from fc_create
        let Some(handle) = (unsafe { FcBoundary::handle(handle) }) else { return FcStatus::InvalidArgument };
        match handle.reset_ui_textures() {
            Some(()) => FcStatus::Ok,
            None => FcStatus::Failed,
        }
    })
}
