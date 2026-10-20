use std::sync::Arc;

use egui::epaint::{Mesh, RectShape};
use egui::text::{LayoutJob, TextFormat};
use egui::{Align2, Color32, CornerRadius, FontId, Galley, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2, pos2};

use crate::theme::{Metrics, Palette, Typeface};

/// Low-level painting helpers shared by all views.
pub struct Draw;

impl Draw {
    const CAPS_SIZE: f32 = 11.5;
    const CAPS_SPACING: f32 = 0.7;

    /// Lays out one line of text. Measure and paint the same galley: each layout copies the text.
    pub fn galley(painter: &Painter, text: impl Into<String>, font: FontId, color: Color32) -> Arc<Galley> {
        painter.layout_no_wrap(text.into(), font, color)
    }

    /// Paints a laid-out galley with its `anchor` at `pos`; returns its rect.
    pub fn place(painter: &Painter, galley: Arc<Galley>, pos: Pos2, anchor: Align2) -> Rect {
        let rect = anchor.anchor_size(pos, galley.size());
        painter.galley(rect.min, galley, Palette::TEXT);
        rect
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

    /// UPPERCASE label with tracking; `text` is given in upper case.
    pub fn caps(painter: &Painter, pos: Pos2, anchor: Align2, text: &str, color: Color32) -> Rect {
        let galley = Self::tracked(painter, text, Typeface::semibold(Self::CAPS_SIZE), color, Self::CAPS_SPACING);
        Self::place(painter, galley, pos, anchor)
    }

    pub fn icon(painter: &Painter, center: Pos2, icon: char, size: f32, color: Color32) -> Rect {
        painter.text(center, Align2::CENTER_CENTER, icon, Typeface::icon(size), color)
    }

    /// A one-pixel vertical line, snapped to the pixel centre.
    pub fn vline(painter: &Painter, x: f32, y0: f32, y1: f32, color: Color32) {
        let x = x.floor() + 0.5;
        painter.line_segment([pos2(x, y0), pos2(x, y1)], Stroke::new(1.0, color));
    }

    /// Soft drop shadow under a rounded rect.
    pub fn shadow(painter: &Painter, rect: Rect, radius: u8, blur: f32, offset: Vec2, color: Color32) {
        let corner = CornerRadius::same(radius.saturating_add(blur as u8 / 2));
        let shape = RectShape::filled(rect.translate(offset).expand(blur * 0.25), corner, color).with_blur_width(blur);
        painter.add(shape);
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

    pub fn fill(painter: &Painter, rect: Rect, radius: u8, color: Color32) {
        painter.rect_filled(rect, CornerRadius::same(radius), color);
    }

    pub fn outline(painter: &Painter, rect: Rect, radius: u8, width: f32, color: Color32) {
        painter.rect_stroke(rect, CornerRadius::same(radius), Stroke::new(width, color), StrokeKind::Inside);
    }

    /// Round glow centered on a point.
    pub fn glow(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
        let rect = Rect::from_center_size(center, Vec2::splat(radius * 2.0));
        let shape = RectShape::filled(rect, CornerRadius::same(radius.min(255.0) as u8), color).with_blur_width(radius);
        painter.add(shape);
    }

    /// Translucent panel over the game picture, with a hairline.
    pub fn overlay_panel(painter: &Painter, rect: Rect) {
        Self::fill(painter, rect, Metrics::RADIUS_PANEL, Palette::BG_OVERLAY);
        Self::outline(painter, rect, Metrics::RADIUS_PANEL, 1.0, Palette::LINE_OVERLAY);
    }

    /// Pill-shaped translucent chip behind text, over the game picture.
    pub fn chip_bg(painter: &Painter, rect: Rect) {
        let radius = (rect.height() * 0.5) as u8;
        Self::fill(painter, rect, radius, Palette::BG_OVERLAY);
        Self::outline(painter, rect, radius, 1.0, Palette::LINE_OVERLAY);
    }
}
