extern crate backtalk;
use backtalk::*;
extern crate futures;
use futures::future::Future;
#[macro_use]
extern crate serde_json;

fn main() {
  let mut server = Server::new();
  use std::sync::Arc;
  let database = Arc::new(memory::MemoryAdapter::new());
  server.resource("/cats", move |req: Request| {
    let database1 = database.clone();
    req
      .and_then(move |req| {
        if req.method() == Method::Delete {
          if let &JsonValue::String(ref password) = req.param("password") {
            if password != "meow" {
              return Error::forbidden("incorrect password");
            }
          } else {
            return Error::unauthorized("please provide a password");
          }
        }
        req.boxed()
      })
      .and_then(move |req| {
        database1.handle(req)
      })
  });
  server.listen("127.0.0.1:3000");
}