use freecam_core::config::FreecamConfig;
use freecam_core::services::FreecamController;
use freecam_core::structs::{
    CameraCommand, CameraPose, FrameOutput, FrameStatus, FreecamState, GuiCommand, ServerInstruction,
};

use crate::support::frames::Frames;

const CTRL: u32 = 0x11;
const P: u32 = 0x50;
const BRACKET: u32 = 0xDB;
const W: u32 = 0x57;
const B: u32 = 0x42;
const Z: u32 = 0x5A;
const ENTER: u32 = 0x0D;
const FRAME_STEP: f64 = 0.05;
const EXIT_WAIT: usize = 10;

struct Rig {
    controller: FreecamController,
    time: f64,
}

impl Rig {
    fn new() -> Self {
        Self { controller: FreecamController::new(FreecamConfig::default()), time: 0.0 }
    }

    fn status(&mut self) -> FrameStatus {
        self.time += FRAME_STEP;
        Frames::status(self.time)
    }

    fn frame(&mut self) -> FrameOutput {
        let status = self.status();
        self.controller.frame(&status)
    }

    fn frame_with(&mut self, edit: impl FnOnce(&mut FrameStatus)) -> FrameOutput {
        let mut status = self.status();
        edit(&mut status);
        self.controller.frame(&status)
    }

    fn frames(&mut self, count: usize) -> Vec<FrameOutput> {
        (0..count).map(|_| self.frame()).collect()
    }

    // presses Ctrl+P and releases P, leaving Ctrl held
    fn press_toggle(&mut self) -> bool {
        self.controller.on_key(CTRL, true);
        let consumed = self.controller.on_key(P, true);
        self.controller.on_key(P, false);
        consumed
    }

    fn start(&mut self) -> FrameOutput {
        self.press_toggle();
        self.frame()
    }

    // toggles freecam off; the returned frame is exit frame 1
    fn begin_exit(&mut self) -> FrameOutput {
        self.press_toggle();
        self.frame()
    }

    // presses Ctrl+key and releases the key, leaving Ctrl held
    fn chord(&mut self, key: u32) {
        self.controller.on_key(CTRL, true);
        self.controller.on_key(key, true);
        self.controller.on_key(key, false);
    }

    // freecam on with the smooth camera off, so the camera stops as soon as keys are released
    fn start_sharp(&mut self) -> CameraPose {
        let pose = self.start().render_override.unwrap();
        self.chord(BRACKET);
        self.controller.on_key(CTRL, false);
        pose
    }

    // walks forward at the scroll speed and returns the pose reached
    fn walk(&mut self, frames: usize) -> CameraPose {
        self.controller.on_key(CTRL, false);
        self.controller.on_key(W, true);
        let mut pose = self.frame().render_override.unwrap();
        for _ in 1..frames {
            pose = self.frame().render_override.unwrap();
        }
        self.controller.on_key(W, false);
        pose
    }

    // starts freecam and records two points `frames` walking frames apart
    fn build_path(&mut self, frames: usize) -> (CameraPose, CameraPose) {
        let first = self.start_sharp();
        self.chord(B);
        let last = self.walk(frames);
        self.chord(B);
        self.controller.on_key(CTRL, false);
        (first, last)
    }

    // the same as build_path but with the smooth camera left on
    fn build_smooth_path(&mut self, frames: usize) -> (CameraPose, CameraPose) {
        let first = Self::pose(&self.start());
        self.chord(B);
        let last = self.walk(frames);
        self.chord(B);
        self.controller.on_key(CTRL, false);
        (first, last)
    }

    // presses Ctrl+Enter, leaving Ctrl held, and runs one frame
    fn play(&mut self) -> FrameOutput {
        self.chord(ENTER);
        self.frame()
    }

    fn pose(out: &FrameOutput) -> CameraPose {
        out.render_override.unwrap()
    }

    fn gap(a: CameraPose, b: CameraPose) -> f32 {
        (a.position - b.position).length()
    }
}

