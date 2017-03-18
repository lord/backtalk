extern crate futures;
extern crate tokio_core;
#[macro_use]
extern crate serde_json;
extern crate hyper;
extern crate queryst;
extern crate uuid;

pub use serde_json::Value as JsonValue;
pub type JsonMap = serde_json::Map<String, JsonValue>; // TODO ADD THIS EVERYWHERE

pub type Params = serde_json::value::Map<String, JsonValue>;

mod request;
pub use request::{Request, Method};

mod server;
pub use server::Server;

mod reply;
pub use reply::Reply;

mod adapter;
pub use adapter::Adapter;

mod resource;
pub use resource::{Resource};

mod channel;
pub use channel::{Channel, Sender};

mod error;
pub use error::{Error, ErrorKind};

pub mod memory;

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
  }
}
