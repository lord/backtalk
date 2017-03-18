use {JsonValue, Sender, Channel, Params};
use std::sync::Mutex;

pub struct MemoryChannel {
  senders: Mutex<Vec<Sender>>,
}

impl MemoryChannel {
  pub fn new() -> MemoryChannel {
    MemoryChannel {
      senders: Mutex::new(Vec::new()),
    }
  }
}

impl Channel for MemoryChannel {
  fn join(&self, sender: Sender, _: Params) {
    self.senders.lock().unwrap().push(sender)
  }

  fn send(&self, message_kind: &str, msg: &JsonValue) {
    for sender in self.senders.lock().unwrap().iter_mut() {
      sender.send(message_kind, msg.clone());
    }
  }
}