#[test]
fn toggle_starts_with_set_hide_and_block() {
    let mut rig = Rig::new();
    let out = rig.start();
    assert_eq!(rig.controller.state(), FreecamState::On);
    assert_eq!(out.camera, CameraCommand::Set(Frames::start_pose()));
    assert_eq!(out.gui, GuiCommand::Hide);
    assert!(out.block_input);
    assert_eq!(out.render_override, Some(Frames::start_pose()));
}

#[test]
fn refuses_when_not_ready() {
    let mut rig = Rig::new();
    rig.press_toggle();
    let out = rig.frame_with(|s| s.ready = false);
    assert_eq!(rig.controller.state(), FreecamState::Off);
    assert_eq!(out.camera, CameraCommand::None);
    assert_eq!(out.gui, GuiCommand::None);
}

#[test]
fn refuses_when_server_camera_active() {
    let mut rig = Rig::new();
    rig.press_toggle();
    let out = rig.frame_with(|s| s.player_camera_active = false);
    assert_eq!(rig.controller.state(), FreecamState::Off);
    assert_eq!(out.camera, CameraCommand::None);
    assert_eq!(out.gui, GuiCommand::None);
}

#[test]
fn refuses_when_menu_open() {
    let mut rig = Rig::new();
    rig.press_toggle();
    let out = rig.frame_with(|s| s.menu_open = true);
    assert_eq!(rig.controller.state(), FreecamState::Off);
    assert_eq!(out.camera, CameraCommand::None);
    assert_eq!(out.gui, GuiCommand::None);
}

#[test]
fn refuses_when_not_in_world() {
    let mut rig = Rig::new();
    rig.press_toggle();
    let out = rig.frame_with(|s| s.in_world = false);
    assert_eq!(rig.controller.state(), FreecamState::Off);
    assert_eq!(out.camera, CameraCommand::None);
}

#[test]
fn exit_waits_then_clears() {
    let mut rig = Rig::new();
    rig.start();
    let mut exit = vec![rig.begin_exit()];
    exit.extend(rig.frames(EXIT_WAIT - 1));
    for out in &exit[..EXIT_WAIT - 1] {
        assert_eq!(out.camera, CameraCommand::None);
        assert_eq!(out.render_override, None);
        assert!(out.block_input);
    }
    let last = exit[EXIT_WAIT - 1];
    assert_eq!(last.camera, CameraCommand::Clear);
    assert_eq!(last.gui, GuiCommand::Show);
    assert!(!last.block_input);
    assert_eq!(rig.controller.state(), FreecamState::Off);
    assert!(exit.iter().all(|out| !matches!(out.camera, CameraCommand::Set(_))));
}

#[test]
fn second_toggle_during_exit_is_ignored() {
    let mut rig = Rig::new();
    rig.start();
    let mut exit = vec![rig.begin_exit()];
    exit.extend(rig.frames(1));
    rig.press_toggle();
    exit.extend(rig.frames(EXIT_WAIT - 2));
    let clears = exit.iter().filter(|out| out.camera == CameraCommand::Clear).count();
    assert_eq!(clears, 1);
    assert_eq!(exit[EXIT_WAIT - 1].camera, CameraCommand::Clear);
    assert_eq!(rig.controller.state(), FreecamState::Off);
    assert!(rig.frames(5).iter().all(|out| out.camera == CameraCommand::None));
}

#[test]
fn gui_left_hidden_if_user_hid_it() {
    let mut rig = Rig::new();
    rig.press_toggle();
    let started = rig.frame_with(|s| s.gui_hidden = true);
    assert_eq!(started.gui, GuiCommand::Hide);
    rig.begin_exit();
    let exit = rig.frames(EXIT_WAIT - 1);
    let last = exit[exit.len() - 1];
    assert_eq!(last.camera, CameraCommand::Clear);
    assert_eq!(last.gui, GuiCommand::None);
}

