use freecam_core::structs::ScreenMarker;

// a numbered control point in pixels from the top left of the screen; the wedge is the end of the short
// line that shows where the point looks, number starts at 1, is_last is 0 or 1
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FcOverlayMarker {
    pub x: f32,
    pub y: f32,
    pub wedge_x: f32,
    pub wedge_y: f32,
    pub number: u32,
    pub is_last: u8,
}

impl From<&ScreenMarker> for FcOverlayMarker {
    fn from(value: &ScreenMarker) -> Self {
        Self {
            x: value.at.0,
            y: value.at.1,
            wedge_x: value.wedge_to.0,
            wedge_y: value.wedge_to.1,
            number: value.number,
            is_last: u8::from(value.is_last),
        }
    }
}
