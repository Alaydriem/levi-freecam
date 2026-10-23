use egui::{Align2, CursorIcon, Rect, Response, Sense, Ui};
use levi_ui::draw::{Corner, Draw};
use levi_ui::font::FontFace;
use levi_ui::state::{DisabledReason, InteractionState, StateStyle};
use levi_ui::tokens::{ColorToken, TextSizeToken, color, text_size};
use levi_ui::widget::{IconButton, TextButton};

use crate::id::FreecamUiId;

/// Puts levi-ui buttons in the fixed rects of the bar (`BarLayout`): each one takes clicks over its rect and paints
/// itself there. A button with a `DisabledReason` takes no clicks and shows the reason when hovered.
pub struct BarButton;

impl BarButton {
    pub fn icon(
        ui: &mut Ui,
        rect: Rect,
        id: FreecamUiId,
        button: &IconButton,
        disabled: Option<DisabledReason>,
    ) -> Response {
        let (response, state) = Self::interact(ui, rect, id, disabled.is_none());
        button.paint(ui.painter(), rect, state);
        Self::finish(response, disabled)
    }

    pub fn text(
        ui: &mut Ui,
        rect: Rect,
        id: FreecamUiId,
        button: &TextButton<'_>,
        disabled: Option<DisabledReason>,
    ) -> Response {
        let (response, state) = Self::interact(ui, rect, id, disabled.is_none());
        button.paint(ui.painter(), rect, state);
        Self::finish(response, disabled)
    }

    /// An icon button for a Segoe Fluent Icons glyph that has no `Icon`: the levi-ui icon button's look, with the
    /// glyph painted in the icon face.
    pub fn glyph(
        ui: &mut Ui,
        rect: Rect,
        id: FreecamUiId,
        glyph: char,
        on: bool,
        disabled: Option<DisabledReason>,
    ) -> Response {
        let (response, state) = Self::interact(ui, rect, id, disabled.is_none());
        let painter = state.painter(ui.painter());
        let style = if on { StateStyle::Soft } else { StateStyle::Clear };
        Draw::fill(&painter, rect, Corner::Control, style.fill(state));
        let glyph_color = match (on, state) {
            (true, _) => <color::AccentHover as ColorToken>::VALUE,
            (false, InteractionState::Hover | InteractionState::Pressed | InteractionState::Focused) => {
                <color::Text as ColorToken>::VALUE
            }
            (false, InteractionState::Rest | InteractionState::Disabled) => <color::TextDim as ColorToken>::VALUE,
        };
        let font = FontFace::Icons.id(<text_size::Icon as TextSizeToken>::VALUE);
        painter.text(rect.center(), Align2::CENTER_CENTER, glyph, font, glyph_color);
        Self::finish(response, disabled)
    }

    fn interact(ui: &mut Ui, rect: Rect, id: FreecamUiId, enabled: bool) -> (Response, InteractionState) {
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let response = ui.interact(rect, ui.scope_id().with(id), sense);
        let state = InteractionState::of(&response, enabled);
        (response, state)
    }

    fn finish(response: Response, disabled: Option<DisabledReason>) -> Response {
        match disabled {
            None => response.on_hover_cursor(CursorIcon::PointingHand),
            Some(reason) => reason.tooltip().attach(response),
        }
    }
}
