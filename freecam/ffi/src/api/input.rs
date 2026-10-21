use freecam_core::structs::ServerInstruction;

use crate::services::{FcBoundary, FcHandle};
use crate::types::FcStatus;

/// A camera instruction reached the client. kind: 0 other, 1 set or clear. While another mod holds the Camera lease
/// it is that mod's, not the server's, and changes nothing. Call it from the game thread.
#[unsafe(no_mangle)]
pub extern "C" fn fc_server_instruction(handle: *const FcHandle, kind: u8) -> FcStatus {
    FcBoundary::guard(|| {
        // SAFETY: the handle came from fc_create
        let Some(handle) = (unsafe { FcBoundary::handle(handle) }) else { return FcStatus::InvalidArgument };
        let instruction = match kind {
            0 => ServerInstruction::Other,
            1 => ServerInstruction::SetOrClear,
            _ => return FcStatus::InvalidArgument,
        };
        match handle.server_instruction(instruction) {
            Some(()) => FcStatus::Ok,
            None => FcStatus::Failed,
        }
    })
}
