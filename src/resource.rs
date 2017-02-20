use futures::{BoxFuture, Future};
use futures::future::{ok, err};
use super::{Adapter, Reply, Req, JsonValue, Method};

pub struct Resource {
  adapter: Box<Adapter>,
  before: Vec<Box<BeforeHook>>,
  after: Vec<Box<AfterHook>>,
}

impl Resource {
  pub fn new<T: Adapter + 'static>(adapt: T) -> Resource {
    Resource {
      adapter: Box::new(adapt),
      before: Vec::new(),
      after: Vec::new(),
    }
  }

  pub fn before<T: BeforeHook + 'static>(&mut self, hook: T) {
    self.before.push(Box::new(hook));
  }

  pub fn after<T: AfterHook + 'static>(&mut self, hook: T) {
    self.after.push(Box::new(hook));
  }

  pub fn handle(&self, req: Req) -> BoxFuture<Reply, Reply> {
    fn make_err(err_str: &str) -> BoxFuture<Reply, Reply> {
      err(Reply::new(400, None, JsonValue::Array(vec![JsonValue::String("error!".to_string()), JsonValue::String(err_str.to_string())]))).boxed()
    }

    let mut req: BoxFuture<Req, Reply> = ok(req).boxed();
    for hook in self.before {
      req = req.and_then(|req| hook.handle(req)).boxed();
    }

    // TODO: THE PROBLEM WITH THIS CODE IS THAT THIS CLOSURES
    // MIGHT OUTLIVE SELF, which is a problem for both self.adapter and self.before and self.after hooks

    let reply = req.and_then(move |req| {
      let res = match (req.method(), req.id()) {
        (&Method::List, _) => self.adapter.find(req.params()),
        (&Method::Post, _) => self.adapter.post(req.data(), req.params()),
        (&Method::Get, &Some(ref id)) => self.adapter.get(id, req.params()),
        (&Method::Delete, &Some(ref id)) => self.adapter.delete(id, req.params()),
        (&Method::Patch, &Some(ref id)) => self.adapter.patch(id, req.data(), req.params()),
        (&Method::Action(_), _) => unimplemented!(),
        (_, &None) => return make_err("missing id in request"),
      };
      res.then(move |res| match res {
        Ok(val) => Ok(req.into_reply(200, val)),
        Err((code, val)) => Err(req.into_reply(code, val)),
      }).boxed()
    }).boxed();

    for hook in self.after {
      reply = reply.and_then(|reply| hook.handle(reply)).boxed();
    }

    reply
  }
}

// TODO ALLOW HOOKS TO RETURN ANY KIND OF FUTURE? so we avoid double boxed allocations
pub trait BeforeHook: Sync + Send {
  fn handle(&self, Req) -> BoxFuture<Req, Reply>;
}

pub trait AfterHook: Sync + Send {
  fn handle(&self, Reply) -> BoxFuture<Reply, Reply>;
}
