use freecam_core::structs::ServerInstruction;

use crate::services::{FcBoundary, FcHandle};
use crate::types::FcStatus;

// consume is set to 1 when the game must not see the key
#[unsafe(no_mangle)]
pub extern "C" fn fc_key(handle: *const FcHandle, code: u32, down: u8, consume: *mut u8) -> FcStatus {
    FcBoundary::guard(|| {
        // SAFETY: the handle came from fc_create; consume is writable for the call
        let (Some(handle), Some(consume)) =
            (unsafe { FcBoundary::handle(handle) }, unsafe { FcBoundary::output(consume) })
        else {
            return FcStatus::InvalidArgument;
        };
        match handle.with_controller(|controller| controller.on_key(code, down != 0)) {
            Some(consumed) => {
                *consume = u8::from(consumed);
                FcStatus::Ok
            }
            None => FcStatus::Failed,
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn fc_mouse_move(handle: *const FcHandle, dx: i32, dy: i32) -> FcStatus {
    FcBoundary::guard(|| {
        // SAFETY: the handle came from fc_create
        let Some(handle) = (unsafe { FcBoundary::handle(handle) }) else { return FcStatus::InvalidArgument };
        match handle.with_controller(|controller| controller.on_mouse_move(dx, dy)) {
            Some(()) => FcStatus::Ok,
            None => FcStatus::Failed,
        }
    })
}

// consume is set to 1 when the game must not see the mouse button or wheel event
#[unsafe(no_mangle)]
pub extern "C" fn fc_mouse_button(handle: *const FcHandle, button: u8, data: i8, consume: *mut u8) -> FcStatus {
    FcBoundary::guard(|| {
        // SAFETY: the handle came from fc_create; consume is writable for the call
        let (Some(handle), Some(consume)) =
            (unsafe { FcBoundary::handle(handle) }, unsafe { FcBoundary::output(consume) })
        else {
            return FcStatus::InvalidArgument;
        };
        match handle.mouse_button(button, data) {
            Some(consumed) => {
                *consume = u8::from(consumed);
                FcStatus::Ok
            }
            None => FcStatus::Failed,
        }
    })
}

// kind: 0 other, 1 set or clear
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
        match handle.with_controller(|controller| controller.on_server_instruction(instruction)) {
            Some(()) => FcStatus::Ok,
            None => FcStatus::Failed,
        }
    })
}
