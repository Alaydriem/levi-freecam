use freecam_core::services::path_store::PATH_CAPACITY;
use freecam_core::services::{OverlayBuilder, PathCurve};
use freecam_core::structs::math::Vec3;
use freecam_core::structs::overlay::path_overlay::OVERLAY_SAMPLES_PER_SEGMENT;
use freecam_core::structs::{CameraPose, PathOverlay};

const WIDTH: f32 = 1920.0;
const HEIGHT: f32 = 1080.0;
const FOV: f32 = 70.0;
const ASPECT: f32 = 16.0 / 9.0;

struct Fixture;

impl Fixture {
    fn pose(x: f32, y: f32, z: f32) -> CameraPose {
        CameraPose::new(Vec3::new(x, y, z), 0.0, 0.0)
    }

    fn build(view: CameraPose, points: &[CameraPose], speed: f32) -> PathOverlay {
        let mut curve = PathCurve::new();
        curve.rebuild(points);
        let mut overlay = PathOverlay::new();
        OverlayBuilder::build(&curve, points, view, FOV, ASPECT, WIDTH, HEIGHT, speed, &mut overlay);
        overlay
    }

    fn assert_all_finite(overlay: &PathOverlay) {
        for segment in &overlay.segments {
            let values = [segment.from.0, segment.from.1, segment.to.0, segment.to.1];
            assert!(values.iter().all(|value| value.is_finite()), "non-finite segment {segment:?}");
        }
        for marker in &overlay.markers {
            let values = [marker.at.0, marker.at.1, marker.wedge_to.0, marker.wedge_to.1];
            assert!(values.iter().all(|value| value.is_finite()), "non-finite marker {marker:?}");
        }
    }
}

#[test]
fn point_straight_ahead_is_screen_center() {
    let overlay = Fixture::build(Fixture::pose(0.0, 0.0, 0.0), &[Fixture::pose(0.0, 0.0, 10.0)], 10.0);

    assert_eq!(overlay.markers.len(), 1);
    let marker = overlay.markers[0];
    assert!((marker.at.0 - WIDTH / 2.0).abs() < 0.5, "x {}", marker.at.0);
    assert!((marker.at.1 - HEIGHT / 2.0).abs() < 0.5, "y {}", marker.at.1);
}

#[test]
fn point_to_the_right_maps_right() {
    let overlay = Fixture::build(Fixture::pose(0.0, 0.0, 0.0), &[Fixture::pose(-5.0, 0.0, 10.0)], 10.0);

    assert_eq!(overlay.markers.len(), 1);
    assert!(overlay.markers[0].at.0 > WIDTH / 2.0);
}

#[test]
fn point_above_maps_up_the_screen() {
    let overlay = Fixture::build(Fixture::pose(0.0, 0.0, 0.0), &[Fixture::pose(0.0, 5.0, 10.0)], 10.0);

    assert!(overlay.markers[0].at.1 < HEIGHT / 2.0);
}

#[test]
fn behind_camera_is_skipped() {
    let view = Fixture::pose(0.0, 0.0, 0.0);
    let behind = Fixture::build(view, &[Fixture::pose(0.0, 0.0, -10.0)], 10.0);
    assert!(behind.markers.is_empty());

    let points = [Fixture::pose(0.0, 0.0, -10.0), Fixture::pose(0.0, 0.0, 10.0)];
    let overlay = Fixture::build(view, &points, 10.0);

    Fixture::assert_all_finite(&overlay);
    assert_eq!(overlay.markers.len(), 1);
    assert_eq!(overlay.markers[0].number, 2);
    assert!(!overlay.segments.is_empty());
    // the line runs straight along the view axis, so every end stays at the horizontal center
    for segment in &overlay.segments {
        assert!((segment.from.0 - WIDTH / 2.0).abs() < 0.5);
        assert!((segment.to.0 - WIDTH / 2.0).abs() < 0.5);
    }
}

#[test]
fn clipped_segment_starts_at_the_near_plane() {
    let view = Fixture::pose(0.0, 0.0, 0.0);
    // a line from behind the camera to ahead of it, offset to the side so the cut shows on screen
    let points = [Fixture::pose(-1.0, 0.0, -1.0), Fixture::pose(-1.0, 0.0, 1.0)];
    let overlay = Fixture::build(view, &points, 10.0);

    Fixture::assert_all_finite(&overlay);
    let first = overlay.segments.first().copied();
    assert!(first.is_some());
    let first = first.unwrap_or_default();
    // at z = 0.05 with the offset of 1 to the right
    let focal = 1.0 / (FOV.to_radians() / 2.0).tan();
    let expected = WIDTH / 2.0 * (1.0 + focal * 1.0 / (0.05 * ASPECT));
    assert!((first.from.0 - expected).abs() < 1.0, "from {} expected {expected}", first.from.0);
}

