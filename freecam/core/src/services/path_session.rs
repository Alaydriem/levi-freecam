use log::info;

use crate::services::path_player::EASE_RAMP_SECONDS;
use crate::services::path_store::PATH_CAPACITY;
use crate::services::{PathCurve, PathPlayer, PathStore};
use crate::structs::CameraPose;
use crate::structs::path::PlaybackStep;

const MIN_PLAY_POINTS: usize = 2;

// the recorded path for one world session, its curve and its playback
pub struct PathSession {
    store: PathStore,
    curve: PathCurve,
    player: PathPlayer,
    pitch_limit: f32,
    settle_frames: u32,
    was_in_world: bool,
    dimension: i32,
}

impl PathSession {
    pub fn new(pitch_limit: f32, settle_frames: u32) -> Self {
        Self {
            store: PathStore::new(PATH_CAPACITY),
            curve: PathCurve::new(),
            player: PathPlayer::new(EASE_RAMP_SECONDS),
            pitch_limit: pitch_limit.abs(),
            settle_frames,
            was_in_world: false,
            dimension: 0,
        }
    }

    pub fn points(&self) -> &[CameraPose] {
        self.store.points()
    }

    pub fn curve(&self) -> &PathCurve {
        &self.curve
    }

    pub fn add(&mut self, pose: CameraPose) {
        if !self.store.push(pose) {
            info!("freecam path full at {PATH_CAPACITY} points");
            return;
        }
        self.curve.rebuild(self.store.points());
    }

    pub fn undo(&mut self) {
        if self.store.undo() {
            self.curve.rebuild(self.store.points());
        }
    }

    // clears the path when the player leaves the world or changes dimension inside one
    pub fn observe_world(&mut self, in_world: bool, dimension: i32) {
        let left = self.was_in_world && !in_world;
        let moved = self.was_in_world && in_world && dimension != self.dimension;
        self.was_in_world = in_world;
        self.dimension = dimension;
        if !left && !moved {
            return;
        }
        if !self.store.is_empty() {
            info!("freecam path cleared: {}", if left { "left the world" } else { "dimension change" });
        }
        self.player.stop();
        self.store.clear();
        self.curve.rebuild(self.store.points());
    }

    // returns false, and changes nothing, when the path has too few points to play
    pub fn start(&mut self, speed: f32) -> bool {
        if self.store.len() < MIN_PLAY_POINTS {
            info!("freecam play needs {MIN_PLAY_POINTS} points, the path has {}", self.store.len());
            return false;
        }
        self.player.start(self.curve.length(), speed, self.settle_frames);
        true
    }

    // the pose to show this frame, with pitch kept inside the configured limit
    pub fn step(&mut self, dt: f64, paused: bool) -> (CameraPose, PlaybackStep) {
        let step = self.player.step(dt, paused);
        let pose = self.curve.pose_at(step.distance);
        let pitch = pose.pitch.clamp(-self.pitch_limit, self.pitch_limit);
        (CameraPose::new(pose.position, pose.yaw, pitch), step)
    }

    pub fn stop(&mut self) {
        self.player.stop();
    }
}
