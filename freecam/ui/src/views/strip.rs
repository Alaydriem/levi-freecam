use egui::{Align2, Rect, Ui, pos2, vec2};
use freecam_core::structs::UiSnapshot;

use crate::widgets::BarDraw;
use studio_ui::theme::{StudioPalette, Typeface};
use studio_ui::widgets::Draw;

/// The chip at the top left: title, point count and shot length.
pub struct Strip;

impl Strip {
    /// Distance from the screen edges, in points.
    pub const MARGIN: f32 = 24.0;
    pub const HEIGHT: f32 = 30.0;

    /// The bottom edge of the strip, in points from the top of the screen.
    pub const fn bottom() -> f32 {
        Self::MARGIN + Self::HEIGHT
    }

    /// Paints the strip and returns its rect.
    pub fn show(ui: &Ui, snapshot: &UiSnapshot) -> Rect {
        let painter = ui.painter();
        let screen = ui.ctx().content_rect();
        let title = BarDraw::tracked(painter, "FREECAM", Typeface::semibold(12.5), StudioPalette::TEXT, 1.0);
        let detail =
            format!("PATH  \u{00B7}  {} points  \u{00B7}  {:.2} s", snapshot.point_count, snapshot.total_seconds);
        let detail = BarDraw::galley(painter, detail, Typeface::ui(13.5), StudioPalette::TEXT_DIM);
        let width = 14.0 + 8.0 + 8.0 + title.size().x + 12.0 + detail.size().x + 16.0;
        let origin = pos2(screen.left() + Self::MARGIN, screen.top() + Self::MARGIN);
        let rect = Rect::from_min_size(origin, vec2(width, Self::HEIGHT));
        Draw::chip_bg(painter, rect);
        let cy = rect.center().y;
        let dot = pos2(rect.left() + 16.0, cy);
        BarDraw::glow(painter, dot, 6.0, StudioPalette::alpha(StudioPalette::ACCENT, 0.5));
        painter.circle_filled(dot, 4.0, StudioPalette::ACCENT_HOVER);
        let title_rect = Draw::place(painter, title, pos2(dot.x + 12.0, cy), Align2::LEFT_CENTER);
        Draw::place(painter, detail, pos2(title_rect.right() + 12.0, cy), Align2::LEFT_CENTER);
        rect
    }
}
