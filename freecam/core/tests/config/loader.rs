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
    assert_eq!(config.keys.delete, 0x2E);
    assert_eq!(config.keys.cursor, 0x45);
}

#[test]
fn tab_as_cursor_key_falls_back_to_e() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.json");
    std::fs::write(&path, r#"{"keys":{"cursor":9}}"#).unwrap();

    assert_eq!(ConfigLoader::load(&path).keys.cursor, 0x45);
}

#[test]
fn old_undo_point_key_still_loads() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.json");
    std::fs::write(&path, r#"{"keys":{"undoPoint":90}}"#).unwrap();

    assert_eq!(ConfigLoader::load(&path).keys.delete, 0x2E);
}

#[test]
fn ui_values_are_clamped() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.json");

    std::fs::write(&path, r#"{"ui":{"eventQueueCapacity":1,"maxVertices":10}}"#).unwrap();
    let low = ConfigLoader::load(&path);
    assert_eq!(low.ui.event_queue_capacity, 16);
    assert_eq!(low.ui.max_vertices, 4096);

    std::fs::write(&path, r#"{"ui":{"eventQueueCapacity":99999,"maxVertices":9000000000}}"#).unwrap();
    let high = ConfigLoader::load(&path);
    assert_eq!(high.ui.event_queue_capacity, 4096);
    assert_eq!(high.ui.max_vertices, 1_048_576);
}

#[test]
fn ui_defaults() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.json");
    std::fs::write(&path, "{}").unwrap();

    let config = ConfigLoader::load(&path);

    assert_eq!(config.ui.event_queue_capacity, 256);
    assert_eq!(config.ui.max_vertices, 65536);
}

#[test]
fn marker_and_look_at_defaults() {
    let config = ConfigLoader::load(Path::new("definitely/not/here/config.json"));

    assert!(config.markers.enabled);
    assert_eq!(config.markers.max_shapes, 4096);
    assert_eq!(config.markers.max_per_send, 400);
    assert_eq!(config.markers.line_weight, 2);
    assert_eq!(config.markers.frustum_depth, 1.2);
    assert!(config.look_at.enabled);
    assert_eq!(config.look_at.max_distance, 256.0);
}

#[test]
fn missing_marker_and_look_at_blocks_load_as_defaults() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.json");
    std::fs::write(&path, r#"{"telemetry":true}"#).unwrap();

    let config = ConfigLoader::load(&path);

    assert_eq!(config.markers, FreecamConfig::default().markers);
    assert_eq!(config.look_at, FreecamConfig::default().look_at);
}

#[test]
fn marker_and_look_at_values_are_clamped() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.json");

    std::fs::write(
        &path,
        r#"{"markers":{"enabled":false,"maxShapes":1,"maxPerSend":1,"lineWeight":0,"frustumDepth":0.0},"lookAt":{"enabled":false,"maxDistance":1.0}}"#,
    )
    .unwrap();
    let low = ConfigLoader::load(&path);
    assert!(!low.markers.enabled);
    assert_eq!(low.markers.max_shapes, 2560);
    assert_eq!(low.markers.max_per_send, 50);
    assert_eq!(low.markers.line_weight, 1);
    assert_eq!(low.markers.frustum_depth, 0.3);
    assert!(!low.look_at.enabled);
    assert_eq!(low.look_at.max_distance, 8.0);

    std::fs::write(
        &path,
        r#"{"markers":{"maxShapes":99999,"maxPerSend":5000,"lineWeight":7,"frustumDepth":40.0},"lookAt":{"maxDistance":9000.0}}"#,
    )
    .unwrap();
    let high = ConfigLoader::load(&path);
    assert_eq!(high.markers.max_shapes, 8192);
    assert_eq!(high.markers.max_per_send, 800);
    assert_eq!(high.markers.line_weight, 2);
    assert_eq!(high.markers.frustum_depth, 5.0);
    assert_eq!(high.look_at.max_distance, 512.0);
}

#[test]
fn non_finite_marker_and_look_at_floats_use_defaults() {
    let mut config = FreecamConfig::default();
    config.markers.frustum_depth = f32::NAN;
    config.look_at.max_distance = f32::INFINITY;

    let config = config.sanitized();

    assert_eq!(config.markers.frustum_depth, 1.2);
    assert_eq!(config.look_at.max_distance, 256.0);
}

#[test]
fn old_config_with_play_and_test_panel_loads() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.json");
    std::fs::write(&path, r#"{"keys":{"play":13,"addPoint":78},"ui":{"testPanel":true,"maxVertices":8192}}"#).unwrap();

    let config = ConfigLoader::load(&path);

    assert_eq!(config.keys.add_point, 78);
    assert_eq!(config.ui.max_vertices, 8192);
    let defaults = FreecamConfig::default();
    assert_eq!(config.keys.toggle, defaults.keys.toggle);
    assert_eq!(config.ui.event_queue_capacity, defaults.ui.event_queue_capacity);
}

#[test]
fn default_folder_is_pictures_screenshots() {
    let config = ConfigLoader::load(Path::new("definitely/not/here/config.json"));

    assert_eq!(config.screenshot.folder, r"%USERPROFILE%\Pictures\Screenshots");
}

#[test]
fn screenshot_folder_is_read_and_a_blank_one_falls_back() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.json");
    std::fs::write(&path, r#"{"screenshot":{"folder":"D:\\shots"}}"#).unwrap();
    assert_eq!(ConfigLoader::load(&path).screenshot.folder, r"D:\shots");

    std::fs::write(&path, r#"{"screenshot":{"folder":"  "}}"#).unwrap();
    assert_eq!(ConfigLoader::load(&path).screenshot.folder, FreecamConfig::default().screenshot.folder);
}
