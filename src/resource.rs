use futures::{BoxFuture, Future};
use futures::future::{ok, err};
use std::sync::Arc;
use std::collections::HashMap;
use super::{Adapter, Reply, Req, JsonValue, Method, Channel};

pub struct Resource {
  adapter: Arc<Box<Adapter>>,
  before: Vec<Arc<Box<Guard>>>,
  after: Vec<Arc<Box<Filter>>>,
  actions: Arc<HashMap<String, Box<Action>>>,
  channel: Option<Arc<Box<Channel>>>,
}

impl Resource {
  pub fn new<T: Adapter + 'static>(adapt: T) -> Resource {
    Resource {
      adapter: Arc::new(Box::new(adapt)),
      before: Vec::new(),
      after: Vec::new(),
      actions: Arc::new(HashMap::new()),
      channel: None,
    }
  }

  pub fn channel<T: Channel + 'static>(&mut self, chan: T) {
    self.channel = Some(Arc::new(Box::new(chan)));
  }

  pub fn before<T: Guard + 'static>(&mut self, hook: T) {
    self.before.push(Arc::new(Box::new(hook)));
  }

  pub fn after<T: Filter + 'static>(&mut self, hook: T) {
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
    let channel = self.channel.clone();
    let channel2 = self.channel.clone();

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
        (Method::Listen, id_opt) => {
          return match channel {
            None => make_err("no channel installed"),
            Some(chan) => {
              let (sender, reply) = Reply::new_streamed(200, Some(req));
              chan.join(sender);
              ok(reply).boxed()
            }
          }
        },
        (_, None) => return make_err("missing id in request"),
      };
      res.then(move |res| match res {
        Ok(val) => {
          match (req.method(), channel2) {
            (&Method::Post, Some(ref chan)) => chan.handle(Method::Post, val.clone()),
            (&Method::Patch, Some(ref chan)) => chan.handle(Method::Patch, val.clone()),
            (&Method::Delete, Some(ref chan)) => chan.handle(Method::Delete, val.clone()),
            (&Method::Action(ref action), Some(ref chan)) => chan.handle(Method::Action(action.to_string()), val.clone()),
            _ => (),
          }
          Ok(req.into_reply(200, val))
        },
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
pub trait Guard: Sync + Send {
  fn handle(&self, Req) -> BoxFuture<Req, Reply>;
}

pub trait Filter: Sync + Send {
  fn handle(&self, Reply) -> BoxFuture<Reply, Reply>;
}

pub trait Action: Sync + Send {
  fn handle(&self, Req) -> BoxFuture<Reply, Reply>;
}
