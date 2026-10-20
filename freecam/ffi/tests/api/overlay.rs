use freecam_ffi::api::input::fc_key;
use freecam_ffi::api::overlay::fc_overlay;
use freecam_ffi::types::{FcOverlayView, FcStatus};

use crate::support::FfiSession;

const CTRL: u32 = 0x11;
const P: u32 = 0x50;
const B: u32 = 0x42;
const W: u32 = 0x57;
const S: u32 = 0x53;
const ENTER: u32 = 0x0D;

struct Keys;

impl Keys {
    fn press(session: &FfiSession, code: u32, down: u8) {
        let mut consume = 0u8;
        assert_eq!(fc_key(session.handle, code, down, &mut consume), FcStatus::Ok);
    }

    fn chord(session: &FfiSession, code: u32) {
        Self::press(session, CTRL, 1);
        Self::press(session, code, 1);
        Self::press(session, code, 0);
    }
}

#[test]
fn overlay_null_args_are_invalid() {
    let session = FfiSession::new();
    let mut out = FcOverlayView::default();
    assert_eq!(fc_overlay(std::ptr::null(), &mut out), FcStatus::InvalidArgument);
    assert_eq!(fc_overlay(session.handle, std::ptr::null_mut()), FcStatus::InvalidArgument);
}

#[test]
fn overlay_after_points_has_markers() {
    let session = FfiSession::new();
    let mut status = FfiSession::ready_status();
    Keys::chord(&session, P);
    session.frame(&status);

    Keys::chord(&session, B);
    Keys::press(&session, CTRL, 0);
    Keys::press(&session, W, 1);
    for _ in 0..20 {
        status.time_seconds += 0.05;
        session.frame(&status);
    }
    Keys::press(&session, W, 0);
    Keys::chord(&session, B);
    Keys::press(&session, CTRL, 0);
    Keys::press(&session, S, 1);
    for _ in 0..40 {
        status.time_seconds += 0.05;
        session.frame(&status);
    }
    Keys::press(&session, S, 0);
    status.time_seconds += 0.05;
    let (code, out) = session.frame(&status);
    assert_eq!(code, FcStatus::Ok);
    assert_eq!(out.show_overlay, 1);

    let mut view = FcOverlayView::default();
    assert_eq!(fc_overlay(session.handle, &mut view), FcStatus::Ok);

    assert_eq!(view.visible, 1);
    assert_eq!(view.point_count, 2);
    assert!(view.marker_count <= 2);
    assert!(view.segment_count > 0);
    assert!(view.shot_seconds.is_finite());
    // SAFETY: the view came from fc_overlay on a live handle with no later fc_overlay or fc_frame call
    let segments = unsafe { std::slice::from_raw_parts(view.segments, view.segment_count as usize) };
    // SAFETY: the same view and the same lifetime as the segments
    let markers = unsafe { std::slice::from_raw_parts(view.markers, view.marker_count as usize) };
    for segment in segments {
        assert!([segment.x0, segment.y0, segment.x1, segment.y1].iter().all(|value| value.is_finite()));
    }
    for marker in markers {
        assert!([marker.x, marker.y, marker.wedge_x, marker.wedge_y].iter().all(|value| value.is_finite()));
    }
}

#[test]
fn overlay_empty_when_off() {
    let session = FfiSession::new();
    session.frame(&FfiSession::ready_status());

    let mut view = FcOverlayView::default();
    assert_eq!(fc_overlay(session.handle, &mut view), FcStatus::Ok);

    assert_eq!(view.visible, 0);
    assert_eq!(view.segment_count, 0);
    assert_eq!(view.marker_count, 0);
    assert_eq!(view.point_count, 0);
}

#[test]
fn overlay_hidden_when_on_with_no_points() {
    let session = FfiSession::new();
    let status = FfiSession::ready_status();
    Keys::chord(&session, P);
    let (code, out) = session.frame(&status);
    assert_eq!(code, FcStatus::Ok);
    assert_eq!(out.camera, 1);
    assert_eq!(out.show_overlay, 0);

    let mut view = FcOverlayView::default();
    assert_eq!(fc_overlay(session.handle, &mut view), FcStatus::Ok);

    assert_eq!(view.visible, 0);
    assert_eq!(view.segment_count, 0);
    assert_eq!(view.marker_count, 0);
    assert_eq!(view.point_count, 0);
}

#[test]
fn overlay_hidden_while_playing() {
    let session = FfiSession::new();
    let mut status = FfiSession::ready_status();
    Keys::chord(&session, P);
    session.frame(&status);

    Keys::chord(&session, B);
    Keys::press(&session, CTRL, 0);
    Keys::press(&session, W, 1);
    for _ in 0..20 {
        status.time_seconds += 0.05;
        session.frame(&status);
    }
    Keys::press(&session, W, 0);
    Keys::chord(&session, B);
    Keys::chord(&session, ENTER);
    Keys::press(&session, CTRL, 0);
    status.time_seconds += 0.05;
    let (code, out) = session.frame(&status);
    assert_eq!(code, FcStatus::Ok);
    // freecam is still on and the path is playing, so the camera is still set
    assert_eq!(out.camera, 1);
    assert_eq!(out.show_overlay, 0);

    let mut view = FcOverlayView::default();
    assert_eq!(fc_overlay(session.handle, &mut view), FcStatus::Ok);

    assert_eq!(view.visible, 0);
    assert_eq!(view.segment_count, 0);
    assert_eq!(view.marker_count, 0);
}
