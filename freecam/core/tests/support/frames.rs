use freecam_core::structs::{CameraPose, FrameStatus, Vec3};

pub struct Frames;

impl Frames {
    pub fn start_pose() -> CameraPose {
        CameraPose::new(Vec3::new(10.0, 64.0, -5.0), 90.0, 10.0)
    }

    // in world, focused, ready, player camera active, health 20, dimension 0
    pub fn status(time_seconds: f64) -> FrameStatus {
        FrameStatus {
            time_seconds,
            in_world: true,
            menu_open: false,
            focused: true,
            ready: true,
            player_camera_active: true,
            start_pose: Self::start_pose(),
            gui_hidden: false,
            health: 20,
            dimension: 0,
            rendered_position: None,
            game_camera_forward: None,
            game_camera_position: None,
            fov_y_degrees: 70.0,
            aspect: 16.0 / 9.0,
            screen_width: 1920.0,
            screen_height: 1080.0,
        }
    }
}
