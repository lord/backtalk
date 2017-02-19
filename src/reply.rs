use super::{JsonValue, Req};

#[derive(Debug)]
pub struct Reply {
  data: JsonValue,
  code: i64, // TODO replace with enum of errors, etc
  req: Option<Req>,
}

impl Reply {
  // TODO refine this? currently only really should be used internally.
  pub fn new(code: i64, req: Option<Req>, data: JsonValue) -> Reply {
    Reply {
      code: code,
      req: req,
      data: data,
    }
  }

  pub fn to_string(self) -> String {
    self.data.to_string()
  }
}
