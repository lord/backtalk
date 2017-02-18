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
  fn into_reply(self, reply: String) -> Reply {
    Reply {
      data: reply,
      // req: self,
    }
  }

  fn from_websocket_string(s: String, route: &str) -> Result<Req, Reply> {
    let raw_dat = serde_json::from_str(&s);
    let raw_arr = match raw_dat {
      Ok(JsonValue::Array(a)) => a,
      Ok(_) => return Err(Reply { data: "was not array error TODO".to_string() }),
      _ => return Err(Reply { data: "could not parse input as json TODO".to_string() }),
    };
    // [method, params, id, data]
    // id and data may be null, depending on the method
    if raw_arr.len() != 4 {
      return Err(Reply { data: "wrong number of args".to_string() });
    }

    let mut raw_iter = raw_arr.into_iter();
    let method = match raw_iter.next().unwrap() {
      JsonValue::String(s) => s,
      _ => return Err(Reply { data: "method must be a string".to_string() }),
    };
    let params = match raw_iter.next().unwrap() {
      JsonValue::Object(o) => o,
      _ => return Err(Reply { data: "params must be an object".to_string() }) // TODO convert null to empty object
    };
    let id = match raw_iter.next().unwrap() {
      JsonValue::String(s) => Some(s),
      JsonValue::Null => None,
      _ => return Err(Reply { data: "id must be a string or null".to_string() }), // TODO allow numeric ids
    };
    let data = raw_iter.next().unwrap();

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
  pub data: String,
  // req: Req,
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
      None => return Err(ws::Error::new(ws::ErrorKind::Internal, "")),
    };
    let out = self.sender.clone();
    match self.server.route_table {
      Some(ref f) => {
        let req = match Req::from_websocket_string(msg.to_string(), route_str) {
          Ok(req) => req,
          Err(e) => {
            out.send(ws::Message::text(e.data));
            return Ok(())
          }
        };
        let prom = f(req).then(move |resp| {
          println!("resp: {:?}", &resp);
          match resp {
            Ok(s) => out.send(ws::Message::text(s.data)),
            Err(s) => out.send(ws::Message::text(s.data)),
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
  #[test]
  fn it_works() {
    let mut s = Server::new();
    s.route(|req| {
      let reply_str = format!("backtalk echo: {:?}", &req);
      ok(req.into_reply(reply_str)).boxed()
    });
    s.listen("127.0.0.1");
  }
}