#[test]
fn damage_starts_exit() {
    let mut rig = Rig::new();
    rig.start();
    let hit = rig.frame_with(|s| s.health = 19);
    assert!(matches!(rig.controller.state(), FreecamState::Exiting { .. }));
    assert_eq!(hit.camera, CameraCommand::None);
    let rest = rig.frames(EXIT_WAIT - 1);
    assert_eq!(rest[EXIT_WAIT - 2].camera, CameraCommand::Clear);
    assert_eq!(rig.controller.state(), FreecamState::Off);
}

#[test]
fn damage_compares_to_previous_frame_health() {
    let mut rig = Rig::new();
    rig.press_toggle();
    rig.frame_with(|s| s.health = 10);
    rig.frame_with(|s| s.health = 18);
    assert_eq!(rig.controller.state(), FreecamState::On);
    rig.frame_with(|s| s.health = 17);
    assert!(matches!(rig.controller.state(), FreecamState::Exiting { .. }));
}

#[test]
fn dimension_change_starts_exit() {
    let mut rig = Rig::new();
    rig.start();
    let out = rig.frame_with(|s| s.dimension = 1);
    assert!(matches!(rig.controller.state(), FreecamState::Exiting { .. }));
    assert_eq!(out.camera, CameraCommand::None);
    let rest = rig.frames(EXIT_WAIT - 1);
    assert_eq!(rest[EXIT_WAIT - 2].camera, CameraCommand::Clear);
}

#[test]
fn server_set_hands_over_without_clear() {
    let mut rig = Rig::new();
    rig.start();
    rig.controller.on_server_instruction(ServerInstruction::SetOrClear);
    let out = rig.frame();
    assert_eq!(rig.controller.state(), FreecamState::Off);
    assert_eq!(out.camera, CameraCommand::None);
    assert_eq!(out.render_override, None);
    assert_eq!(out.gui, GuiCommand::Show);
    assert!(!out.block_input);
    assert!(rig.frames(20).iter().all(|out| out.camera == CameraCommand::None));
}

#[test]
fn server_set_during_exit_cancels_clear() {
    let mut rig = Rig::new();
    rig.start();
    rig.begin_exit();
    rig.frames(2);
    rig.controller.on_server_instruction(ServerInstruction::SetOrClear);
    let out = rig.frame();
    assert_eq!(rig.controller.state(), FreecamState::Off);
    assert_eq!(out.camera, CameraCommand::None);
    assert_eq!(out.gui, GuiCommand::Show);
    assert!(!out.block_input);
    assert!(rig.frames(20).iter().all(|out| out.camera == CameraCommand::None));
}

#[test]
fn server_instruction_while_off_does_not_affect_next_start() {
    let mut rig = Rig::new();
    rig.controller.on_server_instruction(ServerInstruction::SetOrClear);
    let out = rig.start();
    assert_eq!(rig.controller.state(), FreecamState::On);
    assert!(matches!(out.camera, CameraCommand::Set(_)));
}

#[test]
fn server_other_ignored() {
    let mut rig = Rig::new();
    rig.start();
    rig.controller.on_server_instruction(ServerInstruction::Other);
    rig.frame();
    assert_eq!(rig.controller.state(), FreecamState::On);
}

#[test]
fn leaving_world_turns_off_without_clear() {
    let mut rig = Rig::new();
    rig.start();
    let out = rig.frame_with(|s| s.in_world = false);
    assert_eq!(rig.controller.state(), FreecamState::Off);
    assert_eq!(out.gui, GuiCommand::Show);
    assert_eq!(out.camera, CameraCommand::None);
    assert!(!out.block_input);
}

#[test]
fn menu_open_holds_camera() {
    let mut rig = Rig::new();
    rig.start();
    rig.controller.on_key(W, true);
    let out = rig.frame_with(|s| s.menu_open = true);
    assert_eq!(out.camera, CameraCommand::Set(Frames::start_pose()));
    assert_eq!(out.render_override, Some(Frames::start_pose()));
    assert!(out.block_input);
}

#[test]
fn held_key_moves_camera() {
    let mut rig = Rig::new();
    rig.start();
    rig.controller.on_key(W, true);
    rig.frame();
    let out = rig.frame();
    let moved = out.render_override.unwrap().position;
    assert_ne!(moved, Frames::start_pose().position);
}

