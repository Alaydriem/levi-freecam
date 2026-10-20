use freecam_ffi::services::FcBoundary;
use freecam_ffi::types::FcStatus;

#[test]
fn panic_is_stopped_at_boundary() {
    assert_eq!(FcBoundary::guard(|| panic!("x")), FcStatus::Panicked);
    assert_eq!(FcBoundary::guard(|| FcStatus::Ok), FcStatus::Ok);
}
