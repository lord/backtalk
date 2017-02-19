extern crate ws;
extern crate futures;
extern crate tokio_core;
extern crate serde_json;

use futures::{BoxFuture, Future};
use futures::future::{ok, err};
use serde_json::Value as JsonValue;
use serde_json::value::Map;

type Params = Map<String, JsonValue>;

mod req;
pub use req::{Req, Method};

mod server;
pub use server::Server;

#[derive(Debug)]
pub struct Reply {
  data: JsonValue,
  code: i64, // TODO replace with enum of errors, etc
  req: Option<Req>,
}

impl Reply {
  // TODO refine this? currently only really should be used internally.
  pub fn new(code: i64, req: Option<Req>, data: JsonValue) -> Reply {
    Reply {
      code: code,
      req: req,
      data: data,
    }
  }
}

// TODO could a client continue the connection even after the 404? make sure not

// TODO maybe allow adapters to have data be any serializable object? So the trait would have a generic

trait Adapter: Send {
  fn find(&self, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
  fn get(&self, id: &str, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
  fn post(&self, data: &JsonValue, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
  fn patch(&self, id: &str, data: &JsonValue, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
  fn delete(&self, id: &str, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
}

pub struct Resource {
  adapter: Box<Adapter>,
}

impl Resource {
  fn new<T: Adapter + 'static + Send>(adapt: T) -> Resource {
    Resource {
      adapter: Box::new(adapt),
    }
  }

  fn handle(&self, req: Req) -> BoxFuture<Reply, Reply> {
    fn make_err(err_str: &str) -> BoxFuture<Reply, Reply> {
      err(Reply { code: 400, data: JsonValue::Array(vec![JsonValue::String("error!".to_string()), JsonValue::String(err_str.to_string())]), req: None }).boxed()
    }
    let res = match (req.method(), req.id()) {
      (&Method::List, _) => self.adapter.find(req.params()),
      (&Method::Post, _) => self.adapter.post(req.data(), req.params()),
      (&Method::Get, &Some(ref id)) => self.adapter.get(id, req.params()),
      (&Method::Delete, &Some(ref id)) => self.adapter.delete(id, req.params()),
      (&Method::Patch, &Some(ref id)) => self.adapter.patch(id, req.data(), req.params()),
      (&Method::Action(_), _) => unimplemented!(),
      (_, &None) => return make_err("missing id in request"),
    };

    res.then(|res| match res {
      Ok(val) => Ok(req.into_reply(200, val)),
      Err((code, val)) => Err(req.into_reply(code, val)),
    }).boxed()
  }
}

struct MemoryAdapter {}

impl Adapter for MemoryAdapter {
  fn find(&self, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    unimplemented!()
  }

  fn get(&self, id: &str, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    ok(JsonValue::Array(vec![JsonValue::String("foo".to_string())])).boxed()
  }

  fn post(&self, data: &JsonValue, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    unimplemented!()
  }

  fn patch(&self, id: &str, data: &JsonValue, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    unimplemented!()
  }

  fn delete(&self, id: &str, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    unimplemented!()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let mut s = Server::new();
    let mut r = Resource::new(MemoryAdapter{});
    s.mount("/hello", r);
    s.listen("127.0.0.1");
  }
}
