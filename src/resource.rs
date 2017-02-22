use futures::{BoxFuture, Future};
use futures::future::{ok, err};
use std::sync::Arc;
use std::collections::HashMap;
use super::{Adapter, Reply, Req, JsonValue, Method};

pub struct Resource {
  adapter: Arc<Box<Adapter>>,
  before: Vec<Arc<Box<BeforeHook>>>,
  after: Vec<Arc<Box<AfterHook>>>,
  actions: Arc<HashMap<String, Box<Action>>>,
}

impl Resource {
  pub fn new<T: Adapter + 'static>(adapt: T) -> Resource {
    Resource {
      adapter: Arc::new(Box::new(adapt)),
      before: Vec::new(),
      after: Vec::new(),
      actions: Arc::new(HashMap::new()),
    }
  }

  pub fn before<T: BeforeHook + 'static>(&mut self, hook: T) {
    self.before.push(Arc::new(Box::new(hook)));
  }

  pub fn after<T: AfterHook + 'static>(&mut self, hook: T) {
    self.after.push(Arc::new(Box::new(hook)));
  }

  pub fn action<T: Action + 'static, S: Into<String>>(&mut self, name: S, action: T) {
    Arc::get_mut(&mut self.actions).unwrap().insert(name.into(), Box::new(action));
  }

  pub fn handle(&self, req: Req) -> BoxFuture<Reply, Reply> {
    fn make_err(err_str: &str) -> BoxFuture<Reply, Reply> {
      err(Reply::new(400, None, JsonValue::Array(vec![JsonValue::String("error!".to_string()), JsonValue::String(err_str.to_string())]))).boxed()
    }

    let mut req: BoxFuture<Req, Reply> = ok(req).boxed();
    for hook in &self.before {
      let hook = hook.clone(); // TODO SEE IF THIS IS NECESSARY
      req = req.and_then(move |req| hook.handle(req)).boxed();
    }

    let adapter = self.adapter.clone();
    let actions = self.actions.clone();

    let mut reply = req.and_then(move |req| {
      let res = match (req.method().clone(), req.id().clone()) {
        (Method::List, _) => adapter.find(req.params()),
        (Method::Post, _) => adapter.post(req.data(), req.params()),
        (Method::Get, Some(ref id)) => adapter.get(id, req.params()),
        (Method::Delete, Some(ref id)) => adapter.delete(id, req.params()),
        (Method::Patch, Some(ref id)) => adapter.patch(id, req.data(), req.params()),
        (Method::Action(ref action_name), _) => {
          return match actions.get(action_name) {
            Some(action) => action.handle(req),
            None => make_err("action not found"),
          }
        },
        (_, None) => return make_err("missing id in request"),
      };
      res.then(move |res| match res {
        Ok(val) => Ok(req.into_reply(200, val)),
        Err((code, val)) => Err(req.into_reply(code, val)),
      }).boxed()
    }).boxed();

    for hook in &self.after {
      let hook = hook.clone();
      reply = reply.and_then(move |reply| hook.handle(reply)).boxed();
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

pub trait Action: Sync + Send {
  fn handle(&self, Req) -> BoxFuture<Reply, Reply>;
}
