use egui::{Color32, Rect, Ui, vec2};
use freecam_core::structs::{UiAction, UiSnapshot};

use super::{BarControls, BarLayout, BarText, Scrubber};
use crate::theme::{Icon, Palette};
use crate::widgets::{Draw, IconButton};

/// The playback bar: a floating pill at the bottom centre, shown in bar mode while no path plays.
pub struct PlaybackBar;

impl PlaybackBar {
    const RADIUS: u8 = 14;
    const DIVIDER_REACH: f32 = 16.0;

    pub fn is_shown(snapshot: &UiSnapshot) -> bool {
        snapshot.cursor_mode && !snapshot.playing
    }

    /// Paints the bar when it is shown and pushes the actions clicked. Returns the bar rect, or [`Rect::NOTHING`] when
    /// it is hidden. `text` keeps the read-outs laid out between frames.
    pub fn show(ui: &mut Ui, snapshot: &UiSnapshot, text: &mut BarText, actions: &mut Vec<UiAction>) -> Rect {
        if !Self::is_shown(snapshot) {
            return Rect::NOTHING;
        }
        let layout = BarLayout::new(ui.ctx().content_rect());
        Self::pill(ui, &layout);
        BarControls::transport(ui, &layout, snapshot, text, actions);
        BarControls::speed_chip(ui, &layout, snapshot, text);
        Scrubber::show(ui, layout.track, snapshot, text, actions);
        if IconButton::new(Icon::CAMERA, "screenshot", 17.0).show(ui, layout.shot).clicked() {
            actions.push(UiAction::Screenshot);
        }
        match snapshot.selection.filter(|index| *index < snapshot.point_count as usize) {
            Some(index) => BarControls::selected(ui, &layout, index, text, actions),
            None => BarControls::idle(ui, &layout, snapshot, text, actions),
        }
        layout.bar
    }

    fn pill(ui: &Ui, layout: &BarLayout) {
        let painter = ui.painter();
        let rect = layout.bar;
        Draw::shadow(painter, rect, Self::RADIUS, 24.0, vec2(0.0, 8.0), Palette::alpha(Palette::SHADOW, 0.9));
        Draw::fill(painter, rect, Self::RADIUS, Palette::alpha(Palette::BG_PANEL, 0.92));
        Draw::outline(painter, rect, Self::RADIUS, 1.0, Color32::from_white_alpha(20));
        let cy = rect.center().y;
        let (top, bottom) = (cy - Self::DIVIDER_REACH, cy + Self::DIVIDER_REACH);
        Draw::vline(painter, layout.divider_x, top, bottom, Palette::LINE_STRONG);
        Draw::vline(painter, layout.shot_divider_x, top, bottom, Palette::LINE_STRONG);
    }
}
