use freecam_core::structs::FrameStatus;

use crate::types::{FcPose, FcVec3};

// the game state the shim reads once per frame; every u8 flag is 0 or 1, and a has_* flag of 0 means
// the vector beside it is ignored
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FcFrameStatus {
    pub time_seconds: f64,
    pub in_world: u8,
    pub menu_open: u8,
    pub focused: u8,
    pub ready: u8,
    pub player_camera_active: u8,
    pub start_pose: FcPose,
    pub gui_hidden: u8,
    pub health: i32,
    pub dimension: i32,
    pub has_rendered_position: u8,
    pub rendered_position: FcVec3,
    pub has_game_camera_forward: u8,
    pub game_camera_forward: FcVec3,
    pub has_game_camera_position: u8,
    pub game_camera_position: FcVec3,
    // vertical field of view in degrees
    pub fov_y_degrees: f32,
    // width over height of the view
    pub aspect: f32,
    // shim time spent delivering the previous frame's marker batch, 0 when nothing was sent
    pub last_shape_send_us: f32,
    // raised by one each time the shim captures a ClientNetworkHandler; 0 while none was captured
    pub handler_generation: u32,
    /// raised by one on each Present; a screenshot counts its frames in presents, not in fc_frame calls
    pub present_count: u32,
}

impl From<&FcFrameStatus> for FrameStatus {
    fn from(value: &FcFrameStatus) -> Self {
        let optional = |has: u8, vector: &FcVec3| (has != 0).then(|| vector.to_core());
        Self {
            time_seconds: value.time_seconds,
            in_world: value.in_world != 0,
            menu_open: value.menu_open != 0,
            focused: value.focused != 0,
            ready: value.ready != 0,
            player_camera_active: value.player_camera_active != 0,
            start_pose: value.start_pose.to_core(),
            gui_hidden: value.gui_hidden != 0,
            health: value.health,
            dimension: value.dimension,
            rendered_position: optional(value.has_rendered_position, &value.rendered_position),
            game_camera_forward: optional(value.has_game_camera_forward, &value.game_camera_forward),
            game_camera_position: optional(value.has_game_camera_position, &value.game_camera_position),
            fov_y_degrees: value.fov_y_degrees,
            aspect: value.aspect,
            last_shape_send_us: value.last_shape_send_us,
            handler_generation: value.handler_generation,
            present_count: value.present_count,
        }
    }
}
