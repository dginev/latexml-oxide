use crate::common::font::Font;
use std::fmt;
use std::rc::Rc;

pub type LigatureClosure = Rc<dyn Fn(&str) -> String>;
pub type FontTestClosure = Rc<dyn Fn(&Font) -> bool>;

#[derive(Clone)]
pub struct Ligature {
  pub regex: String,
  pub code: LigatureClosure,
  pub font_test: Option<FontTestClosure>,
}

impl fmt::Debug for Ligature {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", self.regex) }
}

impl PartialEq for Ligature {
  fn eq(&self, other: &Ligature) -> bool { self.regex == other.regex }
}
