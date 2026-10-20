use egui::Color32;

/// "Studio Dark" color tokens.
pub struct Palette;

impl Palette {
    pub const BG_APP: Color32 = Color32::from_rgb(0x0B, 0x0C, 0x0F);
    pub const BG_PANEL: Color32 = Color32::from_rgb(0x12, 0x14, 0x19);
    pub const BG_RAISED: Color32 = Color32::from_rgb(0x19, 0x1C, 0x23);
    pub const BG_FIELD: Color32 = Color32::from_rgb(0x20, 0x24, 0x2D);
    pub const BG_HOVER: Color32 = Color32::from_rgb(0x28, 0x2D, 0x38);
    /// Active segment of a segmented control: one step above `bg.hover`.
    pub const BG_ACTIVE: Color32 = Color32::from_rgb(0x30, 0x36, 0x43);
    /// Dark translucent fill of panels drawn over the game picture.
    pub const BG_OVERLAY: Color32 = Color32::from_rgba_unmultiplied_const(8, 9, 12, 168);

    pub const LINE: Color32 = Color32::from_rgb(0x25, 0x2A, 0x33);
    pub const LINE_STRONG: Color32 = Color32::from_rgb(0x34, 0x3A, 0x46);
    /// Hairline around panels drawn over the game picture.
    pub const LINE_OVERLAY: Color32 = Color32::from_rgba_unmultiplied_const(255, 255, 255, 18);

    pub const TEXT: Color32 = Color32::from_rgb(0xE6, 0xE8, 0xEC);
    pub const TEXT_DIM: Color32 = Color32::from_rgb(0x91, 0x98, 0xA5);
    pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x5B, 0x62, 0x70);

    pub const ACCENT: Color32 = Color32::from_rgb(0x8B, 0x7C, 0xFF);
    pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0x9C, 0x8F, 0xFF);
    pub const ACCENT_PRESS: Color32 = Color32::from_rgb(0x7A, 0x6A, 0xF2);
    pub const ACCENT_SOFT: Color32 = Color32::from_rgba_unmultiplied_const(0x8B, 0x7C, 0xFF, 46);

    pub const PLAYHEAD: Color32 = Color32::from_rgb(0xFF, 0x4D, 0x5E);

    /// Path points, matched to the in-world camera boxes: blue for the rest, green for the selected or last point.
    pub const POINT: Color32 = Color32::from_rgb(0x3D, 0xA5, 0xFF);
    pub const POINT_SELECTED: Color32 = Color32::from_rgb(0x4B, 0xE3, 0x5A);

    pub const WHITE: Color32 = Color32::WHITE;
    pub const SHADOW: Color32 = Color32::from_rgba_unmultiplied_const(0, 0, 0, 150);

    /// `color` at `alpha` (0..1), keeping its hue.
    pub fn alpha(color: Color32, alpha: f32) -> Color32 {
        let [r, g, b, a] = color.to_srgba_unmultiplied();
        let scaled = (f32::from(a) * alpha.clamp(0.0, 1.0)).round() as u8;
        Color32::from_rgba_unmultiplied(r, g, b, scaled)
    }

    /// Linear blend in gamma space, `t` = 0 gives `from`.
    pub fn mix(from: Color32, to: Color32, t: f32) -> Color32 {
        from.lerp_to_gamma(to, t.clamp(0.0, 1.0))
    }
}
