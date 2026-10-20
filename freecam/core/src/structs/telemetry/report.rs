// one window of frame pacing measurements
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TelemetryReport {
    pub fps: f32,
    pub min_dt: f32,
    pub max_dt: f32,
    // rendered distance per frame over the distance the previous frame's velocity predicts, 1.0 when nothing qualified
    pub step_ratio_min: f32,
    pub step_ratio_max: f32,
    pub sets_per_second: f32,
    pub camera_updates_per_second: f32,
    // largest angle between the view direction and the game camera direction
    pub max_drift_degrees: f32,
    // frames where freecam expected a rendered position and the shim reported none
    pub missed_overrides: u32,
}
