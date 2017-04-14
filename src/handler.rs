use {Reply, Request, Error};
use futures::{BoxFuture, Future};

/**
Anything that returns a future reply for a request.

You'll probably implement a bunch of these with your application-specific code. For simplicity,
any closure with the signature `Fn(Request) -> Future<Reply, Error>` is automatically a Handler.
*/
pub trait Handler: Send + Sync {
  fn handle(&self, req: Request) -> BoxFuture<Reply, Error>;
}

impl <T, F> Handler for T
  where T: Fn(Request) -> F + Send + Sync,
        F: Future<Item=Reply, Error=Error> + Send + 'static {
  fn handle(&self, req: Request) -> BoxFuture<Reply, Error> {
    self(req).boxed()
  }
}
