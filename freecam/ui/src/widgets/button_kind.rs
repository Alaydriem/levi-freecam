use egui::Color32;

use crate::theme::Palette;

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
            Self::Primary => (Palette::ACCENT, Palette::ACCENT_HOVER, Palette::ACCENT_PRESS, Palette::WHITE),
            Self::Secondary => (Palette::BG_FIELD, Palette::BG_HOVER, Palette::BG_ACTIVE, Palette::TEXT),
            Self::Outline => (Color32::TRANSPARENT, Palette::BG_HOVER, Palette::BG_ACTIVE, Palette::TEXT),
        }
    }
}
