use crate::services::PathCurve;
use crate::services::path_player::EASE_RAMP_SECONDS;
use crate::services::projection::{NEAR_PLANE, Projection};
use crate::structs::CameraPose;
use crate::structs::overlay::path_overlay::OVERLAY_SAMPLES_PER_SEGMENT;
use crate::structs::overlay::{PathOverlay, ScreenMarker, ScreenSegment};

const WEDGE_LENGTH: f32 = 1.5;
const MIN_FOV_DEGREES: f32 = 1.0;
const MAX_FOV_DEGREES: f32 = 179.0;
const MIN_SHOT_POINTS: usize = 2;

// projects a path to screen space for one view
pub struct OverlayBuilder;

impl OverlayBuilder {
    // refills `out` without allocating; `speed` is the cruise speed in blocks per second
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        curve: &PathCurve,
        points: &[CameraPose],
        view: CameraPose,
        fov_y_degrees: f32,
        aspect: f32,
        width: f32,
        height: f32,
        speed: f32,
        out: &mut PathOverlay,
    ) {
        out.clear();
        out.point_count = points.len() as u32;
        if !Self::inputs_usable(points, view, fov_y_degrees, aspect, width, height, speed) {
            return;
        }
        out.shot_seconds = Self::shot_seconds(curve, points.len(), speed);
        let projection = Projection::new(view, fov_y_degrees, aspect, width, height);
        Self::build_line(curve, &projection, out);
        Self::build_markers(points, &projection, out);
    }

    // false for a view that cannot be projected or any input that is not a finite number
    fn inputs_usable(
        points: &[CameraPose],
        view: CameraPose,
        fov_y_degrees: f32,
        aspect: f32,
        width: f32,
        height: f32,
        speed: f32,
    ) -> bool {
        let scalars = [fov_y_degrees, aspect, width, height, speed];
        scalars.iter().all(|value| value.is_finite())
            && fov_y_degrees > MIN_FOV_DEGREES
            && fov_y_degrees < MAX_FOV_DEGREES
            && aspect > 0.0
            && width > 0.0
            && height > 0.0
            && Self::pose_is_finite(&view)
            && points.iter().all(Self::pose_is_finite)
    }

    fn pose_is_finite(pose: &CameraPose) -> bool {
        let position = pose.position;
        [position.x, position.y, position.z, pose.yaw, pose.pitch].iter().all(|value| value.is_finite())
    }

    // the length over the speed, plus the two smoothstep ramps that together cost one ramp time
    fn shot_seconds(curve: &PathCurve, point_count: usize, speed: f32) -> f32 {
        if speed <= 0.0 || point_count < MIN_SHOT_POINTS {
            return 0.0;
        }
        curve.length() / speed + EASE_RAMP_SECONDS
    }

    fn build_line(curve: &PathCurve, projection: &Projection, out: &mut PathOverlay) {
        curve.sample_positions(OVERLAY_SAMPLES_PER_SEGMENT, &mut out.samples);
        for pair in out.samples.windows(2) {
            let ends = (projection.to_camera(pair[0]), projection.to_camera(pair[1]));
            let Some((from, to)) = Projection::clip(ends.0, ends.1) else {
                continue;
            };
            out.segments.push(ScreenSegment { from: projection.to_screen(from), to: projection.to_screen(to) });
        }
    }

    fn build_markers(points: &[CameraPose], projection: &Projection, out: &mut PathOverlay) {
        for (index, point) in points.iter().enumerate() {
            let at_camera = projection.to_camera(point.position);
            if at_camera.z < NEAR_PLANE {
                continue;
            }
            let at = projection.to_screen(at_camera);
            let wedge_camera = projection.to_camera(point.position + point.forward() * WEDGE_LENGTH);
            let wedge_to = if wedge_camera.z < NEAR_PLANE { at } else { projection.to_screen(wedge_camera) };
            out.markers.push(ScreenMarker {
                at,
                wedge_to,
                number: index as u32 + 1,
                is_last: index + 1 == points.len(),
            });
        }
    }
}
