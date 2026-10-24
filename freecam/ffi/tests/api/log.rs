use amm_client::test_support::FakeTable;
use freecam_ffi::api::log::fc_log;
use freecam_ffi::types::FcStatus;

use crate::support::FfiSession;

#[test]
fn log_with_null_message_returns_error() {
    assert_eq!(fc_log(2, std::ptr::null(), 4, std::ptr::null(), 0, 1), FcStatus::InvalidArgument);
}

#[test]
fn log_with_an_unknown_level_returns_error() {
    let message = "hello";
    assert_eq!(fc_log(9, message.as_ptr(), message.len(), std::ptr::null(), 0, 1), FcStatus::InvalidArgument);
}

#[test]
fn log_reaches_the_manager_after_create() {
    let _session = FfiSession::new();
    let message = "shim says hello";
    let file = "FreecamMod.cpp";

    assert_eq!(fc_log(1, message.as_ptr(), message.len(), file.as_ptr(), file.len(), 42), FcStatus::Ok);

    let logs = FakeTable::with(|state| state.logs.clone());
    let line = logs.iter().find(|call| call.message == message).unwrap();
    assert_eq!((line.file.as_str(), line.line), (file, 42));
}