#[test]
fn focus_loss_clears_held_keys() {
    let mut rig = Rig::new();
    rig.start();
    rig.controller.on_key(BRACKET, true);
    rig.controller.on_key(BRACKET, false);
    rig.controller.on_key(W, true);
    rig.frame_with(|s| s.focused = false);
    let positions: Vec<_> = rig.frames(3).iter().map(|out| out.render_override.unwrap().position).collect();
    assert_eq!(positions[0], positions[1]);
    assert_eq!(positions[1], positions[2]);
}

#[test]
fn chord_in_menu_does_not_toggle() {
    let mut rig = Rig::new();
    rig.frame_with(|s| s.menu_open = true);
    assert!(!rig.press_toggle());
    let out = rig.frame();
    assert_eq!(rig.controller.state(), FreecamState::Off);
    assert_eq!(out.camera, CameraCommand::None);
}

#[test]
fn mouse_buttons_blocked_only_while_active() {
    let mut rig = Rig::new();
    assert!(!rig.controller.on_mouse_button(1, 1));
    rig.start();
    assert!(rig.controller.on_mouse_button(1, 1));
}

#[test]
fn input_during_exit_does_not_leak_into_next_session() {
    let mut leaky = Rig::new();
    leaky.start();
    leaky.begin_exit();
    leaky.controller.on_mouse_move(400, -300);
    leaky.controller.on_mouse_button(4, 120);
    leaky.frames(EXIT_WAIT - 1);
    assert_eq!(leaky.controller.state(), FreecamState::Off);
    let first = leaky.start();
    let start = Frames::start_pose();
    let pose = first.render_override.unwrap();
    assert_eq!((pose.yaw, pose.pitch), (start.yaw, start.pitch));
    let on_frame = leaky.frame().render_override.unwrap();
    assert_eq!((on_frame.yaw, on_frame.pitch), (start.yaw, start.pitch));

    let mut clean = Rig::new();
    clean.start();
    clean.begin_exit();
    clean.frames(EXIT_WAIT - 1);
    clean.start();
    clean.frame();
    leaky.controller.on_key(W, true);
    clean.controller.on_key(W, true);
    let leaky_pose = leaky.frame().render_override.unwrap();
    let clean_pose = clean.frame().render_override.unwrap();
    assert_eq!(leaky_pose.position, clean_pose.position);
}

#[test]
fn leaving_world_while_exiting_turns_off_without_clear() {
    let mut rig = Rig::new();
    rig.start();
    rig.begin_exit();
    let out = rig.frame_with(|s| s.in_world = false);
    assert_eq!(rig.controller.state(), FreecamState::Off);
    assert_eq!(out.gui, GuiCommand::Show);
    assert_eq!(out.camera, CameraCommand::None);
    assert!(rig.frames(20).iter().all(|out| out.camera == CameraCommand::None));
}

#[test]
fn add_and_undo_points_while_on() {
    let mut rig = Rig::new();
    let first = rig.start_sharp();
    rig.chord(B);
    assert_eq!(rig.controller.path().points(), &[first]);
    let second = rig.walk(10);
    rig.chord(B);
    assert_eq!(rig.controller.path().points(), &[first, second]);
    rig.chord(Z);
    assert_eq!(rig.controller.path().points(), &[first]);
    rig.chord(Z);
    rig.chord(Z);
    assert!(rig.controller.path().points().is_empty());
}

#[test]
fn play_with_fewer_than_two_points_does_nothing() {
    let mut rig = Rig::new();
    rig.start_sharp();
    rig.chord(B);
    let here = rig.walk(5);
    let out = rig.play();
    assert_eq!(rig.controller.state(), FreecamState::On);
    assert_eq!(Rig::pose(&out), here);
    assert_eq!(Rig::pose(&rig.frame()), here);
}

