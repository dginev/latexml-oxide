use libxml::tree::Node;
use std::fmt;
use std::rc::Rc;

use crate::common::error::Result;
use crate::common::font::Font;
use crate::document::Document;

pub type LigatureClosure = Rc<dyn Fn(&str) -> String>;
pub type FontTestClosure = Rc<dyn Fn(&Font) -> bool>;
pub type LigatureMatcher = Rc<
  dyn Fn(&mut Document, &mut Node) -> Result<Option<(usize, String, MathLigatureOptions)>>,
>;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MathLigatureOptions {
  pub role: Option<String>,
  pub name: Option<String>,
  pub meaning: Option<String>,
}

impl MathLigatureOptions {
  pub fn sorted_each(&self) -> [(&str, Option<&String>); 3] {
    [
      ("meaning", self.meaning.as_ref()),
      ("name", self.name.as_ref()),
      ("role", self.role.as_ref()),
    ]
  }
}

#[derive(Clone, Default)]
pub struct Ligature {
  pub id: usize,
  pub regex: Option<String>,
  pub code: Option<LigatureClosure>,
  pub font_test: Option<FontTestClosure>,
  pub matcher: Option<LigatureMatcher>,
}

impl fmt::Debug for Ligature {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", self.regex) }
}

impl PartialEq for Ligature {
  fn eq(&self, other: &Ligature) -> bool { self.id == other.id }
}
