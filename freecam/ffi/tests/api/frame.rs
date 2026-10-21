use amm_client::test_support::FakeTable;
use amm_core::structs::event::ModEvent;
use freecam_core::services::marker_sync::MARKER_ID_BASE;
use freecam_core::structs::{CameraPose, Vec3};
use freecam_ffi::api::frame::fc_frame;
use stats_alloc::Region;

use crate::GLOBAL;
use freecam_core::structs::FreecamAction;
use freecam_ffi::api::input::fc_server_instruction;
use freecam_ffi::api::markers::fc_ray_result;
use freecam_ffi::services::LinkSession;
use freecam_ffi::types::{FcFrameOutput, FcStatus, FcVec3};

use crate::support::FfiSession;

#[test]
fn null_handle_is_invalid_argument() {
    let status = FfiSession::ready_status();
    let mut out = FcFrameOutput::default();
    assert_eq!(fc_frame(std::ptr::null(), &status, &mut out), FcStatus::InvalidArgument);
}

#[test]
fn null_out_pointers_are_invalid_argument() {
    let session = FfiSession::new();
    let status = FfiSession::ready_status();
    let mut out = FcFrameOutput::default();
    assert_eq!(fc_frame(session.handle, std::ptr::null(), &mut out), FcStatus::InvalidArgument);
    assert_eq!(fc_frame(session.handle, &status, std::ptr::null_mut()), FcStatus::InvalidArgument);
}

#[test]
fn toggle_then_frame_outputs_set_and_takes_the_flight_leases() {
    let session = FfiSession::new();
    FfiSession::tap(FreecamAction::Toggle);
    let status = FfiSession::ready_status();

    let (code, out) = session.frame(&status);

    assert_eq!(code, FcStatus::Ok);
    assert_eq!(out.camera, 1);
    assert_eq!(out.camera_owned, 1);
    assert_eq!(out.pose, status.start_pose);
    assert_eq!(FakeTable::with(|state| state.acquired.clone()), LinkSession::FLIGHT);
}

#[test]
fn marker_batch_pointer_stays_readable_until_next_frame() {
    let session = FfiSession::with_config(FfiSession::NO_LOOK_AT);
    let (time, _) = session.build_path(1);
    let (code, out) = session.frame(&FfiSession::status_at(time + FfiSession::FRAME_STEP, 1));
    assert_eq!(code, FcStatus::Ok);
    assert!(out.marker_count > 0);
    let first = FfiSession::batch(&out);

    // calls to other exports leave the batch alone
    assert_eq!(fc_server_instruction(session.handle, 0), FcStatus::Ok);
    assert_eq!(fc_ray_result(session.handle, 0, 0.0, 0.0, 0.0), FcStatus::Ok);

    assert_eq!(FfiSession::batch(&out), first);
    assert!(first.iter().all(|entry| entry.id >= MARKER_ID_BASE && (1..=3).contains(&entry.kind)));
    assert!(first.len() <= 400);
}

#[test]
fn markers_wait_for_a_captured_handler() {
    let session = FfiSession::with_config(FfiSession::NO_LOOK_AT);
    let (mut time, world) = session.build_path(0);
    time += FfiSession::FRAME_STEP;
    let (_, out) = session.frame(&FfiSession::status_at(time, 0));
    assert!(world.is_empty());
    assert_eq!(out.marker_count, 0);

    time += FfiSession::FRAME_STEP;
    let (_, out) = session.frame(&FfiSession::status_at(time, 1));
    assert!(out.marker_count > 0);
}

#[test]
fn ray_request_crosses_when_core_asks() {
    let session = FfiSession::new();
    FfiSession::tap(FreecamAction::Toggle);
    session.frame(&FfiSession::status_at(0.05, 1));
    let (_, quiet) = session.frame(&FfiSession::status_at(0.10, 1));
    assert_eq!(quiet.ray.active, 0);

    FfiSession::tap(FreecamAction::AddPoint);
    let (code, out) = session.frame(&FfiSession::status_at(0.15, 1));
    assert_eq!(code, FcStatus::Ok);
    let start = FfiSession::start_pose();
    let forward = CameraPose::new(Vec3::new(10.0, 64.0, -5.0), start.yaw, start.pitch).forward();
    assert_eq!(out.ray.active, 1);
    assert_eq!(out.ray.origin, start.position);
    assert_eq!(out.ray.direction, FcVec3::from(&forward));
    assert_eq!(out.ray.max_distance, 256.0);

    let (_, next) = session.frame(&FfiSession::status_at(0.20, 1));
    assert_eq!(next.ray.active, 0);
}

#[test]
fn frame_does_not_allocate_after_warm_up() {
    let session = FfiSession::with_config(FfiSession::NO_LOOK_AT);
    FfiSession::tap(FreecamAction::Toggle);
    let mut time = 0.0;
    let mut frame = |events: usize| {
        for _ in 0..events {
            FakeTable::push_event(ModEvent::PointerDelta { dx: 1.0, dy: 0.5 });
        }
        time += FfiSession::FRAME_STEP;
        let (code, _) = session.frame(&FfiSession::status_at(time, 1));
        assert_eq!(code, FcStatus::Ok);
    };
    // the fake manager's queue grows to its size here, so the measured frames reuse it
    frame(LinkSession::DRAIN_LIMIT);
    for _ in 0..30 {
        frame(4);
    }

    let region = Region::new(&GLOBAL);
    for _ in 0..100 {
        frame(4);
    }

    assert_eq!(region.change().allocations, 0);
}
