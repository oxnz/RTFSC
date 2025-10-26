use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn test_counter() {
    let counter = AtomicUsize::new(0);

    std::thread::scope(|s| {
        for _i in 0..4 {
            let c = &counter;
            s.spawn(move || {
                for _ in 0..1_000_000 {
                    c.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });

    assert_eq!(4000000, counter.load(Ordering::Relaxed));
}
