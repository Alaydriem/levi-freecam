use freecam_core::config::KeyConfig;
use freecam_core::services::InputTracker;
use freecam_core::structs::{HeldKeys, InputAction};

const CTRL: u32 = 0x11;
const P: u32 = 0x50;
const BRACKET: u32 = 0xDB;
const W: u32 = 0x57;
const ESC: u32 = 0x1B;

struct Fixture;

impl Fixture {
    fn tracker() -> InputTracker {
        InputTracker::new(KeyConfig::default())
    }
}

#[test]
fn ctrl_p_requests_toggle() {
    let mut input = Fixture::tracker();
    input.on_key(CTRL, true, false, false);
    let outcome = input.on_key(P, true, false, false);
    assert_eq!(outcome.action, Some(InputAction::ToggleFreecam));
    assert!(outcome.consume);
}

#[test]
fn p_without_ctrl_does_nothing_when_inactive() {
    let mut input = Fixture::tracker();
    let outcome = input.on_key(P, true, false, false);
    assert_eq!(outcome.action, None);
    assert!(!outcome.consume);
}

#[test]
fn chords_ignored_in_menu() {
    let mut input = Fixture::tracker();
    input.on_key(CTRL, true, false, true);
    let outcome = input.on_key(P, true, false, true);
    assert_eq!(outcome.action, None);
    assert!(!outcome.consume);
}

#[test]
fn ctrl_bracket_toggles_smooth_only_when_active() {
    let mut input = Fixture::tracker();
    input.on_key(CTRL, true, true, false);
    let active = input.on_key(BRACKET, true, true, false);
    assert_eq!(active.action, Some(InputAction::ToggleSmooth));

    let mut input = Fixture::tracker();
    input.on_key(CTRL, true, false, false);
    let inactive = input.on_key(BRACKET, true, false, false);
    assert_eq!(inactive.action, None);
}

#[test]
fn active_consumes_key_downs_except_escape() {
    let mut input = Fixture::tracker();
    assert!(input.on_key(W, true, true, false).consume);
    assert!(!input.on_key(ESC, true, true, false).consume);
}

#[test]
fn key_ups_never_consumed() {
    let mut input = Fixture::tracker();
    input.on_key(W, true, true, false);
    assert!(!input.on_key(W, false, true, false).consume);
}

#[test]
fn held_keys_track_always() {
    let mut input = Fixture::tracker();
    input.on_key(W, true, false, false);
    assert!(input.held().forward);
    input.on_key(W, false, true, false);
    assert!(!input.held().forward);
    input.on_key(CTRL, true, false, false);
    assert!(input.held().fast);
}

#[test]
fn mouse_look_only_when_active_and_no_menu() {
    let mut input = Fixture::tracker();
    input.on_mouse_move(5, 5, false, false);
    input.on_mouse_move(5, 5, true, true);
    assert_eq!(input.take_look(), (0.0, 0.0));

    input.on_mouse_move(3, -2, true, false);
    input.on_mouse_move(1, 1, true, false);
    assert_eq!(input.take_look(), (4.0, -1.0));
    assert_eq!(input.take_look(), (0.0, 0.0));
}

#[test]
fn wheel_scrolls_and_is_consumed_when_active() {
    let mut input = Fixture::tracker();
    assert!(input.on_mouse_button(4, 120, true, false));
    assert_eq!(input.take_scroll(), 1);
    assert!(input.on_mouse_button(4, -120, true, false));
    assert_eq!(input.take_scroll(), -1);

    assert!(!input.on_mouse_button(4, 120, false, false));
    assert_eq!(input.take_scroll(), 0);
}

#[test]
fn buttons_consumed_when_active() {
    let mut input = Fixture::tracker();
    assert!(input.on_mouse_button(1, 1, true, false));
    assert!(!input.on_mouse_button(1, 1, false, false));
}

