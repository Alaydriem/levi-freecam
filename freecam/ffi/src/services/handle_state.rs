use freecam_core::services::FreecamController;
use freecam_core::services::path_store::PATH_CAPACITY;
use freecam_core::structs::overlay::path_overlay::OVERLAY_SAMPLES_PER_SEGMENT;

use crate::types::{FcOverlayMarker, FcOverlaySegment, FcOverlayView};

// everything behind the handle's lock: the controller and the C-layout copies of its overlay. The copies
// hold at most PATH_CAPACITY * OVERLAY_SAMPLES_PER_SEGMENT segments and PATH_CAPACITY markers and are
// sized once by new, so their addresses never change
pub struct HandleState {
    controller: FreecamController,
    segments: Vec<FcOverlaySegment>,
    markers: Vec<FcOverlayMarker>,
}

impl HandleState {
    pub fn new(controller: FreecamController) -> Self {
        Self {
            controller,
            segments: Vec::with_capacity(PATH_CAPACITY * OVERLAY_SAMPLES_PER_SEGMENT),
            markers: Vec::with_capacity(PATH_CAPACITY),
        }
    }

    pub fn controller_mut(&mut self) -> &mut FreecamController {
        &mut self.controller
    }

    // refills the copies from the controller's overlay without allocating and returns a view of them
    pub fn refresh_overlay(&mut self) -> FcOverlayView {
        let overlay = self.controller.overlay();
        self.segments.clear();
        self.segments.extend(overlay.segments.iter().take(self.segments.capacity()).map(FcOverlaySegment::from));
        self.markers.clear();
        self.markers.extend(overlay.markers.iter().take(self.markers.capacity()).map(FcOverlayMarker::from));
        FcOverlayView {
            segments: self.segments.as_ptr(),
            segment_count: self.segments.len() as u32,
            markers: self.markers.as_ptr(),
            marker_count: self.markers.len() as u32,
            point_count: overlay.point_count,
            shot_seconds: overlay.shot_seconds,
            visible: u8::from(self.controller.overlay_visible()),
        }
    }
}
