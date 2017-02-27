use std::convert::From;
use hyper::Error;
use hyper::Chunk;
use futures::{Poll, Stream, Async};
use futures::sync::mpsc;

pub type MpscReceiver = mpsc::UnboundedReceiver<Chunk>;

/// A `Stream` for `Chunk`s used in requests and responses.
pub enum Body {
  Once(Option<Chunk>),
  Stream(MpscReceiver),
}

impl Body {
  pub fn pair() -> (mpsc::UnboundedSender<Chunk>, Body) {
    let (tx, rx) = mpsc::unbounded();
    let rx = Body::Stream(rx);
    (tx, rx)
  }
}

impl Stream for Body {
  type Item = Chunk;
  type Error = Error;

  fn poll(&mut self) -> Poll<Option<Chunk>, Error> {
    match self {
      &mut Body::Once(ref mut opt) => Ok(Async::Ready(opt.take())),
      &mut Body::Stream(ref mut stream) => {
        match stream.poll() {
          Ok(u) => Ok(u),
          Err(()) => Err(Error::Incomplete) // TODO FIX THIS ERROR
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
