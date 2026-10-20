use freecam_core::config::KeyConfig;
use freecam_core::structs::{UiEvent, UiFrameInput, UiFrameOutput, UiLegend, UiSnapshot};
use freecam_ui::runtime::UiRuntime;

/// A runtime, a snapshot and a legend for tests, run on a 1920x1080 screen.
pub struct UiFixture {
    pub runtime: UiRuntime,
    pub snapshot: UiSnapshot,
    pub legend: UiLegend,
    pub input: UiFrameInput,
    pub out: UiFrameOutput,
}

impl UiFixture {
    pub const MAX_VERTICES: usize = 65536;
    pub const SETTLE_FRAMES: usize = 12;
    pub const DISTANCES: [f32; 3] = [0.0, 10.0, 40.0];

    pub fn new() -> Self {
        Self::with_max_vertices(Self::MAX_VERTICES)
    }

    pub fn with_max_vertices(max_vertices: usize) -> Self {
        Self {
            runtime: UiRuntime::new(max_vertices),
            snapshot: UiSnapshot { visible: true, point_count: 3, total_seconds: 4.5, ..Default::default() },
            legend: UiLegend::new(&KeyConfig::default()),
            input: UiFrameInput {
                screen_width: 1920.0,
                screen_height: 1080.0,
                pixels_per_point: 1.0,
                time_seconds: 0.0,
            },
            out: UiFrameOutput::new(),
        }
    }

    pub fn cursor_mode(mut self) -> Self {
        self.snapshot.cursor_mode = true;
        self
    }

    /// Bar mode with three points at 0, 10 and 40 blocks on a 40 block path.
    pub fn with_path(mut self) -> Self {
        self.snapshot.cursor_mode = true;
        self.snapshot.point_count = 3;
        self.snapshot.point_distances[..3].copy_from_slice(&Self::DISTANCES);
        self.snapshot.length = 40.0;
        self.snapshot.speed = 6.0;
        self
    }

    /// Bar mode with `points` points one block apart.
    pub fn with_points(mut self, points: usize) -> Self {
        self.snapshot.cursor_mode = true;
        self.snapshot.point_count = points as u32;
        for (index, distance) in self.snapshot.point_distances[..points].iter_mut().enumerate() {
            *distance = index as f32;
        }
        self.snapshot.length = points.saturating_sub(1) as f32;
        self.snapshot.speed = 6.0;
        self
    }

    /// Runs enough empty frames for every hover and selection transition to finish.
    pub fn settle(&mut self) {
        for _ in 0..Self::SETTLE_FRAMES {
            self.frame(&[]);
        }
    }

    pub fn pixels_per_point(mut self, pixels_per_point: f32) -> Self {
        self.input.pixels_per_point = pixels_per_point;
        self
    }

    /// Runs one frame, a sixtieth of a second after the last.
    pub fn frame(&mut self, events: &[UiEvent]) -> &UiFrameOutput {
        self.input.time_seconds += 1.0 / 60.0;
        let mut events = events.iter().copied();
        self.runtime.run(&self.input, &self.snapshot, &self.legend, &mut events, &mut self.out);
        &self.out
    }
}
