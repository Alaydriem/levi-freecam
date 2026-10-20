use std::time::Instant;

use egui::{Context, CursorIcon, Event, FontDefinitions, FullOutput, Pos2, RawInput, Rect, Ui, vec2};
use freecam_core::structs::{UiAction, UiCursor, UiEvent, UiFrameInput, UiFrameOutput, UiLegend, UiSnapshot};

use super::{EventMapper, MeshWriter, UiProfile};
use crate::theme::{FontLoader, StudioTheme};
use crate::views::{BarText, Legend, PlaybackBar, Strip, Toast};

/// Owns the egui context and turns one snapshot into one frame of flat mesh data.
pub struct UiRuntime {
    ctx: Context,
    // kept so that a new context gets the same fonts without reading the files again
    fonts: FontDefinitions,
    max_vertices: usize,
    // cursor mode of the last frame
    cursor_mode: bool,
    mapper: EventMapper,
    profiling: bool,
    profile: UiProfile,
    bar_text: BarText,
}

impl UiRuntime {
    /// Creates the context, installs the Studio Dark theme and loads the fonts. `max_vertices` comes sanitized from
    /// config; a frame with more vertices is skipped.
    pub fn new(max_vertices: usize) -> Self {
        let fonts = FontLoader::definitions();
        Self {
            ctx: Self::context(&fonts),
            fonts,
            max_vertices,
            cursor_mode: false,
            mapper: EventMapper::new(),
            profiling: false,
            profile: UiProfile::default(),
            bar_text: BarText::new(),
        }
    }

    /// Turns the timers of [`Self::profile`] on or off. Off by default, so a normal frame reads no clock.
    pub fn set_profiling(&mut self, profiling: bool) {
        self.profiling = profiling;
        self.profile = UiProfile::default();
    }

    /// Starts a new egui context with the cached fonts, so the next frame sends every texture in full, the font atlas
    /// included. Call it after the renderer lost its textures. Hover animations restart.
    pub fn reset_textures(&mut self) {
        self.ctx = Self::context(&self.fonts);
        self.bar_text.invalidate();
    }

    /// The phase timings of the last frame, while profiling is on.
    pub fn profile(&self) -> UiProfile {
        self.profile
    }

    /// Runs one UI frame and writes it into `out`, whose buffers are cleared first and keep their capacity.
    pub fn run(
        &mut self,
        input: &UiFrameInput,
        snapshot: &UiSnapshot,
        legend: &UiLegend,
        events: &mut dyn Iterator<Item = UiEvent>,
        out: &mut UiFrameOutput,
    ) {
        out.clear();
        let pixels_per_point = Self::sane_pixels_per_point(input.pixels_per_point);
        let screen = [input.screen_width.max(0.0), input.screen_height.max(0.0)];
        let mut raw = self.raw_input(input, screen, pixels_per_point, events);
        // the shim sends no pointer events outside cursor mode, so the last position would stay hovered
        if self.cursor_mode && !snapshot.cursor_mode {
            raw.events.push(Event::PointerGone);
        }
        self.cursor_mode = snapshot.cursor_mode;

        let started = self.clock();
        let actions = &mut out.actions;
        let bar_text = &mut self.bar_text;
        let mut over_panel = false;
        let mut full = self.ctx.run_ui(raw, |ui| {
            if snapshot.visible {
                over_panel = Self::views(ui, snapshot, legend, bar_text, actions);
            }
        });
        out.wants_pointer = snapshot.cursor_mode && (over_panel || self.ctx.egui_is_using_pointer());
        out.cursor = Self::cursor(full.platform_output.cursor_icon);

        let built = self.clock();
        let primitives = self.ctx.tessellate(std::mem::take(&mut full.shapes), full.pixels_per_point);
        let tessellated = self.clock();
        MeshWriter::write_textures(&full.textures_delta, out);
        Self::release_textures(&mut full);
        MeshWriter::write_meshes(&primitives, full.pixels_per_point, screen, self.max_vertices, out);
        let copied = self.clock();

        if let (Some(started), Some(built), Some(tessellated), Some(copied)) = (started, built, tessellated, copied) {
            self.profile =
                UiProfile { run: built - started, tessellate: tessellated - built, copy: copied - tessellated };
        }
    }

    // paints every view and pushes the actions clicked; returns whether the pointer is over a panel
    fn views(
        ui: &mut Ui,
        snapshot: &UiSnapshot,
        legend: &UiLegend,
        bar_text: &mut BarText,
        actions: &mut Vec<UiAction>,
    ) -> bool {
        let strip = Strip::show(ui, snapshot);
        let legend_rect = Legend::show(ui, legend, snapshot.cursor_mode);
        bar_text.begin(ui.ctx());
        let bar = PlaybackBar::show(ui, snapshot, bar_text, actions);
        Toast::show(ui, snapshot, bar_text);
        [strip, legend_rect, bar].into_iter().any(|rect| ui.rect_contains_pointer(rect))
    }

    fn raw_input(
        &mut self,
        input: &UiFrameInput,
        screen: [f32; 2],
        pixels_per_point: f32,
        events: &mut dyn Iterator<Item = UiEvent>,
    ) -> RawInput {
        let size = vec2(screen[0] / pixels_per_point, screen[1] / pixels_per_point);
        let mut raw = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            time: Some(input.time_seconds),
            ..Default::default()
        };
        raw.viewports.entry(raw.viewport_id).or_default().native_pixels_per_point = Some(pixels_per_point);
        for event in events {
            self.mapper.map(event, pixels_per_point, &mut raw.events);
        }
        raw
    }

    // the deltas were copied into the output; egui asserts that a dropped delta was handled
    fn release_textures(full: &mut FullOutput) {
        full.textures_delta.clear();
    }

    fn context(fonts: &FontDefinitions) -> Context {
        let ctx = Context::default();
        StudioTheme::install_with(&ctx, fonts.clone());
        ctx
    }

    fn cursor(icon: CursorIcon) -> UiCursor {
        match icon {
            CursorIcon::PointingHand => UiCursor::PointingHand,
            _ => UiCursor::Default,
        }
    }

    // a zero, negative or NaN scale from the caller falls back to 1.0
    fn sane_pixels_per_point(pixels_per_point: f32) -> f32 {
        if pixels_per_point.is_finite() && pixels_per_point > 0.0 { pixels_per_point } else { 1.0 }
    }

    fn clock(&self) -> Option<Instant> {
        self.profiling.then(Instant::now)
    }
}
