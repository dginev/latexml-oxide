use std::collections::HashMap;
#[derive(Clone, Debug, PartialEq)]
pub struct Font {
  pub family : String,
  pub series : String,
  pub shape : String,
  pub size : String,
  pub color : String,
  pub bg : String,
  pub opacity : String,
  pub encoding : String,
  pub language : String,
  pub mathstyle : String,
  pub forceseries: bool,
  pub forcefamily: bool,
  pub forceshape: bool,
}

// Note: forcefamily, forceseries, forceshape (& forcebold for compatibility)
// are only useful for fonts in math; See the specialize method below.
impl Default for Font {
  fn default() -> Self {
    Font {
      family : String::new(),
      series : String::new(),
      shape : String::new(),
      size : String::new(),
      color : String::new(),
      bg : String::new(),
      opacity : String::new(),
      encoding : String::new(),
      language : String::new(),
      mathstyle : String::new(),
      forceseries: false,
      forcefamily: false,
      forceshape: false,
    }
  }
}

impl Font {
  pub fn merge(&mut self, kv: HashMap<String, String>) {
    for (key, value) in kv {
      match key.as_str() {
        "family" => {self.family = value},
        "series" => {self.series = value},
        "shape" => {self.shape = value},
        "size" => {self.size = value},
        "color" => {self.color = value},
        "bg" => {self.bg = value},
        "encoding" => {self.encoding = value},
        "language" => {self.language = value},
        "mathstyle" => {self.mathstyle = value},
        _ => {}
      }
    }
  }
}