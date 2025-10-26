use std::sync::{
    Arc, Barrier,
    atomic::{AtomicPtr, Ordering},
};

#[derive(Debug)]
struct Node<T> {
    value: T,
    next: AtomicPtr<Node<T>>,
}
#[derive(Debug, Default)]
struct Stack<T> {
    head: AtomicPtr<Node<T>>,
}

impl<T: Copy> Stack<T> {
    pub fn push(&self, value: T) {
        let node = Box::new(Node {
            value,
            next: AtomicPtr::default(),
        });
        let ptr = Box::into_raw(node);
        loop {
            let curr = self.head.load(Ordering::Acquire);
            unsafe {
                (*ptr).next.store(curr, Ordering::Relaxed);
            }
            if self
                .head
                .compare_exchange_weak(curr, ptr, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                break;
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        loop {
            let curr = self.head.load(Ordering::Acquire);
            if curr.is_null() {
                return None;
            }
            let next = unsafe { (*curr).next.load(Ordering::Acquire) };
            if let Ok(ptr) =
                self.head
                    .compare_exchange_weak(curr, next, Ordering::AcqRel, Ordering::Acquire)
            {
                let x = unsafe { Box::from_raw(ptr) };
                let r = Some(x.value);
                // TODO: mem leak
                std::mem::forget(x);
                return r;
            }
        }
    }
}

#[test]
fn test_stack() {
    let stack = Arc::new(Stack::default());
    let barrier = Arc::new(Barrier::new(4));
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let v = Arc::clone(&stack);
            let b = Arc::clone(&barrier);
            std::thread::spawn(move || {
                let nloop = 1000_000;
                for i in 0..nloop {
                    v.push(i);
                }
                b.wait();
                let mut n = 0;
                while let Some(_) = v.pop() {
                    n += 1;
                }
                n
            })
        })
        .collect();
    let mut results = workers
        .into_iter()
        .map(|w| w.join().unwrap_or_default())
        .collect::<Vec<_>>();
    let mut n = 0;
    while let Some(_) = stack.pop() {
        n += 1;
    }
    results.push(n);
    println!("{:?}, +=> {}", results, results.iter().sum::<usize>());
    assert_eq!(4000_000, results.iter().sum::<usize>());
}
