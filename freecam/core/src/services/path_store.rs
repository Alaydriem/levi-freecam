use crate::structs::CameraPose;

pub const PATH_CAPACITY: usize = 256;

pub struct PathStore {
    points: Vec<CameraPose>,
    capacity: usize,
    revision: u64,
}

impl PathStore {
    pub fn new(capacity: usize) -> Self {
        Self { points: Vec::with_capacity(capacity), capacity, revision: 0 }
    }

    // returns false when the store is full
    pub fn push(&mut self, pose: CameraPose) -> bool {
        if self.points.len() >= self.capacity {
            return false;
        }
        self.points.push(pose);
        self.revision += 1;
        true
    }

    // returns false when there is nothing to remove
    pub fn undo(&mut self) -> bool {
        if self.points.pop().is_none() {
            return false;
        }
        self.revision += 1;
        true
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.revision += 1;
    }

    pub fn points(&self) -> &[CameraPose] {
        &self.points
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    // grows on every successful change
    pub fn revision(&self) -> u64 {
        self.revision
    }
}
