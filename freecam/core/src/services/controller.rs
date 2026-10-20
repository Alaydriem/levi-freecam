use log::info;

use crate::config::FreecamConfig;
use crate::services::{CameraMotion, InputTracker, OverlayBuilder, PathSession, SmoothnessTelemetry};
use crate::structs::{
    CameraCommand, CameraPose, FrameOutput, FrameStatus, FreecamState, GuiCommand, InputAction, PathOverlay,
    ServerInstruction, StartRefusal, Vec3,
};

pub struct FreecamController {
    motion: CameraMotion,
    input: InputTracker,
    telemetry: SmoothnessTelemetry,
    path: PathSession,
    overlay: PathOverlay,
    exit_wait_frames: u32,
    state: FreecamState,
    menu_open: bool,
    overlay_shown: bool,
    toggle_requested: bool,
    handover_requested: bool,
    play_requested: bool,
    last_time: Option<f64>,
    // true when freecam hid the GUI and must show it again
    restore_gui: bool,
    health: i32,
    dimension: i32,
}

impl FreecamController {
    pub fn new(config: FreecamConfig) -> Self {
        let FreecamConfig { keys, motion, exit_wait_frames, telemetry, .. } = config.sanitized();
        Self {
            path: PathSession::new(motion.pitch_limit, exit_wait_frames),
            overlay: PathOverlay::new(),
            motion: CameraMotion::new(motion),
            input: InputTracker::new(keys),
            telemetry: SmoothnessTelemetry::new(telemetry),
            exit_wait_frames,
            state: FreecamState::Off,
            menu_open: false,
            overlay_shown: false,
            toggle_requested: false,
            handover_requested: false,
            play_requested: false,
            last_time: None,
            restore_gui: false,
            health: 0,
            dimension: 0,
        }
    }

    // returns true when the game must not see the key
    pub fn on_key(&mut self, code: u32, down: bool) -> bool {
        let outcome = self.input.on_key(code, down, self.is_active(), self.menu_open);
        match outcome.action {
            Some(InputAction::ToggleFreecam) => self.toggle_requested = true,
            Some(InputAction::ToggleSmooth) => {
                let smooth = self.motion.toggle_smooth();
                info!("freecam smooth camera {}", if smooth { "on" } else { "off" });
            }
            Some(InputAction::AddPoint) if self.state == FreecamState::On => self.path.add(self.motion.pose()),
            Some(InputAction::UndoPoint) if self.state == FreecamState::On => self.path.undo(),
            Some(InputAction::TogglePlayback) => self.play_requested = true,
            Some(InputAction::AddPoint | InputAction::UndoPoint) => {}
            None => {}
        }
        outcome.consume
    }

    pub fn on_mouse_move(&mut self, dx: i32, dy: i32) {
        self.input.on_mouse_move(dx, dy, self.is_active(), self.menu_open);
    }

    // returns true when the game must not see the button
    pub fn on_mouse_button(&mut self, button: u8, data: i8) -> bool {
        self.input.on_mouse_button(button, data, self.is_active(), self.menu_open)
    }

    pub fn on_server_instruction(&mut self, kind: ServerInstruction) {
        if kind == ServerInstruction::SetOrClear && self.is_active() {
            self.handover_requested = true;
        }
    }

    pub fn state(&self) -> FreecamState {
        self.state
    }

    pub fn path(&self) -> &PathSession {
        &self.path
    }

    pub fn overlay(&self) -> &PathOverlay {
        &self.overlay
    }

    // the one gate for the overlay: set by the last frame, so the frame's show_overlay, the overlay buffers and
    // this answer always agree
    pub fn overlay_visible(&self) -> bool {
        self.overlay_shown
    }

    pub fn frame(&mut self, status: &FrameStatus) -> FrameOutput {
        let mut output = self.advance(status);
        self.overlay_shown = self.state == FreecamState::On && !self.menu_open && !self.path.points().is_empty();
        output.show_overlay = self.overlay_shown;
        self.refresh_overlay(status, &output);
        output
    }

