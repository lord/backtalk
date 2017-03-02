use futures::{BoxFuture, Future};
use futures::future::{ok, err};
use std::sync::Arc;
use std::collections::HashMap;
use super::{Adapter, Reply, Req, JsonValue, Method, Channel};

pub struct Resource {
  adapter: Arc<Box<Adapter>>,
  actions: Arc<HashMap<String, Box<Action>>>,
  channel: Option<Arc<Box<Channel>>>,
}

impl Resource {
  pub fn new<T: Adapter + 'static>(adapt: T) -> Resource {
    Resource {
      adapter: Arc::new(Box::new(adapt)),
      actions: Arc::new(HashMap::new()),
      channel: None,
    }
  }

  pub fn channel<T: Channel + 'static>(&mut self, chan: T) {
    self.channel = Some(Arc::new(Box::new(chan)));
  }

  pub fn action<T: Action + 'static, S: Into<String>>(&mut self, name: S, action: T) {
    Arc::get_mut(&mut self.actions).unwrap().insert(name.into(), Box::new(action));
  }

  pub fn handle(&self, req: Req) -> BoxFuture<Reply, Reply> {
    fn make_err(err_str: &str) -> BoxFuture<Reply, Reply> {
      err(Reply::new(400, None, JsonValue::Array(vec![JsonValue::String("error!".to_string()), JsonValue::String(err_str.to_string())]))).boxed()
    }

    match req.method().clone() {
      Method::Action(ref action_name) => {
        // return match actions.get(action_name) {
        //   Some(action) => action.handle(req),
        //   None => make_err("action not found"),
        // }
        unimplemented!();
      },
      Method::Listen => match &self.channel {
        &Some(ref chan) => chan.handle(req),
        &None => unimplemented!(),
      },
      _ => self.adapter.handle(req),
    }
  }
}

pub trait Action: Sync + Send {
  fn handle(&self, Req) -> BoxFuture<Reply, Reply>;
}
