use freecam_core::services::PathStore;
use freecam_core::services::path_store::PATH_CAPACITY;
use freecam_core::structs::CameraPose;
use freecam_core::structs::math::Vec3;

struct Fixture;

impl Fixture {
    fn pose(x: f32) -> CameraPose {
        CameraPose::new(Vec3::new(x, 0.0, 0.0), 0.0, 0.0)
    }
}

#[test]
fn push_appends_and_bumps_revision() {
    let mut store = PathStore::new(PATH_CAPACITY);
    let before = store.revision();
    assert!(store.push(Fixture::pose(1.0)));
    assert_eq!(store.len(), 1);
    assert_eq!(store.points(), &[Fixture::pose(1.0)]);
    assert!(store.revision() > before);
}

#[test]
fn undo_removes_last() {
    let mut store = PathStore::new(PATH_CAPACITY);
    store.push(Fixture::pose(1.0));
    store.push(Fixture::pose(2.0));
    let before = store.revision();
    assert!(store.undo());
    assert_eq!(store.points(), &[Fixture::pose(1.0)]);
    assert!(store.revision() > before);
}

#[test]
fn undo_on_empty_returns_false_and_keeps_revision() {
    let mut store = PathStore::new(PATH_CAPACITY);
    let before = store.revision();
    assert!(!store.undo());
    assert_eq!(store.revision(), before);
}

#[test]
fn push_at_capacity_returns_false() {
    let mut store = PathStore::new(PATH_CAPACITY);
    for i in 0..PATH_CAPACITY {
        assert!(store.push(Fixture::pose(i as f32)));
    }
    let before = store.revision();
    assert!(!store.push(Fixture::pose(999.0)));
    assert_eq!(store.len(), 256);
    assert_eq!(store.revision(), before);
}

#[test]
fn clear_empties() {
    let mut store = PathStore::new(PATH_CAPACITY);
    store.push(Fixture::pose(1.0));
    let before = store.revision();
    store.clear();
    assert_eq!(store.len(), 0);
    assert!(store.points().is_empty());
    assert!(store.revision() > before);
}
