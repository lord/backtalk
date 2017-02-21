extern crate backtalk;
extern crate futures;

use backtalk::*;
use futures::{BoxFuture, Future};
use futures::future::{ok, err};

struct MyHook;
impl BeforeHook for MyHook {
  fn handle(&self, req: Req) -> BoxFuture<Req, Reply> {
    ok(req).boxed()
  }
}

fn main() {
  let mut s = Server::new();
  let mut r = Resource::new(MemoryAdapter{});
  // for _ in 0..1000 {
  //   r.before(MyHook{});
  // }
  s.mount("/hello", r);
  s.listen("127.0.0.1");
}
