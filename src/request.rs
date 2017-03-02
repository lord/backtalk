use super::{Params, JsonValue, Reply};

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

  pub fn into_reply(self, code: i64, reply: JsonValue) -> Reply {
    Reply::new(code, Some(self), reply)
  }

  pub fn method(&self) -> &Method {
    &self.method
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

  pub fn data(&self) -> &JsonValue {
    &self.data
  }

  // TODO decide between params_mut and simple get and set methods

  // TODO probably should move this into lib and have a better way (that supports tests) of creating requests
  // maybe a separate function, in this module, that lib.rs doesn't export
}
