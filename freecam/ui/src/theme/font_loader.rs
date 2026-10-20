use std::path::Path;
use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};

use super::Typeface;

/// Builds the font set from the Windows system fonts, with egui's built-in fonts as fallback.
///
/// The Segoe and Cascadia files are read from the machine and never shipped with the mod: their
/// licences do not allow redistribution. The fallback fonts come from egui itself (Ubuntu-Light,
/// Hack: Ubuntu Font Licence and MIT-style licences, both redistributable).
pub struct FontLoader;

impl FontLoader {
    const FONT_DIR: &'static str = r"C:\Windows\Fonts";
    const UI: &'static str = "segoeui.ttf";
    // `seguisb.ttf` is the real Segoe UI Semibold; `segoeuib.ttf` is Bold.
    const SEMIBOLD: [&'static str; 2] = ["seguisb.ttf", "segoeuib.ttf"];
    const MONO: &'static str = "CascadiaMono.ttf";
    const ICONS: &'static str = "SegoeIcons.ttf";

    /// Builds the font set. A missing file is logged and replaced by egui's built-in fonts.
    pub fn definitions() -> FontDefinitions {
        Self::definitions_from(Path::new(Self::FONT_DIR))
    }

    /// Same as [`Self::definitions`], reading the font files from `dir`.
    pub fn definitions_from(dir: &Path) -> FontDefinitions {
        let mut defs = FontDefinitions::default();
        let builtin_proportional = defs.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
        let builtin_mono = defs.families.get(&FontFamily::Monospace).cloned().unwrap_or_default();

        let ui = Self::insert(&mut defs, dir, "segoe-ui", &[Self::UI]);
        let semibold = Self::insert(&mut defs, dir, "segoe-ui-semibold", &Self::SEMIBOLD);
        let mono = Self::insert(&mut defs, dir, "cascadia-mono", &[Self::MONO]);
        let icons = Self::insert(&mut defs, dir, "segoe-fluent-icons", &[Self::ICONS]);

        let ui_chain = Self::chain(&[ui.clone(), icons.clone()], &builtin_proportional);
        let semibold_chain = Self::chain(&[semibold, ui.clone(), icons.clone()], &builtin_proportional);
        let mono_chain = Self::chain(&[mono, ui, icons.clone()], &builtin_mono);
        let icon_chain = Self::chain(&[icons], &builtin_proportional);

        defs.families.insert(FontFamily::Proportional, ui_chain);
        defs.families.insert(FontFamily::Monospace, mono_chain.clone());
        defs.families.insert(FontFamily::Name(Typeface::SEMIBOLD.into()), semibold_chain);
        defs.families.insert(FontFamily::Name(Typeface::MONO.into()), mono_chain);
        defs.families.insert(FontFamily::Name(Typeface::ICONS.into()), icon_chain);
        defs
    }

    // Loads the first file that exists and registers it under `key`.
    fn insert(defs: &mut FontDefinitions, dir: &Path, key: &str, files: &[&str]) -> Option<String> {
        for file in files {
            let path = dir.join(file);
            match std::fs::read(&path) {
                Ok(bytes) => {
                    defs.font_data.insert(key.to_owned(), Arc::new(FontData::from_owned(bytes)));
                    return Some(key.to_owned());
                }
                Err(err) => log::warn!("font {} not loaded, using the built-in fallback: {err}", path.display()),
            }
        }
        None
    }

    fn chain(preferred: &[Option<String>], fallback: &[String]) -> Vec<String> {
        preferred.iter().flatten().cloned().chain(fallback.iter().cloned()).collect()
    }
}