#[test]
fn button_releases_never_consumed() {
    let mut input = Fixture::tracker();
    assert!(!input.on_mouse_button(1, 0, true, false));
    assert!(!input.on_mouse_button(2, 0, true, false));
}

#[test]
fn clear_held_releases_everything() {
    let mut input = Fixture::tracker();
    input.on_key(W, true, false, false);
    input.on_key(CTRL, true, false, false);
    input.clear_held();
    assert_eq!(input.held(), HeldKeys::default());
}

#[test]
fn menu_open_buttons_pass_through() {
    let mut input = Fixture::tracker();
    assert!(!input.on_mouse_button(1, 1, true, true));
    assert!(!input.on_mouse_button(4, 120, true, true));
    assert_eq!(input.take_scroll(), 0);
}

#[test]
fn held_chord_repeat_does_not_retrigger() {
    let mut input = Fixture::tracker();
    input.on_key(CTRL, true, false, false);
    let first = input.on_key(P, true, false, false);
    assert_eq!(first.action, Some(InputAction::ToggleFreecam));
    let repeat = input.on_key(P, true, true, false);
    assert_eq!(repeat.action, None);
    assert!(repeat.consume);
    input.on_key(P, false, true, false);
    let again = input.on_key(P, true, false, false);
    assert_eq!(again.action, Some(InputAction::ToggleFreecam));
}

#[test]
fn active_menu_open_key_passes_through() {
    let mut input = Fixture::tracker();
    assert!(!input.on_key(W, true, true, true).consume);
}

#[test]
fn clear_held_resets_chord_repeat() {
    let mut input = Fixture::tracker();
    input.on_key(CTRL, true, false, false);
    let first = input.on_key(P, true, false, false);
    assert_eq!(first.action, Some(InputAction::ToggleFreecam));
    input.clear_held();
    input.on_key(CTRL, true, false, false);
    let after = input.on_key(P, true, false, false);
    assert_eq!(after.action, Some(InputAction::ToggleFreecam));
}

const B: u32 = 0x42;
const Z: u32 = 0x5A;
const ENTER: u32 = 0x0D;

#[test]
fn ctrl_k_z_enter_fire_only_when_active() {
    let cases = [(B, InputAction::AddPoint), (Z, InputAction::UndoPoint), (ENTER, InputAction::TogglePlayback)];
    for (code, expected) in cases {
        let mut input = Fixture::tracker();
        input.on_key(CTRL, true, true, false);
        let active = input.on_key(code, true, true, false);
        assert_eq!(active.action, Some(expected));
        assert!(active.consume);

        let mut input = Fixture::tracker();
        input.on_key(CTRL, true, false, false);
        let inactive = input.on_key(code, true, false, false);
        assert_eq!(inactive.action, None);

        let mut input = Fixture::tracker();
        let no_ctrl = input.on_key(code, true, true, false);
        assert_eq!(no_ctrl.action, None);
    }
}

#[test]
fn path_chords_ignored_in_menu() {
    for code in [B, Z, ENTER] {
        let mut input = Fixture::tracker();
        input.on_key(CTRL, true, true, true);
        let outcome = input.on_key(code, true, true, true);
        assert_eq!(outcome.action, None);
        assert!(!outcome.consume);
    }
}

#[test]
fn path_chord_repeat_does_not_retrigger() {
    for (code, expected) in
        [(B, InputAction::AddPoint), (Z, InputAction::UndoPoint), (ENTER, InputAction::TogglePlayback)]
    {
        let mut input = Fixture::tracker();
        input.on_key(CTRL, true, true, false);
        assert_eq!(input.on_key(code, true, true, false).action, Some(expected));
        assert_eq!(input.on_key(code, true, true, false).action, None);
        input.on_key(code, false, true, false);
        assert_eq!(input.on_key(code, true, true, false).action, Some(expected));
        input.clear_held();
        input.on_key(CTRL, true, true, false);
        assert_eq!(input.on_key(code, true, true, false).action, Some(expected));
    }
}
