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

  pub fn text_default() -> Self { // TODO
    Font::default()
  }

  pub fn math_default() -> Self { // TODO
    Font::default()
  }

  // NOTE: In math, NORMALLY, setting any one of
  //    family, series or shape
  // will, usually, automatically reset the others to thier defaults!
  // You must arrange this in the calls....
  pub fn merge(&self, kv: HashMap<String, String>) -> Self {
    let mut newfont = self.clone();
    for (key, value) in kv {
      match key.as_str() {
        "family" => {newfont.family = value},
        "series" => {newfont.series = value},
        "shape" => {newfont.shape = value},
        "size" => {newfont.size = value},
        "color" => {newfont.color = value},
        "bg" => {newfont.bg = value},
        "encoding" => {newfont.encoding = value},
        "language" => {newfont.language = value},
        "mathstyle" => {newfont.mathstyle = value},
        _ => {}
      }
    }
    newfont
  }

  pub fn specialize(&self, text: &str) -> Self {
    self.clone()
  }
}
