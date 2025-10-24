# The C++ memory model and operations on atomic types

## Memory model basics

the two aspects to the memory model:

- the basic structural aspects, which relate to how things are laid out in memory
    - import for concurrency
- the concurrency aspects.
    - 3 models
        - sequentially consistent: memory_order_seq_cst
            - all threads must see the same order of operations
        - acquire-release: memory_order_acq_rel
        - relaxed: memory_order_relaxed

objects and memory locations

modification order

## atomic operations and types in C++

`<atomic>`

`std::atomic_shared_ptr<T>`

## synchronizing operations and enforcing ordering

A release operation on an atomic variable synchronizes-with an acquire operation on the same variable that reads the value written (or a later value written by that release).

Synchronizes-with connects threads by linking a release to an acquire on the same atomic variable — it’s what makes “happens-before” cross thread boundaries.

```
Thread 1                          Thread 2
---------                         ---------
data = 42;                        (reads data)
ready.store(true, release);   ---> ready.load(acquire);
                                   ↓
                            all prior writes in T1
                              now visible in T2
```


summary rules

-  Publish data from one thread to another
    - release + acquire on same atomic
- Just atomic counter with no dependency
    - relaxed
- Strong total ordering (debug friendly)
    - seq_cst
- Lock-free data exchange
    - acquire/release pair


## References

- https://en.cppreference.com/w/cpp/atomic/memory_order.html
