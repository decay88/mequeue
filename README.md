# Mequeue
Mequeue is an executor for MEV bots optimized to be able to process multiple transactions concurrently.

The main goal is to make this executor to add as less resource-overhead as possible while keeping it simple to use.

[Package](https://crates.io/crates/mequeue) |
[Chat](https://t.me/mequeue)

```toml
[dependencies]

mequeue = { git = "https://github.com/mekosko/mequeue" }
```
Simple bot example can be found in the [interceptor][interceptor] folder.

[interceptor]: https://github.com/mekosko/mequeue/tree/main/interceptor

# Design decisions and why they were made
It was inspired by how async executors are made. There are several components:

1. Workers, the number of which you can specify when creating an executor.
2. A workers manager that restarts them every time the shared state changes.
3. Shared among workers mpmc queue. Each event can be received by only one worker.
4. Broadcast channel used to update shared between workers state.

## Why we restart workers on every state change?
State change implies that all computations made by the workers are no longer relevant.

We don't want to spend our time on outdated computations, so we just abort workers and start them again.

## Why do we prefer concurrent execution?
Because of concurrent execution while one worker is awaiting data from external source other can work.

Moreover, tokio can run coroutines in parallel. So workers can process events in parallel.
