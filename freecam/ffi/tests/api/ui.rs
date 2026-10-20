use std::sync::atomic::{AtomicBool, Ordering};

use egui::{Pos2, Rect, vec2};
use freecam_core::structs::TelemetryReport;
use freecam_ffi::api::input::fc_mouse_button;
use freecam_ffi::api::ui::{fc_ui_event, fc_ui_frame, fc_ui_reset_textures};
use freecam_ffi::services::FcHandle;
use freecam_ffi::types::{FcFrameOutput, FcStatus, FcUiEvent, FcUiFrame, FcUiFrameInput, FcUiVertex};
use freecam_ui::views::BarLayout;

use crate::support::FfiSession;

const MOVE: u32 = 0;
const BUTTON: u32 = 1;
const RIGHT: u8 = 2;
const DRAG_GRAB: u8 = 3;

fn input(time_seconds: f64) -> FcUiFrameInput {
    FcUiFrameInput { width: 1920.0, height: 1080.0, pixels_per_point: 1.0, time_seconds, gpu_ms: -1.0 }
}

fn event(kind: u32, x: f32, y: f32, pressed: u8) -> FcUiEvent {
    FcUiEvent { kind, x, y, button: 0, pressed, vk: 0 }
}

fn ui_frame(session: &FfiSession, time_seconds: f64) -> FcUiFrame {
    let mut out = FcUiFrame::default();
    assert_eq!(fc_ui_frame(session.handle, &input(time_seconds), &mut out), FcStatus::Ok);
    out
}

// the Clear points button on the bar
fn button_centre() -> Pos2 {
    BarLayout::new(Rect::from_min_size(Pos2::ZERO, vec2(1920.0, 1080.0))).clear.center()
}

// freecam on with one point, so a Clear points click arms the clear
fn session_with_point() -> FfiSession {
    let session = visible_session(FfiSession::NO_LOOK_AT);
    session.chord(FfiSession::B);
    session
}

fn click(session: &FfiSession, at: Pos2) {
    for (kind, pressed) in [(MOVE, 0), (BUTTON, 1), (BUTTON, 0)] {
        assert_eq!(fc_ui_event(session.handle, &event(kind, at.x, at.y, pressed)), FcStatus::Ok);
    }
}

// game frames 50 ms apart until core's telemetry closes a window; telemetry must be on in the config
fn run_until_report(session: &FfiSession) -> TelemetryReport {
    let mut status = FfiSession::ready_status();
    for _ in 0..200 {
        status.time_seconds += 0.05;
        session.frame(&status);
        let report = session.handle().with_controller(|controller| controller.last_telemetry_report()).flatten();
        if let Some(report) = report {
            return report;
        }
    }
    panic!("no telemetry report after 10 seconds of frames");
}

// freecam on, two game frames so the snapshot the UI reads says visible
fn visible_session(json: &str) -> FfiSession {
    let session = FfiSession::with_config(json);
    let status = FfiSession::ready_status();
    session.chord(FfiSession::P);
    session.frame(&status);
    session.frame(&status);
    session
}

#[test]
fn null_arguments_are_rejected() {
    let session = FfiSession::new();
    let mut out = FcUiFrame::default();
    let move_event = event(MOVE, 1.0, 1.0, 0);
    assert_eq!(fc_ui_frame(std::ptr::null(), &input(0.0), &mut out), FcStatus::InvalidArgument);
    assert_eq!(fc_ui_frame(session.handle, std::ptr::null(), &mut out), FcStatus::InvalidArgument);
    assert_eq!(fc_ui_frame(session.handle, &input(0.0), std::ptr::null_mut()), FcStatus::InvalidArgument);
    assert_eq!(fc_ui_event(std::ptr::null(), &move_event), FcStatus::InvalidArgument);
    assert_eq!(fc_ui_event(session.handle, std::ptr::null()), FcStatus::InvalidArgument);
    assert_eq!(fc_ui_reset_textures(std::ptr::null()), FcStatus::InvalidArgument);
}

