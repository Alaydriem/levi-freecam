use std::sync::Arc;

use egui::{Align2, Galley, Rect, Ui, pos2, vec2};
use freecam_core::structs::UiLegend;

use super::Strip;
use crate::widgets::BarDraw;
use studio_ui::theme::{Metrics, StudioPalette, Typeface};
use studio_ui::widgets::Draw;

/// The key legend under the strip: one key cap and one label per row.
pub struct Legend;

impl Legend {
    /// Rows past this many are not drawn. The legend core builds has 5, one of them shown only in bar mode.
    pub const MAX_ROWS: usize = 8;

    const GAP_BELOW_STRIP: f32 = 8.0;
    const ROW: f32 = 26.0;
    const PAD: f32 = 10.0;
    const CAP_HEIGHT: f32 = 22.0;
    const CAP_PAD: f32 = 14.0;
    const LABEL_GAP: f32 = 10.0;
    const KEY_SIZE: f32 = 12.0;
    const LABEL_SIZE: f32 = 13.5;

    /// The height of a legend with `rows` rows, in points.
    pub fn height(rows: usize) -> f32 {
        Self::PAD * 2.0 + Self::ROW * rows.min(Self::MAX_ROWS) as f32 - (Self::ROW - Self::CAP_HEIGHT)
    }

    /// The distance from the top of the screen to the bottom of a legend with `rows` rows, in points.
    pub fn bottom(rows: usize) -> f32 {
        Strip::bottom() + Self::GAP_BELOW_STRIP + Self::height(rows)
    }

    /// Paints the rows shown in this mode and returns the legend's rect. A legend with no row to show paints nothing
    /// and returns [`Rect::NOTHING`].
    pub fn show(ui: &Ui, legend: &UiLegend, cursor_mode: bool) -> Rect {
        let shown = legend.rows.iter().enumerate().filter(|(index, _)| legend.shown(*index, cursor_mode));
        let rows = shown.clone().count().min(Self::MAX_ROWS);
        if rows == 0 {
            return Rect::NOTHING;
        }
        let painter = ui.painter();
        let screen = ui.ctx().content_rect();
        // each row is laid out once, then measured and painted from the same galleys
        let mut galleys: [Option<(Arc<Galley>, Arc<Galley>)>; Self::MAX_ROWS] = Default::default();
        let (mut key_w, mut label_w) = (0.0_f32, 0.0_f32);
        for (slot, (index, row)) in galleys.iter_mut().zip(shown) {
            let key = BarDraw::galley(painter, row.key.as_str(), Typeface::mono(Self::KEY_SIZE), StudioPalette::TEXT);
            let label = legend.label(index, cursor_mode);
            let label = BarDraw::galley(painter, label, Typeface::ui(Self::LABEL_SIZE), StudioPalette::TEXT_DIM);
            key_w = key_w.max(key.size().x);
            label_w = label_w.max(label.size().x);
            *slot = Some((key, label));
        }
        let key_w = key_w + Self::CAP_PAD;
        let top_left = pos2(screen.left() + Strip::MARGIN, screen.top() + Strip::bottom() + Self::GAP_BELOW_STRIP);
        let size = vec2(Self::PAD * 2.0 + key_w + Self::LABEL_GAP + label_w, Self::height(rows));
        let rect = Rect::from_min_size(top_left, size);
        Draw::overlay_panel(painter, rect);
        for (index, (key, label)) in galleys.into_iter().flatten().enumerate() {
            let y = rect.top() + Self::PAD + index as f32 * Self::ROW;
            let cap = Rect::from_min_size(pos2(rect.left() + Self::PAD, y), vec2(key_w, Self::CAP_HEIGHT));
            Draw::fill(painter, cap, Metrics::RADIUS_SMALL, StudioPalette::alpha(StudioPalette::BG_FIELD, 0.9));
            Draw::outline(painter, cap, Metrics::RADIUS_SMALL, 1.0, StudioPalette::LINE_STRONG);
            Draw::place(painter, key, cap.center(), Align2::CENTER_CENTER);
            let label_pos = pos2(cap.right() + Self::LABEL_GAP, cap.center().y);
            Draw::place(painter, label, label_pos, Align2::LEFT_CENTER);
        }
        rect
    }
}
