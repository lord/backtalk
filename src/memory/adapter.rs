use futures::future::{Future, BoxFuture, ok, err};
use {JsonValue, ErrorKind, Adapter, JsonObject};
use std::collections::HashMap;
use std::sync::Mutex;

fn std_error(kind: ErrorKind, err_str: &str) -> (ErrorKind, JsonValue) {
  let val = json!({
    "error": {
      "type": kind.as_string(),
      "message": err_str.to_string(),
    }
  });
  (kind, val)
}

pub struct MemoryAdapter {
  datastore: Mutex<HashMap<String, JsonValue>>,
}

impl MemoryAdapter {
  pub fn new() -> MemoryAdapter {
    MemoryAdapter {
      datastore: Mutex::new(HashMap::new()),
    }
  }
}

impl Adapter for MemoryAdapter {
  fn find(&self, _params: &JsonObject) -> BoxFuture<JsonValue, (ErrorKind, JsonValue)> {
    ok(JsonValue::Array(vec![JsonValue::String("foo".to_string())])).boxed()
  }

  fn get(&self, id: &str, _params: &JsonObject) -> BoxFuture<JsonValue, (ErrorKind, JsonValue)> {
    let datastore = self.datastore.lock().unwrap();
    match datastore.get(id) {
      Some(val) => ok(val.clone()).boxed(),
      None => err(std_error(ErrorKind::NotFound, "couldn't find object with that id")).boxed(),
    }
  }

  fn post(&self, data: &JsonValue, _params: &JsonObject) -> BoxFuture<JsonValue, (ErrorKind, JsonValue)> {
    // let datastore = self.datastore.lock().unwrap();
    // let id = Uuid::new_v4().to_string();
    // let map = match data.clone() {
    //   JsonValue::Map(map) => map,
    //   _ => err(std_error(ErrorKind::NotFound, "values must be maps")).boxed()
    // };
    // match datastore.insert(id) {
    //   Some(val) => ok(val.clone()).boxed(),
    //   None => err(std_error(ErrorKind::NotFound, "couldn't find object with that id")).boxed(),
    // }
    ok(JsonValue::Array(vec![JsonValue::String("foo".to_string())])).boxed()
  }

  fn patch(&self, _id: &str, _data: &JsonValue, _params: &JsonObject) -> BoxFuture<JsonValue, (ErrorKind, JsonValue)> {
    ok(JsonValue::Array(vec![JsonValue::String("foo".to_string())])).boxed()
  }

  fn delete(&self, _id: &str, _params: &JsonObject) -> BoxFuture<JsonValue, (ErrorKind, JsonValue)> {
    ok(JsonValue::String("foo".to_string())).boxed()
  }
}
