use {Reply, Request, Error};
use futures::{BoxFuture, Future};

pub trait Resource: Send + Sync {
  fn handle(&self, req: Request) -> BoxFuture<Reply, Error>;
}

impl <T, F> Resource for T
  where T: Fn(Request) -> F + Send + Sync,
        F: Future<Item=Reply, Error=Error> + Send + 'static {
  fn handle(&self, req: Request) -> BoxFuture<Reply, Error> {
    self(req).boxed()
  }
}