#[test]
fn play_snaps_to_first_point_then_settles_then_moves() {
    let mut rig = Rig::new();
    let (first, last) = rig.build_path(40);
    let mut settle = vec![rig.play()];
    assert_eq!(rig.controller.state(), FreecamState::Playing);
    settle.extend(rig.frames(EXIT_WAIT - 1));
    for out in &settle {
        assert_eq!(out.camera, CameraCommand::Set(first));
        assert_eq!(out.render_override, Some(first));
        assert!(out.block_input);
        assert!(!out.show_overlay);
    }
    let moving = Rig::pose(&rig.frame());
    assert!(Rig::gap(moving, first) > 0.0);
    assert!(Rig::gap(moving, last) < Rig::gap(first, last));
}

#[test]
fn play_uses_scroll_speed_not_fast_multiplier() {
    let mut rig = Rig::new();
    rig.build_path(80);
    rig.play();
    rig.frames(EXIT_WAIT - 1 + 30);
    let before = Rig::pose(&rig.frame());
    let after = Rig::pose(&rig.frame());
    let cruise_step = 6.0 * FRAME_STEP as f32;
    assert!((Rig::gap(before, after) - cruise_step).abs() < 0.01, "step {}", Rig::gap(before, after));
}

#[test]
fn play_finishes_and_holds_last_point() {
    let mut rig = Rig::new();
    let (_, last) = rig.build_path(20);
    rig.play();
    let outs = rig.frames(EXIT_WAIT - 1 + 45);
    assert_eq!(rig.controller.state(), FreecamState::On);
    assert_eq!(Rig::pose(&outs[outs.len() - 1]), last);
    for out in rig.frames(3) {
        assert_eq!(Rig::pose(&out), last);
    }
}

#[test]
fn toggle_during_play_stops_in_place() {
    let mut rig = Rig::new();
    rig.build_path(80);
    rig.play();
    rig.frames(EXIT_WAIT - 1 + 20);
    let here = Rig::pose(&rig.frame());
    let stopped = rig.play();
    assert_eq!(rig.controller.state(), FreecamState::On);
    assert_eq!(Rig::pose(&stopped), here);
    for out in rig.frames(3) {
        assert_eq!(Rig::pose(&out), here);
    }
}

#[test]
fn add_undo_ignored_while_playing() {
    let mut rig = Rig::new();
    let (first, last) = rig.build_path(40);
    rig.play();
    rig.frames(EXIT_WAIT + 5);
    rig.chord(B);
    rig.chord(Z);
    rig.chord(Z);
    rig.frame();
    assert_eq!(rig.controller.state(), FreecamState::Playing);
    assert_eq!(rig.controller.path().points(), &[first, last]);
}

#[test]
fn menu_pauses_playback() {
    let mut rig = Rig::new();
    rig.build_path(80);
    rig.play();
    rig.frames(EXIT_WAIT + 20);
    let before = Rig::pose(&rig.frame());
    for _ in 0..6 {
        let out = rig.frame_with(|s| s.menu_open = true);
        assert_eq!(Rig::pose(&out), before);
    }
    assert_eq!(rig.controller.state(), FreecamState::Playing);
    let resumed = Rig::pose(&rig.frame());
    assert!(Rig::gap(before, resumed) < 0.5);
}

#[test]
fn look_input_ignored_while_playing() {
    let mut noisy = Rig::new();
    let mut quiet = Rig::new();
    for rig in [&mut noisy, &mut quiet] {
        rig.build_path(80);
        rig.play();
        rig.frames(EXIT_WAIT + 10);
    }
    noisy.controller.on_mouse_move(400, -300);
    noisy.controller.on_mouse_button(4, 120);
    noisy.controller.on_key(W, true);
    assert_eq!(noisy.frame().render_override, quiet.frame().render_override);
    noisy.controller.on_key(W, false);
    for rig in [&mut noisy, &mut quiet] {
        rig.play();
    }
    assert_eq!(noisy.controller.state(), FreecamState::On);
    for _ in 0..3 {
        assert_eq!(noisy.frame().render_override, quiet.frame().render_override);
    }
}