    // rebuilt while visible, empty otherwise
    fn refresh_overlay(&mut self, status: &FrameStatus, output: &FrameOutput) {
        let Some(view) = output.render_override.filter(|_| self.overlay_shown) else {
            self.overlay.clear();
            return;
        };
        OverlayBuilder::build(
            self.path.curve(),
            self.path.points(),
            view,
            status.fov_y_degrees,
            status.aspect,
            status.screen_width,
            status.screen_height,
            self.motion.speed(),
            &mut self.overlay,
        );
    }

    fn advance(&mut self, status: &FrameStatus) -> FrameOutput {
        let dt = self.advance_clock(status.time_seconds);
        self.menu_open = status.menu_open;
        self.path.observe_world(status.in_world, status.dimension);
        if !status.focused {
            self.input.clear_held();
        }
        let toggle = std::mem::take(&mut self.toggle_requested);
        let handover = std::mem::take(&mut self.handover_requested);
        let play = std::mem::take(&mut self.play_requested);
        if self.is_active() {
            if !status.in_world {
                return self.finish("left the world");
            }
            if handover {
                return self.finish("the server took the camera");
            }
        }
        match self.state {
            FreecamState::Off if toggle => self.try_start(status),
            FreecamState::Off => FrameOutput::default(),
            FreecamState::On => self.run(status, dt, toggle, play),
            FreecamState::Playing => self.play(status, dt, toggle, play),
            FreecamState::Exiting { frames_left } => self.count_down(frames_left),
        }
    }

    fn is_active(&self) -> bool {
        self.state != FreecamState::Off
    }

    // seconds since the previous frame, 0 on the first, never clamped
    fn advance_clock(&mut self, time_seconds: f64) -> f64 {
        let dt = self.last_time.map_or(0.0, |last| (time_seconds - last).max(0.0));
        self.last_time = Some(time_seconds);
        dt
    }

    fn refusal(status: &FrameStatus) -> Option<StartRefusal> {
        if !status.in_world {
            Some(StartRefusal::NotInWorld)
        } else if status.menu_open {
            Some(StartRefusal::MenuOpen)
        } else if !status.ready {
            Some(StartRefusal::NotReady)
        } else if !status.player_camera_active {
            Some(StartRefusal::ServerCameraActive)
        } else {
            None
        }
    }

    fn try_start(&mut self, status: &FrameStatus) -> FrameOutput {
        if let Some(refusal) = Self::refusal(status) {
            info!("freecam start refused: {refusal:?}");
            return FrameOutput::default();
        }
        self.input.take_look();
        self.input.take_scroll();
        self.motion.reset(status.start_pose);
        self.telemetry.restart();
        self.restore_gui = !status.gui_hidden;
        self.health = status.health;
        self.dimension = status.dimension;
        self.state = FreecamState::On;
        info!("freecam on");
        Self::active_output(status.start_pose, GuiCommand::Hide, true)
    }

    fn run(&mut self, status: &FrameStatus, dt: f64, toggle: bool, play: bool) -> FrameOutput {
        if let Some(output) = self.check_exit(status, toggle) {
            return output;
        }
        if play && !status.menu_open && self.path.start(self.motion.speed()) {
            self.state = FreecamState::Playing;
            info!("freecam playing the path");
            return self.play_frame(status, dt);
        }
        let dt = dt as f32;
        let pose = if status.menu_open { self.hold_pose() } else { self.step_pose(dt) };
        let output = Self::active_output(pose, GuiCommand::None, true);
        self.record_telemetry(status, dt, &output, self.motion.velocity());
        output
    }

    // movement and look input are drained and discarded while the path plays
    fn play(&mut self, status: &FrameStatus, dt: f64, toggle: bool, stop: bool) -> FrameOutput {
        if let Some(output) = self.check_exit(status, toggle) {
            return output;
        }
        self.input.take_look();
        self.input.take_scroll();
        if stop {
            return self.end_playback("stopped");
        }
        self.play_frame(status, dt)
    }

