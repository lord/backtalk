use super::{JsonValue, Reply, Req, Resource};
use ws;
use tokio_core;
use futures::future::{ok, err};
use futures::{BoxFuture, Future};
use std::collections::HashMap;
use tokio_core::reactor::Core;
use std::thread;
use futures;

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
    match self.route_table.get(req.resource()) {
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