#[test]
fn damage_during_playback_exits() {
    let mut rig = Rig::new();
    rig.build_path(40);
    rig.play();
    rig.frames(EXIT_WAIT + 5);
    let hit = rig.frame_with(|s| s.health = 19);
    assert!(matches!(rig.controller.state(), FreecamState::Exiting { .. }));
    assert_eq!(hit.camera, CameraCommand::None);
    let rest = rig.frames(EXIT_WAIT - 1);
    assert_eq!(rest[EXIT_WAIT - 2].camera, CameraCommand::Clear);
    assert_eq!(rig.controller.state(), FreecamState::Off);
}

#[test]
fn server_set_during_playback_hands_over() {
    let mut rig = Rig::new();
    rig.build_path(40);
    rig.play();
    rig.frames(EXIT_WAIT + 5);
    rig.controller.on_server_instruction(ServerInstruction::SetOrClear);
    let out = rig.frame();
    assert_eq!(rig.controller.state(), FreecamState::Off);
    assert_eq!(out.camera, CameraCommand::None);
    assert_eq!(out.gui, GuiCommand::Show);
    assert!(rig.frames(20).iter().all(|out| out.camera == CameraCommand::None));
}

#[test]
fn ctrl_p_during_playback_exits_with_clear() {
    let mut rig = Rig::new();
    rig.build_path(40);
    rig.play();
    rig.frames(EXIT_WAIT + 5);
    let mut exit = vec![rig.begin_exit()];
    assert!(matches!(rig.controller.state(), FreecamState::Exiting { .. }));
    exit.extend(rig.frames(EXIT_WAIT - 1));
    assert_eq!(exit[EXIT_WAIT - 1].camera, CameraCommand::Clear);
    assert_eq!(rig.controller.state(), FreecamState::Off);
    let restarted = rig.start();
    assert_eq!(rig.controller.state(), FreecamState::On);
    assert_eq!(restarted.camera, CameraCommand::Set(Frames::start_pose()));
    rig.frames(3);
    assert_eq!(rig.controller.state(), FreecamState::On);
}

#[test]
fn path_survives_freecam_off_on() {
    let mut rig = Rig::new();
    let (first, last) = rig.build_path(20);
    rig.begin_exit();
    rig.frames(EXIT_WAIT - 1);
    assert_eq!(rig.controller.state(), FreecamState::Off);
    rig.start();
    assert_eq!(rig.controller.path().points(), &[first, last]);
    let out = rig.play();
    assert_eq!(rig.controller.state(), FreecamState::Playing);
    assert_eq!(Rig::pose(&out), first);
}

#[test]
fn path_cleared_on_dimension_change_while_off() {
    let mut rig = Rig::new();
    rig.build_path(20);
    rig.begin_exit();
    rig.frames(EXIT_WAIT - 1);
    rig.frames(3);
    assert_eq!(rig.controller.path().points().len(), 2);
    rig.frame_with(|s| s.dimension = 1);
    assert!(rig.controller.path().points().is_empty());
}

#[test]
fn path_cleared_on_leaving_world() {
    let mut off = Rig::new();
    off.build_path(20);
    off.begin_exit();
    off.frames(EXIT_WAIT - 1);
    off.frame_with(|s| s.in_world = false);
    assert!(off.controller.path().points().is_empty());

    let mut on = Rig::new();
    on.build_path(20);
    on.frame_with(|s| s.in_world = false);
    assert_eq!(on.controller.state(), FreecamState::Off);
    assert!(on.controller.path().points().is_empty());
    on.start();
    on.play();
    assert_eq!(on.controller.state(), FreecamState::On);
}

#[test]
fn overlay_shown_only_in_on() {
    let mut rig = Rig::new();
    assert!(!rig.frame().show_overlay);
    rig.start_sharp();
    rig.chord(B);
    assert!(rig.frame().show_overlay);
    rig.walk(10);
    rig.chord(B);
    assert!(rig.frame().show_overlay);
    assert!(!rig.play().show_overlay);
    assert!(rig.frames(EXIT_WAIT + 5).iter().all(|out| !out.show_overlay));
    let stopped = rig.play();
    assert_eq!(rig.controller.state(), FreecamState::On);
    assert!(stopped.show_overlay);
    let exit = rig.begin_exit();
    assert!(!exit.show_overlay);
    assert!(rig.frames(EXIT_WAIT + 2).iter().all(|out| !out.show_overlay));
}

