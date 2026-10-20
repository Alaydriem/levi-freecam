/// Corner radii and motion timing.
pub struct Metrics;

impl Metrics {
    pub const RADIUS_SMALL: u8 = 4;
    pub const RADIUS_CONTROL: u8 = 6;
    pub const RADIUS_PANEL: u8 = 10;

    /// Hover / press transition length (about 120 ms ease-out).
    pub const MOTION_SECONDS: f32 = 0.12;
}
