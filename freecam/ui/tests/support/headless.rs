use egui::{Context, Event, FullOutput, Pos2, RawInput, Rect, Ui, vec2};

/// Runs one egui frame without a window.
pub struct Headless;

impl Headless {
    /// Like [`Self::frame_raw`], with the texture deltas discarded: egui panics when a delta is dropped unapplied.
    pub fn frame(ctx: &Context, size: [f32; 2], events: Vec<Event>, ui: impl FnMut(&mut Ui)) -> FullOutput {
        let mut output = Self::frame_raw(ctx, size, events, ui);
        output.textures_delta.clear();
        output
    }

    /// Runs `ui` for one frame on a `size` screen at 1.0 pixels per point, feeding `events`. The caller owns the texture deltas.
    pub fn frame_raw(ctx: &Context, size: [f32; 2], events: Vec<Event>, ui: impl FnMut(&mut Ui)) -> FullOutput {
        let mut input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(size[0], size[1]))),
            events,
            ..Default::default()
        };
        input.viewports.entry(input.viewport_id).or_default().native_pixels_per_point = Some(1.0);
        ctx.run_ui(input, ui)
    }
}