#[test]
fn unknown_event_kind_is_rejected() {
    let session = FfiSession::new();
    assert_eq!(fc_ui_event(session.handle, &event(99, 0.0, 0.0, 0)), FcStatus::InvalidArgument);
    assert_eq!(
        fc_ui_event(session.handle, &FcUiEvent { button: 7, ..event(BUTTON, 0.0, 0.0, 1) }),
        FcStatus::InvalidArgument
    );
    assert_eq!(fc_ui_event(session.handle, &event(MOVE, 0.0, 0.0, 0)), FcStatus::Ok);
}

#[test]
fn hidden_ui_gives_an_empty_frame() {
    let session = FfiSession::new();
    session.frame(&FfiSession::ready_status());

    let out = ui_frame(&session, 0.1);

    assert_eq!(out.vertex_count, 0);
    assert_eq!(out.index_count, 0);
    assert_eq!(out.draw_count, 0);
}

#[test]
fn visible_ui_gives_a_frame_with_fonts() {
    let session = visible_session("{}");

    let out = ui_frame(&session, 0.1);

    assert!(out.vertex_count > 0);
    assert!(out.index_count > 0);
    assert!(out.draw_count > 0);
    assert!(out.texture_count > 0);
    assert_eq!(out.skipped, 0);
    assert_eq!(out.measure_gpu, 0);
    // SAFETY: the frame came from fc_ui_frame on a live handle with no later fc_ui_frame call
    let textures = unsafe { std::slice::from_raw_parts(out.textures, out.texture_count as usize) };
    let font = textures[0];
    assert_eq!(font.full, 1);
    assert!(!font.rgba.is_null());
    assert_eq!(font.byte_len, font.width as usize * font.height as usize * 4);
}

#[test]
fn telemetry_config_asks_for_gpu_timing() {
    let session = visible_session(r#"{"telemetry": true}"#);

    let out = ui_frame(&session, 0.1);

    assert_eq!(out.measure_gpu, 1);
}

#[test]
fn pointers_stay_valid_until_the_next_call() {
    let session = visible_session("{}");
    let out = ui_frame(&session, 0.1);
    // SAFETY: the frame came from fc_ui_frame on a live handle with no later fc_ui_frame call
    let first: FcUiVertex = unsafe { *out.vertices };

    session.frame(&FfiSession::ready_status());
    assert_eq!(fc_ui_event(session.handle, &event(MOVE, 5.0, 5.0, 0)), FcStatus::Ok);
    assert_eq!(fc_ui_reset_textures(session.handle), FcStatus::Ok);

    // SAFETY: fc_frame, fc_ui_event and fc_ui_reset_textures do not touch the frame buffers; no fc_ui_frame call
    // happened since
    let again: FcUiVertex = unsafe { *out.vertices };
    assert_eq!(first, again);
}

#[test]
fn event_queue_drops_oldest_when_full() {
    let session = visible_session(r#"{"telemetry": true, "ui": {"eventQueueCapacity": 16}}"#);
    for i in 0..20 {
        assert_eq!(fc_ui_event(session.handle, &event(MOVE, i as f32, 10.0, 0)), FcStatus::Ok);
    }

    ui_frame(&session, 0.1);
    let report = run_until_report(&session);

    assert_eq!(report.ui_dropped_events, 4);
}

#[test]
fn every_ui_frame_between_game_frames_reaches_telemetry() {
    let session = visible_session(r#"{"telemetry": true}"#);
    for (i, gpu_ms) in [1.0, 5.0, 2.0].into_iter().enumerate() {
        let mut out = FcUiFrame::default();
        let frame_input = FcUiFrameInput { gpu_ms, ..input(0.1 * (i + 1) as f64) };
        assert_eq!(fc_ui_frame(session.handle, &frame_input, &mut out), FcStatus::Ok);
    }

    let report = run_until_report(&session);

    assert!((report.ui_gpu_ms_max - 5.0).abs() < 1e-5);
    assert!((report.ui_gpu_ms_avg - 8.0 / 3.0).abs() < 1e-5);
    assert!(report.ui_cpu_ms_max > 0.0);
}

#[test]
fn game_frame_hands_off_while_a_ui_frame_runs() {
    let session = session_with_point();
    let mut status = FfiSession::ready_status();
    session.chord(FfiSession::E);
    status.time_seconds += 0.05;
    session.frame(&status);
    ui_frame(&session, 0.1);
    click(&session, button_centre());
    ui_frame(&session, 0.2);
    // the next UI frame builds the font atlas again, so it runs for a long time
    assert_eq!(fc_ui_reset_textures(session.handle), FcStatus::Ok);
    let address = session.handle as usize;
    let started = AtomicBool::new(false);

    let (ui_frame_still_running, armed) = std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            started.store(true, Ordering::SeqCst);
            let mut out = FcUiFrame::default();
            fc_ui_frame(address as *const FcHandle, &input(0.3), &mut out)
        });
        while !started.load(Ordering::SeqCst) {
            std::thread::yield_now();
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
        status.time_seconds += 0.05;
        session.frame(&status);
        let still_running = !worker.is_finished();
        let armed = session.handle().with_controller(|controller| controller.ui_snapshot().clear_armed).unwrap();
        assert_eq!(worker.join().unwrap(), FcStatus::Ok);
        (still_running, armed)
    });

    assert!(ui_frame_still_running, "the UI frame ended before the game frame, so this run proves nothing");
    assert!(armed);
}

