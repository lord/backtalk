extern crate ws;
extern crate futures;
extern crate tokio_core;
extern crate serde_json;

use futures::{BoxFuture, Future};
use futures::future::IntoFuture;
use futures::future::{ok, err, FutureResult};
use tokio_core::reactor::Core;
use std::thread;
use serde_json::Value as JsonValue;
use serde_json::value::Map;
use std::collections::HashMap;

type Params = Map<String, JsonValue>;

#[derive(Debug)]
pub struct Req {
  id: Option<String>,
  params: Params,
  data: JsonValue,
  resource: String,
  method: Method,
}

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

impl Req {
  fn into_reply(self, code: i64, reply: JsonValue) -> Reply {
    Reply {
      code: code,
      data: reply,
      req: Some(self),
    }
  }

  fn from_websocket_string(s: String, route: &str) -> Result<Req, Reply> {
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

#[derive(Debug)]
pub struct Reply {
  data: JsonValue,
  code: i64, // TODO replace with enum of errors, etc
  req: Option<Req>,
}

struct WebSocketHandler<'a> {
    sender: ws::Sender,
    route: Option<String>, // TODO better routing method than strings, like maybe a route index or something
    server: &'a Server,
    eloop: tokio_core::reactor::Remote,
}

impl <'a> ws::Handler for WebSocketHandler<'a> {
  fn on_request(&mut self, req: &ws::Request) -> ws::Result<ws::Response> {
    let mut resp = ws::Response::from_request(req)?;
    if self.server.has_resource(req.resource()) {
      self.route = Some(req.resource().to_string());
    } else {
      resp.set_status(404);
    }
    Ok(resp)
  }

  fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
    let route_str = match self.route {
      Some(ref r) => r,
      None => return Err(ws::Error::new(ws::ErrorKind::Internal, "route was unspecified")),
    };
    let out = self.sender.clone();
    let req = match Req::from_websocket_string(msg.to_string(), route_str) {
      Ok(req) => req,
      Err(e) => {
        out.send(ws::Message::text(e.data.to_string()));
        return Ok(())
      }
    };
    let prom = self.server.handle(req).then(move |resp| {
      match resp {
        Ok(s) => out.send(ws::Message::text(s.data.to_string())),
        Err(s) => out.send(ws::Message::text(s.data.to_string())),
      };
      ok(())
    });
    self.eloop.spawn(|_| prom);
    Ok(())
  }
}

pub struct Server {
  route_table: HashMap<String, Resource>
}

impl Server {
  pub fn new() -> Server {
    Server{
      route_table: HashMap::new()
    }
  }

  fn has_resource(&self, s: &str) -> bool {
    self.route_table.get(s).is_some()
  }

  pub fn handle(&self, req: Req) -> BoxFuture<Reply, Reply> {
    // TODO maybe instead do some sort of indexing instead of all this string hashing, so like, the webhooks calls get_route_ref or something
    match self.route_table.get(&req.resource) {
      Some(resource) => resource.handle(req),
      None => err(req.into_reply(404, JsonValue::String("TODO not found error here".to_string()))).boxed()
    }
  }

  pub fn mount<T: Into<String>>(&mut self, route: T, resource: Resource) {
    self.route_table.insert(route.into(), resource);
  }

  pub fn listen<T: Into<String> + Send + 'static>(self, bind_addr: T) {
    let mut eloop = Core::new().unwrap();
    let addr: String = bind_addr.into();
    let eloop_remote = eloop.remote();
    thread::spawn(move || {
      let server = &self;
      ws::listen((addr + ":3333").as_str(), |out| {
        return WebSocketHandler {
          sender: out.clone(),
          route: None,
          eloop: eloop_remote.clone(),
          server: server,
        };
      })
    });
    eloop.run(futures::future::empty::<(), ()>());
  }
}

// TODO could a client continue the connection even after the 404? make sure not

// TODO maybe allow adapters to have data be any serializable object? So the trait would have a generic

trait Adapter: Send {
  fn find(&self, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
  fn get(&self, id: &str, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
  fn post(&self, data: &JsonValue, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
  fn patch(&self, id: &str, data: &JsonValue, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
  fn delete(&self, id: &str, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)>;
}

pub struct Resource {
  adapter: Box<Adapter>,
}

impl Resource {
  fn new<T: Adapter + 'static + Send>(adapt: T) -> Resource {
    Resource {
      adapter: Box::new(adapt),
    }
  }

  fn handle(&self, req: Req) -> BoxFuture<Reply, Reply> {
    fn make_err(err_str: &str) -> BoxFuture<Reply, Reply> {
      err(Reply { code: 400, data: JsonValue::Array(vec![JsonValue::String("error!".to_string()), JsonValue::String(err_str.to_string())]), req: None }).boxed()
    }
    let res = match (&req.method, &req.id) {
      (&Method::List, _) => self.adapter.find(&req.params),
      (&Method::Post, _) => self.adapter.post(&req.data, &req.params),
      (&Method::Get, &Some(ref id)) => self.adapter.get(id, &req.params),
      (&Method::Delete, &Some(ref id)) => self.adapter.delete(id, &req.params),
      (&Method::Patch, &Some(ref id)) => self.adapter.patch(id, &req.data, &req.params),
      (&Method::Action(_), _) => unimplemented!(),
      (_, &None) => return make_err("missing id in request"),
    };

    res.then(|res| match res {
      Ok(val) => Ok(req.into_reply(200, val)),
      Err((code, val)) => Err(req.into_reply(code, val)),
    }).boxed()
  }
}

struct MemoryAdapter {}

impl Adapter for MemoryAdapter {
  fn find(&self, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    unimplemented!()
  }

  fn get(&self, id: &str, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    ok(JsonValue::Array(vec![JsonValue::String("foo".to_string())])).boxed()
  }

  fn post(&self, data: &JsonValue, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    unimplemented!()
  }

  fn patch(&self, id: &str, data: &JsonValue, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    unimplemented!()
  }

  fn delete(&self, id: &str, params: &Params) -> BoxFuture<JsonValue, (i64, JsonValue)> {
    // TODO
    unimplemented!()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let mut s = Server::new();
    let mut r = Resource::new(MemoryAdapter{});
    s.mount("/hello", r);
    s.listen("127.0.0.1");
  }
}
