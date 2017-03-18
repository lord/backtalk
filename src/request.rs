use super::{Params, JsonValue, Reply, Error};
use reply::make_reply;
use futures::future::{IntoFuture, ok, FutureResult, AndThen, BoxFuture, Future};

#[derive(Debug, Clone)]
pub enum Method {
  // indempotent methods (must be able to call many times and it'll have the same effect/return value as just once)
  List, // -> GET /resource
  Get, // -> GET /resource/123
  Delete, // -> DELETE /resource/123
  // not indempotent
  Post, // -> POST /resource
  Patch, // -> PATCH /resource/123
  Listen, // -> GET /resource or (maybe?) GET /resource/123 with content-type text/event-stream
  Action(String), // -> POST /resource/123/actionname
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

#[derive(Debug)]
pub struct Request {
  id: Option<String>,
  params: Params,
  data: JsonValue,
  resource: String,
  method: Method,
}

impl Request {
  pub fn new(resource: String, method: Method, id: Option<String>, data: JsonValue, params: Params) -> Request {
    Request {
      resource: resource,
      method: method,
      id: id,
      data: data,
      params: params
      
    }
  }

  pub fn into_reply(self, reply: JsonValue) -> Reply {
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

  pub fn params(&self) -> &Params {
    &self.params
  }

  pub fn params_mut(&mut self) -> &mut Params {
    &mut self.params
  }

  pub fn param(&self, key: &str) -> Option<&JsonValue> {
    self.params.get(key)
  }

  pub fn set_param(&mut self, key: String, val: JsonValue) {
    self.params.insert(key, val);
  }

  pub fn data(&self) -> &JsonValue {
    &self.data
  }

  pub fn data_mut(&mut self) -> &mut JsonValue {
    &mut self.data
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

