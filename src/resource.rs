use {Reply, Request, Error};
use futures::BoxFuture;

pub trait Resource: Send + Sync {
  fn handle(&self, req: Request) -> BoxFuture<Reply, Error>;
}

impl <T> Resource for T where T: Fn(Request) -> BoxFuture<Reply, Error> + Send + Sync {
  fn handle(&self, req: Request) -> BoxFuture<Reply, Error> {
    self(req)
  }
}
