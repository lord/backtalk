use ::JsonValue;
use futures;
use hyper;
use hyper::Chunk;

type ChunkSender = futures::sync::mpsc::Sender<Result<Chunk, hyper::Error>>;

pub struct Sender {
  chunk_sender: ChunkSender,
  capacity: u64,
}

impl Sender {
  fn new(cap: u64, chunk_sender: ChunkSender) -> Sender {
    Sender {
      chunk_sender: chunk_sender,
      capacity: cap,
    }
  }
}

// TODO also maybe would be nice to be able to store authentication information somewhere here

pub trait Channel {
  fn join(&self, Sender);
  fn handle(&self, JsonValue);
}
