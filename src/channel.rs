use ::JsonValue;
use futures;
use hyper::Chunk;

type ChunkSender = futures::sync::mpsc::UnboundedSender<Chunk>;

pub struct Sender {
  inner: ChunkSender,
}

impl Sender {
  pub fn new(chunk_sender: ChunkSender) -> Sender {
    Sender {
      inner: chunk_sender,
    }
  }

  pub fn send(&mut self, val: JsonValue) -> Result<(), ()> {
    let wrapped_str = format!("data:{}\n\n", val);
    self.inner.send(wrapped_str.into()).map_err(|_| ())
  }
}

// TODO also maybe would be nice to be able to store authentication information somewhere here

pub trait Channel: Send + Sync {
  fn join(&self, Sender);
  fn handle(&self, JsonValue);
}
