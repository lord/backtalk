use {JsonValue, Request, Reply};
use futures::Future;
use futures;
use futures::future::ok;
use futures::future::BoxFuture;
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

pub trait Channel: Send + Sync {
  fn join(&self, Sender);
  fn send(&self, &JsonValue);

  fn handle(&self, req: Request) -> BoxFuture<Reply, Reply> {
    let (sender, reply) = Reply::new_streamed(200, Some(req));
    self.join(sender);
    ok(reply).boxed()
  }
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

  fn send(&self, msg: &JsonValue) {
    for sender in self.senders.lock().unwrap().iter_mut() {
      sender.send(msg.clone());
    }
  }
}
