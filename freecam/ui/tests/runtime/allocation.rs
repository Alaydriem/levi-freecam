use stats_alloc::Region;

use crate::GLOBAL;
use crate::support::ui_fixture::UiFixture;

const WARM_UP_FRAMES: usize = 10;
const MEASURED_FRAMES: usize = 200;

fn capacities(fixture: &UiFixture) -> [usize; 3] {
    [fixture.out.vertices.capacity(), fixture.out.indices.capacity(), fixture.out.draws.capacity()]
}

#[test]
fn output_buffers_stop_growing_after_warm_up() {
    let mut fixture = UiFixture::new().cursor_mode();
    for _ in 0..WARM_UP_FRAMES {
        fixture.frame(&[]);
    }
    let warm = capacities(&fixture);

    for _ in 0..MEASURED_FRAMES {
        fixture.frame(&[]);
    }

    assert_eq!(capacities(&fixture), warm);
}

#[test]
fn report_egui_allocations_per_frame() {
    let scenes = [
        ("hidden", UiFixture::new()),
        ("strip + legend", UiFixture::new()),
        ("strip + legend + bar, 3 points", UiFixture::new().with_path()),
        ("strip + legend + bar, 256 points", UiFixture::new().with_points(256)),
    ];
    for (name, mut fixture) in scenes {
        fixture.snapshot.visible = name != "hidden";
        for _ in 0..WARM_UP_FRAMES {
            fixture.frame(&[]);
        }
        let region = Region::new(&GLOBAL);
        let mut frames = 0;
        for _ in 0..MEASURED_FRAMES {
            fixture.frame(&[]);
            frames += 1;
        }
        let change = region.change();
        let per_frame = |count: usize| count as f64 / frames as f64;
        println!(
            "{name}: {:.1} allocations, {:.1} reallocations, {:.0} bytes allocated per frame over {frames} frames",
            per_frame(change.allocations),
            per_frame(change.reallocations),
            per_frame(change.bytes_allocated),
        );
        assert_eq!(frames, MEASURED_FRAMES);
    }
}

// allocations per frame over the measured frames, after the warm-up
fn allocations_per_frame(fixture: &mut UiFixture) -> usize {
    for _ in 0..WARM_UP_FRAMES {
        fixture.frame(&[]);
    }
    let region = Region::new(&GLOBAL);
    for _ in 0..MEASURED_FRAMES {
        fixture.frame(&[]);
    }
    region.change().allocations / MEASURED_FRAMES
}

#[test]
fn bar_allocations_do_not_grow_with_points() {
    let few = allocations_per_frame(&mut UiFixture::new().with_points(3));
    let many = allocations_per_frame(&mut UiFixture::new().with_points(256));

    assert!(many <= few, "3 points: {few} allocations per frame, 256 points: {many}");
}
