use JsonValue;
use reply::Body;
use hyper::server as http;
use hyper::header::ContentLength;
use hyper::status::StatusCode;

pub struct Error {
  data: JsonValue,
  kind: ErrorKind,
}

pub enum ErrorKind {
  Unauthorized,
  Forbidden,
  TooManyRequests,
  NotFound,
  BadRequest,
  ServerError,
  Unavailable,
  MethodNotAllowed,
}

impl ErrorKind {
  fn to_hyper_status(&self) -> StatusCode {
    match self {
      &ErrorKind::Unauthorized => StatusCode::Unauthorized,
      &ErrorKind::Forbidden => StatusCode::Forbidden,
      &ErrorKind::TooManyRequests => StatusCode::TooManyRequests,
      &ErrorKind::NotFound => StatusCode::NotFound,
      &ErrorKind::BadRequest => StatusCode::BadRequest,
      &ErrorKind::ServerError => StatusCode::InternalServerError,
      &ErrorKind::Unavailable => StatusCode::ServiceUnavailable,
      &ErrorKind::MethodNotAllowed => StatusCode::MethodNotAllowed,
    }
  }
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
      .with_status(self.kind.to_hyper_status())
      .with_header(ContentLength(resp_str.len() as u64))
      .with_body(Body::Once(Some(resp_str.into())))
  }
}
