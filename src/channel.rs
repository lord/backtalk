use {JsonValue, Request, Reply, Error, Params};
use reply::make_streamed_reply;
use futures::Future;
use futures;
use futures::future::ok;
use futures::future::BoxFuture;

type ValueSender = futures::sync::mpsc::UnboundedSender<(String, JsonValue)>;

pub struct Sender {
  inner: ValueSender,
}

impl Sender {
  pub fn new(chunk_sender: ValueSender) -> Sender {
    Sender {
      inner: chunk_sender,
    }
  }

  pub fn send<S: Into<String>>(&mut self, event_type: S, val: JsonValue) -> Result<(), ()> {
    self.inner.send((event_type.into(), val)).map_err(|_| ())
  }
}

pub trait Channel: Send + Sync {
  fn join(&self, Sender, Params);
  fn send(&self, &str, &JsonValue);

  fn handle(&self, req: Request) -> BoxFuture<Reply, Error> {
    let params = req.params().clone();
    let (sender, reply) = make_streamed_reply(req);
    self.join(sender, params);
    ok(reply).boxed()
  }
}
