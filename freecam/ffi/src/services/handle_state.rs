use freecam_core::services::FreecamController;
use freecam_core::structs::MarkerEntry;

use crate::types::FcMarkerEntry;

// everything behind the handle's lock
//
// Bound: each marker buffer holds at most the controller's marker capacity (`markers.maxShapes`), reserved once in
// `new()`. A fill never grows past it, so the buffer is never reallocated and a borrowed pointer stays put until the
// next fill of the same buffer.
pub struct HandleState {
    controller: FreecamController,
    // the last fc_frame marker batch
    markers: Vec<FcMarkerEntry>,
    // the last fc_markers_shutdown batch
    shutdown_markers: Vec<FcMarkerEntry>,
}

impl HandleState {
    pub fn new(controller: FreecamController) -> Self {
        let capacity = controller.marker_capacity();
        Self { controller, markers: Vec::with_capacity(capacity), shutdown_markers: Vec::with_capacity(capacity) }
    }

    pub fn controller_mut(&mut self) -> &mut FreecamController {
        &mut self.controller
    }

    // copies the frame's marker batch into the reused buffer; null with a count of 0 when the batch is empty
    pub fn publish_markers(&mut self) -> (*const FcMarkerEntry, usize) {
        Self::fill(&mut self.markers, self.controller.marker_batch())
    }

    // a remove for every live marker, copied into its own reused buffer so the last frame batch stays readable
    pub fn shutdown_markers(&mut self) -> (*const FcMarkerEntry, usize) {
        Self::fill(&mut self.shutdown_markers, self.controller.marker_shutdown_batch())
    }

    fn fill(buffer: &mut Vec<FcMarkerEntry>, batch: &[MarkerEntry]) -> (*const FcMarkerEntry, usize) {
        buffer.clear();
        let room = buffer.capacity();
        buffer.extend(batch.iter().take(room).map(FcMarkerEntry::from));
        if buffer.is_empty() { (std::ptr::null(), 0) } else { (buffer.as_ptr(), buffer.len()) }
    }
}
