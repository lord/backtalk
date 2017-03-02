use futures::{BoxFuture, Future};
use futures::future::{ok, err};
use std::sync::Arc;
use std::collections::HashMap;
use super::{Adapter, Reply, Req, JsonValue, Method, Channel};

pub trait Resource: Send + Sync {
  fn handle(&self, req: Req) -> BoxFuture<Reply, Reply>;
}

impl <T> Resource for T where T: Fn(Req) -> BoxFuture<Reply, Reply> + Send + Sync {
  fn handle(&self, req: Req) -> BoxFuture<Reply, Reply> {
    self(req)
  }
}
