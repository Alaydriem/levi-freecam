use std::fmt::Write;

use egui::{Align2, Color32, CursorIcon, Pos2, Rect, Sense, Stroke, Ui, pos2, vec2};
use freecam_core::structs::{UiAction, UiSnapshot};

use super::{BarLayout, BarText, Timecode};
use crate::theme::{Icon, Metrics, Palette, Typeface};
use crate::widgets::{ButtonKind, ButtonState, Draw, IconButton, Motion, TextButton};

/// The bar's buttons and read-outs around the scrubber.
pub struct BarControls;

impl BarControls {
    const CLEAR_LABEL: &'static str = "Clear points";
    const CLEAR_ARMED_LABEL: &'static str = "Click again to clear";

    /// Rewind, play and the timecode.
    pub fn transport(
        ui: &mut Ui,
        layout: &BarLayout,
        snapshot: &UiSnapshot,
        text: &mut BarText,
        actions: &mut Vec<UiAction>,
    ) {
        if IconButton::new(Icon::PREVIOUS, "rewind", 15.0).show(ui, layout.rewind).clicked() {
            actions.push(UiAction::Rewind);
        }
        if Self::play_button(ui, layout.play) {
            actions.push(UiAction::Play);
        }
        Self::timecode(ui, pos2(layout.timecode_x, layout.bar.center().y), snapshot, text);
    }

    /// The read-only speed chip: playback speed follows the scroll wheel.
    pub fn speed_chip(ui: &Ui, layout: &BarLayout, snapshot: &UiSnapshot, text: &mut BarText) {
        let painter = ui.painter();
        let rect = layout.speed;
        let radius = (rect.height() * 0.5) as u8;
        let cy = rect.center().y;
        Draw::fill(painter, rect, radius, Palette::alpha(Palette::BG_FIELD, 0.9));
        Draw::outline(painter, rect, radius, 1.0, Palette::LINE_STRONG);
        Draw::caps(painter, pos2(rect.left() + 12.0, cy), Align2::LEFT_CENTER, "SPEED", Palette::TEXT_FAINT);
        let generation = text.generation();
        let value = text.speed.galley(painter, generation, |out| write!(out, "{:.1} b/s", snapshot.speed));
        Draw::place_tinted(painter, value, pos2(rect.right() - 12.0, cy), Align2::RIGHT_CENTER, Palette::TEXT);
    }

    /// Clear points, Add point and the point count, shown with no selection.
    pub fn idle(
        ui: &mut Ui,
        layout: &BarLayout,
        snapshot: &UiSnapshot,
        text: &mut BarText,
        actions: &mut Vec<UiAction>,
    ) {
        let armed = snapshot.clear_armed && snapshot.point_count > 0;
        let state = match (snapshot.point_count, armed) {
            (0, _) => ButtonState::Disabled,
            (_, true) => ButtonState::Armed,
            (_, false) => ButtonState::Normal,
        };
        let clear = IconButton::with_state(Icon::CLEAR, "clear-points", 15.0, state).show(ui, layout.clear);
        if clear.clicked() {
            actions.push(UiAction::ClearPoints);
        }
        let tip_anchor = pos2(layout.clear.center().x, layout.bar.top());
        if armed {
            Self::tip(ui, tip_anchor, Self::CLEAR_ARMED_LABEL);
        } else if clear.hovered() {
            Self::tip(ui, tip_anchor, Self::CLEAR_LABEL);
        }
        let add =
            TextButton::styled("Add point", "add-point", ButtonState::Normal, ButtonKind::Primary, Some(Icon::ADD));
        if add.show(ui, layout.add).clicked() {
            actions.push(UiAction::AddPoint);
        }
        let painter = ui.painter();
        let cy = layout.bar.center().y;
        let generation = text.generation();
        let count = text.count.galley(painter, generation, |out| match snapshot.point_count {
            1 => out.write_str("1 point"),
            count => write!(out, "{count} points"),
        });
        Draw::place_tinted(painter, count, pos2(layout.count_x, cy - 8.0), Align2::LEFT_CENTER, Palette::TEXT);
        let hint_pos = pos2(layout.count_x, cy + 10.0);
        Draw::text(
            painter,
            hint_pos,
            Align2::LEFT_CENTER,
            "Adds at the playhead",
            Typeface::ui(12.5),
            Palette::TEXT_FAINT,
        );
    }

