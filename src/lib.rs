extern crate ws;
extern crate futures;
extern crate tokio_core;

use futures::{BoxFuture, Future};
use futures::future::IntoFuture;
use futures::future::{ok, err, FutureResult};
use tokio_core::reactor::Core;
use std::thread;

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

// PLAN:
// methods:
//   - indempotent
//     - list -> GET /resource
//     - get -> GET /resource/123
//     - delete -> DELETE /resource/123 (NOTE! MUST BE INDEPOTENT, that is, you can call it many times and it'll have the same effect as just once)
//   - not indempotent
//     - create -> POST /resource
//     - patch -> PATCH /resource/123
//     - (custom) -> POST /resource/123/actionname (a la stripe)

#[derive(Debug)]
pub struct Req {
  pub data: String,
}

#[derive(Debug)]
pub struct Reply {
  pub data: String,
  req: Req,
}

impl Req {
  fn into_reply(self, reply: String) -> Reply {
    Reply {
      data: reply,
      req: self,
    }
  }
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
    let msg = format!("{}:{}", route_str, msg);
    let out = self.sender.clone();
    match self.server.route_table {
      Some(ref f) => {
        let req = Req {
          data: msg,
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
      let reply_str = format!("backtalk echo: {}", &req.data);
      ok(req.into_reply(reply_str)).boxed()
    });
    s.listen("127.0.0.1");
  }
}
