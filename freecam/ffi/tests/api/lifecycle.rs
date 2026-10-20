use freecam_ffi::api::lifecycle::{fc_create, fc_destroy};
use freecam_ffi::types::FcStr;

#[test]
fn null_config_path_gives_a_null_handle() {
    assert!(fc_create(std::ptr::null(), None).is_null());
}

#[test]
fn non_utf8_config_path_gives_a_null_handle() {
    let bad = [0xFFu8];
    let path = FcStr { ptr: bad.as_ptr(), len: 1 };
    assert!(fc_create(&path, None).is_null());
}

#[test]
fn create_with_missing_config_works() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("absent.json");
    let text = missing.to_str().unwrap();
    let path = FcStr { ptr: text.as_ptr(), len: text.len() };
    let handle = fc_create(&path, None);
    assert!(!handle.is_null());
    fc_destroy(handle);
}

#[test]
fn destroying_a_null_handle_is_harmless() {
    fc_destroy(std::ptr::null_mut());
}
