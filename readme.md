<p align="center">
  <img src="https://raw.githubusercontent.com/lord/img/master/logo-backtalk.png" alt="Backtalk: API Web Server" width="226">
  <br>
  <a href="https://travis-ci.org/lord/backtalk"><img src="https://travis-ci.org/lord/backtalk.svg?branch=master" alt="Build Status"></a>
  <a href="https://crates.io/crates/backtalk"><img src="https://img.shields.io/crates/v/backtalk.svg" alt="Crate Info"></a>
  <a href="https://docs.rs/backtalk"><img src="https://img.shields.io/badge/docs.rs-visit-brightgreen.svg" alt="Documentation"></a>
</p>

Backtalk is a web framework for Rust. It's currently **very incomplete, undocumented, and in development**, and much is subject to change. The code is still messy and unperformant right now.

- **Asynchronous** – use Futures for everything, handle thousands of concurrent connections.
- **Realtime** – expose a streaming API, and push live events to clients.
- **Simple** – only a couple hundred lines of code.
- **Opinionated** – exclusively for JSON-based RESTful APIs.
- **Magicless** – no macros, and runs on stable Rust.

A simple server example:

```rust
let mut tasks = Resource::new(MemoryAdapter{});
tasks.guard(Methods::Post, backtalk_validate::require("title"));

let mut srv = Server::new();
srv.mount("/tasks", tasks);
srv.listen("127.0.0.1");
```

## Why SEE instead of websockets?

- SSE works automatically over our existing SSL
- SSE works over HTTP/2. Websockets does not.
- SSE has reconnection and message replay/catchup built in to it automatically
- websockets supports requests in both directions, but we can just AJAX it
  - we'd worry about AJAX overhead, but thanks to HTTP/2, that's not a concern
- once Hyper supports HTTP/2, we'll have built-in multiplexing over a single TCP connection handled by the browser

## Things

- `Resource` is an object that receives requests. Usually corresponds to a particular type of object. Allows adding hooks.
- `Adapter` is a database adapter passed in to the resource. It talks to the database with some simple CRUD functions.
- `Req` is a request, either over HTTP or Websockets
- `Reply` is a response object representing JSON data that will be returned to the client, and a HTTP status (from a subset of subset of the messages)
- `BeforeHook` accepts a Req and returns a Future<Req, Reply>.
- `AfterHook` accepts a Reply and returns a Future<Reply, Reply>. If the future gets resolved to an error reply, it's sent directly, skipping any other hooks.
- `Channel` accepts incoming events and determines which outgoing connections to broadcast them to.

## Inspiration

- Feathers.js
- Phoenix
- Rocket.rs
