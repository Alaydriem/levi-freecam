use crate::config::KeyConfig;
use crate::services::KeyNames;
use crate::structs::ui::LegendRow;

// the key legend, built once from the key config
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLegend {
    pub rows: Vec<LegendRow>,
}

impl UiLegend {
    pub fn new(keys: &KeyConfig) -> Self {
        let modifier = KeyNames::name(keys.modifier);
        let chord = |code: u32| format!("{modifier}+{}", KeyNames::name(code));
        let row = |key: String, label: &str, label_cursor_on: Option<&str>| LegendRow {
            key,
            label: label.to_owned(),
            label_cursor_on: label_cursor_on.map(str::to_owned),
            cursor_only: false,
        };
        let drag = LegendRow { cursor_only: true, ..row("RMB drag".to_owned(), "look", None) };
        Self {
            rows: vec![
                row(chord(keys.cursor), "show bar", Some("hide bar")),
                drag,
                row(chord(keys.add_point), "add point", None),
                row(KeyNames::name(keys.delete), "delete point", None),
                row(chord(keys.toggle), "exit freecam", None),
            ],
        }
    }

    // false for a row that does not exist
    pub fn shown(&self, row: usize, cursor_mode: bool) -> bool {
        self.rows.get(row).is_some_and(|row| cursor_mode || !row.cursor_only)
    }

    // an empty string for a row that does not exist
    pub fn label(&self, row: usize, cursor_mode: bool) -> &str {
        let Some(row) = self.rows.get(row) else {
            return "";
        };
        match (&row.label_cursor_on, cursor_mode) {
            (Some(label), true) => label,
            _ => &row.label,
        }
    }
}
