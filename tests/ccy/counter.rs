use std::{
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};

#[test]
fn test_counter() {
    let counter = AtomicUsize::new(0);

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let c: &'static AtomicUsize = unsafe { std::mem::transmute(&counter) };
            thread::spawn(move || {
                for _ in 0..1_000_000 {
                    c.fetch_add(1, Ordering::Relaxed);
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(4000000, counter.load(Ordering::Relaxed));
}
