use ::JsonValue;
use ::Method;
use futures;
use hyper::Chunk;
use std::sync::Mutex;

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
  fn handle(&self, Method, JsonValue);
}

pub struct BroadcastChannel {
  senders: Mutex<Vec<Sender>>,
}

impl BroadcastChannel {
  pub fn new() -> BroadcastChannel {
    BroadcastChannel {
      senders: Mutex::new(Vec::new()),
    }
  }
}

impl Channel for BroadcastChannel {
  fn join(&self, sender: Sender) {
    self.senders.lock().unwrap().push(sender)
  }

  fn handle(&self, _: Method, msg: JsonValue) {
    for sender in self.senders.lock().unwrap().iter_mut() {
      sender.send(msg.clone());
    }
  }
}
