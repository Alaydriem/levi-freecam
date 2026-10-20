use crate::structs::{FrameStatus, TelemetryReport, Vec3};

// seconds of recorded frame time per report
const WINDOW_SECONDS: f32 = 2.0;
// absorbs float rounding when frame times add up to exactly one window
const WINDOW_TOLERANCE: f32 = 1e-3;
// slower frames are not steady enough to judge step size
const MIN_STEADY_SPEED: f32 = 0.5;
// largest speed change between frames that still counts as steady
const STEADY_SPEED_TOLERANCE: f32 = 0.02;

pub struct SmoothnessTelemetry {
    enabled: bool,
    elapsed: f32,
    frames: u32,
    min_dt: f32,
    max_dt: f32,
    step_ratio_min: f32,
    step_ratio_max: f32,
    step_samples: u32,
    sets: u32,
    camera_updates: u32,
    max_drift_degrees: f32,
    previous_rendered: Option<Vec3>,
    previous_speed: Option<f32>,
    previous_expected_step: f32,
    previous_steady: bool,
    missed_overrides: u32,
    previous_camera: Option<Vec3>,
    report: Option<TelemetryReport>,
}

impl SmoothnessTelemetry {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            elapsed: 0.0,
            frames: 0,
            min_dt: f32::MAX,
            max_dt: 0.0,
            step_ratio_min: f32::MAX,
            step_ratio_max: 0.0,
            step_samples: 0,
            sets: 0,
            camera_updates: 0,
            max_drift_degrees: 0.0,
            previous_rendered: None,
            previous_speed: None,
            previous_expected_step: 0.0,
            previous_steady: false,
            missed_overrides: 0,
            previous_camera: None,
            report: None,
        }
    }

    // forgets the previous frame and the open window, for a new freecam session
    pub fn restart(&mut self) {
        *self = Self::new(self.enabled);
    }

    pub fn record(&mut self, dt: f32, status: &FrameStatus, sent_set: bool, velocity: Vec3, view_forward: Vec3) {
        if !self.enabled {
            return;
        }
        self.elapsed += dt;
        self.frames += 1;
        self.min_dt = self.min_dt.min(dt);
        self.max_dt = self.max_dt.max(dt);
        self.sets += u32::from(sent_set);
        self.record_step(dt, status.rendered_position, velocity);
        self.record_camera(status);
        if let Some(game_forward) = status.game_camera_forward {
            let drift = Self::angle_degrees(view_forward, game_forward);
            self.max_drift_degrees = self.max_drift_degrees.max(drift);
        }
        if self.elapsed >= WINDOW_SECONDS - WINDOW_TOLERANCE {
            self.report = Some(self.build_report());
            self.reset_window();
        }
    }

    pub fn take_report(&mut self) -> Option<TelemetryReport> {
        self.report.take()
    }

    // the rendered position is the previous frame's write, so it is judged against the previous frame's step
    fn record_step(&mut self, dt: f32, rendered: Option<Vec3>, velocity: Vec3) {
        let speed = velocity.length();
        if rendered.is_none() && self.previous_speed.is_some() {
            self.missed_overrides += 1;
        }
        if let (Some(now), Some(before)) = (rendered, self.previous_rendered)
            && self.previous_steady
            && self.previous_expected_step > 0.0
        {
            let ratio = (now - before).length() / self.previous_expected_step;
            self.step_ratio_min = self.step_ratio_min.min(ratio);
            self.step_ratio_max = self.step_ratio_max.max(ratio);
            self.step_samples += 1;
        }
        let steady =
            self.previous_speed.is_some_and(|previous| (speed - previous).abs() < previous * STEADY_SPEED_TOLERANCE);
        self.previous_steady = steady && speed >= MIN_STEADY_SPEED && dt > 0.0;
        self.previous_expected_step = speed * dt;
        self.previous_rendered = rendered;
        self.previous_speed = Some(speed);
    }

    fn record_camera(&mut self, status: &FrameStatus) {
        if status.game_camera_position.is_some() && status.game_camera_position != self.previous_camera {
            self.camera_updates += 1;
        }
        self.previous_camera = status.game_camera_position;
    }

    fn angle_degrees(a: Vec3, b: Vec3) -> f32 {
        a.normalized().dot(b.normalized()).clamp(-1.0, 1.0).acos().to_degrees()
    }

    fn build_report(&self) -> TelemetryReport {
        let (step_ratio_min, step_ratio_max) =
            if self.step_samples > 0 { (self.step_ratio_min, self.step_ratio_max) } else { (1.0, 1.0) };
        TelemetryReport {
            fps: self.frames as f32 / self.elapsed,
            min_dt: self.min_dt,
            max_dt: self.max_dt,
            step_ratio_min,
            step_ratio_max,
            sets_per_second: self.sets as f32 / self.elapsed,
            camera_updates_per_second: self.camera_updates as f32 / self.elapsed,
            max_drift_degrees: self.max_drift_degrees,
            missed_overrides: self.missed_overrides,
        }
    }

    fn reset_window(&mut self) {
        self.elapsed = 0.0;
        self.frames = 0;
        self.min_dt = f32::MAX;
        self.max_dt = 0.0;
        self.step_ratio_min = f32::MAX;
        self.step_ratio_max = 0.0;
        self.step_samples = 0;
        self.sets = 0;
        self.camera_updates = 0;
        self.max_drift_degrees = 0.0;
        self.missed_overrides = 0;
    }
}
