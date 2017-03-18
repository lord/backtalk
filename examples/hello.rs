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
  let adapter = memory::MemoryChannel::new();
  let channel = Arc::new(memory::MemoryChannel::new());
  s.resource("/hello2", move |req: Request| {
    req
      .and_then(|req| {
        req.into_reply(json!({"meow": "foobar"}))
      }).boxed()
  });
  s.resource("/hello", move |req: Request| {
    let res = match req.method().clone() {
      // Method::Action(ref action_name) => {
      //   unimplemented!();
      // },
      Method::Listen => channel.handle(req),
      _ => adapter.handle(req),
    };
    let channel2 = channel.clone();
    res.map(move |reply| {
      if let Some(dat) = reply.data() {
        channel2.send("test kind", dat);
      }
      reply
    }).boxed()
  });
  s.listen("127.0.0.1:3000");
}
