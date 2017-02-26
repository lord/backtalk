use ::JsonValue;

// TODO sender should probably be a custom struct that is willing
// to accept JsonValue as an argument.
// also maybe would be nice to be able to store authentication information somewhere here

pub struct Sender {}

pub trait Channel {
  fn join(&self, Sender); // string is obvs temp value
  fn leave(&self, Sender); // string is obvs temp value
  fn handle(&self, JsonValue);
}