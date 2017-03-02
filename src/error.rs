use JsonValue;
use reply::Body;
use hyper::server as http;
use hyper::header::ContentLength;

pub struct Error {
  data: JsonValue,
  kind: ErrorKind,
}

pub enum ErrorKind {
  Unauthorized,
  NotFound,
  TODO,
}

impl Error {
  pub fn new(kind: ErrorKind, data: JsonValue) -> Error {
    Error {
      kind: kind,
      data: data,
    }
  }

  pub fn to_http(self) -> http::Response<Body> {
    let resp = http::Response::new();
    let resp_str = self.data.to_string();
    // TODO SET STATUS
    resp
      .with_header(ContentLength(resp_str.len() as u64))
      .with_body(Body::Once(Some(resp_str.into())))
  }
}