    /// "Point N", Re-place and Delete, shown with a selection.
    pub fn selected(ui: &mut Ui, layout: &BarLayout, index: usize, text: &mut BarText, actions: &mut Vec<UiAction>) {
        Self::point_tag(ui, layout.tag, index, text);
        let replace =
            TextButton::styled("Re-place", "replace", ButtonState::Normal, ButtonKind::Secondary, Some(Icon::REFRESH));
        if replace.show(ui, layout.replace).clicked() {
            actions.push(UiAction::Replace);
        }
        let delete =
            TextButton::styled("Delete", "delete", ButtonState::Normal, ButtonKind::Outline, Some(Icon::DELETE));
        if delete.show(ui, layout.delete).clicked() {
            actions.push(UiAction::Delete);
        }
    }

    // returns true on the frame it is clicked
    fn play_button(ui: &mut Ui, rect: Rect) -> bool {
        let response =
            ui.interact(rect, ui.scope_id().with("play"), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
        let (hover, press) = Motion::states(ui.ctx(), &response);
        let painter = ui.painter();
        let center = rect.center();
        let radius = rect.width() * 0.5;
        let glow = Palette::alpha(Palette::ACCENT, 0.28 + 0.12 * hover);
        Draw::glow(painter, center + vec2(0.0, 1.0), radius + 2.0 + hover * 2.0, glow);
        let fill =
            Palette::mix(Palette::mix(Palette::ACCENT, Palette::ACCENT_HOVER, hover), Palette::ACCENT_PRESS, press);
        painter.circle_filled(center, radius - press * 0.8, fill);
        painter.circle_stroke(center, radius - 0.5, Stroke::new(1.0, Color32::from_white_alpha(22)));
        // the play triangle reads as centred one point right of the true centre
        Draw::icon(painter, center + vec2(1.0, 0.0), Icon::PLAY, 16.0, Palette::WHITE);
        response.clicked()
    }

    fn timecode(ui: &Ui, left_center: Pos2, snapshot: &UiSnapshot, text: &mut BarText) {
        let painter = ui.painter();
        let generation = text.generation();
        let main =
            text.timecode.galley(painter, generation, |out| Timecode::write_short(out, snapshot.playhead_seconds));
        let total = text.total.galley(painter, generation, |out| {
            out.write_str(" / ")?;
            Timecode::write_short(out, snapshot.total_seconds)
        });
        let main_rect = Draw::place_tinted(painter, main, left_center, Align2::LEFT_CENTER, Palette::TEXT);
        let total_pos = pos2(main_rect.right() + 1.0, main_rect.bottom() - 3.0);
        Draw::place_tinted(painter, total, total_pos, Align2::LEFT_BOTTOM, Palette::TEXT_DIM);
    }

    fn point_tag(ui: &Ui, rect: Rect, index: usize, text: &mut BarText) {
        let painter = ui.painter();
        let cy = rect.center().y;
        let radius = Metrics::RADIUS_CONTROL;
        Draw::fill(painter, rect, radius, Palette::alpha(Palette::POINT_SELECTED, 0.12));
        Draw::outline(painter, rect, radius, 1.0, Palette::alpha(Palette::POINT_SELECTED, 0.45));
        Draw::diamond(
            painter,
            pos2(rect.left() + 18.0, cy),
            5.0,
            Palette::POINT_SELECTED,
            Stroke::new(1.0, Palette::BG_APP),
        );
        let generation = text.generation();
        let tag = text.tag.galley(painter, generation, |out| write!(out, "Point {}", index + 1));
        Draw::place_tinted(painter, tag, pos2(rect.left() + 30.0, cy), Align2::LEFT_CENTER, Palette::TEXT);
    }

    // a small label centred above `anchor`, with a gap
    fn tip(ui: &Ui, anchor: Pos2, text: &str) {
        let painter = ui.painter();
        let galley = Draw::galley(painter, text, Typeface::ui(12.5), Palette::TEXT);
        let size = galley.size() + vec2(16.0, 8.0);
        let rect = Rect::from_min_size(pos2(anchor.x - size.x * 0.5, anchor.y - 12.0 - size.y), size);
        Draw::shadow(painter, rect, Metrics::RADIUS_CONTROL, 8.0, vec2(0.0, 2.0), Palette::SHADOW);
        Draw::fill(painter, rect, Metrics::RADIUS_CONTROL, Palette::BG_RAISED);
        Draw::outline(painter, rect, Metrics::RADIUS_CONTROL, 1.0, Palette::LINE_STRONG);
        Draw::place(painter, galley, rect.center(), Align2::CENTER_CENTER);
    }
}
