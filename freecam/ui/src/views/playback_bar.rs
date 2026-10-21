use egui::{Area, Color32, Id, Order, Rect, Ui, vec2};
use freecam_core::structs::{UiAction, UiSnapshot};

use super::{BarControls, BarLayout, BarText, Scrubber};
use crate::widgets::{BarDraw, ButtonState, IconButton};
use studio_ui::theme::{Icon, StudioPalette};
use studio_ui::widgets::Draw;

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
    ///
    /// The bar lives in its own area over exactly the bar rect: egui counts the pointer as over egui only above an
    /// area, never above the root background, and the mod manager hands a click to a layer only while egui wants it
    pub fn show(ui: &mut Ui, snapshot: &UiSnapshot, text: &mut BarText, actions: &mut Vec<UiAction>) -> Rect {
        if !Self::is_shown(snapshot) {
            return Rect::NOTHING;
        }
        let layout = BarLayout::new(ui.ctx().content_rect());
        Area::new(Id::unique("freecam.playback_bar")).order(Order::Middle).fixed_pos(layout.bar.min).show(
            ui.ctx(),
            |ui| {
                ui.set_min_size(layout.bar.size());
                Self::contents(ui, &layout, snapshot, text, actions);
            },
        );
        layout.bar
    }

    fn contents(
        ui: &mut Ui,
        layout: &BarLayout,
        snapshot: &UiSnapshot,
        text: &mut BarText,
        actions: &mut Vec<UiAction>,
    ) {
        Self::pill(ui, layout);
        BarControls::transport(ui, layout, snapshot, text, actions);
        BarControls::speed_chip(ui, layout, snapshot, text);
        Scrubber::show(ui, layout.track, snapshot, text, actions);
        let shot = if snapshot.can_screenshot { ButtonState::Normal } else { ButtonState::Disabled };
        if IconButton::with_state(Icon::CAMERA, "screenshot", 17.0, shot).show(ui, layout.shot).clicked() {
            actions.push(UiAction::Screenshot);
        }
        match snapshot.selection.filter(|index| *index < snapshot.point_count as usize) {
            Some(index) => BarControls::selected(ui, layout, index, text, actions),
            None => BarControls::idle(ui, layout, snapshot, text, actions),
        }
    }

    fn pill(ui: &Ui, layout: &BarLayout) {
        let painter = ui.painter();
        let rect = layout.bar;
        Draw::shadow(
            painter,
            rect,
            Self::RADIUS,
            24.0,
            vec2(0.0, 8.0),
            StudioPalette::alpha(StudioPalette::SHADOW, 0.9),
        );
        Draw::fill(painter, rect, Self::RADIUS, StudioPalette::alpha(StudioPalette::BG_PANEL, 0.92));
        Draw::outline(painter, rect, Self::RADIUS, 1.0, Color32::from_white_alpha(20));
        let cy = rect.center().y;
        let (top, bottom) = (cy - Self::DIVIDER_REACH, cy + Self::DIVIDER_REACH);
        BarDraw::vline(painter, layout.divider_x, top, bottom, StudioPalette::LINE_STRONG);
        BarDraw::vline(painter, layout.shot_divider_x, top, bottom, StudioPalette::LINE_STRONG);
    }
}
