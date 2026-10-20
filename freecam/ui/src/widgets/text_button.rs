use egui::{Align2, AsIdSalt, Color32, CursorIcon, IdSalt, Rect, Response, Sense, Ui, pos2, vec2};

use super::{ButtonKind, ButtonState, Draw, Motion};
use crate::theme::{Metrics, Palette, Typeface};

/// A labelled button with an optional leading icon, a weight from [`ButtonKind`] and a hover and press transition.
pub struct TextButton {
    label: String,
    salt: IdSalt,
    state: ButtonState,
    kind: ButtonKind,
    icon: Option<char>,
}

impl TextButton {
    const FONT_SIZE: f32 = 14.0;
    const ICON_SIZE: f32 = 15.0;
    const ICON_GAP: f32 = 8.0;

    /// `label` is moved into the text layout, so an owned `String` costs no extra copy. `salt` keeps the id stable when
    /// the label text changes.
    pub fn new(label: impl Into<String>, salt: impl AsIdSalt) -> Self {
        Self::with_state(label, salt, ButtonState::Normal)
    }

    /// Like [`Self::new`], with a look and click behaviour from `state`.
    pub fn with_state(label: impl Into<String>, salt: impl AsIdSalt, state: ButtonState) -> Self {
        Self::styled(label, salt, state, ButtonKind::Secondary, None)
    }

    /// Like [`Self::with_state`], with a weight and an optional icon before the label.
    pub fn styled(
        label: impl Into<String>,
        salt: impl AsIdSalt,
        state: ButtonState,
        kind: ButtonKind,
        icon: Option<char>,
    ) -> Self {
        Self { label: label.into(), salt: IdSalt::new(salt), state, kind, icon }
    }

    pub fn show(self, ui: &mut Ui, rect: Rect) -> Response {
        let id = ui.scope_id().with_salt(self.salt).with(("text-button", rect.min.x as i32, rect.min.y as i32));
        let disabled = self.state == ButtonState::Disabled;
        let sense = if disabled { Sense::hover() } else { Sense::click() };
        let mut response = ui.interact(rect, id, sense);
        if !disabled {
            response = response.on_hover_cursor(CursorIcon::PointingHand);
        }
        let (hover, press) = if disabled { (0.0, 0.0) } else { Motion::states(ui.ctx(), &response) };
        let painter = ui.painter();

        let (rest, hovered, pressed, text_color) = self.kind.colors();
        let radius = Metrics::RADIUS_CONTROL;
        if self.kind == ButtonKind::Primary {
            Draw::shadow(painter, rect, radius, 6.0, vec2(0.0, 2.0), Palette::alpha(rest, 0.25 + 0.15 * hover));
        }
        Draw::fill(painter, rect, radius, Palette::mix(Palette::mix(rest, hovered, hover), pressed, press));
        match (self.state, self.kind) {
            (ButtonState::Armed, _) => Draw::outline(painter, rect, radius, 1.5, Palette::ACCENT_HOVER),
            (_, ButtonKind::Outline) => {
                Draw::outline(painter, rect, radius, 1.0, Palette::mix(Palette::LINE_STRONG, Palette::BG_ACTIVE, hover))
            }
            (_, ButtonKind::Primary) => Draw::outline(painter, rect, radius, 1.0, Color32::from_white_alpha(18)),
            _ => {}
        }

        let color = if disabled { Palette::TEXT_DIM } else { text_color };
        let galley = Draw::galley(painter, self.label, Typeface::semibold(Self::FONT_SIZE), color);
        let lead = if self.icon.is_some() { Self::ICON_SIZE + Self::ICON_GAP } else { 0.0 };
        let left = rect.center().x - (galley.size().x + lead) * 0.5;
        let cy = rect.center().y;
        if let Some(icon) = self.icon {
            Draw::icon(painter, pos2(left + Self::ICON_SIZE * 0.5, cy), icon, Self::ICON_SIZE, color);
        }
        Draw::place(painter, galley, pos2(left + lead, cy), Align2::LEFT_CENTER);
        response
    }
}
