use egui::{Event, Key, Modifiers, MouseWheelSource, MouseWheelUnit, PointerButton, Pos2, TouchPhase, pos2, vec2};
use freecam_core::structs::UiEvent;

/// Turns [`UiEvent`]s into egui events, converting physical pixels to points and tracking the modifier keys.
pub(crate) struct EventMapper {
    modifiers: Modifiers,
    // held state of the primary, secondary and middle buttons, released on focus loss
    held: [bool; 3],
    // the last pointer position in points
    pointer: Pos2,
}

impl EventMapper {
    const VK_SHIFT: [u32; 3] = [0x10, 0xA0, 0xA1];
    const VK_CONTROL: [u32; 3] = [0x11, 0xA2, 0xA3];
    const VK_ALT: [u32; 3] = [0x12, 0xA4, 0xA5];

    const LETTERS: [Key; 26] = [
        Key::A,
        Key::B,
        Key::C,
        Key::D,
        Key::E,
        Key::F,
        Key::G,
        Key::H,
        Key::I,
        Key::J,
        Key::K,
        Key::L,
        Key::M,
        Key::N,
        Key::O,
        Key::P,
        Key::Q,
        Key::R,
        Key::S,
        Key::T,
        Key::U,
        Key::V,
        Key::W,
        Key::X,
        Key::Y,
        Key::Z,
    ];
    const DIGITS: [Key; 10] =
        [Key::Num0, Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6, Key::Num7, Key::Num8, Key::Num9];
    const FUNCTION: [Key; 12] =
        [Key::F1, Key::F2, Key::F3, Key::F4, Key::F5, Key::F6, Key::F7, Key::F8, Key::F9, Key::F10, Key::F11, Key::F12];

    pub(crate) fn new() -> Self {
        Self { modifiers: Modifiers::NONE, held: [false; 3], pointer: Pos2::ZERO }
    }

    /// Appends the egui events for `event` to `out`. An event egui has no use for appends nothing.
    pub(crate) fn map(&mut self, event: UiEvent, pixels_per_point: f32, out: &mut Vec<Event>) {
        let to_points = |x: f32, y: f32| pos2(x / pixels_per_point, y / pixels_per_point);
        match event {
            UiEvent::PointerMoved { x, y } => {
                self.pointer = to_points(x, y);
                out.push(Event::PointerMoved(self.pointer));
            }
            UiEvent::PointerButton { x, y, button, pressed } => {
                if let Some(mapped) = Self::button(button) {
                    self.pointer = to_points(x, y);
                    if let Some(held) = self.held.get_mut(usize::from(button)) {
                        *held = pressed;
                    }
                    out.push(Event::PointerButton {
                        pos: self.pointer,
                        button: mapped,
                        pressed,
                        modifiers: self.modifiers,
                    });
                }
            }
            UiEvent::Wheel { dx, dy } => out.push(Event::MouseWheel {
                unit: MouseWheelUnit::Line,
                delta: vec2(dx, dy),
                phase: TouchPhase::Move,
                source: MouseWheelSource::Wheel,
                modifiers: self.modifiers,
            }),
            UiEvent::Key { vk, pressed } => self.key(vk, pressed, out),
            UiEvent::PointerGone => out.push(Event::PointerGone),
            UiEvent::FocusLost => {
                self.modifiers = Modifiers::NONE;
                out.push(Event::WindowFocused(false));
                self.release_buttons(out);
                out.push(Event::PointerGone);
            }
        }
    }

    // egui clears keys on focus loss but not mouse buttons, so a button held through Alt+Tab is released here
    fn release_buttons(&mut self, out: &mut Vec<Event>) {
        for (index, held) in self.held.iter_mut().enumerate() {
            if let (true, Some(button)) = (*held, Self::button(index as u8)) {
                out.push(Event::PointerButton {
                    pos: self.pointer,
                    button,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                });
            }
            *held = false;
        }
    }

    fn key(&mut self, vk: u32, pressed: bool, out: &mut Vec<Event>) {
        if self.modifier(vk, pressed) {
            out.push(Event::ModifiersChanged(self.modifiers));
            return;
        }
        if let Some(key) = Self::named_key(vk) {
            out.push(Event::Key { key, physical_key: None, pressed, repeat: false, modifiers: self.modifiers });
        }
    }

    // updates the held modifiers; false when `vk` is not a modifier key
    fn modifier(&mut self, vk: u32, pressed: bool) -> bool {
        if Self::VK_SHIFT.contains(&vk) {
            self.modifiers.shift = pressed;
        } else if Self::VK_CONTROL.contains(&vk) {
            self.modifiers.ctrl = pressed;
            self.modifiers.command = pressed;
        } else if Self::VK_ALT.contains(&vk) {
            self.modifiers.alt = pressed;
        } else {
            return false;
        }
        true
    }

    fn button(button: u8) -> Option<PointerButton> {
        match button {
            0 => Some(PointerButton::Primary),
            1 => Some(PointerButton::Secondary),
            2 => Some(PointerButton::Middle),
            _ => None,
        }
    }

    // Tab, Enter, Space and the arrows are never mapped: they move egui focus and activate the focused widget, so the
    // keyboard (Space is fly-up) could click a button
    fn named_key(vk: u32) -> Option<Key> {
        let index = |first: u32| (vk - first) as usize;
        match vk {
            0x08 => Some(Key::Backspace),
            0x1B => Some(Key::Escape),
            0x21 => Some(Key::PageUp),
            0x22 => Some(Key::PageDown),
            0x23 => Some(Key::End),
            0x24 => Some(Key::Home),
            0x2D => Some(Key::Insert),
            0x2E => Some(Key::Delete),
            0x30..=0x39 => Self::DIGITS.get(index(0x30)).copied(),
            0x41..=0x5A => Self::LETTERS.get(index(0x41)).copied(),
            0x70..=0x7B => Self::FUNCTION.get(index(0x70)).copied(),
            _ => None,
        }
    }
}
