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

// defmodule HelloPhoenix.RoomChannel do
//   use Phoenix.Channel

//   def join("room:lobby", _message, socket) do
//     {:ok, socket}
//   end
//   def join("room:" <> _private_room_id, _params, _socket) do
//     {:error, %{reason: "unauthorized"}}
//   end

//   def handle_in("new_msg", %{"body" => body}, socket) do
//     broadcast! socket, "new_msg", %{body: body}
//     {:noreply, socket}
//   end

//   def handle_out("new_msg", payload, socket) do
//     push socket, "new_msg", payload
//     {:noreply, socket}
//   end
// end

// TODO: eventually should be https://docs.rs/futures/0.1/futures/future/trait.IntoFuture.html

// don't support PUT? https://tools.ietf.org/html/rfc7396 and http://williamdurand.fr/2014/02/14/please-do-not-patch-like-an-idiot/

// TODO be able to return a future of anything that can be IntoReply instead of just Reply?

// TODO I think macros can help with reducing usage of BoxFuture which is slower?
//      it would be cool if we used futures in a zero-cost way
//      also, it would be nice if we didn't have to write ok(fut).boxed() everywhere
//      see Rocket for inspiration

#[derive(Debug)]
pub struct Req {
  id: Option<String>,
  params: Map<String, JsonValue>,
  data: JsonValue,
  resource: String, // TODO should routes be strings?
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
  fn into_reply(self, code: i64, reply: JsonValue) -> BoxFuture<Reply, Reply> {
    ok(Reply {
      code: code,
      data: reply,
      req: Some(self),
    }).boxed()
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

    // TODO check that the right things are present

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
    match req.resource() {
      "/one" => self.route = Some("/one".to_string()),
      "/two" => self.route = Some("/two".to_string()),

      _ => {
        let mut resp = ws::Response::from_request(req)?;
        resp.set_status(404);
        return Ok(resp);
      },
    }
    ws::Response::from_request(req)
  }

  fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
    let route_str = match self.route {
      Some(ref r) => r,
      None => return Err(ws::Error::new(ws::ErrorKind::Internal, "route was unspecified")),
    };
    let out = self.sender.clone();
    match self.server.route_table {
      Some(ref f) => {
        let req = match Req::from_websocket_string(msg.to_string(), route_str) {
          Ok(req) => req,
          Err(e) => {
            out.send(ws::Message::text(e.data.to_string()));
            return Ok(())
          }
        };
        let prom = f(req).then(move |resp| {
          match resp {
            Ok(s) => out.send(ws::Message::text(s.data.to_string())),
            Err(s) => out.send(ws::Message::text(s.data.to_string())),
          };
          ok(())
        });
        self.eloop.spawn(|_| prom);
      },
      None => unimplemented!(),
    }
    Ok(())
  }
}

pub struct Server {
  route_table: Option<Box<Fn(Req) -> BoxFuture<Reply, Reply> + Send>>
}

impl Server {
  pub fn new() -> Server {
    let s = Server{route_table: None};
    s
  }

  pub fn route<T>(&mut self, r: T)
    where T: Fn(Req) -> BoxFuture<Reply, Reply> + 'static + Send {
    self.route_table = Some(Box::new(r));
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

#[cfg(test)]
mod tests {
  use super::*;
  use futures;

  #[test]
  fn it_works() {
    let mut s = Server::new();
    s.route(|req| {
      let reply_str = format!("backtalk echo: {:?}", &req);
      req.into_reply(200, JsonValue::Array(vec![JsonValue::String(reply_str)]))
    });
    s.listen("127.0.0.1");
  }
}
