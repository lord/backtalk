use {JsonValue};
use reply::Body;
use hyper::server as http;
use hyper::header::{ContentLength,ContentType};
use hyper::mime;
use hyper::StatusCode;
use futures::future::{err, BoxFuture, Future};

/**
An error response to be sent back to the client.

Contains a JSON error that the client can reply to. The easiest way to create one of these
is the various `Error::bad_request`, `Error::unavailable` etc. functions, which automatically
return a `BoxFuture<T, Error>`, so you can return them directly from an `and_then` closure without
wrapping in a future or boxing.

If you need custom JSON in your error, you can use the `Error::new` function directly.

Currently there's no way to access or change the insides of an `Error`, but that probably will
change in the near future.
*/
#[derive(Debug)]
pub struct Error {
  data: JsonValue,
  kind: ErrorKind,
}

/**
A type of error, for instance "Bad Request" or "Server Error".
*/
#[derive(Debug)]
pub enum ErrorKind {
  /// The route requires authorization, and it was not provided or was invalid.
  Unauthorized,
  /// The authorization was valid, but the authorized user has insufficient permissions for this route.
  Forbidden,
  /// The client has been sending requests too quickly, and needs to slow down.
  RateLimited,
  /// The URL wasn't found.
  NotFound,
  /// The request was invalid or bad for some reason.
  BadRequest,
  /// The request was invalid or bad for some reason.
  ServerError,
  /// The server is temporarily overloaded or down for maintenance.
  Unavailable,
  /// This HTTP method isn't allowed at this URL, and another method would be valid.
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

  /**
  Returns the string version of the error. For instance, `BadRequest` returns `"bad_request"`.
  */
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
  /**

  */
  pub fn new(kind: ErrorKind, data: JsonValue) -> Error {
    Error {
      kind: kind,
      data: data,
    }
  }

  pub fn unauthorized<T: Send + 'static>(msg: &str) -> BoxFuture<T, Error> {
    err(std_error(ErrorKind::Unauthorized, msg)).boxed()
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
      .with_header(ContentType(mime::APPLICATION_JSON))
      .with_body(Body::Once(Some(resp_str.into())))
  }
}
