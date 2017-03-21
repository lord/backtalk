use super::{JsonObject, JsonValue, Reply, Error};
use reply::make_reply;
use futures::future::{IntoFuture, ok, FutureResult, AndThen, Future, BoxFuture};

/**
A type of request, for instance "List" or "Post".

These mostly correspond to the HTTP methods, with the addition of `List`, `Listen`, and `Action`.

- `List` is a `GET` request with an ID on a resource, such as `GET /cats`.
- `Listen` is a `GET`
request with a `Accept: text/event-stream` header. `Listen` requests may or may not have IDs, so
both `GET /cats` and `GET /cats/123` with the `event-stream` header would be a `Listen` request.
- `Action` is a custom action on a specific resource ID. For instance, `POST /cats/123/feed` would
be `Action("feed")`.

Note that we don't support `PUT` requests currently, for simplicity.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
  /// `GET /resource`, indempotent
  List,
  /// `GET /resource/123`, indempotent
  Get,
  /// `DELETE /resource/123`, indempotent
  Delete,
  /// `POST /resource`
  Post,
  /// `PATCH /resource/123`
  Patch,
  /// Either `GET /resource/` or `GET /resource/123`, with the `Accept: text/event-stream` header
  Listen,
  /// `POST /resource/123/actionname`
  Action(String),
}

impl Method {
  pub fn as_string(&self) -> String {
    match self {
      &Method::List => "list",
      &Method::Get => "get",
      &Method::Delete => "delete",
      &Method::Post => "post",
      &Method::Patch => "patch",
      &Method::Listen => "listen",
      &Method::Action(ref action) => action,
    }.to_string()
  }
}

/**
A request containing data from the client.
*/
#[derive(Debug)]
pub struct Request {
  id: Option<String>,
  params: JsonObject,
  data: JsonObject,
  resource: String,
  method: Method,
  null: JsonValue,
}

impl Request {
  pub fn new(resource: String, method: Method, id: Option<String>, data: JsonObject, params: JsonObject) -> Request {
    Request {
      resource: resource,
      method: method,
      id: id,
      data: data,
      params: params,
      null: JsonValue::Null,
    }
  }

  pub fn into_reply(self, reply: JsonObject) -> Reply {
    make_reply(self, reply)
  }

  // TODO data_then accepts a function that returns a future<JsonValue, Error>

  pub fn method(&self) -> Method {
    self.method.clone()
  }

  pub fn resource(&self) -> &str {
    &self.resource
  }

  pub fn id(&self) -> &Option<String> {
    &self.id
  }

  pub fn params(&self) -> &JsonObject {
    &self.params
  }

  pub fn params_mut(&mut self) -> &mut JsonObject {
    &mut self.params
  }

  pub fn param(&self, key: &str) -> &JsonValue {
    self.params.get(key).unwrap_or(&self.null)
  }

  pub fn set_param(&mut self, key: String, val: JsonValue) {
    self.params.insert(key, val);
  }

  pub fn data(&self) -> &JsonObject {
    &self.data
  }

  pub fn data_mut(&mut self) -> &mut JsonObject {
    &mut self.data
  }

  pub fn boxed(self) -> BoxFuture<Request, Error> {
    ok(self).boxed()
  }

  pub fn and_then<F, B>(self, f: F) -> AndThen<FutureResult<Request, Error>, B, F>
    where F: FnOnce(Request) -> B,
          B: IntoFuture<Error=Error>
  {
    ok::<Request, Error>(self).and_then(f)
  }
}

impl IntoFuture for Request {
  type Item = Request;
  type Error = Error;
  type Future = FutureResult<Request, Error>;
  fn into_future(self) -> Self::Future {
    ok(self)
  }
}

