use std::collections::HashMap;
use std::path::Path;

use freecam_ffi::api::frame::fc_frame;
use freecam_ffi::api::input::fc_key;
use freecam_ffi::api::lifecycle::{fc_create, fc_destroy};
use freecam_ffi::services::FcHandle;
use freecam_ffi::types::{FcFrameOutput, FcFrameStatus, FcMarkerEntry, FcPose, FcStatus, FcStr, FcVec3};

// a live handle created from a config path that does not exist, destroyed on drop
pub struct FfiSession {
    pub handle: *mut FcHandle,
}

impl FfiSession {
    pub const CTRL: u32 = 0x11;
    pub const P: u32 = 0x50;
    pub const E: u32 = 0x45;
    pub const B: u32 = 0x42;
    pub const W: u32 = 0x57;
    pub const FRAME_STEP: f64 = 0.05;
    pub const NO_LOOK_AT: &str = r#"{"lookAt":{"enabled":false}}"#;

    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        Self::create(&dir.path().join("absent.json"))
    }

    // a live handle created from a config file holding `json`
    pub fn with_config(json: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("config.json");
        std::fs::write(&file, json).unwrap();
        Self::create(&file)
    }

    fn create(file: &Path) -> Self {
        let text = file.to_str().unwrap();
        let path = FcStr { ptr: text.as_ptr(), len: text.len() };
        let handle = fc_create(&path, None);
        assert!(!handle.is_null());
        Self { handle }
    }

    // presses and releases the modifier chord with `code`, then releases the modifier
    pub fn chord(&self, code: u32) {
        let mut consume = 0u8;
        for (key, down) in [(Self::CTRL, 1), (code, 1), (code, 0), (Self::CTRL, 0)] {
            assert_eq!(fc_key(self.handle, key, down, &mut consume), FcStatus::Ok);
        }
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
        // SAFETY: the pointer came from fc_create, was checked non-null, and lives until drop
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
        self.chord(Self::P);
        step(time, &mut world);
        self.chord(Self::B);
        let mut consume = 0u8;
        assert_eq!(fc_key(self.handle, Self::W, 1, &mut consume), FcStatus::Ok);
        for _ in 0..20 {
            time += Self::FRAME_STEP;
            step(time, &mut world);
        }
        assert_eq!(fc_key(self.handle, Self::W, 0, &mut consume), FcStatus::Ok);
        self.chord(Self::B);
        (time, world)
    }
}

impl Drop for FfiSession {
    fn drop(&mut self) {
        fc_destroy(self.handle);
    }
}
