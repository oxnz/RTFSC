mod epoll;
mod kqueue;
mod select;

pub use kqueue::{Action, Filter, KEvent as Event, KQueue as EventQueue};

#[test]
fn test() {
    let mut q = EventQueue::try_new().unwrap();
    let mut events = vec![Event::new(
        libc::STDOUT_FILENO as usize,
        Action::add(),
        Filter::new().write(),
    )];
    let r = q.update(&events);
    assert!(r.is_ok());
    let n = q.poll(&mut events, None);
    assert_eq!(n.ok(), Some(1));
}
