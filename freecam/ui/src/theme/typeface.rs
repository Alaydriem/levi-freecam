use std::sync::LazyLock;

use egui::{FontFamily, FontId};

/// Font ids for the views. The named families are built once, so a font id costs a refcount, not an allocation.
pub struct Typeface;

static SEMIBOLD_FAMILY: LazyLock<FontFamily> = LazyLock::new(|| FontFamily::Name(Typeface::SEMIBOLD.into()));
static MONO_FAMILY: LazyLock<FontFamily> = LazyLock::new(|| FontFamily::Name(Typeface::MONO.into()));
static ICON_FAMILY: LazyLock<FontFamily> = LazyLock::new(|| FontFamily::Name(Typeface::ICONS.into()));

impl Typeface {
    pub const SEMIBOLD: &'static str = "ui-semibold";
    pub const MONO: &'static str = "ui-mono";
    pub const ICONS: &'static str = "ui-icons";

    pub fn ui(size: f32) -> FontId {
        FontId::new(size, FontFamily::Proportional)
    }

    pub fn semibold(size: f32) -> FontId {
        FontId::new(size, SEMIBOLD_FAMILY.clone())
    }

    pub fn mono(size: f32) -> FontId {
        FontId::new(size, MONO_FAMILY.clone())
    }

    pub fn icon(size: f32) -> FontId {
        FontId::new(size, ICON_FAMILY.clone())
    }
}
