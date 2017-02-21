use ws::Sender;
use ::JsonValue;

// TODO sender should probably be a custom struct that is willing
// to accept JsonValue as an argument.

pub trait Channel {
  fn join(Sender);
  fn leave(Sender);
  fn handle(JsonValue);
}