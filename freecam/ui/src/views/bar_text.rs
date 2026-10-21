use egui::Context;
use freecam_core::structs::UiSnapshot;

use super::PointLabels;
use crate::widgets::{CachedText, FontWatch};
use studio_ui::theme::Typeface;

/// The bar's and the toast's changing text, kept between frames. A read-out is laid out again only when its value
/// or the fonts change, so a frame with the same values allocates no text.
pub struct BarText {
    fonts: FontWatch,
    generation: u64,
    pub(crate) timecode: CachedText,
    pub(crate) total: CachedText,
    pub(crate) speed: CachedText,
    pub(crate) count: CachedText,
    pub(crate) tag: CachedText,
    pub(crate) tip: CachedText,
    pub(crate) toast: CachedText,
    pub(crate) labels: PointLabels,
}

impl BarText {
    pub fn new() -> Self {
        let points = UiSnapshot::default().point_distances.len();
        Self {
            fonts: FontWatch::new(),
            generation: 0,
            timecode: CachedText::new(Typeface::mono(18.0)),
            total: CachedText::new(Typeface::mono(12.5)),
            speed: CachedText::new(Typeface::mono(12.5)),
            count: CachedText::new(Typeface::semibold(13.5)),
            tag: CachedText::new(Typeface::semibold(13.5)),
            tip: CachedText::new(Typeface::mono(12.5)),
            toast: CachedText::new(Typeface::semibold(13.5)),
            labels: PointLabels::new(Typeface::mono(10.5), points),
        }
    }

    /// Looks at the fonts once per pass, before the bar or the toast paints.
    pub fn begin(&mut self, ctx: &Context) {
        self.generation = self.fonts.observe(ctx);
    }

    /// Drops every kept galley, for a new egui context.
    pub fn invalidate(&mut self) {
        self.fonts.invalidate();
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
}

impl Default for BarText {
    fn default() -> Self {
        Self::new()
    }
}
