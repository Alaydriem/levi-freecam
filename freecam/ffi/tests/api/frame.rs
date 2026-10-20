use freecam_ffi::api::frame::fc_frame;
use freecam_ffi::api::input::fc_key;
use freecam_ffi::types::{FcFrameOutput, FcStatus};

use crate::support::FfiSession;

#[test]
fn null_handle_is_invalid_argument() {
    let status = FfiSession::ready_status();
    let mut out = FcFrameOutput::default();
    assert_eq!(fc_frame(std::ptr::null(), &status, &mut out), FcStatus::InvalidArgument);
}

#[test]
fn null_out_pointers_are_invalid_argument() {
    let session = FfiSession::new();
    let status = FfiSession::ready_status();
    let mut out = FcFrameOutput::default();
    assert_eq!(fc_frame(session.handle, std::ptr::null(), &mut out), FcStatus::InvalidArgument);
    assert_eq!(fc_frame(session.handle, &status, std::ptr::null_mut()), FcStatus::InvalidArgument);
}

#[test]
fn ctrl_p_then_frame_outputs_set() {
    let session = FfiSession::new();
    let mut consume = 0u8;
    assert_eq!(fc_key(session.handle, 0x11, 1, &mut consume), FcStatus::Ok);
    assert_eq!(fc_key(session.handle, 0x50, 1, &mut consume), FcStatus::Ok);
    let status = FfiSession::ready_status();
    let (code, out) = session.frame(&status);
    assert_eq!(code, FcStatus::Ok);
    assert_eq!(out.camera, 1);
    assert_eq!(out.gui, 1);
    assert_eq!(out.block_input, 1);
    assert_eq!(out.pose, status.start_pose);
}
