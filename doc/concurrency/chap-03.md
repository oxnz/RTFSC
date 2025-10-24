# Sharing data between threads

## race condition

- spotting race condition inherent in interfaces

## Deadlock

- avoid nested locks
- avoid calling user-supplied code with holding a lock
- aquire locks in a fixed order
    - std::lock(m1, m2, ...)
    - std::scoped_lock(m1, m2, ...)
- use a lock hierarchy
    - deadlocks between hierarchical mutexes are impossible, cause the mutexes themselves enforce the lock ordering.
    - drawback: impractical for a hand-over-hand locking shemes (require each mutex in the chain has a lower hierarchy value than prior one)
- locking at an appropriate granularity
    - guideline: choose a sufficiently coarse lock granularity to ensure the required data is protected, also to ensure that a lock is held only for the operations that require it.
    - isn't only about the amount of data locked; it's also about how long the lock is held and what operations are performed while the lock is held.
    - fine-grained lock
    - coarse-grained lock


### `std::unique_lock`

- more flexibility than `std::lock_guard`
    - doesn't always own the mutex that it's associated with
    - can construct with mutex unlocked with `std::defer_lock`
    - can release lock early before destruction with `unlock()`
- can transfer mutex ownership between scopes


## Alternative facilities for protecting shared data

- protecting shared data during initialization
    - std::once_flag and std::call_once
        - std::once_flag cannot be copied or moved.
    - static local variable initialization after c++11 happen exactly once, can be alternative of call_once
- protecting rarely updated data structures
    - reader-writer lock
        - std::shared_muex
        - std::shared_timed_mutex
- recursive locking
    - `std::recursive_mutex`
        - can acquire multiple locks on a single instance from the same thread
        - the lock and unlock number needs to match
    - most of the time, should consider change design before adopt recursive mutex

> The classic double-checked locking (DCL) pattern is tricky because it involves reordering issues, which are very easy to get wrong without proper memory ordering.
> can fix it by using an atomic pointer with proper acquire-release semantics
