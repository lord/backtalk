extern crate backtalk;
extern crate futures;

use backtalk::*;

fn main() {
  let mut s = Server::new();
  let adapter = MemoryAdapter{};
  let channel = BroadcastChannel::new();
  s.resource("/hello", move |req: Req| {
    match req.method().clone() {
      // Method::Action(ref action_name) => {
      //   unimplemented!();
      // },
      Method::Listen => channel.handle(req),
      _ => adapter.handle(req),
    }
  });
  s.listen("127.0.0.1:3000");
}
