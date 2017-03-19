use {JsonObject, Request, Reply, Method, ErrorKind, Error};
use futures::{BoxFuture, Future};
use serde_json::Value as JsonValue;

pub trait Adapter: Send + Sync {
  fn find(&self, params: &JsonObject) -> BoxFuture<JsonObject, (ErrorKind, JsonValue)>;
  fn get(&self, id: &str, params: &JsonObject) -> BoxFuture<JsonObject, (ErrorKind, JsonValue)>;
  fn post(&self, data: &JsonObject, params: &JsonObject) -> BoxFuture<JsonObject, (ErrorKind, JsonValue)>;
  fn patch(&self, id: &str, data: &JsonObject, params: &JsonObject) -> BoxFuture<JsonObject, (ErrorKind, JsonValue)>;
  fn delete(&self, id: &str, params: &JsonObject) -> BoxFuture<JsonObject, (ErrorKind, JsonValue)>;

  fn handle(&self, req: Request) -> BoxFuture<Reply, Error> {
    let res = match (req.method().clone(), req.id().clone()) {
      (Method::List, _) => self.find(req.params()),
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
