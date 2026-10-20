use serde::{Deserialize, Serialize};

const MIN_EVENT_QUEUE_CAPACITY: usize = 16;
const MAX_EVENT_QUEUE_CAPACITY: usize = 4096;
const MIN_MAX_VERTICES: usize = 4096;
const MAX_MAX_VERTICES: usize = 1_048_576;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UiConfig {
    // bound of the queue that carries input events to the UI
    pub event_queue_capacity: usize,
    // a UI frame with more vertices than this is not drawn
    pub max_vertices: usize,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self { event_queue_capacity: 256, max_vertices: 65536 }
    }
}

impl UiConfig {
    // returns a copy with every value moved into its usable range
    pub fn sanitized(self) -> Self {
        Self {
            event_queue_capacity: self.event_queue_capacity.clamp(MIN_EVENT_QUEUE_CAPACITY, MAX_EVENT_QUEUE_CAPACITY),
            max_vertices: self.max_vertices.clamp(MIN_MAX_VERTICES, MAX_MAX_VERTICES),
        }
    }
}