    fn play_frame(&mut self, status: &FrameStatus, dt: f64) -> FrameOutput {
        let before = self.motion.pose().position;
        let (pose, step) = self.path.step(dt, status.menu_open);
        self.motion.reset(pose);
        if step.finished {
            return self.end_playback("path end");
        }
        let output = Self::active_output(pose, GuiCommand::None, false);
        if !step.settling {
            let dt = dt as f32;
            let velocity = if dt > 0.0 { (pose.position - before) * (1.0 / dt) } else { Vec3::default() };
            self.record_telemetry(status, dt, &output, velocity);
        }
        output
    }

    // the camera stands still where playback left it and freecam control resumes
    fn end_playback(&mut self, reason: &str) -> FrameOutput {
        self.path.stop();
        self.state = FreecamState::On;
        info!("freecam playback over: {reason}");
        Self::active_output(self.motion.pose(), GuiCommand::None, true)
    }

    // the exits shared by editing and playback
    fn check_exit(&mut self, status: &FrameStatus, toggle: bool) -> Option<FrameOutput> {
        let damaged = status.health < self.health;
        self.health = status.health;
        if toggle {
            return Some(self.begin_exit("toggle"));
        }
        if damaged {
            return Some(self.begin_exit("damage"));
        }
        if status.dimension != self.dimension {
            return Some(self.begin_exit("dimension change"));
        }
        None
    }

    fn record_telemetry(&mut self, status: &FrameStatus, dt: f32, output: &FrameOutput, velocity: Vec3) {
        let sent_set = matches!(output.camera, CameraCommand::Set(_));
        let forward = output.render_override.map_or(Vec3::default(), |pose| pose.forward());
        self.telemetry.record(dt, status, sent_set, velocity, forward);
        if let Some(report) = self.telemetry.take_report() {
            info!("freecam telemetry: {report:?}");
        }
    }

    fn hold_pose(&mut self) -> CameraPose {
        self.motion.hold();
        self.input.take_look();
        self.input.take_scroll();
        self.motion.pose()
    }

    fn step_pose(&mut self, dt: f32) -> CameraPose {
        let (dx, dy) = self.input.take_look();
        self.motion.add_look(dx, dy);
        self.motion.scroll(self.input.take_scroll());
        self.motion.step(dt, self.input.held())
    }

    fn active_output(pose: CameraPose, gui: GuiCommand, show_overlay: bool) -> FrameOutput {
        FrameOutput {
            camera: CameraCommand::Set(pose),
            render_override: Some(pose),
            gui,
            block_input: true,
            show_overlay,
        }
    }

    // the frame that starts the exit counts as exit frame 1
    fn begin_exit(&mut self, reason: &str) -> FrameOutput {
        self.path.stop();
        let frames_left = self.exit_wait_frames.saturating_sub(1);
        self.state = FreecamState::Exiting { frames_left };
        info!("freecam exiting: {reason}");
        Self::waiting_output()
    }

    fn count_down(&mut self, frames_left: u32) -> FrameOutput {
        if frames_left > 1 {
            self.state = FreecamState::Exiting { frames_left: frames_left - 1 };
            return Self::waiting_output();
        }
        let mut out = self.finish("exit wait over");
        out.camera = CameraCommand::Clear;
        out
    }

    fn waiting_output() -> FrameOutput {
        FrameOutput { block_input: true, ..FrameOutput::default() }
    }

    // switches off, gives the GUI back and sends no camera command
    fn finish(&mut self, reason: &str) -> FrameOutput {
        self.path.stop();
        self.state = FreecamState::Off;
        info!("freecam off: {reason}");
        let gui = if self.restore_gui { GuiCommand::Show } else { GuiCommand::None };
        FrameOutput { gui, ..FrameOutput::default() }
    }
}
