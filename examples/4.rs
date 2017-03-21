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
    database.handle(req).and_then(|mut reply| {
      {
          let mut data = reply.data_mut().unwrap();
          data.insert("example".to_string(), json!("data"));
      }
      reply
    })
  });
  server.listen("127.0.0.1:3000");
}