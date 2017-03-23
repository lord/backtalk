extern crate backtalk;
use backtalk::*;
extern crate futures;
use futures::Future;
#[macro_use]
extern crate serde_json;

fn main() {
  let mut server = Server::new();
  use std::sync::Arc;
  use std::ops::Deref;
  let database = Arc::new(memory::MemoryAdapter::new());
  let chan = Arc::new(memory::MemoryChannel::new());
  server.resource("/cats", move |req: Request| {
    let database1 = database.clone();
    let chan1 = chan.clone();
    let chan2 = chan.clone();
    req
      .and_then(move |req| {
        match req.method() {
          Method::Listen => chan1.handle(req),
          _ => database1.handle(req),
        }
      })
      .and_then(move |reply| {
        util::send_from_reply(reply, chan2.deref())
      })
  });
  server.listen("127.0.0.1:3000");
}