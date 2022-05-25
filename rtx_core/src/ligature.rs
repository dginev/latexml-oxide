use libxml::tree::Node;
use std::fmt;
use std::sync::Arc;

use crate::common::error::Result;
use crate::common::font::Font;
use crate::document::Document;
use crate::state::State;

pub type LigatureClosure = Arc<dyn Fn(&str) -> String>;
pub type FontTestClosure = Arc<dyn Fn(&Font) -> bool>;
pub type LigatureMatcher = Arc<dyn Fn(&mut Document, &mut Node, &State) -> Result<Option<(usize, String, MathLigatureOptions)>>>;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MathLigatureOptions {
  pub role: Option<String>,
  pub name: Option<String>,
  pub meaning: Option<String>,
}

impl MathLigatureOptions {
  pub fn sorted_each(&self) -> Vec<(&str, &Option<String>)> { vec![("meaning", &self.meaning), ("name", &self.name), ("role", &self.role)] }
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