#[test]
fn camera_on_point_is_skipped() {
    let view = Fixture::pose(3.0, 4.0, 5.0);
    let points = [view, Fixture::pose(3.0, 4.0, 15.0)];
    let overlay = Fixture::build(view, &points, 10.0);

    Fixture::assert_all_finite(&overlay);
    assert_eq!(overlay.markers.len(), 1);
    assert_eq!(overlay.markers[0].number, 2);
}

#[test]
fn last_marker_flagged() {
    let points = [Fixture::pose(0.0, 0.0, 5.0), Fixture::pose(1.0, 0.0, 10.0), Fixture::pose(2.0, 0.0, 15.0)];
    let overlay = Fixture::build(Fixture::pose(0.0, 0.0, 0.0), &points, 10.0);

    let flags: Vec<bool> = overlay.markers.iter().map(|marker| marker.is_last).collect();
    let numbers: Vec<u32> = overlay.markers.iter().map(|marker| marker.number).collect();
    assert_eq!(flags, vec![false, false, true]);
    assert_eq!(numbers, vec![1, 2, 3]);
    assert_eq!(overlay.point_count, 3);
}

#[test]
fn wedge_points_toward_where_the_point_looks() {
    let mut point = Fixture::pose(0.0, 0.0, 10.0);
    // yaw 90 looks along -X, which is screen right
    point.yaw = 90.0;
    let overlay = Fixture::build(Fixture::pose(0.0, 0.0, 0.0), &[point], 10.0);

    let marker = overlay.markers[0];
    assert!(marker.wedge_to.0 > marker.at.0);
}

#[test]
fn wedge_behind_the_near_plane_collapses_to_the_marker() {
    let mut point = Fixture::pose(0.0, 0.0, 10.0);
    // facing -Z, so the wedge end lies behind a camera standing just in front of the point
    point.yaw = 180.0;
    let view = Fixture::pose(0.0, 0.0, 9.9);
    let overlay = Fixture::build(view, &[point], 10.0);

    let marker = overlay.markers[0];
    assert_eq!(marker.wedge_to, marker.at);
}

#[test]
fn shot_seconds_matches_length_over_speed() {
    let points = [Fixture::pose(0.0, 0.0, 0.0), Fixture::pose(0.0, 0.0, 40.0)];
    let mut curve = PathCurve::new();
    curve.rebuild(&points);
    let overlay = Fixture::build(Fixture::pose(0.0, 0.0, -5.0), &points, 8.0);

    let ramp = overlay.shot_seconds - curve.length() / 8.0;
    assert!(ramp > 0.0, "ramp {ramp}");
    assert!((ramp - freecam_core::services::path_player::EASE_RAMP_SECONDS).abs() < 1e-4);

    assert_eq!(Fixture::build(Fixture::pose(0.0, 0.0, -5.0), &points, 0.0).shot_seconds, 0.0);
    assert_eq!(Fixture::build(Fixture::pose(0.0, 0.0, -5.0), &points[..1], 8.0).shot_seconds, 0.0);
}

#[test]
fn no_reallocation_across_builds() {
    let points: Vec<CameraPose> = (0..PATH_CAPACITY)
        .map(|index| Fixture::pose((index % 7) as f32, (index % 3) as f32, index as f32 + 2.0))
        .collect();
    let mut curve = PathCurve::new();
    curve.rebuild(&points);
    let mut overlay = PathOverlay::new();
    let capacities = (overlay.segments.capacity(), overlay.markers.capacity());

    for _ in 0..2 {
        OverlayBuilder::build(
            &curve,
            &points,
            Fixture::pose(0.0, 0.0, 0.0),
            FOV,
            ASPECT,
            WIDTH,
            HEIGHT,
            10.0,
            &mut overlay,
        );
    }

    assert_eq!((overlay.segments.capacity(), overlay.markers.capacity()), capacities);
    assert_eq!(overlay.markers.len(), PATH_CAPACITY);
    assert!(overlay.segments.len() <= PATH_CAPACITY * 16);
}

