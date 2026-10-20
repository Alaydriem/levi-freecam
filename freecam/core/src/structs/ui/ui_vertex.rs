// one UI vertex: position in physical pixels, texture coordinates, premultiplied sRGBA colour
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct UiVertex {
    pub x: f32,
    pub y: f32,
    pub u: f32,
    pub v: f32,
    pub rgba: [u8; 4],
}
