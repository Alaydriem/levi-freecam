use freecam_ffi::api::input::{fc_key, fc_mouse_button, fc_mouse_move, fc_server_instruction};
use freecam_ffi::types::FcStatus;

use crate::support::FfiSession;

#[test]
fn null_handle_is_invalid_argument() {
    let mut consume = 0u8;
    assert_eq!(fc_key(std::ptr::null(), 0x11, 1, &mut consume), FcStatus::InvalidArgument);
    assert_eq!(fc_mouse_move(std::ptr::null(), 1, 1), FcStatus::InvalidArgument);
    assert_eq!(fc_mouse_button(std::ptr::null(), 0, 0, &mut consume), FcStatus::InvalidArgument);
    assert_eq!(fc_server_instruction(std::ptr::null(), 1), FcStatus::InvalidArgument);
}

#[test]
fn null_out_pointers_are_invalid_argument() {
    let session = FfiSession::new();
    assert_eq!(fc_key(session.handle, 0x11, 1, std::ptr::null_mut()), FcStatus::InvalidArgument);
    assert_eq!(fc_mouse_button(session.handle, 0, 0, std::ptr::null_mut()), FcStatus::InvalidArgument);
}

#[test]
fn unknown_server_instruction_kind_is_invalid_argument() {
    let session = FfiSession::new();
    assert_eq!(fc_server_instruction(session.handle, 9), FcStatus::InvalidArgument);
    assert_eq!(fc_server_instruction(session.handle, 0), FcStatus::Ok);
    assert_eq!(fc_server_instruction(session.handle, 1), FcStatus::Ok);
}

#[test]
fn ctrl_p_chord_is_consumed_through_the_c_api() {
    let session = FfiSession::new();
    let mut consume = 0u8;
    assert_eq!(fc_key(session.handle, 0x11, 1, &mut consume), FcStatus::Ok);
    assert_eq!(fc_key(session.handle, 0x50, 1, &mut consume), FcStatus::Ok);
    assert_eq!(consume, 1);
    assert_eq!(fc_mouse_move(session.handle, 3, -2), FcStatus::Ok);
}