#[test]
fn unusable_view_inputs_give_an_empty_overlay() {
    let view = Fixture::pose(0.0, 0.0, 0.0);
    let points = [Fixture::pose(0.0, 0.0, 5.0), Fixture::pose(1.0, 0.0, 10.0)];
    let mut curve = PathCurve::new();
    curve.rebuild(&points);
    let nan = f32::NAN;
    let inf = f32::INFINITY;
    let bad_view = Fixture::pose(nan, 0.0, 0.0);
    let bad_points = [Fixture::pose(0.0, 0.0, 5.0), Fixture::pose(inf, 0.0, 10.0)];
    let cases: [(&str, CameraPose, f32, f32, f32, f32, f32); 12] = [
        ("fov zero", view, 0.0, ASPECT, WIDTH, HEIGHT, 10.0),
        ("fov one", view, 1.0, ASPECT, WIDTH, HEIGHT, 10.0),
        ("fov 179", view, 179.0, ASPECT, WIDTH, HEIGHT, 10.0),
        ("fov nan", view, nan, ASPECT, WIDTH, HEIGHT, 10.0),
        ("aspect zero", view, FOV, 0.0, WIDTH, HEIGHT, 10.0),
        ("aspect inf", view, FOV, inf, WIDTH, HEIGHT, 10.0),
        ("width zero", view, FOV, ASPECT, 0.0, HEIGHT, 10.0),
        ("height negative", view, FOV, ASPECT, WIDTH, -1.0, 10.0),
        ("width nan", view, FOV, ASPECT, nan, HEIGHT, 10.0),
        ("view nan", bad_view, FOV, ASPECT, WIDTH, HEIGHT, 10.0),
        ("speed nan", view, FOV, ASPECT, WIDTH, HEIGHT, nan),
        ("speed inf", view, FOV, ASPECT, WIDTH, HEIGHT, inf),
    ];
    for (name, view, fov, aspect, width, height, speed) in cases {
        let mut overlay = PathOverlay::new();
        OverlayBuilder::build(&curve, &points, view, fov, aspect, width, height, speed, &mut overlay);
        assert!(overlay.segments.is_empty(), "{name}: segments");
        assert!(overlay.markers.is_empty(), "{name}: markers");
        assert_eq!(overlay.point_count, 2, "{name}: point count");
        assert_eq!(overlay.shot_seconds, 0.0, "{name}: shot seconds");
    }

    let mut bad_curve = PathCurve::new();
    bad_curve.rebuild(&bad_points);
    let mut overlay = PathOverlay::new();
    OverlayBuilder::build(&bad_curve, &bad_points, view, FOV, ASPECT, WIDTH, HEIGHT, 10.0, &mut overlay);
    Fixture::assert_all_finite(&overlay);
    assert!(overlay.segments.is_empty());
    assert!(overlay.markers.is_empty());
    assert_eq!(overlay.shot_seconds, 0.0);
}

#[test]
fn guarded_build_clears_a_previously_filled_overlay() {
    let view = Fixture::pose(0.0, 0.0, 0.0);
    let points = [Fixture::pose(0.0, 0.0, 5.0), Fixture::pose(1.0, 0.0, 10.0)];
    let mut curve = PathCurve::new();
    curve.rebuild(&points);
    let mut overlay = PathOverlay::new();
    OverlayBuilder::build(&curve, &points, view, FOV, ASPECT, WIDTH, HEIGHT, 10.0, &mut overlay);
    assert!(!overlay.markers.is_empty());

    OverlayBuilder::build(&curve, &points, view, 0.0, ASPECT, WIDTH, HEIGHT, 10.0, &mut overlay);

    assert!(overlay.segments.is_empty());
    assert!(overlay.markers.is_empty());
    assert_eq!(overlay.point_count, 2);
}

// the frame after the second Ctrl+B: the camera sits on point 2 and looks back along the path, with the game's
// real projection and back buffer; every segment must be finite and the count stays inside one curve segment
#[test]
fn camera_on_the_newest_point_gives_finite_segments_with_game_inputs() {
    let first = CameraPose::new(Vec3::new(10.0, 70.0, 10.0), 30.0, -10.0);
    let second = CameraPose::new(Vec3::new(14.0, 71.0, 18.0), 200.0, -5.0);
    let mut curve = PathCurve::new();
    curve.rebuild(&[first, second]);
    let mut overlay = PathOverlay::new();

    OverlayBuilder::build(&curve, &[first, second], second, 70.0, 1.3617964, 2153.0, 1581.0, 10.0, &mut overlay);

    Fixture::assert_all_finite(&overlay);
    assert!(!overlay.segments.is_empty());
    assert!(overlay.segments.len() <= OVERLAY_SAMPLES_PER_SEGMENT);
}
