use freecam_core::services::PathSession;
use freecam_core::services::path_store::PATH_CAPACITY;
use freecam_core::structs::CameraPose;
use freecam_core::structs::math::Vec3;

const SETTLE: u32 = 3;
const DT: f64 = 0.05;
const MAX_STEPS: usize = 10_000;

struct Fixture;

impl Fixture {
    fn pose(x: f32) -> CameraPose {
        CameraPose::new(Vec3::new(x, 0.0, 0.0), 0.0, 0.0)
    }

    // a session holding two points, ten blocks apart
    fn two_points(session: &mut PathSession) {
        session.add(Self::pose(0.0));
        session.add(Self::pose(10.0));
    }
}

#[test]
fn playback_pitch_stays_inside_the_configured_limit() {
    let mut session = PathSession::new(45.0, SETTLE);
    session.add(CameraPose::new(Vec3::new(0.0, 0.0, 0.0), 0.0, 80.0));
    session.add(CameraPose::new(Vec3::new(10.0, 0.0, 0.0), 30.0, -80.0));
    session.add(CameraPose::new(Vec3::new(20.0, 5.0, 0.0), 60.0, 80.0));
    assert!(session.start(6.0));
    let mut steps = 0;
    loop {
        let (pose, step) = session.step(DT, false);
        assert!(pose.pitch.abs() <= 45.0, "pitch {}", pose.pitch);
        steps += 1;
        if step.finished || steps >= MAX_STEPS {
            break;
        }
    }
    assert!(steps < MAX_STEPS);
}

#[test]
fn start_needs_two_points() {
    let mut session = PathSession::new(89.9, SETTLE);
    assert!(!session.start(6.0));
    assert!(session.step(DT, false).1.finished);
    session.add(Fixture::pose(0.0));
    assert!(!session.start(6.0));
    let (pose, step) = session.step(DT, false);
    assert!(step.finished);
    assert_eq!(pose, Fixture::pose(0.0));
    assert_eq!(session.points(), &[Fixture::pose(0.0)]);
}

#[test]
fn start_with_two_points_settles_at_the_first_point() {
    let mut session = PathSession::new(89.9, SETTLE);
    Fixture::two_points(&mut session);
    assert!(session.start(6.0));
    for _ in 0..SETTLE {
        let (pose, step) = session.step(DT, false);
        assert!(step.settling);
        assert_eq!(pose, Fixture::pose(0.0));
    }
    assert!(!session.step(DT, false).1.settling);
}

#[test]
fn add_at_capacity_is_refused() {
    let mut session = PathSession::new(89.9, SETTLE);
    for index in 0..PATH_CAPACITY {
        session.add(Fixture::pose(index as f32));
    }
    let length = session.curve().length();
    session.add(Fixture::pose(9999.0));
    assert_eq!(session.points().len(), PATH_CAPACITY);
    assert_eq!(session.points()[PATH_CAPACITY - 1], Fixture::pose((PATH_CAPACITY - 1) as f32));
    assert_eq!(session.curve().length(), length);
}

#[test]
fn first_frame_in_another_dimension_keeps_the_path() {
    let mut session = PathSession::new(89.9, SETTLE);
    Fixture::two_points(&mut session);
    session.observe_world(true, 1);
    assert_eq!(session.points().len(), 2);
    session.observe_world(true, 1);
    assert_eq!(session.points().len(), 2);
}

#[test]
fn dimension_change_inside_a_world_clears() {
    let mut session = PathSession::new(89.9, SETTLE);
    session.observe_world(true, 0);
    Fixture::two_points(&mut session);
    session.observe_world(true, 1);
    assert!(session.points().is_empty());
    assert_eq!(session.curve().length(), 0.0);
}

#[test]
fn dimension_change_while_out_of_the_world_does_not_clear() {
    let mut session = PathSession::new(89.9, SETTLE);
    session.observe_world(true, 0);
    session.observe_world(false, 0);
    Fixture::two_points(&mut session);
    session.observe_world(false, 1);
    assert_eq!(session.points().len(), 2);
    session.observe_world(true, 1);
    assert_eq!(session.points().len(), 2);
}

#[test]
fn leaving_clears_and_rejoining_does_not_clear_again() {
    let mut session = PathSession::new(89.9, SETTLE);
    session.observe_world(true, 0);
    Fixture::two_points(&mut session);
    session.observe_world(false, 0);
    assert!(session.points().is_empty());
    assert_eq!(session.curve().length(), 0.0);
    Fixture::two_points(&mut session);
    session.observe_world(true, 0);
    assert_eq!(session.points().len(), 2);
}

#[test]
fn leaving_stops_playback() {
    let mut session = PathSession::new(89.9, SETTLE);
    session.observe_world(true, 0);
    Fixture::two_points(&mut session);
    assert!(session.start(6.0));
    session.observe_world(false, 0);
    assert!(session.step(DT, false).1.finished);
}
