use std::collections::HashSet;

use freecam_core::structs::{MarkerEntry, MarkerKind, MarkerShape, Vec3};
use freecam_ffi::api::markers::{fc_markers_shutdown, fc_ray_result};
use freecam_ffi::types::{FcMarkerEntry, FcStatus, FcVec3};

use crate::support::FfiSession;

#[test]
fn marker_entry_converts_every_kind() {
    let kinds = [(MarkerKind::Remove, 0), (MarkerKind::Line, 1), (MarkerKind::Box, 2), (MarkerKind::Arrow, 3)];
    for (index, (kind, code)) in kinds.into_iter().enumerate() {
        let id = 7 + index as u64;
        let shape = MarkerShape::new(kind, Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0), 0xFF3D_A5FF);
        let entry = MarkerEntry::new(id, shape, -1);

        let converted = FcMarkerEntry::from(&entry);

        assert_eq!(converted.kind, code);
        assert_eq!(converted.id, id);
        assert_eq!(converted.location, FcVec3 { x: 1.0, y: 2.0, z: 3.0 });
        assert_eq!(converted.end, FcVec3 { x: 4.0, y: 5.0, z: 6.0 });
        assert_eq!(converted.color, 0xFF3D_A5FF);
        assert_eq!(converted.dimension, -1);
    }
}

#[test]
fn ray_result_null_handle_is_error() {
    assert_eq!(fc_ray_result(std::ptr::null_mut(), 1, 0.0, 0.0, 0.0), FcStatus::InvalidArgument);
}

#[test]
fn ray_result_reaches_core() {
    let session = FfiSession::new();
    session.chord(FfiSession::P);
    session.frame(&FfiSession::status_at(0.05, 1));
    session.chord(FfiSession::B);
    let (_, out) = session.frame(&FfiSession::status_at(0.10, 1));
    assert_eq!(out.ray.active, 1);

    assert_eq!(fc_ray_result(session.handle, 1, 20.0, 64.0, -5.0), FcStatus::Ok);

    let keys = session.handle().with_controller(|controller| controller.path().keys().to_vec()).unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].target, Some(Vec3::new(20.0, 64.0, -5.0)));
}

#[test]
fn shutdown_batch_removes_every_live_slot() {
    let session = FfiSession::with_config(FfiSession::NO_LOOK_AT);
    let (mut time, mut world) = session.build_path(1);
    for _ in 0..3 {
        time += FfiSession::FRAME_STEP;
        let (_, out) = session.frame(&FfiSession::status_at(time, 1));
        FfiSession::apply(&mut world, &FfiSession::batch(&out));
    }
    assert!(!world.is_empty());

    let mut ptr = std::ptr::null();
    let mut count = usize::MAX;
    assert_eq!(fc_markers_shutdown(session.handle, &mut ptr, &mut count), FcStatus::Ok);
    let removes = FfiSession::entries(ptr, count);

    assert!(removes.iter().all(|entry| entry.kind == 0));
    let removed: HashSet<u64> = removes.iter().map(|entry| entry.id).collect();
    let live: HashSet<u64> = world.keys().copied().collect();
    assert_eq!(removes.len(), live.len());
    assert_eq!(removed, live);

    // nothing stays live, so a second call has nothing to remove
    assert_eq!(fc_markers_shutdown(session.handle, &mut ptr, &mut count), FcStatus::Ok);
    assert_eq!(count, 0);
}

#[test]
fn shutdown_null_out_is_error() {
    let session = FfiSession::new();
    let mut ptr = std::ptr::null();
    let mut count = 0usize;
    assert_eq!(fc_markers_shutdown(std::ptr::null_mut(), &mut ptr, &mut count), FcStatus::InvalidArgument);
    assert_eq!(fc_markers_shutdown(session.handle, std::ptr::null_mut(), &mut count), FcStatus::InvalidArgument);
    assert_eq!(fc_markers_shutdown(session.handle, &mut ptr, std::ptr::null_mut()), FcStatus::InvalidArgument);
}
