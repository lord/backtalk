use {JsonObject, Request, Reply, Method, ErrorKind, Error};
use futures::{BoxFuture, Future};
use serde_json::Value as JsonValue;

pub trait Adapter: Send + Sync {
  fn list(&self, params: &JsonObject) -> BoxFuture<JsonObject, (ErrorKind, JsonValue)>;
  fn get(&self, id: &str, params: &JsonObject) -> BoxFuture<JsonObject, (ErrorKind, JsonValue)>;
  fn post(&self, data: &JsonObject, params: &JsonObject) -> BoxFuture<JsonObject, (ErrorKind, JsonValue)>;
  fn patch(&self, id: &str, data: &JsonObject, params: &JsonObject) -> BoxFuture<JsonObject, (ErrorKind, JsonValue)>;
  fn delete(&self, id: &str, params: &JsonObject) -> BoxFuture<JsonObject, (ErrorKind, JsonValue)>;

  fn handle(&self, req: Request) -> BoxFuture<Reply, Error> {
    let res = match (req.method().clone(), req.id().clone()) {
      (Method::List, _) => self.list(req.params()),
      (Method::Post, _) => self.post(req.data(), req.params()),
      (Method::Get, Some(ref id)) => self.get(id, req.params()),
      (Method::Delete, Some(ref id)) => self.delete(id, req.params()),
      (Method::Patch, Some(ref id)) => self.patch(id, req.data(), req.params()),
      (_, None) => return Error::bad_request("missing id in request"),
      (Method::Listen, _) => return Error::server_error("passed listen request to database adapter"),
      (Method::Action(_), _) => return Error::server_error("passed action request to database adapter"),
    };
    res.then(move |res| match res {
      Ok(val) => Ok(req.into_reply(val)),
      Err((kind, val)) => Err(Error::new(kind, val)),
    }).boxed()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use futures::future::{ok, err};
  struct TestAdapter;

  impl Adapter for TestAdapter {
    fn list(&self, params: &JsonObject) -> BoxFuture<JsonObject, (ErrorKind, JsonValue)> {
      let mut obj = JsonObject::new();
      obj.insert("method".to_string(), JsonValue::String("find".to_string()));
      ok(obj).boxed()
    }
    fn get(&self, id: &str, params: &JsonObject) -> BoxFuture<JsonObject, (ErrorKind, JsonValue)> {
      let mut obj = JsonObject::new();
      obj.insert("method".to_string(), JsonValue::String("get".to_string()));
      ok(obj).boxed()
    }
    fn post(&self, data: &JsonObject, params: &JsonObject) -> BoxFuture<JsonObject, (ErrorKind, JsonValue)> {
      err((ErrorKind::ServerError, json!({"error": "testerror"}))).boxed()
    }
    fn patch(&self, id: &str, data: &JsonObject, params: &JsonObject) -> BoxFuture<JsonObject, (ErrorKind, JsonValue)> {
      let mut obj = JsonObject::new();
      obj.insert("method".to_string(), JsonValue::String("patch".to_string()));
      ok(obj).boxed()
    }
    fn delete(&self, id: &str, params: &JsonObject) -> BoxFuture<JsonObject, (ErrorKind, JsonValue)> {
      let mut obj = JsonObject::new();
      obj.insert("method".to_string(), JsonValue::String("delete".to_string()));
      ok(obj).boxed()
    }
  }

  fn make_req(m: Method, id: Option<&str>) -> Request {
    Request::new("resource".to_string(), m, id.map(|s| s.to_string()), JsonObject::new(), JsonObject::new())
  }

  #[test]
  fn adapter_can_list() {
    let adapter = TestAdapter{};
    let res = adapter.handle(make_req(Method::List, None)).wait().unwrap();
    assert!(res.data().unwrap().get("method").unwrap() == "find");
  }

  fn adapter_can_get() {
    let adapter = TestAdapter{};
    let res = adapter.handle(make_req(Method::Get, Some("12"))).wait().unwrap();
    assert!(res.data().unwrap().get("method").unwrap() == "get");
  }

  fn adapter_can_patch() {
    let adapter = TestAdapter{};
    let res = adapter.handle(make_req(Method::Patch, Some("12"))).wait().unwrap();
    assert!(res.data().unwrap().get("method").unwrap() == "patch");
  }

  fn adapter_can_delete() {
    let adapter = TestAdapter{};
    let res = adapter.handle(make_req(Method::Patch, Some("12"))).wait().unwrap();
    assert!(res.data().unwrap().get("method").unwrap() == "delete");
  }

  #[test]
  fn adapter_rejects_without_id() {
    let adapter = TestAdapter{};
    for method in vec![Method::Patch, Method::Delete, Method::Get] {
      let _res = adapter.handle(make_req(method, None)).wait().unwrap_err();
    }
  }

  #[test]
  fn adapter_can_show_errors() {
    let adapter = TestAdapter{};
    let _res = adapter.handle(make_req(Method::Post, None)).wait().unwrap_err();
  }
}
