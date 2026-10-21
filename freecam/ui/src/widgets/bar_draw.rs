use std::sync::Arc;

use egui::epaint::{Mesh, RectShape};
use egui::text::{LayoutJob, TextFormat};
use egui::{Align2, Color32, CornerRadius, FontId, Galley, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2, pos2};
use studio_ui::theme::Typeface;
use studio_ui::widgets::Draw;

/// The painting helpers the bar needs that `studio_ui::widgets::Draw` does not have, or has with an allocation the
/// bar avoids.
pub struct BarDraw;

impl BarDraw {
    const CAPS_SIZE: f32 = 11.5;
    const CAPS_SPACING: f32 = 0.7;

    /// Lays out one line of text. Measure and paint the same galley: each layout copies the text.
    pub fn galley(painter: &Painter, text: impl Into<String>, font: FontId, color: Color32) -> Arc<Galley> {
        painter.layout_no_wrap(text.into(), font, color)
    }

    /// Paints a galley laid out with a placeholder colour in `color`, with its `anchor` at `pos`; returns its rect.
    pub fn place_tinted(painter: &Painter, galley: Arc<Galley>, pos: Pos2, anchor: Align2, color: Color32) -> Rect {
        let rect = anchor.anchor_size(pos, galley.size());
        painter.galley(rect.min, galley, color);
        rect
    }

    /// Tracked text in an arbitrary font (chips, badges).
    pub fn tracked(painter: &Painter, text: &str, font: FontId, color: Color32, spacing: f32) -> Arc<Galley> {
        let format = TextFormat { font_id: font, color, extra_letter_spacing: spacing, ..Default::default() };
        painter.layout_job(LayoutJob::single_section(text.to_owned(), format))
    }

    /// Paints one line of text with its `anchor` at `pos`; returns its rect.
    pub fn text(
        painter: &Painter,
        pos: Pos2,
        anchor: Align2,
        text: impl ToString,
        font: FontId,
        color: Color32,
    ) -> Rect {
        painter.text(pos, anchor, text, font, color)
    }

    /// UPPERCASE label with tracking; `text` is given in upper case, so nothing is converted per frame.
    pub fn caps(painter: &Painter, pos: Pos2, anchor: Align2, text: &str, color: Color32) -> Rect {
        let galley = Self::tracked(painter, text, Typeface::semibold(Self::CAPS_SIZE), color, Self::CAPS_SPACING);
        Draw::place(painter, galley, pos, anchor)
    }

    /// A one-pixel vertical line, snapped to the pixel centre.
    pub fn vline(painter: &Painter, x: f32, y0: f32, y1: f32, color: Color32) {
        let x = x.floor() + 0.5;
        painter.line_segment([pos2(x, y0), pos2(x, y1)], Stroke::new(1.0, color));
    }

    /// Diamond point glyph: a square turned 45 degrees with its corners `half` from `center`. A turned rect needs no
    /// point list, so a diamond allocates nothing.
    pub fn diamond(painter: &Painter, center: Pos2, half: f32, fill: Color32, stroke: Stroke) {
        let side = half * std::f32::consts::SQRT_2;
        let rect = Rect::from_center_size(center, Vec2::splat(side));
        let shape = RectShape::new(rect, CornerRadius::ZERO, fill, stroke, StrokeKind::Middle)
            .with_round_to_pixels(false)
            .with_angle(std::f32::consts::FRAC_PI_4);
        painter.add(shape);
    }

    /// Horizontal gradient from `from` at the left edge to `to` at the right edge.
    pub fn hgradient(painter: &Painter, rect: Rect, from: Color32, to: Color32) {
        let mut mesh = Mesh::default();
        mesh.colored_vertex(rect.left_top(), from);
        mesh.colored_vertex(rect.left_bottom(), from);
        mesh.colored_vertex(rect.right_top(), to);
        mesh.colored_vertex(rect.right_bottom(), to);
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(1, 2, 3);
        painter.add(Shape::mesh(mesh));
    }

    /// Round glow centered on a point.
    pub fn glow(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
        let rect = Rect::from_center_size(center, Vec2::splat(radius * 2.0));
        let shape = RectShape::filled(rect, CornerRadius::same(radius.min(255.0) as u8), color).with_blur_width(radius);
        painter.add(shape);
    }
}
