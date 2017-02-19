use super::{Params, JsonValue, Reply};
use serde_json;

#[derive(Debug)]
pub enum Method {
  // indempotent methods (must be able to call many times and it'll have the same effect/return value as just once)
  List, // -> GET /resource
  Get, // -> GET /resource/123
  Delete, // -> DELETE /resource/123
  // not indempotent
  Post, // -> POST /resource
  Patch, // -> PATCH /resource/123
  Action(String), // -> POST /resource/123/actionname
}

impl Method {
  fn from_str(s: String) -> Method {
    match s.as_str() {
      "list" => Method::List,
      "get" => Method::Get,
      "delete" => Method::Delete,
      "post" => Method::Post,
      "patch" => Method::Patch,
      _ => Method::Action(s),
    }
  }
}

#[derive(Debug)]
pub struct Req {
  id: Option<String>,
  params: Params,
  data: JsonValue,
  resource: String,
  method: Method,
}

impl Req {
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
  pub fn from_websocket_string(s: String, route: &str) -> Result<Req, Reply> {
    fn err(err_str: &str) -> Result<Req, Reply> {
      Err(Reply { code: 400, data: JsonValue::Array(vec![JsonValue::String("error!".to_string()), JsonValue::String(err_str.to_string())]), req: None })
    }
    let raw_dat = serde_json::from_str(&s);
    let mut raw_iter = match raw_dat {
      Ok(JsonValue::Array(a)) => a.into_iter(),
      Ok(_) => return err("was not array error TODO"),
      _ => return err("could not parse input as json TODO"),
    };

    // [method, params, id, data]
    // id and data may be null, depending on the method
    let method = match raw_iter.next() {
      Some(JsonValue::String(s)) => s,
      Some(_) => return err("method must be a string"),
      None => return err("missing method in request"),
    };
    let params = match raw_iter.next() {
      Some(JsonValue::Object(o)) => o,
      Some(_) => return err("params must be an object"),
      None => return err("missing params in request"), // TODO convert null to empty object
    };
    let id = match raw_iter.next() {
      Some(JsonValue::String(s)) => Some(s),
      Some(JsonValue::Null) => None,
      Some(_) => return err("id must be a string or null"),
      None => return err("missing id in request"), // TODO allow numeric ids
    };
    let data = match raw_iter.next() {
      Some(o) => o,
      None => return err("missing data in request"),
    };

    let req = Req {
      resource: route.to_string(),
      method: Method::from_str(method),
      params: params,
      id: id,
      data: data,
    };

    Ok(req)
  }
}
