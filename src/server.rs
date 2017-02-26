use super::{JsonValue, Reply, Req, Resource, Method};
use ws;
use tokio_core;
use futures::future::{ok, err};
use futures::{BoxFuture, Future};
use std::collections::HashMap;
use std::time::Duration;
use futures::Sink;
use tokio_core::reactor::Core;
use std::thread;
use futures;
use hyper;
use hyper::StatusCode;
use hyper::header::{ContentLength, ContentType, Accept};
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::server as http;
use hyper::Method as HttpMethod;
use futures::Stream;
use futures::future::FutureResult;
use std::sync::Arc;
use queryst::parse as query_parse;
use serde_json::Map;
use serde_json;

pub fn http_to_req(method: &HttpMethod, path: &str, query: &str, body: Option<Vec<u8>>, server: &Arc<Server>) -> Result<Req, Reply> {
  fn err(err_str: &str) -> Result<Req, Reply> {
    Err(Reply::new(400, None, JsonValue::Array(vec![JsonValue::String("error!".to_string()), JsonValue::String(err_str.to_string())])))
  }
  let body = if let Some(b) = body {
    b
  } else {
    return err("TODO error in request body");
  };
  let body_str = match String::from_utf8(body) {
    Ok(s) => s,
    _ => return err("TODO invalid unicode in request body"),
  };
  let body_obj = if body_str == "" {
    JsonValue::Null
  } else {
    match serde_json::from_str(&body_str) {
      Ok(o) => o,
      _ => return err("TODO invalid JSON in request body"),
    }
  };
  let query = match query_parse(query) {
    Ok(JsonValue::Null) => Map::new(),
    Ok(JsonValue::Object(u)) => u,
    _ => return err("failed to parse query string")
  };
  let mut parts: Vec<&str> = path.split("/").skip(1).collect();
  // remove trailing `/` part if present
  if let Some(&"") = parts.last() {
    parts.pop();
  }

  let resource_url = format!("/{}", parts.join("/"));
  if server.has_resource(&resource_url) {
    if method == &HttpMethod::Get {
      return Ok(Req::new(
        resource_url,
        Method::List,
        None,
        JsonValue::Null,
        query
      ))
    } else if method == &HttpMethod::Post {
      return Ok(Req::new(
        resource_url,
        Method::Post,
        None,
        body_obj,
        query
      ))
    } else {
      return err("TODO invalid http method")
    }
  }

  let (id, parts) = match parts.split_last() {
    Some(t) => t,
    None => return err("TODO 404 not found")
  };
  let resource_url = format!("/{}", parts.join("/"));
  if server.has_resource(&resource_url) {
    if method == &HttpMethod::Get {
      return Ok(Req::new(
        resource_url,
        Method::Get,
        Some(id.to_string()),
        JsonValue::Null,
        query
      ))
    } else if method == &HttpMethod::Patch {
      return Ok(Req::new(
        resource_url,
        Method::Patch,
        Some(id.to_string()),
        body_obj,
        query
      ))
    } else if method == &HttpMethod::Delete {
      return Ok(Req::new(
        resource_url,
        Method::Delete,
        Some(id.to_string()),
        JsonValue::Null,
        query
      ))
    } else {
      return err("TODO invalid http method")
    }
  }

  let action_name = id;
  let (id, parts) = match parts.split_last() {
    Some(t) => t,
    None => return err("TODO 404 not found")
  };
  let resource_url = format!("/{}", parts.join("/"));
  if server.has_resource(&resource_url) {
    if method == &HttpMethod::Post {
      return Ok(Req::new(
        resource_url,
        Method::Action(action_name.to_string()),
        Some(id.to_string()),
        JsonValue::Null,
        query
      ))
    } else {
      return err("TODO invalid http method")
    }
  }

  err("404 resource not found")
}

