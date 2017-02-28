use super::{Params};
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
