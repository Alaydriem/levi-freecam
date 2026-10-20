// a numbered control point on screen, in pixels from the top left
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ScreenMarker {
    pub at: (f32, f32),
    // the end of the short line that shows where the point looks
    pub wedge_to: (f32, f32),
    // starts at 1
    pub number: u32,
    pub is_last: bool,
}
