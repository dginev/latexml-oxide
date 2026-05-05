use libxml::tree::Node;
use std::fmt;
use std::rc::Rc;

use crate::common::error::Result;
use crate::common::font::Font;
use crate::document::Document;

pub type LigatureClosure = Rc<dyn Fn(&str) -> String>;
pub type FontTestClosure = Rc<dyn Fn(&Font) -> bool>;
pub type LigatureMatcher =
  Rc<dyn Fn(&mut Document, &mut Node) -> Result<Option<(usize, String, MathLigatureOptions)>>>;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MathLigatureOptions {
  pub role:    Option<String>,
  pub name:    Option<String>,
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
  pub id:        usize,
  pub regex:     Option<String>,
  pub code:      Option<LigatureClosure>,
  pub font_test: Option<FontTestClosure>,
  pub matcher:   Option<LigatureMatcher>,
}

impl fmt::Debug for Ligature {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", self.regex) }
}

impl PartialEq for Ligature {
  fn eq(&self, other: &Ligature) -> bool { self.id == other.id }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn math_ligature_options_default_is_none_triple() {
    let m = MathLigatureOptions::default();
    assert!(m.role.is_none());
    assert!(m.name.is_none());
    assert!(m.meaning.is_none());
  }

  #[test]
  fn math_ligature_options_equality() {
    let a = MathLigatureOptions {
      role:    Some("ADDOP".into()),
      name:    None,
      meaning: Some("plus".into()),
    };
    let b = MathLigatureOptions {
      role:    Some("ADDOP".into()),
      name:    None,
      meaning: Some("plus".into()),
    };
    let c = MathLigatureOptions {
      role:    Some("RELOP".into()), // differs
      name:    None,
      meaning: Some("plus".into()),
    };
    assert_eq!(a, b);
    assert_ne!(a, c);
  }

  #[test]
  fn math_ligature_sorted_each_fixed_order() {
    // Perl parity: output always ordered (meaning, name, role).
    let m = MathLigatureOptions {
      role:    Some("r".into()),
      name:    Some("n".into()),
      meaning: Some("m".into()),
    };
    let ordered = m.sorted_each();
    assert_eq!(ordered[0].0, "meaning");
    assert_eq!(ordered[1].0, "name");
    assert_eq!(ordered[2].0, "role");
  }

  #[test]
  fn math_ligature_sorted_each_none_values_preserved() {
    // sorted_each reports None when a field is None.
    let m = MathLigatureOptions::default();
    let ordered = m.sorted_each();
    for (_, v) in ordered {
      assert!(v.is_none());
    }
  }

  #[test]
  fn ligature_default_has_zero_id_and_none_fields() {
    let l = Ligature::default();
    assert_eq!(l.id, 0);
    assert!(l.regex.is_none());
    assert!(l.code.is_none());
    assert!(l.font_test.is_none());
    assert!(l.matcher.is_none());
  }

  #[test]
  fn ligature_equality_by_id_only() {
    let mut a = Ligature::default();
    let mut b = Ligature::default();
    a.id = 1;
    b.id = 1;
    // equal ids compare equal even when other fields (regex, code)
    // differ — Perl parity.
    assert_eq!(a, b);
    b.id = 2;
    assert_ne!(a, b);
  }

  #[test]
  fn ligature_debug_format_uses_regex() {
    // Debug writes just the regex (whatever formatting the Option
    // picks). Verify it doesn't panic and uses the regex field.
    let l = Ligature {
      regex: Some("test_regex".to_string()),
      ..Default::default()
    };
    let out = format!("{l:?}");
    assert!(out.contains("test_regex"), "got {out:?}");
  }
}
