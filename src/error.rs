use {JsonValue};
use reply::Body;
use hyper::server as http;
use hyper::header::{ContentLength,ContentType};
use hyper::mime;
use hyper::status::StatusCode;
use futures::future::{err, BoxFuture, Future};

#[derive(Debug)]
pub struct Error {
  data: JsonValue,
  kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
  Unauthorized,
  Forbidden,
  RateLimited,
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
      &ErrorKind::RateLimited => StatusCode::TooManyRequests,
      &ErrorKind::NotFound => StatusCode::NotFound,
      &ErrorKind::BadRequest => StatusCode::BadRequest,
      &ErrorKind::ServerError => StatusCode::InternalServerError,
      &ErrorKind::Unavailable => StatusCode::ServiceUnavailable,
      &ErrorKind::MethodNotAllowed => StatusCode::MethodNotAllowed,
    }
  }

  pub fn as_string(&self) -> String {
    match self {
      &ErrorKind::Unauthorized => "authorization",
      &ErrorKind::Forbidden => "authorization",
      &ErrorKind::RateLimited => "rate_limit",
      &ErrorKind::NotFound => "not_found",
      &ErrorKind::BadRequest => "bad_request",
      &ErrorKind::ServerError => "server",
      &ErrorKind::Unavailable => "server",
      &ErrorKind::MethodNotAllowed => "bad_request",
    }.to_string()
  }
}

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

impl Error {
  pub fn new(kind: ErrorKind, data: JsonValue) -> Error {
    Error {
      kind: kind,
      data: data,
    }
  }

  pub fn forbidden<T: Send + 'static>(msg: &str) -> BoxFuture<T, Error> {
    err(std_error(ErrorKind::Forbidden, msg)).boxed()
  }
  pub fn rate_limited<T: Send + 'static>(msg: &str) -> BoxFuture<T, Error> {
    err(std_error(ErrorKind::RateLimited, msg)).boxed()
  }
  pub fn not_found<T: Send + 'static>(msg: &str) -> BoxFuture<T, Error> {
    err(std_error(ErrorKind::NotFound, msg)).boxed()
  }
  pub fn bad_request<T: Send + 'static>(msg: &str) -> BoxFuture<T, Error> {
    err(std_error(ErrorKind::BadRequest, msg)).boxed()
  }
  pub fn server_error<T: Send + 'static>(msg: &str) -> BoxFuture<T, Error> {
    err(std_error(ErrorKind::ServerError, msg)).boxed()
  }
  pub fn unavailable<T: Send + 'static>(msg: &str) -> BoxFuture<T, Error> {
    err(std_error(ErrorKind::Unavailable, msg)).boxed()
  }
  pub fn method_not_allowed<T: Send + 'static>(msg: &str) -> BoxFuture<T, Error> {
    err(std_error(ErrorKind::MethodNotAllowed, msg)).boxed()
  }

  pub fn to_http(self) -> http::Response<Body> {
    let resp = http::Response::new();
    let resp_str = self.data.to_string();
    resp
      .with_status(self.kind.to_hyper_status())
      .with_header(ContentLength(resp_str.len() as u64))
      .with_header(ContentType(
        mime::Mime(
          mime::TopLevel::Application,
          mime::SubLevel::Json,
          vec![(mime::Attr::Charset, mime::Value::Utf8)]
        )
      ))
      .with_body(Body::Once(Some(resp_str.into())))
  }
}
