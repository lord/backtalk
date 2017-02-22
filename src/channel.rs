use ws::Sender;
use ::JsonValue;

// TODO sender should probably be a custom struct that is willing
// to accept JsonValue as an argument.
// also maybe would be nice to be able to store authentication information somewhere here

pub trait Channel {
  fn join(&self, Sender);
  fn leave(&self, Sender);
  fn handle(&self, JsonValue);
}