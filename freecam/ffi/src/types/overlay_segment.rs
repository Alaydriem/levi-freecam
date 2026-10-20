use freecam_core::structs::ScreenSegment;

// a line of the path in pixels from the top left of the screen
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FcOverlaySegment {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl From<&ScreenSegment> for FcOverlaySegment {
    fn from(value: &ScreenSegment) -> Self {
        Self { x0: value.from.0, y0: value.from.1, x1: value.to.0, y1: value.to.1 }
    }
}
