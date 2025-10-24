# Synchronizaing concurrent operations

## Waiting for an event

- Waiting for a condition with condition variables
- thread safe concurrent queue

## Waiting for one-off events with futures

- Returning values from background tasks
    - Using std::future to get the return value of an asynchronous task
- Making (std::)promises
- Saving an exception for the future
- Waiting from multiple threads
    - std::shared_future copyable

## Waiting with a time limit

topics

- clocks
    - steady
- duration
- timepoint
- Functions that accept timeouts
    - `_for`
    - `_until`

## Using the synchronization of operations to simplify code

### concurrent programming paradigm

- FP (functional programming)
- CSP (Communication Sequential Processes)
    - threads are conceptually entirely separate, with no shared data but with communication channels that allow messages to be passed between them