pub fn websocket_to_req(s: String, route: &str) -> Result<Req, Reply> {
  fn err(err_str: &str) -> Result<Req, Reply> {
    Err(Reply::new(400, None, JsonValue::Array(vec![JsonValue::String("error!".to_string()), JsonValue::String(err_str.to_string())])))
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

  Ok(Req::new(route.to_string(), Method::from_str(method), id, data, params))
}

// only one is created
#[derive(Clone)]
struct HttpService {
  server: Arc<Server>,
}

impl http::Service for HttpService {
  type Request = http::Request;
  type Response = http::Response;
  type Error = hyper::Error;
  type Future = BoxFuture<http::Response, hyper::Error>;

  fn call(&self, http_req: http::Request) -> Self::Future {
    let (method, uri, _, headers, body) = http_req.deconstruct();

    let default_accept = Accept::star();
    let accepts = headers.get::<Accept>().unwrap_or(&default_accept).as_slice().iter();
    // TODO better and actually spec compliant Accept header matching
    // should throw error if can't return either eventsource or json
    let (_, is_eventsource) = accepts.fold((0, false), |prev, ref quality_item| {
      let (mut best_qual, mut is_eventsource) = prev;
      let this_quality = quality_item.quality.0;
      if this_quality > best_qual {
        best_qual = this_quality;
        let Mime(ref top_level, ref sub_level, _) = quality_item.item;
        is_eventsource = top_level == &TopLevel::Text && sub_level == &SubLevel::EventStream;
      }
      (best_qual, is_eventsource)
    });

    let server = self.server.clone();
    let body_prom = body.fold(Vec::new(), |mut a, b| -> FutureResult<Vec<u8>, hyper::Error> { a.extend_from_slice(&b[..]); ok(a) });

    if is_eventsource {
      let (chunk_sender, body) = hyper::Body::pair();
      thread::spawn(|| {
        let mut chunk_sender = chunk_sender;
        let mut n = 0;
        loop {
          n+=1;
          if n == 10 {
            return
          }
          chunk_sender = match chunk_sender.send(Ok(hyper::Chunk::from("data: meow\n\n"))).wait() {
            Ok(sender) => sender,
            Err(_) => return
          };
          thread::sleep(Duration::from_millis(500));
        }
      });
      let resp = http::Response::new()
        .with_header(ContentType(Mime(TopLevel::Text, SubLevel::EventStream, vec![(hyper::mime::Attr::Charset, hyper::mime::Value::Utf8)])))
        .with_body(body);
      return ok(resp).boxed();
    }


    body_prom.then(move |body_res| {
      match http_to_req(&method, uri.path(), uri.query().unwrap_or(""), body_res.ok(), &server) {
        Ok(req) => server.handle(req),
        Err(reply) => err(reply).boxed(),
      }
    }).then(|resp| -> BoxFuture<hyper::server::Response, hyper::Error> {
      let http_resp = match resp {
        Ok(s) => {
          let resp_str = s.to_string();
          http::Response::new()
            .with_header(ContentLength(resp_str.len() as u64))
            .with_body(resp_str)
        }
        Err(s) => {
          let resp_str = s.to_string();
          http::Response::new()
            .with_status(StatusCode::NotFound) // TODO MAKE THIS A PROPER STATUS CODE
            .with_header(ContentLength(resp_str.len() as u64))
            .with_body(resp_str)
        },
      };
      ok(http_resp).boxed()
    }).boxed()
  }
}

// one is created for each incoming connection
struct WebSocketHandler {
  sender: ws::Sender,
  route: Option<String>, // TODO better routing method than strings, like maybe a route index or something
  server: Arc<Server>,
  eloop: tokio_core::reactor::Remote,
}

impl ws::Handler for WebSocketHandler {
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
    let req = match websocket_to_req(msg.to_string(), route_str) {
      Ok(req) => req,
      Err(e) => {
        return out.send(ws::Message::text(e.to_string()));
      }
    };
    let prom = self.server.handle(req).then(move |resp| {
      match resp {
        Ok(s) => out.send(ws::Message::text(s.to_string())),
        Err(s) => out.send(ws::Message::text(s.to_string())),
      }
    }).map_err(|e| {
      println!("TODO HANDLE ERROR: {:?}", e);
      ()
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
    let http_addr = (addr.clone() + ":3334").as_str().parse().unwrap();
    let server_arc = Arc::new(self);
    let server_clone = server_arc.clone();
    thread::spawn(move || {
      let server = http::Http::new().bind(&http_addr, move || {
        Ok(HttpService{server: (&server_clone).clone()})
      }).unwrap();
      println!("Listening on http://{} with 1 thread.", server.local_addr().unwrap());
      server.run().unwrap();
    });
    let server_clone = server_arc.clone();
    thread::spawn(move || {
      ws::listen((addr + ":3333").as_str(), |out| {
        return WebSocketHandler {
          sender: out.clone(),
          route: None,
          eloop: eloop_remote.clone(),
          server: (&server_clone).clone(),
        };
      })
    });
    eloop.run(futures::future::empty::<(), ()>()).unwrap();
  }
}
