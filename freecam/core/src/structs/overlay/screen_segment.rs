// a line of the path on screen, in pixels from the top left
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ScreenSegment {
    pub from: (f32, f32),
    pub to: (f32, f32),
}
