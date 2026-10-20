use egui::{Context, FontFamily};
use freecam_ui::theme::{StudioTheme, Typeface};

use crate::support::headless::Headless;

#[test]
fn install_registers_font_families() {
    let ctx = Context::default();

    StudioTheme::install(&ctx);
    Headless::frame(&ctx, [200.0, 100.0], Vec::new(), |_| {});

    let families = ctx.fonts(|fonts| fonts.definitions().families.clone());
    for name in [Typeface::SEMIBOLD, Typeface::MONO, Typeface::ICONS] {
        let chain = families.get(&FontFamily::Name(name.into()));
        assert!(chain.is_some_and(|chain| !chain.is_empty()), "family {name} missing or empty");
    }
}
