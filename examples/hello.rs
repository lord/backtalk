extern crate backtalk;
extern crate futures;

use backtalk::*;
use futures::{BoxFuture, Future};
use futures::future::ok;

fn main() {
  let mut s = Server::new();
  let mut r = Resource::new(MemoryAdapter{});
  r.channel(BroadcastChannel::new());
  s.mount("/hello", r);
  s.listen("127.0.0.1:3000");
}
