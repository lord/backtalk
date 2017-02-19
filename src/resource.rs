use futures::{BoxFuture, Future};
use futures::future::err;
use super::{Adapter, Reply, Req, JsonValue, Method};

pub struct Resource {
  adapter: Box<Adapter>,
}

impl Resource {
  pub fn new<T: Adapter + 'static + Send>(adapt: T) -> Resource {
    Resource {
      adapter: Box::new(adapt),
    }
  }

  pub fn handle(&self, req: Req) -> BoxFuture<Reply, Reply> {
    fn make_err(err_str: &str) -> BoxFuture<Reply, Reply> {
      err(Reply::new(400, None, JsonValue::Array(vec![JsonValue::String("error!".to_string()), JsonValue::String(err_str.to_string())]))).boxed()
    }
    let res = match (req.method(), req.id()) {
      (&Method::List, _) => self.adapter.find(req.params()),
      (&Method::Post, _) => self.adapter.post(req.data(), req.params()),
      (&Method::Get, &Some(ref id)) => self.adapter.get(id, req.params()),
      (&Method::Delete, &Some(ref id)) => self.adapter.delete(id, req.params()),
      (&Method::Patch, &Some(ref id)) => self.adapter.patch(id, req.data(), req.params()),
      (&Method::Action(_), _) => unimplemented!(),
      (_, &None) => return make_err("missing id in request"),
    };

    res.then(|res| match res {
      Ok(val) => Ok(req.into_reply(200, val)),
      Err((code, val)) => Err(req.into_reply(code, val)),
    }).boxed()
  }
}
