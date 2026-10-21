use egui::Color32;

use studio_ui::theme::StudioPalette;

/// Visual weight of a [`super::TextButton`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    /// Accent fill, for the main action.
    Primary,
    Secondary,
    /// No fill at rest, with a hairline outline.
    Outline,
}

impl ButtonKind {
    /// (rest fill, hover fill, press fill, text colour)
    pub fn colors(self) -> (Color32, Color32, Color32, Color32) {
        match self {
            Self::Primary => {
                (StudioPalette::ACCENT, StudioPalette::ACCENT_HOVER, StudioPalette::ACCENT_PRESS, StudioPalette::WHITE)
            }
            Self::Secondary => {
                (StudioPalette::BG_FIELD, StudioPalette::BG_HOVER, StudioPalette::BG_ACTIVE, StudioPalette::TEXT)
            }
            Self::Outline => {
                (Color32::TRANSPARENT, StudioPalette::BG_HOVER, StudioPalette::BG_ACTIVE, StudioPalette::TEXT)
            }
        }
    }
}
