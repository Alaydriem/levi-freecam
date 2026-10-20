// what the UI needs to know about the screen for one frame
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiFrameInput {
    // physical pixels
    pub screen_width: f32,
    // physical pixels
    pub screen_height: f32,
    pub pixels_per_point: f32,
    // a clock that only goes forward, used for hover animations
    pub time_seconds: f64,
}
