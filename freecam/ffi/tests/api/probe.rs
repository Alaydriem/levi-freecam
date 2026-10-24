use freecam_ffi::api::probe::fc_probe_key;
use freecam_ffi::types::FcStatus;

use crate::support::FfiSession;

#[test]
fn probe_with_null_handle_returns_error() {
    assert_eq!(fc_probe_key(std::ptr::null(), 1.0), FcStatus::InvalidArgument);
}

#[test]
fn probe_on_a_live_handle_is_ok() {
    let session = FfiSession::new();
    assert_eq!(fc_probe_key(session.handle, 1.0), FcStatus::Ok);
}
