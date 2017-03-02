extern crate futures;
extern crate tokio_core;
extern crate serde_json;
extern crate hyper;
extern crate queryst;

use serde_json::Value as JsonValue;

pub type Params = serde_json::value::Map<String, JsonValue>;

mod request;
pub use request::{Request, Method};

mod server;
pub use server::Server;

mod reply;
pub use reply::Reply;

mod adapter;
pub use adapter::{Adapter, MemoryAdapter};

mod resource;
pub use resource::{Resource};

mod channel;
pub use channel::{Channel, Sender, BroadcastChannel};

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
  }
}
