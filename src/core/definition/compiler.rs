use regex::{Captures,Regex};
use std::sync::Arc;
use core::definition::Definition;
use libxml::tree::Node;

use core::definition::constructor::{Constructor, ReplacementClosure};

lazy_static! {
  static ref VALUE_RE : Regex = Regex::new(r"(\\#|\\&[\\w\\:]*\\()").unwrap();
  static ref COND_RE : Regex = Regex::new(r"\\?(\\#|\\&[\\w\\:]*\\()").unwrap();
  // Attempt to follow XML Spec, Appendix B
  static ref QNAME_RE : Regex = Regex::new(r"((?:\\p{Ll}|\\p{Lu}|\\p{Lo}|\\p{Lt}|\\p{Nl}|_|:)(?:\\p{Ll}|\\p{Lu}|\\p{Lo}|\\p{Lt}|\\p{Nl}|_|:|\\p{M}|\\p{Lm}|\\p{Nd}|\\.|\\-)*)").unwrap();
  static ref TEXT_RE : Regex = Regex::new(r"(.[^\\#<\\?\\)\\&\\,]*)").unwrap();
  static ref NONW_RE : Regex = Regex::new(r"\W").unwrap();
  static ref FLOAT_RE : Regex = Regex::new(r"^(\^+)\s*").unwrap();
}

#[macro_export]
macro_rules! TranslateConstructor(
  ($replacement:expr, $floats:expr) => (
  {

  }
));

impl Constructor {
  pub fn compile_replacement(&self) -> Option<ReplacementClosure> {
    if self.replacement.is_empty() {
      return None
    }
    let cs = self.get_cs();
    let name = NONW_RE.replace_all(&self.get_cs_name(), "");
    let nargs = self.get_num_args();

    let mut floats : Option<String> = None;
    let replacement = FLOAT_RE.replace(&self.replacement, |caps: &Captures| {
      floats = match caps.at(1) { // Grab float marker.
        None => None,
        Some(subs) => Some(subs.to_owned())
      };
      String::new()
    });

    println_stderr!("-- Preparing translation closure for: \n{:?}\n", replacement);
    Some(Arc::new(Box::new(|document, args, props, state| {
      let mut savenode : Option<Node> = None;
      TranslateConstructor!(replacement, floats);
      match savenode {
        None => {},
        Some(savenode) => document.set_node(savenode)
      };
      return;
    })))
  }
}
