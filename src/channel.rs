use ::JsonValue;
use ::Method;
use futures;
use hyper::Chunk;
use std::sync::Mutex;

type ValueSender = futures::sync::mpsc::UnboundedSender<JsonValue>;

pub struct Sender {
  inner: ValueSender,
}

impl Sender {
  pub fn new(chunk_sender: ValueSender) -> Sender {
    Sender {
      inner: chunk_sender,
    }
  }

  pub fn send(&mut self, val: JsonValue) -> Result<(), ()> {
    self.inner.send(val).map_err(|_| ())
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