#[test]
fn click_round_trip() {
    let session = session_with_point();
    let status = FfiSession::ready_status();
    session.chord(FfiSession::E);
    let (_, out) = session.frame(&status);
    assert_eq!(out.cursor_mode, 1);
    ui_frame(&session, 0.1);

    click(&session, button_centre());
    ui_frame(&session, 0.2);
    session.frame(&status);
    let after = ui_frame(&session, 0.3);

    let armed = session.handle().with_controller(|controller| controller.ui_snapshot().clear_armed);
    assert_eq!(armed, Some(true));
    assert_eq!(after.skipped, 0);
}

#[test]
fn reset_textures_sends_the_font_again() {
    let session = visible_session("{}");
    ui_frame(&session, 0.1);
    let quiet = ui_frame(&session, 0.2);
    assert_eq!(quiet.texture_count, 0);

    assert_eq!(fc_ui_reset_textures(session.handle), FcStatus::Ok);
    let out = ui_frame(&session, 0.3);

    assert!(out.texture_count > 0);
    assert!(out.vertex_count > 0);
}

// freecam and bar mode on, the pointer moved to `at` and seen by one UI frame, then a right press and one game frame
fn right_press_at(at: Pos2) -> FcFrameOutput {
    let session = session_with_point();
    let status = FfiSession::ready_status();
    session.chord(FfiSession::E);
    let (_, out) = session.frame(&status);
    assert_eq!(out.cursor_mode, 1);
    ui_frame(&session, 0.1);
    assert_eq!(fc_ui_event(session.handle, &event(MOVE, at.x, at.y, 0)), FcStatus::Ok);
    ui_frame(&session, 0.2);

    let mut consume = 0u8;
    assert_eq!(fc_mouse_button(session.handle, RIGHT, 1, &mut consume), FcStatus::Ok);
    let (_, out) = session.frame(&status);
    out
}

#[test]
fn right_press_over_the_bar_starts_no_drag() {
    let out = right_press_at(button_centre());

    assert_eq!(out.look_drag, 0);
    assert_eq!(out.mouse, 0);
    assert_eq!(out.cursor_mode, 1);
}

#[test]
fn right_press_away_from_the_bar_starts_a_drag() {
    let out = right_press_at(Pos2::new(960.0, 400.0));

    assert_eq!(out.look_drag, 1);
    assert_eq!(out.mouse, DRAG_GRAB);
    assert_eq!(out.cursor_mode, 1);
}
