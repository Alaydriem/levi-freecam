use std::collections::HashMap;
use std::path::Path;

use amm_client::test_support::FakeTable;
use amm_core::structs::event::{KeybindPhase, ModEvent};
use freecam_core::structs::FreecamAction;
use freecam_ffi::api::frame::fc_frame;
use freecam_ffi::api::lifecycle::fc_destroy;
use freecam_ffi::services::FcHandle;
use freecam_ffi::types::{FcFrameOutput, FcFrameStatus, FcMarkerEntry, FcPose, FcStatus, FcVec3};

// a live handle registered with this thread's fake mod manager, destroyed on drop
pub struct FfiSession {
    pub handle: *mut FcHandle,
}

impl FfiSession {
    pub const FRAME_STEP: f64 = 0.05;
    pub const NO_LOOK_AT: &str = "look_at { enabled = false }";

    pub fn new() -> Self {
        Self::with_config("")
    }

    // a live handle whose config.hcl, as the fake manager hands it over, is `hcl`
    pub fn with_config(hcl: &str) -> Self {
        FakeTable::set_config(hcl);
        let handle = FcHandle::create(FakeTable::table(), Path::new(FakeTable::CONFIG_DIR)).unwrap();
        Self { handle: Box::into_raw(Box::new(handle)) }
    }

    // queues the manager's keybind event for the next fc_frame
    pub fn keybind(action: FreecamAction, phase: KeybindPhase) {
        FakeTable::push_event(ModEvent::Keybind { action: action.index(), phase });
    }

    // presses and releases the keybind
    pub fn tap(action: FreecamAction) {
        Self::keybind(action, KeybindPhase::Pressed);
        Self::keybind(action, KeybindPhase::Released);
    }

    pub fn start_pose() -> FcPose {
        FcPose { position: FcVec3 { x: 10.0, y: 64.0, z: -5.0 }, yaw: 90.0, pitch: 10.0 }
    }

    // in world, focused, ready, handler captured once, player camera active, health 20
    pub fn ready_status() -> FcFrameStatus {
        FcFrameStatus {
            time_seconds: 0.05,
            handler_generation: 1,
            in_world: 1,
            focused: 1,
            ready: 1,
            player_camera_active: 1,
            start_pose: Self::start_pose(),
            health: 20,
            fov_y_degrees: 70.0,
            aspect: 16.0 / 9.0,
            ..FcFrameStatus::default()
        }
    }

    // the live handle behind the raw pointer, for the behavioural accessors
    pub fn handle(&self) -> &FcHandle {
        // SAFETY: the pointer came from Box::into_raw over a created handle, and lives until drop
        unsafe { &*self.handle }
    }

    pub fn frame(&self, status: &FcFrameStatus) -> (FcStatus, FcFrameOutput) {
        let mut out = FcFrameOutput::default();
        let code = fc_frame(self.handle, status, &mut out);
        (code, out)
    }

    // the ready status at `time_seconds` with this handler capture count
    pub fn status_at(time_seconds: f64, handler_generation: u32) -> FcFrameStatus {
        FcFrameStatus { time_seconds, handler_generation, ..Self::ready_status() }
    }

    // a copy of the frame's marker batch, read through the borrowed pointer
    pub fn batch(out: &FcFrameOutput) -> Vec<FcMarkerEntry> {
        Self::entries(out.markers, out.marker_count)
    }

    pub fn entries(ptr: *const FcMarkerEntry, count: usize) -> Vec<FcMarkerEntry> {
        if count == 0 {
            return Vec::new();
        }
        assert!(!ptr.is_null());
        // SAFETY: fc_frame or fc_markers_shutdown returned this pointer for `count` entries, and no call
        // to the same export happened since
        unsafe { std::slice::from_raw_parts(ptr, count) }.to_vec()
    }

    // applies a batch the way the client does: a remove drops the id, any other kind draws it
    pub fn apply(world: &mut HashMap<u64, FcMarkerEntry>, batch: &[FcMarkerEntry]) {
        for entry in batch {
            if entry.kind == 0 {
                world.remove(&entry.id);
            } else {
                world.insert(entry.id, *entry);
            }
        }
    }

    // turns freecam on and records two points 20 walking frames apart, every frame with this handler
    // capture count; returns the time of the last frame and the shapes the client shows
    pub fn build_path(&self, handler_generation: u32) -> (f64, HashMap<u64, FcMarkerEntry>) {
        let mut world = HashMap::new();
        let mut time = Self::FRAME_STEP;
        let step = |time: f64, world: &mut HashMap<u64, FcMarkerEntry>| {
            let (code, out) = self.frame(&Self::status_at(time, handler_generation));
            assert_eq!(code, FcStatus::Ok);
            Self::apply(world, &Self::batch(&out));
        };
        Self::tap(FreecamAction::Toggle);
        step(time, &mut world);
        Self::tap(FreecamAction::AddPoint);
        Self::keybind(FreecamAction::FlyForward, KeybindPhase::Pressed);
        for _ in 0..20 {
            time += Self::FRAME_STEP;
            step(time, &mut world);
        }
        Self::keybind(FreecamAction::FlyForward, KeybindPhase::Released);
        Self::tap(FreecamAction::AddPoint);
        (time, world)
    }
}

impl Drop for FfiSession {
    fn drop(&mut self) {
        fc_destroy(self.handle);
    }
}
