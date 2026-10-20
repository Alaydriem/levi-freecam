use egui::{Context, Event, Modifiers, PointerButton, Pos2, Rect, pos2, vec2};
use freecam_ui::widgets::TextButton;

use crate::support::headless::Headless;

const SIZE: [f32; 2] = [400.0, 300.0];

fn button_rect() -> Rect {
    Rect::from_min_size(pos2(100.0, 100.0), vec2(120.0, 28.0))
}

fn button(pressed: bool, pos: Pos2) -> Event {
    Event::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE }
}

fn frame(ctx: &Context, events: Vec<Event>) -> bool {
    let mut clicked = false;
    Headless::frame(ctx, SIZE, events, |ui| {
        clicked = TextButton::new("Go", "go").show(ui, button_rect()).clicked();
    });
    clicked
}

// Widgets register with the hit test one frame late, so the first frame only lays the button out.
fn settled_context() -> Context {
    let ctx = Context::default();
    frame(&ctx, Vec::new());
    ctx
}

#[test]
fn press_and_release_inside_clicks() {
    let ctx = settled_context();
    let inside = pos2(150.0, 114.0);

    let clicked = frame(&ctx, vec![Event::PointerMoved(inside), button(true, inside), button(false, inside)]);

    assert!(clicked);
}

#[test]
fn release_outside_does_not_click() {
    let ctx = settled_context();
    let inside = pos2(150.0, 114.0);
    let outside = pos2(300.0, 250.0);

    let pressed = frame(&ctx, vec![Event::PointerMoved(inside), button(true, inside)]);
    let released = frame(&ctx, vec![Event::PointerMoved(outside), button(false, outside)]);

    assert!(!pressed);
    assert!(!released);
}
