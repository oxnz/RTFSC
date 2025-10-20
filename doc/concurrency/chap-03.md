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