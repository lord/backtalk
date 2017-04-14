/*!
Backtalk is an experimental mini-library for writing asynchronous, realtime JSON web APIs served
over HTTP. If you're getting started, it's probably easiest to dive into the examples in the
`examples` folder, which should be well-documented with comments.

Backtalk is still being refined, but (especially once we build a database adapter for a proper
database) it should be useful enough at this point to build real things with it! If you have
feedback, comments, suggestions, pull requests, bug reports, they'd be much appreciated, since I
don't really know what I'm doing tbqh.
*/

extern crate futures;
extern crate tokio_core;
#[macro_use]
extern crate serde_json;
extern crate hyper;
extern crate queryst;
extern crate uuid;

pub use serde_json::Value as JsonValue;
pub type JsonObject = serde_json::value::Map<String, JsonValue>;

mod request;
pub use request::{Request, Method};

mod server;
pub use server::Server;

mod reply;
pub use reply::Reply;

mod adapter;
pub use adapter::Adapter;

mod handler;
pub use handler::{Handler};

mod channel;
pub use channel::{Channel, Sender};

mod error;
pub use error::{Error, ErrorKind};

pub mod memory;
pub mod util;
