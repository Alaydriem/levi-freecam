use freecam_ffi::api::frame::fc_frame;
use freecam_ffi::api::lifecycle::{fc_create, fc_destroy};
use freecam_ffi::services::FcHandle;
use freecam_ffi::types::{FcFrameOutput, FcFrameStatus, FcPose, FcStatus, FcStr, FcVec3};

// a live handle created from a config path that does not exist, destroyed on drop
pub struct FfiSession {
    pub handle: *mut FcHandle,
}

impl FfiSession {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("absent.json");
        let text = missing.to_str().unwrap();
        let path = FcStr { ptr: text.as_ptr(), len: text.len() };
        Self { handle: fc_create(&path, None) }
    }

    pub fn start_pose() -> FcPose {
        FcPose { position: FcVec3 { x: 10.0, y: 64.0, z: -5.0 }, yaw: 90.0, pitch: 10.0 }
    }

    // in world, focused, ready, player camera active, health 20
    pub fn ready_status() -> FcFrameStatus {
        FcFrameStatus {
            time_seconds: 0.05,
            in_world: 1,
            focused: 1,
            ready: 1,
            player_camera_active: 1,
            start_pose: Self::start_pose(),
            health: 20,
            fov_y_degrees: 70.0,
            aspect: 16.0 / 9.0,
            screen_width: 1920.0,
            screen_height: 1080.0,
            ..FcFrameStatus::default()
        }
    }

    pub fn frame(&self, status: &FcFrameStatus) -> (FcStatus, FcFrameOutput) {
        let mut out = FcFrameOutput::default();
        let code = fc_frame(self.handle, status, &mut out);
        (code, out)
    }
}

impl Drop for FfiSession {
    fn drop(&mut self) {
        fc_destroy(self.handle);
    }
}
