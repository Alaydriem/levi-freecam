//! Headless UI bench: runs the UI for a fixed number of frames with a fixed snapshot and prints the cost of each phase.
//!
//! `mise run bench-ui`. A dev tool, never loaded by the game, so it prints to stdout.

use std::time::{Duration, Instant};

use freecam_core::config::KeyConfig;
use freecam_core::structs::{UiFrameInput, UiFrameOutput, UiLegend, UiSnapshot};
use freecam_ui::runtime::UiRuntime;

struct Bench;

impl Bench {
    const FRAMES: usize = 600;
    const MAX_VERTICES: usize = 65536;

    const SCENES: [usize; 2] = [5, 256];
    const POINT_SPACING: f32 = 4.0;

    fn run() {
        for points in Self::SCENES {
            Self::scene(points);
        }
    }

    // bar mode with `points` points evenly spread, the last one selected
    fn snapshot(points: usize) -> UiSnapshot {
        let mut snapshot = UiSnapshot {
            visible: true,
            cursor_mode: true,
            point_count: points as u32,
            length: (points - 1) as f32 * Self::POINT_SPACING,
            playhead: 2.0,
            playhead_seconds: 0.5,
            total_seconds: 4.5,
            speed: 6.0,
            selection: Some(points - 1),
            ..UiSnapshot::default()
        };
        for (index, distance) in snapshot.point_distances[..points].iter_mut().enumerate() {
            *distance = index as f32 * Self::POINT_SPACING;
        }
        snapshot
    }

    fn scene(points: usize) {
        let mut runtime = UiRuntime::new(Self::MAX_VERTICES);
        runtime.set_profiling(true);
        let snapshot = Self::snapshot(points);
        let legend = UiLegend::new(&KeyConfig::default());
        let mut input =
            UiFrameInput { screen_width: 1920.0, screen_height: 1080.0, pixels_per_point: 1.0, time_seconds: 0.0 };
        let mut out = UiFrameOutput::new();

        let mut total = Vec::with_capacity(Self::FRAMES);
        let mut run = Vec::with_capacity(Self::FRAMES);
        let mut tessellate = Vec::with_capacity(Self::FRAMES);
        let mut copy = Vec::with_capacity(Self::FRAMES);
        let mut first = Duration::ZERO;
        for frame in 0..Self::FRAMES {
            input.time_seconds += 1.0 / 60.0;
            let started = Instant::now();
            runtime.run(&input, &snapshot, &legend, &mut std::iter::empty(), &mut out);
            let elapsed = started.elapsed();
            if frame == 0 {
                first = elapsed;
                continue;
            }
            let profile = runtime.profile();
            total.push(elapsed);
            run.push(profile.run);
            tessellate.push(profile.tessellate);
            copy.push(profile.copy);
        }

        println!("ui_bench: {} frames at 1920x1080, strip + legend + bar with {points} points", Self::FRAMES);
        println!("first frame (fonts, atlas upload): {:.3} ms", Self::ms(first));
        println!(
            "last frame: {} vertices, {} indices, {} draws",
            out.vertices.len(),
            out.indices.len(),
            out.draws.len()
        );
        println!("phase          avg ms    p95 ms   (frames 2..{})", Self::FRAMES);
        for (name, samples) in
            [("run", &mut run), ("tessellate", &mut tessellate), ("copy", &mut copy), ("total", &mut total)]
        {
            let (avg, p95) = Self::stats(samples);
            println!("{name:<12} {avg:>8.4}  {p95:>8.4}");
        }
    }

    fn stats(samples: &mut [Duration]) -> (f64, f64) {
        if samples.is_empty() {
            return (0.0, 0.0);
        }
        samples.sort_unstable();
        let avg = samples.iter().map(|sample| Self::ms(*sample)).sum::<f64>() / samples.len() as f64;
        let index = ((samples.len() as f64 * 0.95).ceil() as usize).saturating_sub(1).min(samples.len() - 1);
        (avg, Self::ms(samples[index]))
    }

    fn ms(duration: Duration) -> f64 {
        duration.as_secs_f64() * 1000.0
    }
}

fn main() {
    Bench::run();
}
