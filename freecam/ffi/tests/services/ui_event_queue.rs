use freecam_core::structs::UiEvent;
use freecam_ffi::services::UiEventQueue;

fn moved(x: f32) -> UiEvent {
    UiEvent::PointerMoved { x, y: 0.0 }
}

#[test]
fn push_past_capacity_evicts_the_oldest_and_counts_it() {
    let queue = UiEventQueue::new(16);

    for i in 0..20 {
        queue.push(moved(i as f32));
    }

    assert_eq!(queue.len(), 16);
    assert_eq!(queue.dropped(), 4);
    let kept: Vec<UiEvent> = std::iter::from_fn(|| queue.pop()).collect();
    assert_eq!(kept, (4..20).map(|i| moved(i as f32)).collect::<Vec<_>>());
}

#[test]
fn zero_capacity_still_keeps_the_newest_event() {
    let queue = UiEventQueue::new(0);

    queue.push(moved(1.0));
    queue.push(moved(2.0));

    assert_eq!(queue.pop(), Some(moved(2.0)));
    assert_eq!(queue.dropped(), 1);
}
