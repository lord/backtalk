use super::{Params, Request, Reply, Method};
use futures::{BoxFuture, Future};
use futures::future::ok;
use serde_json::Value as JsonValue;

// TODO maybe allow adapters to have data be any serializable object? So the trait would have a generic
// TODO have a better way of doing this than just clone, but a system that still will
pub trait Adapter: Send + Sync {
  fn find(&self, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
  fn get(&self, id: &str, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
  fn post(&self, data: &JsonValue, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
  fn patch(&self, id: &str, data: &JsonValue, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
  fn delete(&self, id: &str, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;

  fn handle(&self, req: Request) -> BoxFuture<Reply, Reply> {
    let res = match (req.method().clone(), req.id().clone()) {
      (Method::List, _) => self.find(req.params()),
      (Method::Post, _) => self.post(req.data(), req.params()),
      (Method::Get, Some(ref id)) => self.get(id, req.params()),
      (Method::Delete, Some(ref id)) => self.delete(id, req.params()),
      (Method::Patch, Some(ref id)) => self.patch(id, req.data(), req.params()),
      _ => unimplemented!(),
    };
    res.then(move |res| match res {
      Ok(val) => Ok(req.into_reply(200, val)),
      Err((code, val)) => Err(req.into_reply(code, val)),
    }).boxed()
  }
}

#[derive(Clone)]
pub struct MemoryAdapter {}

impl Adapter for MemoryAdapter {
  fn find(&self, _params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    ok(JsonValue::Array(vec![JsonValue::String("foo".to_string())])).boxed()
  }

  fn get(&self, _id: &str, _params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    ok(JsonValue::Array(vec![JsonValue::String("foo".to_string())])).boxed()
  }

  fn post(&self, _data: &JsonValue, _params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    ok(JsonValue::Array(vec![JsonValue::String("foo".to_string())])).boxed()
  }

  fn patch(&self, _id: &str, _data: &JsonValue, _params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    ok(JsonValue::Array(vec![JsonValue::String("foo".to_string())])).boxed()
  }

  fn delete(&self, _id: &str, _params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    ok(JsonValue::String("foo".to_string())).boxed()
  }
}
