use std::sync::atomic::{AtomicU64, Ordering};

use crossbeam_queue::ArrayQueue;
use freecam_core::structs::UiEvent;

// the lock-free queue that carries input events from the window thread to the render thread. It holds at most
// `capacity` events (ui.eventQueueCapacity, at least 1); a push into a full queue evicts the oldest event and counts it
pub struct UiEventQueue {
    events: ArrayQueue<UiEvent>,
    dropped: AtomicU64,
}

impl UiEventQueue {
    pub fn new(capacity: usize) -> Self {
        Self { events: ArrayQueue::new(capacity.max(1)), dropped: AtomicU64::new(0) }
    }

    pub fn push(&self, event: UiEvent) {
        if self.events.force_push(event).is_some() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn pop(&self) -> Option<UiEvent> {
        self.events.pop()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.events.capacity()
    }

    // events evicted since the queue was made
    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }
}
