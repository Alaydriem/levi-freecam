use egui::{Context, Id, Response};

use crate::theme::Metrics;

/// 120 ms ease-out transitions for hover and press.
pub struct Motion;

impl Motion {
    pub fn flag(ctx: &Context, id: Id, on: bool) -> f32 {
        ctx.animate_bool_with_time_and_easing(id, on, Metrics::MOTION_SECONDS, egui::emath::easing::cubic_out)
    }

    /// (hover, press) amounts for a response.
    pub fn states(ctx: &Context, response: &Response) -> (f32, f32) {
        let hover = Self::flag(ctx, response.id.with("hover"), response.hovered() || response.has_focus());
        let press = Self::flag(ctx, response.id.with("press"), response.is_pointer_button_down_on());
        (hover, press)
    }
}
