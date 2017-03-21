extern crate backtalk;
use backtalk::*;
extern crate futures;
use futures::Future;
#[macro_use]
extern crate serde_json;

fn main() {
  let mut server = Server::new();
  let database = memory::MemoryAdapter::new();
  server.resource("/cats", move |req: Request| {
    database.handle(req)
  });
  server.listen("127.0.0.1:3000");
}