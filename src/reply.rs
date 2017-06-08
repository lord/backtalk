use {JsonValue, Request, Method, JsonObject, Error, channel};
use std::fmt;
use hyper::server as http;
use hyper::Error as HyperError;
use hyper::header::{ContentLength, ContentType};
use hyper::mime;
use hyper::Chunk as HyperChunk;
use futures::{Poll, Stream, Async, IntoFuture};
use futures::future::{ok, FutureResult, BoxFuture, Future};
use futures::stream::BoxStream;
use futures::sync::mpsc;
use Sender;

type ChunkReceiver = BoxStream<HyperChunk, ()>;

/**
A successful response with JSON data to be sent back to the client.

There are two kinds of replies. Static replies represent JSON data that is ready. Most requests
return static replies. Streaming replies represent a stream of JSON data that will stream from
a `Channel` directly to the client. You can't access the data of a streaming reply through the
`Reply` struct, since it's not ready yet. If you want to transform or edit the reply data for a
stream, you'll need to implement a custom `Channel` instead.

These are several ways to create a Reply:

- pass a Request to an Adapter to get a static response from a database
- pass a Request to a Channel to get a streaming response
- in your custom Resource, call `request.into_reply(data)` to create a Reply object.

Reply implements `IntoFuture`, so you can return it directly from a `and_then` block.
*/
#[derive(Debug)]
pub struct Reply {
  data: ReplyData,
  req: Request,
}

enum ReplyData {
  Value(JsonObject),
  Stream(ChunkReceiver),
}

impl fmt::Debug for ReplyData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &ReplyData::Value(ref val) => write!(f, "ReplyData::Value({:?})", val),
      &ReplyData::Stream(_) => write!(f, "ReplyData::Stream(<stream>)"),
    }
  }
}

// only used internally
pub fn make_reply(req: Request, data: JsonObject) -> Reply {
  Reply {
    req: req,
    data: ReplyData::Value(data),
  }
}

// only used internally
pub fn make_streamed_reply(req: Request) -> (Sender, Reply) {
  let (tx, rx) = mpsc::unbounded();
  let rx = rx
    .map(|val: (String, JsonObject)| -> HyperChunk {
      format!("event:{}\ndata:{}\n\n", val.0, JsonValue::Object(val.1)).into()
    })
    .boxed();
  let reply = Reply {
    req: req,
    data: ReplyData::Stream(rx)
  };
  let sender = channel::new_sender(tx);
  (sender, reply)
}

impl Reply {
  pub fn data(&self) -> Option<&JsonObject> {
    match self.data {
      ReplyData::Value(ref dat) => Some(dat),
      _ => None,
    }
  }

  pub fn data_mut(&mut self) -> Option<&mut JsonObject> {
    match self.data {
      ReplyData::Value(ref mut dat) => Some(dat),
      _ => None,
    }
  }

  // TODO data_then accepts a function that returns a future<JsonObject, Error>

  pub fn to_http(self) -> http::Response<Body> {
    let resp = http::Response::new();

    match self.data {
      ReplyData::Value(val) => {
        let resp_str = JsonValue::Object(val).to_string();
        resp
          .with_header(ContentLength(resp_str.len() as u64))
          .with_header(ContentType(mime::APPLICATION_JSON))
          .with_body(Body::Once(Some(resp_str.into())))
      },
      ReplyData::Stream(stream) => {
        resp
          .with_header(ContentType(mime::TEXT_EVENT_STREAM))
          .with_body(Body::Stream(stream))
      },
    }
  }

  pub fn method(&self) -> Method {
    self.req.method()
  }

  pub fn resource(&self) -> &str {
    &self.req.resource()
  }

  pub fn id(&self) -> &Option<String> {
    &self.req.id()
  }

  pub fn params(&self) -> &JsonObject {
    &self.req.params()
  }

  pub fn param(&self, key: &str) -> &JsonValue {
    self.req.param(key)
  }

  pub fn boxed(self) -> BoxFuture<Reply, Error> {
    ok(self).boxed()
  }

  pub fn request_data(&self) -> &JsonObject {
    self.req.data()
  }
}

impl IntoFuture for Reply {
  type Item = Reply;
  type Error = Error;
  type Future = FutureResult<Reply, Error>;
  fn into_future(self) -> Self::Future {
    ok(self)
  }
}

/// A `Stream` for `HyperChunk`s used in requests and responses.
pub enum Body {
  Once(Option<HyperChunk>),
  Stream(ChunkReceiver),
}

impl Stream for Body {
  type Item = HyperChunk;
  type Error = HyperError;

  fn poll(&mut self) -> Poll<Option<HyperChunk>, HyperError> {
    match self {
      &mut Body::Once(ref mut opt) => Ok(Async::Ready(opt.take())),
      &mut Body::Stream(ref mut stream) => {
        match stream.poll() {
          Ok(u) => Ok(u),
          Err(()) => Ok(Async::Ready(None)), // this probably can never happen?
        }
      }
    }
  }
}
