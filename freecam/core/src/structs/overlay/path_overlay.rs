use crate::services::path_store::PATH_CAPACITY;
use crate::structs::math::Vec3;
use crate::structs::overlay::{ScreenMarker, ScreenSegment};

// path samples joined per curve segment
pub const OVERLAY_SAMPLES_PER_SEGMENT: usize = 16;

// the 2D geometry the shim draws while editing; every buffer is sized once by new
#[derive(Debug, Clone)]
pub struct PathOverlay {
    pub segments: Vec<ScreenSegment>,
    pub markers: Vec<ScreenMarker>,
    pub point_count: u32,
    pub shot_seconds: f32,
    // world positions sampled along the curve
    pub(crate) samples: Vec<Vec3>,
}

impl PathOverlay {
    pub fn new() -> Self {
        Self {
            segments: Vec::with_capacity(PATH_CAPACITY * OVERLAY_SAMPLES_PER_SEGMENT),
            markers: Vec::with_capacity(PATH_CAPACITY),
            point_count: 0,
            shot_seconds: 0.0,
            samples: Vec::with_capacity(PATH_CAPACITY * OVERLAY_SAMPLES_PER_SEGMENT + 1),
        }
    }

    // empties the geometry and keeps the buffers
    pub fn clear(&mut self) {
        self.segments.clear();
        self.markers.clear();
        self.samples.clear();
        self.point_count = 0;
        self.shot_seconds = 0.0;
    }
}

impl Default for PathOverlay {
    fn default() -> Self {
        Self::new()
    }
}
