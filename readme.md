<p align="center">
  <img src="https://raw.githubusercontent.com/lord/img/master/logo-backtalk.png" alt="Backtalk: API Web Server" width="226">
  <br>
  <a href="https://travis-ci.org/lord/backtalk"><img src="https://travis-ci.org/lord/backtalk.svg?branch=master" alt="Build Status"></a>
  <!--<a href="https://crates.io/crates/backtalk"><img src="https://img.shields.io/crates/v/backtalk.svg" alt="Crate Info"></a>
  <a href="https://docs.rs/backtalk"><img src="https://img.shields.io/badge/docs.rs-visit-brightgreen.svg" alt="Documentation"></a>-->
</p>

Backtalk is a web framework for Rust. Much is subject to change and it's probably not ready for writing production sites, but the structure is there, and I'm glad to answer questions/help out if the documentation isn't enough.

- **Asynchronous** – use Futures for everything, handle thousands of concurrent connections.
- **Realtime** – expose a streaming API, and push live events to clients.
- **Simple** – only a couple hundred lines of code.
- **Opinionated** – exclusively for JSON-based RESTful APIs.
- **Magicless** – no macros, no unsafe, runs on stable Rust.

A simple server example:

```rust
let mut server = Server::new();
let database = memory::MemoryAdapter::new();
server.resource("/cats", move |req: Request| {
  database.handle(req)
});
server.listen("127.0.0.1:3000");
```

You can look in the `examples` directory for more information, or the [blog post](https://lord.io/blog/2017/backtalk) walking through the examples.

## Inspiration

- Feathers.js
- Phoenix
- Rocket.rs
