#[derive(Debug, Clone, PartialEq)]
pub struct Locator {
  from: String,
  to: String
}

impl Default for Locator {
  fn default() -> Self {
    Locator {
      from: String::new(),
      to: String::new()
    }
  }
}
