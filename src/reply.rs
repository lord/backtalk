use {JsonValue, Request};
use hyper::server as http;
use hyper::Error as HyperError;
use hyper::header::{ContentLength, ContentType};
use hyper::mime;
use hyper::Chunk as HyperChunk;
use futures::{Poll, Stream, Async};
use futures::stream::BoxStream;
use futures::sync::mpsc;
use Sender;

type ChunkReceiver = BoxStream<HyperChunk, ()>;

pub struct Reply {
  data: ReplyData,
  req: Request,
}

enum ReplyData {
  Value(JsonValue),
  Stream(ChunkReceiver),
}

// only used internally, by Response to make replies
pub fn make_reply(req: Request, data: JsonValue) -> Reply {
  Reply {
    req: req,
    data: ReplyData::Value(data),
  }
}

pub fn make_streamed_reply(req: Request) -> (Sender, Reply) {
  let (tx, rx) = mpsc::unbounded();
  let rx = rx
    .map(|val| -> HyperChunk {
      format!("data:{}\n\n", val).into()
    })
    .boxed();
  let reply = Reply {
    req: req,
    data: ReplyData::Stream(rx)
  };
  let sender = Sender::new(tx);
  (sender, reply)
}

impl Reply {
  pub fn data(&self) -> Option<&JsonValue> {
    match self.data {
      ReplyData::Value(ref dat) => Some(dat),
      _ => None,
    }
  }

  pub fn to_http(self) -> http::Response<Body> {
    let resp = http::Response::new();

    match self.data {
      ReplyData::Value(val) => {
        let resp_str = val.to_string();
        resp
          .with_header(ContentLength(resp_str.len() as u64))
          .with_header(ContentType(
            mime::Mime(
              mime::TopLevel::Application,
              mime::SubLevel::Json,
              vec![(mime::Attr::Charset, mime::Value::Utf8)]
            )
          ))
          .with_body(Body::Once(Some(resp_str.into())))
      },
      ReplyData::Stream(stream) => {
        resp
          .with_header(ContentType(
            mime::Mime(
              mime::TopLevel::Text,
              mime::SubLevel::EventStream,
              vec![(mime::Attr::Charset, mime::Value::Utf8)]
            )
          ))
          .with_body(Body::Stream(stream))
      },
    }
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
          Err(()) => Err(HyperError::Incomplete)
        }
      }
    }
  }
}
