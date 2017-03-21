use {Request, Reply, Error, JsonObject, Method};
use reply::make_streamed_reply;
use futures::Future;
use futures;
use futures::future::ok;
use futures::future::BoxFuture;

type ValueSender = futures::sync::mpsc::UnboundedSender<(String, JsonObject)>;

/**
Sends JSON objects over a realtime stream to a single connected client

You almost certainly will only use a `Sender` inside of a `Channel`.
*/

pub struct Sender {
  inner: ValueSender,
}

// only used internally
pub fn new_sender(chunk_sender: ValueSender) -> Sender {
  Sender {
    inner: chunk_sender,
  }
}

impl Sender {
  /**
  Sends the object `val` to the client connected to this `Sender`.

  `event_type` is some sort of event type, like `"post"` or `"delete"`, but it can be whatever
  you'd like. The `Result` returned is `Ok(())` if the send was successful, and `Err(())` if the
  send was a failure. Note that this function returns instantly; the messages are queued in memory
  before being sent out to the client.
  */
  pub fn send<S: Into<String>>(&mut self, event_type: S, val: JsonObject) -> Result<(), ()> {
    self.inner.send((event_type.into(), val)).map_err(|_| ())
  }
}

/**
Converts a Request into a streaming Reply, and routes incoming messages to outgoing streams.

If you're using a pre-existing `Channel` implementation, you probably just want to use the `handle`
function, which creates the streaming replies. Third-party `Channel`s that implement `join` and
`send` get the `handle` function for free.
*/
pub trait Channel: Send + Sync {
  /**
  Called when a new streamed reply is created in the `handle` function. The `Sender` is the object
  representing the connected client — you can call `sender.send` to send messages to this client.
  The `Option<String>` is the ID in the URL — for instance, a subscription to `/cats` would
  have an ID of `None`, and a subscription to `/cats/123` would have an ID of `Some("123")`. The
  `JsonObject` is the params object of the request, and can be used for authenticating users.
  */
  fn join(&self, Sender, Option<String>, JsonObject);

  /**
  Called by application code to send a new message to connected clients. Channel implementors are
  also free to add additional functions that send messages with additional paramaters, such as
  specifying which users
  get notified.
  */
  fn send(&self, &str, &JsonObject);

  /**
  Takes a `Request` and returns a `Reply` future with a streaming `Reply` body. If you're using a
  channel in your server's application code, this is the function you'll want to use.
  */
  fn handle(&self, req: Request) -> BoxFuture<Reply, Error> {
    if req.method().clone() != Method::Listen {
      return Error::server_error("passed a non-listen request to channel")
    }
    let params = req.params().clone();
    let id = req.id().clone();
    let (sender, reply) = make_streamed_reply(req);
    self.join(sender, id, params);
    ok(reply).boxed()
  }
}
