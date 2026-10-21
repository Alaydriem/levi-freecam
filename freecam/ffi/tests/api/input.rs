use amm_client::test_support::FakeTable;
use amm_core::structs::descriptor::Capability;
use freecam_core::structs::{FreecamAction, FreecamState};
use freecam_ffi::api::input::fc_server_instruction;
use freecam_ffi::types::FcStatus;

use crate::support::FfiSession;

// freecam on, one frame in
fn flying() -> FfiSession {
    let session = FfiSession::new();
    FfiSession::tap(FreecamAction::Toggle);
    session.frame(&FfiSession::ready_status());
    session
}

fn state(session: &FfiSession) -> FreecamState {
    session.handle().with_controller(|controller| controller.state()).unwrap()
}

#[test]
fn null_handle_is_invalid_argument() {
    assert_eq!(fc_server_instruction(std::ptr::null(), 1), FcStatus::InvalidArgument);
}

#[test]
fn unknown_server_instruction_kind_is_invalid_argument() {
    let session = FfiSession::new();
    assert_eq!(fc_server_instruction(session.handle, 9), FcStatus::InvalidArgument);
    assert_eq!(fc_server_instruction(session.handle, 0), FcStatus::Ok);
    assert_eq!(fc_server_instruction(session.handle, 1), FcStatus::Ok);
}

#[test]
fn server_set_exits_while_no_other_mod_holds_camera() {
    let session = flying();

    assert_eq!(fc_server_instruction(session.handle, 1), FcStatus::Ok);
    session.frame(&FfiSession::status_at(0.10, 1));

    assert_eq!(state(&session), FreecamState::Off);
}

#[test]
fn server_set_ignored_while_other_mod_holds_camera() {
    let session = flying();
    FakeTable::set_held_by_other(Capability::Camera, true);

    assert_eq!(fc_server_instruction(session.handle, 1), FcStatus::Ok);
    session.frame(&FfiSession::status_at(0.10, 1));

    assert_eq!(state(&session), FreecamState::On);
}
