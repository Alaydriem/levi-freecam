use std::path::Path;

use freecam_core::config::{ConfigLoader, FreecamConfig};
use tempfile::TempDir;

#[test]
fn missing_file_gives_defaults() {
    let config = ConfigLoader::load(Path::new("definitely/not/here/config.json"));

    assert_eq!(config.keys.toggle, 0x50);
    assert_eq!(config.keys.smooth_toggle, 0xDB);
    assert_eq!(config.motion.start_speed, 6.0);
    assert_eq!(config.motion.max_speed, 80.0);
    assert_eq!(config.exit_wait_frames, 10);
    assert!(!config.telemetry);
}

#[test]
fn broken_json_gives_defaults() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.json");
    std::fs::write(&path, "{not json").unwrap();

    assert_eq!(ConfigLoader::load(&path), FreecamConfig::default());
}

#[test]
fn partial_json_keeps_other_defaults() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.json");
    std::fs::write(&path, r#"{"motion":{"sensitivity":0.3}}"#).unwrap();

    let config = ConfigLoader::load(&path);

    assert_eq!(config.motion.sensitivity, 0.3);
    assert_eq!(config.motion.look_ease_rate, 14.0);
}

#[test]
fn sanitized_fixes_bad_ranges() {
    let mut config = FreecamConfig::default();
    config.motion.min_speed = -1.0;
    config.motion.max_speed = 0.1;
    config.motion.start_speed = 500.0;
    config.motion.pitch_limit = 120.0;
    config.motion.max_frame_dt = 0.0;
    config.exit_wait_frames = 0;

    let config = config.sanitized();

    assert_eq!(config.motion.min_speed, 0.5);
    assert!(config.motion.max_speed >= config.motion.min_speed);
    assert_eq!(config.motion.start_speed, config.motion.max_speed);
    assert_eq!(config.motion.pitch_limit, 89.9);
    assert_eq!(config.motion.max_frame_dt, 0.1);
    assert_eq!(config.exit_wait_frames, 4);
}

#[test]
fn new_keys_have_defaults() {
    let config = ConfigLoader::load(Path::new("definitely/not/here/config.json"));

    assert_eq!(config.keys.add_point, 0x42);
    assert_eq!(config.keys.undo_point, 0x5A);
    assert_eq!(config.keys.play, 0x0D);
}
