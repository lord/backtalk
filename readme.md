<p align="center">
  <img src="https://raw.githubusercontent.com/lord/img/master/logo-backtalk.png" alt="Backtalk: API Web Server" width="226">
  <br>
  <a href="https://travis-ci.org/lord/slate"><img src="https://travis-ci.org/lord/backtalk.svg?branch=master" alt="Build Status"></a>
  <a href="https://crates.io/crates/backtalk"><img src="https://img.shields.io/crates/v/backtalk.svg" alt="Crate Info"></a>
  <a href="https://docs.rs/backtalk"><img src="https://img.shields.io/badge/docs.rs-visit-green.svg" alt="Documentation"></a>
</p>

Backtalk is a web framework for Rust. It's:

- **Asynchronous** – use Futures for everything.
- **Realtime** – expose the same API over both websockets and HTTP, and push live events to clients.
- **Simple** – minimalist tools, easily composed.
- **Opinionated** – exclusively for rapidly building JSON-based RESTful APIs.

A simple server example:

```rust
let mut tasks = Resource::new(MemoryAdapter{});
tasks.guard(Methods::Post, backtalk_validate::require("title"));

let mut srv = Server::new();
srv.mount("/tasks", tasks);
srv.listen("127.0.0.1");
```

## Tasks

- [x] add JSON parsing and serialization into Req and Reply objects
- [ ] add `Resource` trait with various methods
- [ ] add `Filter` and `Guard` traits
- [ ] add proper routing to `Server`

## Objects

- `Resource` is an object that receives requests. Usually corresponds to a particular type of object. Allows adding hooks and methods and error handlers.
- `Request` is a request for data, either over HTTP or Websockets
- `Reply` is a response object representing JSON/BSON data that will be returned to the client, and a HTTP status (from a subset of subset of the messages)
- `Guard` is a function that accepts a Request and returns a Future<Request, Reply>.
- `Filter` is a function that accepts a Reply and returns a Future<Reply, Reply>.

## Inspiration

- Feathers.js
- Phoenix
- Rocket.rs
