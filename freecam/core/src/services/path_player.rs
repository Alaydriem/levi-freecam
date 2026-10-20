use crate::structs::path::PlaybackStep;

pub const EASE_RAMP_SECONDS: f32 = 1.0;

pub struct PathPlayer {
    ramp_seconds: f32,
    running: bool,
    length: f64,
    speed: f64,
    // ramp time after shrinking so both ramps fit in the path
    ramp: f64,
    total_seconds: f64,
    settle_remaining: u32,
    elapsed: f64,
    held_distance: f64,
}

impl PathPlayer {
    pub fn new(ramp_seconds: f32) -> Self {
        Self {
            ramp_seconds: ramp_seconds.max(0.0),
            running: false,
            length: 0.0,
            speed: 0.0,
            ramp: 0.0,
            total_seconds: 0.0,
            settle_remaining: 0,
            elapsed: 0.0,
            held_distance: 0.0,
        }
    }

    pub fn start(&mut self, length: f32, speed: f32, settle_frames: u32) {
        self.running = true;
        self.length = f64::from(length.max(0.0));
        self.speed = f64::from(speed.max(0.0));
        self.settle_remaining = settle_frames;
        self.elapsed = 0.0;
        self.held_distance = 0.0;
        if self.speed <= 0.0 || self.length <= 0.0 {
            self.length = 0.0;
            self.ramp = 0.0;
            self.total_seconds = 0.0;
            return;
        }
        self.ramp = f64::from(self.ramp_seconds).min(self.length / self.speed);
        self.total_seconds = self.length / self.speed + self.ramp;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn stop(&mut self) {
        if self.running {
            self.held_distance = if self.settle_remaining > 0 { 0.0 } else { self.distance_at(self.elapsed) };
        }
        self.running = false;
    }

    pub fn step(&mut self, dt: f64, paused: bool) -> PlaybackStep {
        if !self.running {
            return PlaybackStep { distance: self.held_distance as f32, settling: false, finished: true };
        }
        if paused {
            let distance = if self.settle_remaining > 0 { 0.0 } else { self.distance_at(self.elapsed) };
            return PlaybackStep { distance: distance as f32, settling: self.settle_remaining > 0, finished: false };
        }
        if self.settle_remaining > 0 {
            self.settle_remaining -= 1;
            return PlaybackStep { distance: 0.0, settling: true, finished: false };
        }
        self.elapsed += dt.max(0.0);
        if self.elapsed >= self.total_seconds {
            self.running = false;
            self.held_distance = self.length;
            return PlaybackStep { distance: self.length as f32, settling: false, finished: true };
        }
        PlaybackStep { distance: self.distance_at(self.elapsed) as f32, settling: false, finished: false }
    }

    fn distance_at(&self, seconds: f64) -> f64 {
        if self.total_seconds <= 0.0 {
            return 0.0;
        }
        let ramp_distance = self.speed * self.ramp / 2.0;
        let down_start = self.total_seconds - self.ramp;
        let distance = if seconds < self.ramp {
            Self::ramp_distance(self.speed, self.ramp, seconds / self.ramp)
        } else if seconds < down_start {
            ramp_distance + self.speed * (seconds - self.ramp)
        } else {
            let remaining = (self.total_seconds - seconds) / self.ramp;
            self.length - Self::ramp_distance(self.speed, self.ramp, remaining)
        };
        distance.clamp(0.0, self.length)
    }

    fn ramp_distance(speed: f64, ramp: f64, fraction: f64) -> f64 {
        speed * ramp * (fraction.powi(3) - fraction.powi(4) / 2.0)
    }
}