#[test]
fn overlay_filled_only_while_on() {
    const S: u32 = 0x53;
    let mut rig = Rig::new();
    rig.frame();
    assert!(rig.controller.overlay().markers.is_empty());
    assert_eq!(rig.controller.overlay().point_count, 0);

    rig.start_sharp();
    rig.chord(B);
    rig.controller.on_key(CTRL, false);
    rig.controller.on_key(S, true);
    rig.frames(5);
    rig.controller.on_key(S, false);
    rig.frame();
    assert_eq!(rig.controller.overlay().point_count, 1);
    assert!(!rig.controller.overlay().markers.is_empty());

    rig.chord(B);
    rig.play();
    assert_eq!(rig.controller.state(), FreecamState::Playing);
    assert_eq!(rig.controller.overlay().point_count, 0);
    assert!(rig.controller.overlay().markers.is_empty());
    assert!(rig.controller.overlay().segments.is_empty());

    rig.begin_exit();
    assert!(matches!(rig.controller.state(), FreecamState::Exiting { .. }));
    assert!(rig.controller.overlay().markers.is_empty());
    assert!(rig.controller.overlay().segments.is_empty());
    rig.frames(EXIT_WAIT);
    assert_eq!(rig.controller.state(), FreecamState::Off);
    assert!(rig.controller.overlay().markers.is_empty());
    assert_eq!(rig.controller.overlay().point_count, 0);
}

#[test]
fn overlay_hidden_in_on_with_no_points() {
    let mut rig = Rig::new();
    rig.start_sharp();
    let out = rig.frame();
    assert_eq!(rig.controller.state(), FreecamState::On);
    assert!(!out.show_overlay);
    assert!(!rig.controller.overlay_visible());
    assert_eq!(rig.controller.overlay().point_count, 0);
    assert!(rig.controller.overlay().markers.is_empty());
    assert!(rig.controller.overlay().segments.is_empty());
}

#[test]
fn overlay_visible_in_on_with_a_point_and_no_menu() {
    let mut rig = Rig::new();
    rig.start_sharp();
    rig.chord(B);
    let out = rig.frame();
    assert!(out.show_overlay);
    assert!(rig.controller.overlay_visible());
    assert_eq!(rig.controller.overlay().point_count, 1);
}

#[test]
fn overlay_hidden_while_a_menu_is_open() {
    let mut rig = Rig::new();
    rig.start_sharp();
    rig.chord(B);
    rig.frame();
    let out = rig.frame_with(|s| s.menu_open = true);
    assert_eq!(rig.controller.state(), FreecamState::On);
    assert!(!out.show_overlay);
    assert!(!rig.controller.overlay_visible());
    assert!(rig.controller.overlay().markers.is_empty());
    assert!(rig.controller.overlay().segments.is_empty());
    let closed = rig.frame();
    assert!(closed.show_overlay);
    assert!(rig.controller.overlay_visible());
}

#[test]
fn overlay_hidden_when_playing_off_or_exiting() {
    let mut rig = Rig::new();
    assert!(!rig.controller.overlay_visible());
    rig.build_path(20);
    rig.frame();
    assert!(rig.controller.overlay_visible());
    rig.play();
    assert_eq!(rig.controller.state(), FreecamState::Playing);
    assert!(!rig.controller.overlay_visible());
    rig.frames(EXIT_WAIT + 5);
    rig.begin_exit();
    assert!(matches!(rig.controller.state(), FreecamState::Exiting { .. }));
    assert!(!rig.controller.overlay_visible());
    rig.frames(EXIT_WAIT);
    assert_eq!(rig.controller.state(), FreecamState::Off);
    assert!(!rig.controller.overlay_visible());
}

