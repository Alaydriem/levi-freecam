use freecam_core::structs::UiFrameInput;

/// What the UI needs to know for one frame.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FcUiFrameInput {
    /// Physical pixels
    pub width: f32,
    /// Physical pixels
    pub height: f32,
    /// Zero, negative or NaN means 1.0
    pub pixels_per_point: f32,
    /// A clock that only goes forward
    pub time_seconds: f64,
    /// GPU time of the last UI draw; < 0 means none
    pub gpu_ms: f32,
}

impl From<&FcUiFrameInput> for UiFrameInput {
    fn from(value: &FcUiFrameInput) -> Self {
        Self {
            screen_width: value.width,
            screen_height: value.height,
            pixels_per_point: value.pixels_per_point,
            time_seconds: value.time_seconds,
        }
    }
}
