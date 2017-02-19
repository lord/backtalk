use super::{Params};
use futures::{BoxFuture, Future};
use futures::future::ok;
use serde_json::Value as JsonValue;

// TODO maybe allow adapters to have data be any serializable object? So the trait would have a generic
pub trait Adapter: Send + Sync {
  fn find(&self, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
  fn get(&self, id: &str, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
  fn post(&self, data: &JsonValue, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
  fn patch(&self, id: &str, data: &JsonValue, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
  fn delete(&self, id: &str, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
}

pub struct MemoryAdapter {}

impl Adapter for MemoryAdapter {
  fn find(&self, _params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    unimplemented!()
  }

  fn get(&self, _id: &str, _params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    ok(JsonValue::Array(vec![JsonValue::String("foo".to_string())])).boxed()
  }

  fn post(&self, _data: &JsonValue, _params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    unimplemented!()
  }

  fn patch(&self, _id: &str, _data: &JsonValue, _params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    unimplemented!()
  }

  fn delete(&self, _id: &str, _params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    unimplemented!()
  }
}
