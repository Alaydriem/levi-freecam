use egui::Color32;

/// The colours only freecam draws, next to the levi-ui tokens.
pub struct FreecamPalette;

impl FreecamPalette {
    /// Path points, matched to the in-world camera boxes: blue for the rest, green for the selected or last point.
    pub const POINT: Color32 = Color32::from_rgb(0x3D, 0xA5, 0xFF);
    pub const POINT_SELECTED: Color32 = Color32::from_rgb(0x4B, 0xE3, 0x5A);
}
