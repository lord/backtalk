extern crate ws;
extern crate futures;
extern crate tokio_core;
extern crate serde_json;
extern crate hyper;

use serde_json::Value as JsonValue;

pub type Params = serde_json::value::Map<String, JsonValue>;

mod req;
pub use req::{Req, Method};

mod server;
pub use server::Server;

mod reply;
pub use reply::Reply;

mod adapter;
pub use adapter::{Adapter, MemoryAdapter}; // TODO memory adapter should probably eventually go in its own crate

mod resource;
pub use resource::{Resource};

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let mut s = Server::new();
    let r = Resource::new(MemoryAdapter{});
    s.mount("/hello", r);
    s.listen("127.0.0.1");
  }
}
