use egui::FontFamily;
use freecam_core::structs::UiAction;
use freecam_ui::views::BarLayout;
use studio_ui::theme::FontLoader;

use crate::support::ui_fixture::{UiEvent, UiFixture};

fn click_at(x: f32, y: f32) -> [UiEvent; 3] {
    [
        UiEvent::PointerMoved { x, y },
        UiEvent::PointerButton { x, y, button: 0, pressed: true },
        UiEvent::PointerButton { x, y, button: 0, pressed: false },
    ]
}

fn layout() -> BarLayout {
    BarLayout::new(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(UiFixture::SCREEN[0], UiFixture::SCREEN[1])))
}

#[test]
fn show_draws_nothing_when_the_snapshot_is_hidden() {
    let mut fixture = UiFixture::new();
    fixture.snapshot.visible = false;

    let out = fixture.frame(&[]);

    assert!(out.rect.is_none());
    assert!(out.vertices.is_empty());
}

#[test]
fn fly_mode_draws_strip_and_legend_only() {
    let mut fixture = UiFixture::new();
    fixture.settle();

    let out = fixture.frame(&[]);

    let rect = out.rect.unwrap();
    assert!(rect.bottom() < layout().bar.top(), "{rect:?}");
    let lowest = out.vertices.iter().map(|v| v.y).fold(0.0_f32, f32::max);
    assert!(lowest < layout().bar.top(), "{lowest}");
}

#[test]
fn bar_mode_reaches_the_bar() {
    let mut fixture = UiFixture::new().with_path();
    fixture.settle();

    let rect = fixture.frame(&[]).rect.unwrap();

    assert!(rect.bottom() >= layout().bar.bottom());
}

#[test]
fn screenshot_button_hidden_when_capture_unavailable() {
    let shot = layout().shot.center();
    let mut available = UiFixture::new().with_path();
    available.settle();
    let mut unavailable = UiFixture::new().with_path();
    unavailable.snapshot.can_screenshot = false;
    unavailable.settle();

    let clicked = available.frame(&click_at(shot.x, shot.y)).actions.clone();
    let refused = unavailable.frame(&click_at(shot.x, shot.y)).actions.clone();

    assert_eq!(clicked, [UiAction::Screenshot]);
    assert!(refused.is_empty(), "{refused:?}");
}

#[test]
fn uses_studio_font() {
    let mut fixture = UiFixture::new();
    fixture.frame(&[]);

    let fonts = fixture.ctx.fonts(|fonts| fonts.definitions().clone());

    assert_eq!(fonts.families[&FontFamily::Proportional].first().map(String::as_str), Some(FontLoader::TEXT_KEY));
}

#[test]
fn textures_lost_keeps_drawing_the_same_picture() {
    let mut fixture = UiFixture::new().with_path();
    fixture.settle();
    let before = fixture.frame(&[]).vertices.len();

    fixture.ui.textures_lost();
    let after = fixture.frame(&[]).vertices.len();

    assert_eq!(before, after);
}

#[test]
fn bar_wants_the_pointer_over_the_bar_only() {
    let mut fixture = UiFixture::new().with_path();
    fixture.settle();
    let bar = layout().bar;
    let wants_at = |fixture: &mut UiFixture, x: f32, y: f32| {
        fixture.frame(&[UiEvent::PointerMoved { x, y }]);
        fixture.ctx.egui_wants_pointer_input()
    };

    // the mod manager hands a click to a layer only while egui wants the pointer there
    let over_bar = wants_at(&mut fixture, bar.center().x, bar.center().y);
    let away = wants_at(&mut fixture, bar.center().x, bar.top() - 300.0);

    assert!(over_bar);
    assert!(!away);
}
