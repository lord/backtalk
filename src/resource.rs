use {Reply, Request};
use futures::BoxFuture;

pub trait Resource: Send + Sync {
  fn handle(&self, req: Request) -> BoxFuture<Reply, Reply>;
}

impl <T> Resource for T where T: Fn(Request) -> BoxFuture<Reply, Reply> + Send + Sync {
  fn handle(&self, req: Request) -> BoxFuture<Reply, Reply> {
    self(req)
  }
}
