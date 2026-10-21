use egui::{AsIdSalt, CursorIcon, IdSalt, Rect, Response, Sense, Ui};

use super::ButtonState;
use studio_ui::theme::{Metrics, StudioPalette};
use studio_ui::widgets::{Draw, Motion};

/// A square button with no fill at rest, holding one icon glyph.
pub struct IconButton {
    icon: char,
    salt: IdSalt,
    icon_size: f32,
    state: ButtonState,
}

impl IconButton {
    pub fn new(icon: char, salt: impl AsIdSalt, icon_size: f32) -> Self {
        Self::with_state(icon, salt, icon_size, ButtonState::Normal)
    }

    /// Like [`Self::new`], with a look and click behaviour from `state`.
    pub fn with_state(icon: char, salt: impl AsIdSalt, icon_size: f32, state: ButtonState) -> Self {
        Self { icon, salt: IdSalt::new(salt), icon_size, state }
    }

    pub fn show(self, ui: &mut Ui, rect: Rect) -> Response {
        let id = ui.scope_id().with_salt(self.salt).with("icon-button");
        let disabled = self.state == ButtonState::Disabled;
        let sense = if disabled { Sense::hover() } else { Sense::click() };
        let mut response = ui.interact(rect, id, sense);
        if !disabled {
            response = response.on_hover_cursor(CursorIcon::PointingHand);
        }
        let (hover, press) = if disabled { (0.0, 0.0) } else { Motion::states(ui.ctx(), &response) };
        let painter = ui.painter();

        Draw::fill(painter, rect, Metrics::RADIUS_CONTROL, StudioPalette::alpha(StudioPalette::BG_HOVER, hover));
        if press > 0.0 {
            Draw::fill(
                painter,
                rect,
                Metrics::RADIUS_CONTROL,
                StudioPalette::alpha(StudioPalette::BG_ACTIVE, press * 0.6),
            );
        }
        let color = match self.state {
            ButtonState::Disabled => StudioPalette::TEXT_FAINT,
            ButtonState::Armed => {
                Draw::outline(painter, rect, Metrics::RADIUS_CONTROL, 1.5, StudioPalette::ACCENT_HOVER);
                StudioPalette::ACCENT_HOVER
            }
            ButtonState::Normal => StudioPalette::mix(StudioPalette::TEXT_DIM, StudioPalette::TEXT, hover),
        };
        Draw::icon(painter, rect.center(), self.icon, self.icon_size, color);
        response
    }
}
