# Mequeue
Mequeue is an executor for MEV bots.

The main goal is to make this executor to add as less resource-overhead as possible while keeping it simple to use.

[Package](https://crates.io/crates/mequeue) |
[Chat](https://t.me/mequeue)

# Design
The only overhead source is the callback search algorithm, it works in O(n) where n is the number of callbacks.

Callbacks may produce events, they are dispatched as usual events.

Executor is mainly built with generics. It works in async context, every inner call is async.

There's no inner queue despite the name.

# Usage
```toml
[dependencies]

mequeue = { git = "https://github.com/mekosko/mequeue" }
```
Simple bot example can be found in the [interceptor][interceptor] folder.

[interceptor]: https://github.com/mekosko/mequeue/tree/main/interceptor
