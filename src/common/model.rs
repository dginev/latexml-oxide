pub struct Model {
  foo : String
}

impl Default for Model {
  fn default() -> Self {
    Model {
      foo : "default model".to_string()
    }
  }
}

impl Model {
  pub fn load_schema(&mut self) {}
}