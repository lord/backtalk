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
- [x] add `Resource` trait with various methods
- [ ] add `Filter` and `Guard` traits
- [ ] add proper routing to `Server`, with RouteRef or something like that instead of a string. RouteRef would also contain, like, url params or something maybe? hmm. would be nice if 
- [ ] broadcasting events to event listeners

elixir shit:

```elixir
defmodule HelloPhoenix.RoomChannel do
  use Phoenix.Channel
  def join("room:lobby", _message, socket) do
    {:ok, socket}
  end
  def join("room:" <> _private_room_id, _params, _socket) do
    {:error, %{reason: "unauthorized"}}
  end
  def handle_in("new_msg", %{"body" => body}, socket) do
    broadcast! socket, "new_msg", %{body: body}
    {:noreply, socket}
  end
  def handle_out("new_msg", payload, socket) do
    push socket, "new_msg", payload
    {:noreply, socket}
  end
end
```

// TODO: eventually should be https://docs.rs/futures/0.1/futures/future/trait.IntoFuture.html

// TODO could a client continue the connection even after the 404? make sure not

// don't support PUT? https://tools.ietf.org/html/rfc7396 and http://williamdurand.fr/2014/02/14/please-do-not-patch-like-an-idiot/

// TODO be able to return a future of anything that can be IntoReply instead of just Reply?

// TODO I think macros can help with reducing usage of BoxFuture which is slower?
//      it would be cool if we used futures in a zero-cost way
//      also, it would be nice if we didn't have to write ok(fut).boxed() everywhere
//      see Rocket for inspiration

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
