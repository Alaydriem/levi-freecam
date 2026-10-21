use amm_client::test_support::FakeTable;
use amm_core::structs::event::ModEvent;
use freecam_core::structs::FreecamAction;
use freecam_ffi::api::lifecycle::{fc_create, fc_destroy};
use freecam_ffi::types::FcStr;

use crate::support::FfiSession;

#[test]
fn null_config_path_gives_a_null_handle() {
    assert!(fc_create(std::ptr::null()).is_null());
}

#[test]
fn non_utf8_config_path_gives_a_null_handle() {
    let bad = [0xFFu8];
    let path = FcStr { ptr: bad.as_ptr(), len: 1 };
    assert!(fc_create(&path).is_null());
}

#[test]
fn create_without_manager_returns_null() {
    // no alaydriem-mod-manager.dll is loaded in the test process
    let text = FakeTable::CONFIG_DIR;
    let path = FcStr { ptr: text.as_ptr(), len: text.len() };

    assert!(fc_create(&path).is_null());
}

#[test]
fn create_registers_the_freecam_descriptor() {
    let _session = FfiSession::new();

    let descriptor = FakeTable::with(|state| state.descriptor.clone()).unwrap();
    assert!(descriptor.contains("id                = \"alaydriem-freecam\""), "{descriptor}");
    assert_eq!(FakeTable::with(|state| state.config_dir.clone()).as_deref(), Some(FakeTable::CONFIG_DIR));
}

#[test]
fn create_reads_config_from_the_link() {
    let session = FfiSession::with_config("motion {\n  start_speed = 12.0\n}\n");

    let speed = session.handle().with_controller(|controller| controller.ui_snapshot().speed).unwrap();

    assert_eq!(speed, 12.0);
}

#[test]
fn settings_changed_rereads_config() {
    let session = FfiSession::new();
    FfiSession::tap(FreecamAction::Toggle);
    session.frame(&FfiSession::status_at(0.05, 1));
    FakeTable::set_config("keys {\n  toggle = \"Ctrl+Shift+F10\"\n}\n");
    FakeTable::push_event(ModEvent::SettingsChanged { paths: 1 });

    session.frame(&FfiSession::status_at(0.10, 1));

    let legend = session.handle().with_controller(|controller| controller.legend().clone()).unwrap();
    assert!(legend.rows.iter().any(|row| row.key == "Ctrl+Shift+F10"), "{legend:?}");
}

#[test]
fn destroy_unregisters() {
    let session = FfiSession::new();

    drop(session);

    assert_eq!(FakeTable::with(|state| state.unregistered.len()), 1);
}

#[test]
fn destroying_a_null_handle_is_harmless() {
    fc_destroy(std::ptr::null_mut());
}
