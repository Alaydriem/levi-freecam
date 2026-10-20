use crate::types::{FcOverlayMarker, FcOverlaySegment};

// the path geometry to draw; the pointers borrow buffers the handle owns, point_count counts every
// control point, including those off screen, and visible is 1 only while the overlay is to be drawn
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FcOverlayView {
    pub segments: *const FcOverlaySegment,
    pub segment_count: u32,
    pub markers: *const FcOverlayMarker,
    pub marker_count: u32,
    pub point_count: u32,
    pub shot_seconds: f32,
    pub visible: u8,
}

impl Default for FcOverlayView {
    fn default() -> Self {
        Self {
            segments: std::ptr::null(),
            segment_count: 0,
            markers: std::ptr::null(),
            marker_count: 0,
            point_count: 0,
            shot_seconds: 0.0,
            visible: 0,
        }
    }
}
