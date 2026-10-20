use std::fmt;
use std::sync::Arc;

use egui::{Color32, FontId, Galley, Painter};

/// One line of text that is laid out again only when its words or the font generation change. Each frame writes
/// the text into a kept buffer and compares it with the text on show, so a frame whose text did not change
/// allocates nothing. The galley has a placeholder colour; [`super::Draw::place_tinted`] paints it in any colour.
pub struct CachedText {
    font: FontId,
    // the text written this frame
    scratch: String,
    // the text the galley holds
    shown: String,
    galley: Option<Arc<Galley>>,
    // the font generation the galley was laid out in
    generation: u64,
}

impl CachedText {
    pub fn new(font: FontId) -> Self {
        Self { font, scratch: String::new(), shown: String::new(), galley: None, generation: 0 }
    }

    /// The galley for the text `write` puts into the buffer. A write that fails shows no text.
    pub fn galley(
        &mut self,
        painter: &Painter,
        generation: u64,
        write: impl FnOnce(&mut String) -> fmt::Result,
    ) -> Arc<Galley> {
        self.scratch.clear();
        if write(&mut self.scratch).is_err() {
            self.scratch.clear();
        }
        match &self.galley {
            Some(galley) if self.generation == generation && self.scratch == self.shown => Arc::clone(galley),
            _ => {
                std::mem::swap(&mut self.scratch, &mut self.shown);
                let galley = painter.layout_no_wrap(self.shown.clone(), self.font.clone(), Color32::PLACEHOLDER);
                self.galley = Some(Arc::clone(&galley));
                self.generation = generation;
                galley
            }
        }
    }
}
