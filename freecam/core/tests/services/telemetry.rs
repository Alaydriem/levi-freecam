use freecam_core::services::SmoothnessTelemetry;
use freecam_core::structs::{FrameStatus, TelemetryReport, Vec3};

use crate::support::frames::Frames;

const DT: f32 = 1.0 / 120.0;

struct Fixture;

impl Fixture {
    fn status(z: f32, game_forward: Option<Vec3>) -> FrameStatus {
        FrameStatus {
            rendered_position: Some(Vec3::new(0.0, 0.0, z)),
            game_camera_forward: game_forward,
            ..Frames::status(0.0)
        }
    }

    // feeds frames and returns the last report produced
    fn run(
        telemetry: &mut SmoothnessTelemetry,
        frames: usize,
        step_for: impl Fn(usize) -> f32,
        game_forward: Option<Vec3>,
    ) -> Option<TelemetryReport> {
        let view = Vec3::new(0.0, 0.0, 1.0);
        let mut z = 0.0;
        let mut report = None;
        for frame in 0..frames {
            z += step_for(frame);
            telemetry.record(DT, &Self::status(z, game_forward), true, Vec3::new(0.0, 0.0, 6.0), view);
            if let Some(ready) = telemetry.take_report() {
                report = Some(ready);
            }
        }
        report
    }
}

#[test]
fn even_steps_give_ratio_one() {
    let mut telemetry = SmoothnessTelemetry::new(true);
    let report = Fixture::run(&mut telemetry, 240, |_| 6.0 * DT, None).unwrap();
    assert!((report.step_ratio_min - 1.0).abs() < 0.01);
    assert!((report.step_ratio_max - 1.0).abs() < 0.01);
    assert!((report.fps - 120.0).abs() < 1.0);
    assert!((report.sets_per_second - 120.0).abs() < 1.0);
}

#[test]
fn every_other_frame_steps_are_caught() {
    let mut telemetry = SmoothnessTelemetry::new(true);
    let report = Fixture::run(&mut telemetry, 240, |f| if f % 2 == 0 { 12.0 * DT } else { 0.0 }, None).unwrap();
    assert!(report.step_ratio_max >= 1.9);
    assert!(report.step_ratio_min <= 0.1);
}

#[test]
fn uneven_frame_times_with_even_steps_give_ratio_one() {
    let mut telemetry = SmoothnessTelemetry::new(true);
    let view = Vec3::new(0.0, 0.0, 1.0);
    let mut rendered_z = 0.0;
    let mut written_z = 0.0;
    let mut report = None;
    for frame in 0..400 {
        let dt = if frame % 2 == 0 { 0.007 } else { 0.016 };
        // the shim reports the pose written on the previous frame
        rendered_z = written_z;
        written_z += 6.0 * dt;
        telemetry.record(dt, &Fixture::status(rendered_z, None), true, Vec3::new(0.0, 0.0, 6.0), view);
        if let Some(ready) = telemetry.take_report() {
            report = Some(ready);
        }
    }
    let report = report.unwrap();
    assert!((report.step_ratio_min - 1.0).abs() < 0.01);
    assert!((report.step_ratio_max - 1.0).abs() < 0.01);
    assert!(rendered_z > 0.0);
}

#[test]
fn missing_rendered_position_counts_as_missed_override() {
    let mut telemetry = SmoothnessTelemetry::new(true);
    let view = Vec3::new(0.0, 0.0, 1.0);
    let mut report = None;
    for frame in 0..240 {
        let status = if frame == 100 { Frames::status(0.0) } else { Fixture::status(frame as f32, None) };
        telemetry.record(DT, &status, true, Vec3::new(0.0, 0.0, 6.0), view);
        if let Some(ready) = telemetry.take_report() {
            report = Some(ready);
        }
    }
    assert_eq!(report.unwrap().missed_overrides, 1);
}

#[test]
fn drift_is_measured() {
    let mut telemetry = SmoothnessTelemetry::new(true);
    let radians = 10.0_f32.to_radians();
    let game = Vec3::new(radians.sin(), 0.0, radians.cos());
    let report = Fixture::run(&mut telemetry, 240, |_| 6.0 * DT, Some(game)).unwrap();
    assert!((report.max_drift_degrees - 10.0).abs() < 0.1);
}

#[test]
fn camera_updates_count_position_changes() {
    let mut telemetry = SmoothnessTelemetry::new(true);
    let mut report = None;
    for frame in 0..240 {
        let status =
            FrameStatus { game_camera_position: Some(Vec3::new((frame / 2) as f32, 0.0, 0.0)), ..Frames::status(0.0) };
        telemetry.record(DT, &status, true, Vec3::default(), Vec3::new(0.0, 0.0, 1.0));
        if let Some(ready) = telemetry.take_report() {
            report = Some(ready);
        }
    }
    let report = report.unwrap();
    assert!((report.camera_updates_per_second - 60.0).abs() < 1.0);
}

#[test]
fn no_report_before_two_seconds() {
    let mut telemetry = SmoothnessTelemetry::new(true);
    assert!(Fixture::run(&mut telemetry, 228, |_| 6.0 * DT, None).is_none());
}

#[test]
fn disabled_reports_nothing() {
    let mut telemetry = SmoothnessTelemetry::new(false);
    assert!(Fixture::run(&mut telemetry, 360, |_| 6.0 * DT, None).is_none());
}

#[test]
fn window_resets_after_a_report() {
    let mut telemetry = SmoothnessTelemetry::new(true);
    assert!(Fixture::run(&mut telemetry, 240, |_| 6.0 * DT, None).is_some());
    assert!(telemetry.take_report().is_none());
    assert!(Fixture::run(&mut telemetry, 120, |_| 6.0 * DT, None).is_none());
}
