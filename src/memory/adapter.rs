use futures::future::{Future, BoxFuture, ok};
use {JsonValue, ErrorKind, Adapter, Params};

#[derive(Clone)]
pub struct MemoryAdapter {}

impl Adapter for MemoryAdapter {
  fn find(&self, _params: &Params) -> BoxFuture<JsonValue, (ErrorKind, JsonValue)> {
    ok(JsonValue::Array(vec![JsonValue::String("foo".to_string())])).boxed()
  }

  fn get(&self, _id: &str, _params: &Params) -> BoxFuture<JsonValue, (ErrorKind, JsonValue)> {
    ok(JsonValue::Array(vec![JsonValue::String("foo".to_string())])).boxed()
  }

  fn post(&self, _data: &JsonValue, _params: &Params) -> BoxFuture<JsonValue, (ErrorKind, JsonValue)> {
    ok(JsonValue::Array(vec![JsonValue::String("foo".to_string())])).boxed()
  }

  fn patch(&self, _id: &str, _data: &JsonValue, _params: &Params) -> BoxFuture<JsonValue, (ErrorKind, JsonValue)> {
    ok(JsonValue::Array(vec![JsonValue::String("foo".to_string())])).boxed()
  }

  fn delete(&self, _id: &str, _params: &Params) -> BoxFuture<JsonValue, (ErrorKind, JsonValue)> {
    ok(JsonValue::String("foo".to_string())).boxed()
  }
}
