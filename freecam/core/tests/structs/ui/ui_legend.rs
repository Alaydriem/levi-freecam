use freecam_core::config::KeyConfig;
use freecam_core::structs::UiLegend;

// the rows shown in this mode, as key and label
fn shown(legend: &UiLegend, cursor_mode: bool) -> Vec<(&str, &str)> {
    (0..legend.rows.len())
        .filter(|row| legend.shown(*row, cursor_mode))
        .map(|row| (legend.rows[row].key.as_str(), legend.label(row, cursor_mode)))
        .collect()
}

#[test]
fn legend_has_r3_rows() {
    let legend = UiLegend::new(&KeyConfig::default());

    assert_eq!(
        shown(&legend, false),
        [("Ctrl+E", "show bar"), ("Ctrl+B", "add point"), ("Del", "delete point"), ("Ctrl+P", "exit freecam")]
    );
    assert!(legend.rows.iter().all(|row| !row.key.contains("Enter")));
}

#[test]
fn bar_mode_adds_the_drag_row_and_relabels_the_bar_row() {
    let legend = UiLegend::new(&KeyConfig::default());

    assert_eq!(
        shown(&legend, true),
        [
            ("Ctrl+E", "hide bar"),
            ("RMB drag", "look"),
            ("Ctrl+B", "add point"),
            ("Del", "delete point"),
            ("Ctrl+P", "exit freecam")
        ]
    );
}

#[test]
fn a_row_past_the_end_is_not_shown() {
    let legend = UiLegend::new(&KeyConfig::default());

    assert!(!legend.shown(legend.rows.len(), true));
    assert_eq!(legend.label(legend.rows.len(), true), "");
}

#[test]
fn legend_follows_configured_keys() {
    let keys = KeyConfig { add_point: 0x4E, delete: 0x08, ..KeyConfig::default() };

    let legend = UiLegend::new(&keys);

    let shown = shown(&legend, false);
    assert_eq!(shown[1].0, "Ctrl+N");
    assert_eq!(shown[2].0, "Backspace");
}
