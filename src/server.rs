use super::{JsonValue, Reply, Req, Resource, Method};
use futures::future::{ok, err};
use futures::{BoxFuture, Future};
use std::collections::HashMap;
use std::time::Duration;
use std::thread;
use hyper;
use hyper::StatusCode;
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::header::{ContentLength, ContentType, Accept};
use hyper::server as http;
use hyper::Method as HttpMethod;
use futures::Stream;
use futures::future::FutureResult;
use std::sync::Arc;
use queryst::parse as query_parse;
use serde_json::Map;
use serde_json;
use ::Sender;
use reply::Body;

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

// only one is created
#[derive(Clone)]
struct HttpService {
  server: Arc<Server>,
}

impl http::Service for HttpService {
  type Request = http::Request;
  type Response = http::Response<Body>;
  type Error = hyper::Error;
  type Future = BoxFuture<Self::Response, Self::Error>;

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

    // if is_eventsource {
    //   let (sender, _) = Reply::new_streamed(None);
    //   thread::spawn(move || {
    //     let mut n = 0;
    //     loop {
    //       n+=1;
    //       if n == 10 {
    //         return
    //       }
    //       match sender.send(JsonValue::Array(vec![JsonValue::String("meow".to_string())])) {
    //         Err(_) => return,
    //         _ => (),
    //       };
    //       match sender.send(JsonValue::Array(vec![JsonValue::String("meow".to_string())])) {
    //         Err(_) => return,
    //         _ => (),
    //       };
    //       thread::sleep(Duration::from_millis(500));
    //     }
    //   });
    //   let resp = http::Response::new()
    //     .with_header(ContentType(Mime(TopLevel::Text, SubLevel::EventStream, vec![(hyper::mime::Attr::Charset, hyper::mime::Value::Utf8)])))
    //     .with_body(body);
    //   return ok(resp).boxed();
    // }

    body_prom.then(move |body_res| {
      match http_to_req(&method, uri.path(), uri.query().unwrap_or(""), body_res.ok(), &server) {
        Ok(req) => server.handle(req),
        Err(reply) => err(reply).boxed(),
      }
    }).then(|reply| -> BoxFuture<hyper::server::Response<Body>, hyper::Error> {
      let http_resp = match reply {
        Ok(r) => r.to_http(),
        Err(r) => r.to_http(),
      };
      ok(http_resp).boxed()
    }).boxed()
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
    let addr: String = bind_addr.into();
    let http_addr = addr.as_str().parse().unwrap();
    let server_arc = Arc::new(self);
    let server_clone = server_arc.clone();
    let server = http::Http::new().bind::<_, Body>(&http_addr, move || {
      Ok(HttpService{server: (&server_clone).clone()})
    }).unwrap();
    println!("Listening on http://{} with 1 thread.", server.local_addr().unwrap());
    server.run().unwrap();
  }
}
