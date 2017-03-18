extern crate backtalk;
extern crate futures;
#[macro_use]
extern crate serde_json;

use backtalk::*;
use std::sync::Arc;
use futures::future::Future;

fn main() {
  let mut s = Server::new();
  s.resource("/meow", |req: Request| {
    Error::forbidden("not allowed! sorry.")
  });
  let adapter = Arc::new(memory::MemoryChannel::new());
  let channel = Arc::new(memory::MemoryChannel::new());
  s.resource("/hello2", move |req: Request| {
    req
      .and_then(|req| {
        req.into_reply(json!({"meow": "foobar"}))
      })
  });
  s.resource("/hello", move |req: Request| {
    let adapter = adapter.clone();
    let channel1 = channel.clone();
    let channel2 = channel.clone();
    req
      .and_then(move |req| match req.method().clone() {
        Method::Action(ref action_name) => Error::forbidden("not allowed! sorry."),
        Method::Listen => channel1.handle(req),
        _ => adapter.handle(req),
      })
      .and_then(move |reply| {
        if let Some(dat) = reply.data() {
          channel2.send("test kind", dat);
        }
        reply
      })
  });
  s.listen("127.0.0.1:3000");
}
