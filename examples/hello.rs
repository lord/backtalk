extern crate backtalk;
extern crate futures;

use backtalk::*;
use std::sync::{Arc, Mutex};
use futures::future::{Future, BoxFuture, ok};

fn main() {
  let mut s = Server::new();
  let adapter = TestAdapter{};
  let channel = Arc::new(TestChannel::new());
  s.resource("/hello", move |req: Request| {
    let res = match req.method().clone() {
      // Method::Action(ref action_name) => {
      //   unimplemented!();
      // },
      Method::Listen => channel.handle(req),
      _ => adapter.handle(req),
    };
    let channel2 = channel.clone();
    res.map(move |reply| {
      if let Some(dat) = reply.data() {
        channel2.send("test kind", dat);
      }
      reply
    }).boxed()
  });
  s.listen("127.0.0.1:3000");
}

#[derive(Clone)]
struct TestAdapter {}

impl Adapter for TestAdapter {
  fn find(&self, _params: &Params) -> BoxFuture<JsonValue, (ErrorKind, JsonValue)> {
    ok(JsonValue::Array(vec![JsonValue::String("foo".to_string())])).boxed()
  }

  fn get(&self, _id: &str, _params: &Params) -> BoxFuture<JsonValue, (ErrorKind, JsonValue)> {
    ok(JsonValue::Array(vec![JsonValue::String("foo".to_string())])).boxed()
  }

  fn post(&self, _data: &JsonValue, _params: &Params) -> BoxFuture<JsonValue, (ErrorKind, JsonValue)> {
    ok(JsonValue::Array(vec![JsonValue::String("foo".to_string())])).boxed()
  }

  fn patch(&self, _id: &str, _data: &JsonValue, _params: &Params) -> BoxFuture<JsonValue, (ErrorKind, JsonValue)> {
    ok(JsonValue::Array(vec![JsonValue::String("foo".to_string())])).boxed()
  }

  fn delete(&self, _id: &str, _params: &Params) -> BoxFuture<JsonValue, (ErrorKind, JsonValue)> {
    ok(JsonValue::String("foo".to_string())).boxed()
  }
}

pub struct TestChannel {
  senders: Mutex<Vec<Sender>>,
}

impl TestChannel {
  pub fn new() -> TestChannel {
    TestChannel {
      senders: Mutex::new(Vec::new()),
    }
  }
}

impl Channel for TestChannel {
  fn join(&self, sender: Sender) {
    self.senders.lock().unwrap().push(sender)
  }

  fn send(&self, message_kind: &str, msg: &JsonValue) {
    for sender in self.senders.lock().unwrap().iter_mut() {
      sender.send(message_kind, msg.clone());
    }
  }
}
