use futures::BoxFuture;
use {Reply, Req};

pub trait Resource: Send + Sync {
  fn handle(&self, req: Req) -> BoxFuture<Reply, Reply>;
}

impl <T> Resource for T where T: Fn(Req) -> BoxFuture<Reply, Reply> + Send + Sync {
  fn handle(&self, req: Req) -> BoxFuture<Reply, Reply> {
    self(req)
  }
}
