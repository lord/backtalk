use {JsonValue, JsonObject, Reply, Request, Resource, Method, Error, ErrorKind};
use futures::future::{ok, err};
use futures::{BoxFuture, Future};
use std::collections::HashMap;
use hyper;
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::header::{Accept};
use hyper::server as http;
use hyper::Method as HttpMethod;
use futures::Stream;
use futures::future::FutureResult;
use std::sync::Arc;
use queryst::parse as query_parse;
use serde_json::value::Map;
use reply::Body;
use serde_json;

fn std_error(kind: ErrorKind, err_str: &str) -> Error {
  let val = json!({
    "error": {
      "type": kind.as_string(),
      "message": err_str.to_string(),
    }
  });
  Error::new(
    kind,
    val
  )
}

pub fn http_to_req(method: &HttpMethod, path: &str, query: &str, headers: &hyper::Headers, body: Option<Vec<u8>>, server: &Arc<Server>) -> Result<Request, Error> {
  let default_accept = Accept::star();
  let accepts = headers.get::<Accept>().unwrap_or(&default_accept).as_slice().iter();
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

  let body = if let Some(b) = body {
    b
  } else {
    return Err(std_error(ErrorKind::BadRequest, "TODO error in request body"));
  };
  let body_str = match String::from_utf8(body) {
    Ok(s) => s,
    _ => return Err(std_error(ErrorKind::BadRequest, "TODO invalid unicode in request body")),
  };
  let body_obj = if body_str == "" {
    JsonObject::new()
  } else {
    match serde_json::from_str(&body_str) {
      Ok(o) => o,
      _ => return Err(std_error(ErrorKind::BadRequest, "TODO invalid json in request body")),
    }
  };
  let query = match query_parse(query) {
    Ok(JsonValue::Null) => Map::new(),
    Ok(JsonValue::Object(u)) => u,
    _ => return Err(std_error(ErrorKind::BadRequest, "TODO failed to parse query string"))
  };
  let mut parts: Vec<&str> = path.split("/").skip(1).collect();
  // remove trailing `/` part if present
  if let Some(&"") = parts.last() {
    parts.pop();
  }

  let resource_url = format!("/{}", parts.join("/"));
  if server.has_resource(&resource_url) {
    if is_eventsource { // TODO should only work for GET? 403 otherwise? better spec compliance
      return Ok(Request::new(
        resource_url,
        Method::Listen,
        None,
        JsonObject::new(),
        query
      ))
    } else if method == &HttpMethod::Get {
      return Ok(Request::new(
        resource_url,
        Method::List,
        None,
        JsonObject::new(),
        query
      ))
    } else if method == &HttpMethod::Post {
      return Ok(Request::new(
        resource_url,
        Method::Post,
        None,
        body_obj,
        query
      ))
    } else {
      return Err(std_error(ErrorKind::MethodNotAllowed, "invalid HTTP method for this URL"));
    }
  }

  let (id, parts) = match parts.split_last() {
    Some(t) => t,
    None => return Err(std_error(ErrorKind::NotFound, "resource not found"))
  };
  let resource_url = format!("/{}", parts.join("/"));
  if server.has_resource(&resource_url) {
    if is_eventsource { 
      return Ok(Request::new(
        resource_url,
        Method::Listen,
        Some(id.to_string()),
        JsonObject::new(),
        query
      ))
    } else if method == &HttpMethod::Get {
      return Ok(Request::new(
        resource_url,
        Method::Get,
        Some(id.to_string()),
        JsonObject::new(),
        query
      ))
    } else if method == &HttpMethod::Patch {
      return Ok(Request::new(
        resource_url,
        Method::Patch,
        Some(id.to_string()),
        body_obj,
        query
      ))
    } else if method == &HttpMethod::Delete {
      return Ok(Request::new(
        resource_url,
        Method::Delete,
        Some(id.to_string()),
        JsonObject::new(),
        query
      ))
    } else {
      return Err(std_error(ErrorKind::MethodNotAllowed, "invalid HTTP method for this URL"));
    }
  }

  let action_name = id;
  let (id, parts) = match parts.split_last() {
    Some(t) => t,
    None => return Err(std_error(ErrorKind::NotFound, "resource not found"))
  };
  let resource_url = format!("/{}", parts.join("/"));
  if server.has_resource(&resource_url) {
    if method == &HttpMethod::Post {
      return Ok(Request::new(
        resource_url,
        Method::Action(action_name.to_string()),
        Some(id.to_string()),
        JsonObject::new(),
        query
      ))
    } else {
      return Err(std_error(ErrorKind::MethodNotAllowed, "invalid HTTP method for this URL"));
    }
  }

  Err(std_error(ErrorKind::NotFound, "resource not found"))
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

    let server = self.server.clone();
    let body_prom = body.fold(Vec::new(), |mut a, b| -> FutureResult<Vec<u8>, hyper::Error> { a.extend_from_slice(&b[..]); ok(a) });

    body_prom.then(move |body_res| {
      match http_to_req(&method, uri.path(), uri.query().unwrap_or(""), &headers, body_res.ok(), &server) {
        Ok(req) => server.handle(req),
        Err(reply) => err(reply).boxed(),
      }
    }).then(|reply| {
      let http_resp = match reply {
        Ok(r) => r.to_http(),
        Err(r) => r.to_http(),
      };
      ok(http_resp)
    }).boxed()
  }
}

pub struct Server {
  route_table: HashMap<String, Box<Resource>>
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

  pub fn handle(&self, req: Request) -> BoxFuture<Reply, Error> {
    // TODO maybe instead do some sort of indexing instead of all this string hashing, so like, the webhooks calls get_route_ref or something
    match self.route_table.get(req.resource()) {
      Some(resource) => resource.handle(req),
      None => err(Error::new(ErrorKind::NotFound, JsonValue::String("TODO not found error here".to_string()))).boxed()
    }
  }

  pub fn resource<T: Into<String>, R: Resource + 'static>(&mut self, route: T, resource: R) {
    self.route_table.insert(route.into(), Box::new(resource));
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
