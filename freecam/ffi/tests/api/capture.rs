use egui::{Pos2, Rect, vec2};
use freecam_core::structs::ToastKind;
use freecam_ffi::api::capture::{fc_capture_taken, fc_capture_written, fc_screenshot_folder};
use freecam_ffi::api::ui::{fc_ui_event, fc_ui_frame};
use freecam_ffi::types::{FcFrameOutput, FcFrameStatus, FcStatus, FcStr, FcUiEvent, FcUiFrame, FcUiFrameInput};
use freecam_ui::views::BarLayout;

use crate::support::FfiSession;

const MOVE: u32 = 0;
const BUTTON: u32 = 1;
const MAX_FRAMES: usize = 60;

fn ui_frame(session: &FfiSession, time_seconds: f64) {
    let input = FcUiFrameInput { width: 1920.0, height: 1080.0, pixels_per_point: 1.0, time_seconds, gpu_ms: -1.0 };
    let mut out = FcUiFrame::default();
    assert_eq!(fc_ui_frame(session.handle, &input, &mut out), FcStatus::Ok);
}

fn click(session: &FfiSession, at: Pos2) {
    for (kind, pressed) in [(MOVE, 0), (BUTTON, 1), (BUTTON, 0)] {
        let event = FcUiEvent { kind, x: at.x, y: at.y, button: 0, pressed, vk: 0 };
        assert_eq!(fc_ui_event(session.handle, &event), FcStatus::Ok);
    }
}

fn read(value: FcStr) -> String {
    assert!(!value.ptr.is_null());
    // SAFETY: fc_screenshot_folder returned this view on a live handle; it stays valid until fc_destroy
    let bytes = unsafe { std::slice::from_raw_parts(value.ptr, value.len) };
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn folder(session: &FfiSession) -> FcStr {
    let mut out = FcStr { ptr: std::ptr::null(), len: 0 };
    assert_eq!(fc_screenshot_folder(session.handle, &mut out), FcStatus::Ok);
    out
}

fn written(session: &FfiSession, sequence: u32, ok: u8, name: &str) -> FcStatus {
    fc_capture_written(session.handle, sequence, ok, name.as_ptr(), name.len())
}

// freecam on, bar mode on, the screenshot button clicked; returns the first frame that asks for a capture and the
// status it ran with. Every game frame is a new present
fn request_capture(session: &FfiSession) -> (FcFrameOutput, FcFrameStatus) {
    let mut status = FfiSession::ready_status();
    session.chord(FfiSession::P);
    session.frame(&status);
    session.chord(FfiSession::E);
    status.time_seconds += 0.05;
    session.frame(&status);
    ui_frame(session, 0.1);
    click(session, BarLayout::new(Rect::from_min_size(Pos2::ZERO, vec2(1920.0, 1080.0))).shot.center());
    ui_frame(session, 0.2);
    for _ in 0..MAX_FRAMES {
        status.time_seconds += 0.05;
        status.present_count += 1;
        let (code, out) = session.frame(&status);
        assert_eq!(code, FcStatus::Ok);
        if out.capture_active == 1 {
            return (out, status);
        }
    }
    panic!("no capture request after {MAX_FRAMES} frames");
}

#[test]
fn capture_request_crosses() {
    let session = FfiSession::new();

    let (out, mut status) = request_capture(&session);
    assert_eq!(out.hide_ui, 1);
    assert_eq!(out.ui_visible, 1);
    assert_eq!(fc_capture_taken(session.handle, out.capture_sequence, 1), FcStatus::Ok);
    status.time_seconds += 0.05;
    status.present_count += 1;
    let (_, after) = session.frame(&status);
    let toast_after_taken = session.handle().with_controller(|controller| controller.ui_snapshot().toast).unwrap();
    let name = "freecam-20261020-101010.png";
    assert_eq!(written(&session, out.capture_sequence, 1, name), FcStatus::Ok);

    assert_eq!(after.capture_active, 0);
    assert_eq!(after.hide_ui, 0);
    assert_eq!(toast_after_taken, ToastKind::None);
    let snapshot = session.handle().with_controller(|controller| controller.ui_snapshot()).unwrap();
    assert_eq!(snapshot.toast, ToastKind::Saved);
    assert_eq!(&snapshot.toast_file[..usize::from(snapshot.toast_file_len)], name.as_bytes());
}

#[test]
fn refused_capture_shows_the_failed_toast() {
    let session = FfiSession::new();
    let (out, _) = request_capture(&session);

    assert_eq!(fc_capture_taken(session.handle, out.capture_sequence, 0), FcStatus::Ok);

    let toast = session.handle().with_controller(|controller| controller.ui_snapshot().toast).unwrap();
    assert_eq!(toast, ToastKind::Failed);
}

#[test]
fn capture_result_null_handle_is_error() {
    let session = FfiSession::new();
    let broken = [0xFF_u8, 0xFE];

    assert_eq!(fc_capture_taken(std::ptr::null_mut(), 1, 1), FcStatus::InvalidArgument);
    assert_eq!(fc_capture_written(std::ptr::null_mut(), 1, 1, b"a.png".as_ptr(), 5), FcStatus::InvalidArgument);
    assert_eq!(fc_capture_written(session.handle, 1, 1, broken.as_ptr(), broken.len()), FcStatus::InvalidArgument);
    assert_eq!(fc_capture_written(session.handle, 1, 1, std::ptr::null(), 4), FcStatus::InvalidArgument);
    assert_eq!(fc_capture_written(session.handle, 1, 0, std::ptr::null(), 0), FcStatus::Ok);
    assert_eq!(fc_capture_taken(session.handle, 1, 1), FcStatus::Ok);
}

#[test]
fn screenshot_folder_is_borrowed_until_destroy() {
    let session = FfiSession::with_config(r#"{"screenshot":{"folder":"D:\\Shots\\Freecam"}}"#);
    let first = folder(&session);

    let mut status = FfiSession::ready_status();
    session.chord(FfiSession::P);
    for _ in 0..3 {
        status.time_seconds += 0.05;
        session.frame(&status);
    }
    ui_frame(&session, 0.1);
    let second = folder(&session);

    assert_eq!(read(first), r"D:\Shots\Freecam");
    assert_eq!(second.ptr, first.ptr);
    assert_eq!(second.len, first.len);
}

#[test]
fn screenshot_folder_defaults_to_pictures_screenshots() {
    let session = FfiSession::with_config(r#"{"screenshot":{"folder":"  "}}"#);
    let mut out = FcStr { ptr: std::ptr::null(), len: 0 };

    assert_eq!(read(folder(&session)), r"%USERPROFILE%\Pictures\Screenshots");
    assert_eq!(fc_screenshot_folder(std::ptr::null(), &mut out), FcStatus::InvalidArgument);
    assert_eq!(fc_screenshot_folder(session.handle, std::ptr::null_mut()), FcStatus::InvalidArgument);
}
