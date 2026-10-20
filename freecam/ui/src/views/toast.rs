use std::fmt;

use egui::{Align2, Rect, Ui, pos2, vec2};
use freecam_core::structs::{ToastKind, UiSnapshot};

use super::{BarLayout, BarText};
use crate::theme::Palette;
use crate::widgets::Draw;

/// The screenshot message above the bar.
pub struct Toast;

impl Toast {
    const HEIGHT: f32 = 30.0;
    const GAP_ABOVE_BAR: f32 = 12.0;

    /// The text for the snapshot's toast, or None when no toast shows. A file name that is not UTF-8 shows as empty.
    pub fn message(snapshot: &UiSnapshot) -> Option<String> {
        if snapshot.toast == ToastKind::None {
            return None;
        }
        let mut text = String::new();
        if Self::write_message(snapshot, &mut text).is_err() {
            text.clear();
        }
        Some(text)
    }

    /// Writes [`Self::message`] into `out` without allocating; writes nothing when no toast shows.
    pub fn write_message(snapshot: &UiSnapshot, out: &mut impl fmt::Write) -> fmt::Result {
        match snapshot.toast {
            ToastKind::None => Ok(()),
            ToastKind::Failed => out.write_str("Screenshot failed"),
            ToastKind::Saved => {
                let len = usize::from(snapshot.toast_file_len).min(snapshot.toast_file.len());
                let name = std::str::from_utf8(&snapshot.toast_file[..len]).unwrap_or("");
                write!(out, "Screenshot saved: {name}")
            }
        }
    }

    /// Paints the toast centred above where the bar sits. Returns its rect, or [`Rect::NOTHING`] with no toast.
    pub fn show(ui: &Ui, snapshot: &UiSnapshot, text: &mut BarText) -> Rect {
        if snapshot.toast == ToastKind::None {
            return Rect::NOTHING;
        }
        let painter = ui.painter();
        let layout = BarLayout::new(ui.ctx().content_rect());
        let galley = text.toast.galley(painter, text.generation(), |out| Self::write_message(snapshot, out));
        let width = 16.0 + 8.0 + 10.0 + galley.size().x + 16.0;
        let bottom = layout.bar.top() - Self::GAP_ABOVE_BAR;
        let rect = Rect::from_min_size(
            pos2(layout.bar.center().x - width * 0.5, bottom - Self::HEIGHT),
            vec2(width, Self::HEIGHT),
        );
        Draw::chip_bg(painter, rect);
        let dot_color = if snapshot.toast == ToastKind::Saved { Palette::POINT_SELECTED } else { Palette::PLAYHEAD };
        let dot = pos2(rect.left() + 20.0, rect.center().y);
        painter.circle_filled(dot, 4.0, dot_color);
        Draw::place_tinted(painter, galley, pos2(dot.x + 14.0, dot.y), Align2::LEFT_CENTER, Palette::TEXT);
        rect
    }
}