#[test]
fn smooth_stop_leaves_camera_still() {
    let mut rig = Rig::new();
    rig.build_smooth_path(80);
    rig.play();
    rig.frames(EXIT_WAIT - 1 + 20);
    let here = Rig::pose(&rig.frame());
    let stopped = rig.play();
    assert_eq!(rig.controller.state(), FreecamState::On);
    assert_eq!(Rig::pose(&stopped), here);
    rig.controller.on_key(CTRL, false);
    for out in rig.frames(5) {
        assert_eq!(Rig::pose(&out), here);
    }
}

#[test]
fn smooth_finish_leaves_camera_still_on_last_point() {
    let mut rig = Rig::new();
    let (_, last) = rig.build_smooth_path(20);
    rig.play();
    let outs = rig.frames(EXIT_WAIT - 1 + 60);
    assert_eq!(rig.controller.state(), FreecamState::On);
    assert_eq!(Rig::pose(&outs[outs.len() - 1]), last);
    rig.controller.on_key(CTRL, false);
    for out in rig.frames(5) {
        assert_eq!(Rig::pose(&out), last);
    }
}

#[test]
fn leaving_world_during_playback_turns_off_without_clear() {
    let mut rig = Rig::new();
    rig.build_path(40);
    rig.play();
    rig.frames(EXIT_WAIT + 5);
    let out = rig.frame_with(|s| s.in_world = false);
    assert_eq!(rig.controller.state(), FreecamState::Off);
    assert_eq!(out.camera, CameraCommand::None);
    assert_eq!(out.gui, GuiCommand::Show);
    assert!(!out.block_input);
    assert!(rig.controller.path().points().is_empty());
    assert!(rig.frames(20).iter().all(|out| out.camera == CameraCommand::None));
}

#[test]
fn dimension_change_during_playback_exits_and_clears_path() {
    let mut rig = Rig::new();
    rig.build_path(40);
    rig.play();
    rig.frames(EXIT_WAIT + 5);
    let mut exit = vec![rig.frame_with(|s| s.dimension = 1)];
    assert!(matches!(rig.controller.state(), FreecamState::Exiting { .. }));
    assert!(rig.controller.path().points().is_empty());
    for _ in 1..EXIT_WAIT {
        exit.push(rig.frame_with(|s| s.dimension = 1));
    }
    for out in &exit[..EXIT_WAIT - 1] {
        assert_eq!(out.camera, CameraCommand::None);
    }
    assert_eq!(exit[EXIT_WAIT - 1].camera, CameraCommand::Clear);
    assert_eq!(exit[EXIT_WAIT - 1].gui, GuiCommand::Show);
    assert_eq!(rig.controller.state(), FreecamState::Off);
}

#[test]
fn death_during_playback_waits_then_clears() {
    let mut rig = Rig::new();
    rig.build_path(40);
    rig.play();
    rig.frames(EXIT_WAIT + 5);
    let mut exit = vec![rig.frame_with(|s| s.health = 0)];
    for _ in 1..EXIT_WAIT {
        exit.push(rig.frame_with(|s| s.health = 0));
    }
    for out in &exit[..EXIT_WAIT - 1] {
        assert_eq!(out.camera, CameraCommand::None);
        assert!(out.block_input);
    }
    assert_eq!(exit[EXIT_WAIT - 1].camera, CameraCommand::Clear);
    assert_eq!(exit[EXIT_WAIT - 1].gui, GuiCommand::Show);
    assert_eq!(rig.controller.state(), FreecamState::Off);
}

#[test]
fn ctrl_p_during_playback_sends_nothing_until_the_clear() {
    let mut rig = Rig::new();
    rig.build_path(40);
    rig.play();
    rig.frames(EXIT_WAIT + 5);
    let mut exit = vec![rig.begin_exit()];
    exit.extend(rig.frames(EXIT_WAIT - 1));
    for out in &exit[..EXIT_WAIT - 1] {
        assert_eq!(out.camera, CameraCommand::None);
        assert_eq!(out.render_override, None);
        assert!(out.block_input);
    }
    assert_eq!(exit[EXIT_WAIT - 1].camera, CameraCommand::Clear);
    assert_eq!(exit[EXIT_WAIT - 1].gui, GuiCommand::Show);
}
