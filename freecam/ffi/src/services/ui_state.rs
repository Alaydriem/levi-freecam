use freecam_core::config::UiConfig;
use freecam_core::structs::{UiAction, UiCursor, UiEvent, UiFrameInput, UiFrameOutput, UiLegend, UiSnapshot, UiTiming};
use freecam_ui::runtime::UiRuntime;

use crate::services::UiEventQueue;
use crate::types::{FcUiDraw, FcUiFrame, FcUiFrameInput, FcUiTextureDelta, FcUiVertex};

// the render thread's UI: the egui runtime, the legend and the C-layout copies of the last frame. The game thread
// never touches it. `drained` holds at most the event queue capacity; the frame copies are cleared each frame and grow
// only when a frame is larger than every frame before it
pub struct UiState {
    runtime: UiRuntime,
    legend: UiLegend,
    out: UiFrameOutput,
    vertices: Vec<FcUiVertex>,
    draws: Vec<FcUiDraw>,
    textures: Vec<FcUiTextureDelta>,
    drained: Vec<UiEvent>,
    skipped_frames: u64,
    measure: bool,
}

impl UiState {
    // `config` is sanitized; `measure` is the telemetry switch
    pub fn new(config: &UiConfig, measure: bool, legend: UiLegend) -> Self {
        Self {
            runtime: UiRuntime::new(config.max_vertices),
            legend,
            out: UiFrameOutput::new(),
            vertices: Vec::new(),
            draws: Vec::new(),
            textures: Vec::new(),
            drained: Vec::with_capacity(config.event_queue_capacity.max(1)),
            skipped_frames: 0,
            measure,
        }
    }

    // moves queued events into the drain buffer, at most its capacity, so a busy window thread cannot keep it looping
    pub fn take_events(&mut self, queue: &UiEventQueue) {
        while self.drained.len() < self.drained.capacity() {
            match queue.pop() {
                Some(event) => self.drained.push(event),
                None => break,
            }
        }
    }

    // runs one UI frame over the drained events and returns a view of its buffers
    pub fn run(&mut self, input: &FcUiFrameInput, snapshot: &UiSnapshot) -> FcUiFrame {
        let mut events = self.drained.drain(..);
        self.runtime.run(&UiFrameInput::from(input), snapshot, &self.legend, &mut events, &mut self.out);
        drop(events);
        self.copy_frame();
        self.skipped_frames += u64::from(self.out.skipped);
        self.view()
    }

    // what the user did in the last frame
    pub fn actions(&self) -> &[UiAction] {
        &self.out.actions
    }

    // the cost and counters of the last frame; a negative or NaN gpu_ms means none
    pub fn timing(&self, cpu_ms: f32, gpu_ms: f32, dropped_events: u64) -> UiTiming {
        UiTiming {
            cpu_ms,
            gpu_ms: (gpu_ms >= 0.0).then_some(gpu_ms),
            vertices: Self::count(self.vertices.len()),
            draws: Self::count(self.draws.len()),
            dropped_events,
            skipped_frames: self.skipped_frames,
        }
    }

    pub fn reset_textures(&mut self) {
        self.runtime.reset_textures();
    }

    fn copy_frame(&mut self) {
        self.vertices.clear();
        self.vertices.extend(self.out.vertices.iter().map(FcUiVertex::from));
        self.draws.clear();
        self.draws.extend(self.out.draws.iter().map(FcUiDraw::from));
        self.textures.clear();
        let bytes = &self.out.texture_bytes;
        self.textures.extend(self.out.textures.iter().map(|delta| FcUiTextureDelta::new(delta, bytes)));
    }

    fn view(&self) -> FcUiFrame {
        FcUiFrame {
            vertices: self.vertices.as_ptr(),
            vertex_count: Self::count(self.vertices.len()),
            indices: self.out.indices.as_ptr(),
            index_count: Self::count(self.out.indices.len()),
            draws: self.draws.as_ptr(),
            draw_count: Self::count(self.draws.len()),
            textures: self.textures.as_ptr(),
            texture_count: Self::count(self.textures.len()),
            freed: self.out.freed.as_ptr(),
            freed_count: Self::count(self.out.freed.len()),
            wants_pointer: u8::from(self.out.wants_pointer),
            cursor: match self.out.cursor {
                UiCursor::Default => 0,
                UiCursor::PointingHand => 1,
            },
            skipped: u8::from(self.out.skipped),
            measure_gpu: u8::from(self.measure),
        }
    }

    // a count past u32::MAX is cut down, so C++ reads fewer elements than exist, never more
    fn count(len: usize) -> u32 {
        u32::try_from(len).unwrap_or(u32::MAX)
    }
}
