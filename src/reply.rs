use super::{JsonValue, Req};
use std::convert::From;
use hyper::server as http;
use hyper::Error as HyperError;
use hyper::header::{ContentLength, ContentType, Accept};
use hyper::Chunk as HyperChunk;
use futures::{Poll, Stream, Async};
use futures::sync::mpsc;
use Sender;

type MpscReceiver = mpsc::UnboundedReceiver<HyperChunk>;

pub struct Reply {
  data: ReplyData,
  code: i64, // TODO replace with enum of errors, etc
  req: Option<Req>,
}

enum ReplyData {
  Value(JsonValue),
  Stream(MpscReceiver),
}

impl Reply {
  // TODO refine this? currently only really should be used internally.
  pub fn new(code: i64, req: Option<Req>, data: JsonValue) -> Reply {
    Reply {
      code: code,
      req: req,
      data: ReplyData::Value(data),
    }
  }

  pub fn to_http(self) -> http::Response<Body> {
    let resp_str = self.to_string();
    http::Response::new()
      .with_header(ContentLength(resp_str.len() as u64))
      .with_body(resp_str)
    //   let resp = http::Response::new()
    //     .with_header(ContentType(Mime(TopLevel::Text, SubLevel::EventStream, vec![(hyper::mime::Attr::Charset, hyper::mime::Value::Utf8)])))
    //     .with_body(body);
  }

  pub fn new_streamed(code: i64, req: Option<Req>) -> (Sender, Reply) {
    let (tx, rx) = mpsc::unbounded();
    let reply = Reply {
      code: code,
      req: req,
      data: ReplyData::Stream(rx)
    };
    let sender = Sender::new(tx);
    (sender, reply)
  }

  pub fn to_string(self) -> String {
    match self.data {
      ReplyData::Value(val) => val.to_string(),
      ReplyData::Stream(_) => unimplemented!(),
    }
  }
}

/// A `Stream` for `HyperChunk`s used in requests and responses.
pub enum Body {
  Once(Option<HyperChunk>),
  Stream(MpscReceiver),
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
          Err(()) => Err(HyperError::Incomplete) // TODO FIX THIS ERROR
        }
      }
    }
  }
}

impl From<String> for Body {
  fn from(s: String) -> Body {
    Body::Once(Some(s.into()))
  }
}

