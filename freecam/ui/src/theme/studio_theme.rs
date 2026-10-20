use egui::{
    Context, CornerRadius, FontDefinitions, Stroke, Visuals,
    style::{Selection, WidgetVisuals},
};

use super::{FontLoader, Metrics, Palette};

/// Installs the Studio Dark look into an egui context.
pub struct StudioTheme;

impl StudioTheme {
    /// Sets fonts, dark visuals and the hover animation time. Call once when the context is created, never inside a frame.
    pub fn install(ctx: &Context) {
        Self::install_with(ctx, FontLoader::definitions());
    }

    /// Same as [`Self::install`] with fonts that are already loaded, so no file is read.
    pub fn install_with(ctx: &Context, fonts: FontDefinitions) {
        ctx.set_fonts(fonts);
        ctx.set_visuals(Self::visuals());
        ctx.global_style_mut(|style| style.animation_time = Metrics::MOTION_SECONDS);
        // the shim owns pixels_per_point; a keyboard zoom would change egui's scale behind it
        ctx.options_mut(|options| options.zoom_with_keyboard = false);
    }

    fn visuals() -> Visuals {
        let mut visuals = Visuals::dark();
        visuals.panel_fill = Palette::BG_PANEL;
        visuals.window_fill = Palette::BG_PANEL;
        visuals.extreme_bg_color = Palette::BG_APP;
        visuals.faint_bg_color = Palette::BG_RAISED;
        visuals.override_text_color = Some(Palette::TEXT);
        visuals.selection = Selection { bg_fill: Palette::ACCENT_SOFT, stroke: Stroke::new(1.0, Palette::ACCENT) };
        visuals.window_stroke = Stroke::new(1.0, Palette::LINE);
        let radius = CornerRadius::same(Metrics::RADIUS_CONTROL);
        visuals.widgets.noninteractive = Self::widget(Palette::BG_PANEL, Palette::LINE, Palette::TEXT_DIM, radius);
        visuals.widgets.inactive = Self::widget(Palette::BG_FIELD, Palette::LINE, Palette::TEXT, radius);
        visuals.widgets.hovered = Self::widget(Palette::BG_HOVER, Palette::LINE_STRONG, Palette::TEXT, radius);
        visuals.widgets.active = Self::widget(Palette::BG_ACTIVE, Palette::LINE_STRONG, Palette::TEXT, radius);
        visuals.widgets.open = Self::widget(Palette::BG_ACTIVE, Palette::LINE_STRONG, Palette::TEXT, radius);
        visuals
    }

    fn widget(fill: egui::Color32, line: egui::Color32, text: egui::Color32, radius: CornerRadius) -> WidgetVisuals {
        WidgetVisuals {
            bg_fill: fill,
            weak_bg_fill: fill,
            bg_stroke: Stroke::new(1.0, line),
            fg_stroke: Stroke::new(1.0, text),
            corner_radius: radius,
            expansion: 0.0,
        }
    }
}
