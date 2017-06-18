#[derive(Clone, Debug, PartialEq)]

/// This struct is a little interesting, as we want to pass overrides that partially modify (via a merge) the current font,
/// in each definitional binding. To accommodate that with this struct, every single field needs to be an Option,
/// in order to unambiguously tell the "intend" of override (Some) vs no intent (None).
pub struct Font {
  pub family : Option<String>,
  pub series : Option<String>,
  pub shape : Option<String>,
  pub size : Option<String>,
  pub color : Option<String>,
  pub bg : Option<String>,
  pub opacity : Option<String>,
  pub encoding : Option<String>,
  pub language : Option<String>,
  pub mathstyle : Option<String>,
  pub forceseries: Option<bool>,
  pub forcefamily: Option<bool>,
  pub forceshape: Option<bool>,
}

// Note: forcefamily, forceseries, forceshape (& forcebold for compatibility)
// are only useful for fonts in math; See the specialize method below.
impl Default for Font {
  fn default() -> Self {
    Font {
      family : None,
      series : None,
      shape : None,
      size : None,
      color : None,
      bg : None,
      opacity : None,
      encoding : None,
      language : None,
      mathstyle : None,
      forceseries: None,
      forcefamily: None,
      forceshape: None,
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
  pub fn merge(&self, other: Font) -> Self {
    let mut newfont = self.clone();
    if let Some(value) = other.family {
      newfont.family = Some(value);
    }
    if let Some(value) = other.series {
      newfont.series = Some(value);
    }
    if let Some(value) = other.shape {
      newfont.shape = Some(value);
    }
    if let Some(value) = other.size {
      newfont.size = Some(value);
    }
    if let Some(value) = other.color {
      newfont.color = Some(value);
    }
    if let Some(value) = other.bg {
      newfont.bg = Some(value);
    }
    if let Some(value) = other.opacity {
      newfont.opacity = Some(value);
    }
    if let Some(value) = other.encoding {
      newfont.encoding = Some(value);
    }
    if let Some(value) = other.language {
      newfont.language = Some(value);
    }
    if let Some(value) = other.mathstyle {
      newfont.mathstyle = Some(value);
    }
    if let Some(value) = other.forceseries {
      newfont.forceseries = Some(value);
    }
    if let Some(value) = other.forcefamily {
      newfont.forcefamily = Some(value);
    }
    if let Some(value) = other.forceshape {
      newfont.forceshape = Some(value);
    }
    newfont
  }

  pub fn specialize(&self, _text: &str) -> Self {
    self.clone()
  }

  pub fn to_attribute(&self) -> String {
    let mut serialized = String::new();
    if let Some(ref value) = self.family {
      serialized = serialized + " " + value;
    }
    if let Some(ref value) = self.series {
      if value != "medium" {// TODO: this is a Hack for alltt.tex, ensure this is generalized
        serialized = serialized + " " + value;
      }
    }
    serialized.trim().to_string()
  }

  pub fn distance(&self, other_opt: Option<&Font>) -> i8 {
    if let Some(other) = other_opt {
      let mut distance = 0;
      if self.family != other.family { distance += 1; }
      if self.series != other.series { distance += 1; }
      if self.shape != other.shape { distance += 1; }
      if self.size != other.size { distance += 1; }
      if self.color != other.color { distance += 1; }
      if self.bg != other.bg { distance += 1; }
      if self.opacity != other.opacity { distance += 1; }
      if self.encoding != other.encoding { distance += 1; }
      if self.language != other.language { distance += 1; }
      if self.mathstyle != other.mathstyle { distance += 1; }
      if self.forceseries != other.forceseries { distance += 1; }
      if self.forcefamily != other.forcefamily { distance += 1; }
      if self.forceshape != other.forceshape { distance += 1; }
      distance
    } else { 0 }
  }
}
