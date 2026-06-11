use std::{borrow::Cow, fmt};

use libxml::tree::Node;

use crate::{
  BoxOps, NO_PROPERTIES,
  common::{
    arena::SymHashMap as HashMap, dimension::Dimension, error::*, font::Font,
    numeric_ops::NumericOps, object::Object, store::Stored,
  },
  definition::register::RegisterValue,
  document::Document,
  tokens::{NO_TOKENS, Tokens},
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Comment(pub String);

impl fmt::Display for Comment {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "") }
}
impl Object for Comment {
  fn revert(&self) -> Result<Tokens> { Ok(NO_TOKENS) }
}
impl BoxOps for Comment {
  fn get_properties(&self) -> &HashMap<Stored> { &NO_PROPERTIES }
  fn with_properties<R, FnR>(&self, caller: FnR) -> R
  where FnR: FnOnce(&HashMap<Stored>) -> R {
    caller(&NO_PROPERTIES)
  }
  fn get_property(&self, key: &str) -> Option<Cow<'_, Stored>> {
    // Perl: Comment->getProperty('isEmpty') returns 1
    if key == "isEmpty" {
      Some(Cow::Owned(Stored::Bool(true)))
    } else {
      None
    }
  }
  fn set_property<T: Into<Stored>>(&mut self, _key: &str, _value: T) {} // no-op
  fn get_string(&self) -> Result<Cow<'_, str>> { Ok(Cow::Borrowed(&self.0)) }
  fn be_absorbed(&self, document: &mut Document) -> Result<Vec<Node>> {
    document.insert_comment(&self.0)?;
    Ok(Vec::new())
  }
  fn get_font(&self) -> Result<Option<Cow<'_, Font>>> { Ok(None) }
  fn get_width(&self, _options: Option<HashMap<Stored>>) -> Result<Option<RegisterValue>> {
    Ok(Some(RegisterValue::Dimension(Dimension::new(0))))
  }

  fn compute_size(&self, _options: HashMap<Stored>) -> Result<(Dimension, Dimension, Dimension)> {
    Ok((
      Dimension::default(),
      Dimension::default(),
      Dimension::default(),
    ))
  }

  // sub getHeight      { return Dimension(0); }
  // sub getTotalHeight { return Dimension(0); }
  // sub getDepth       { return Dimension(0); }
  // sub getSize { return (Dimension(0), Dimension(0), Dimension(0), Dimension(0), Dimension(0),
  // Dimension(0)); }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn comment_default_is_empty_string() {
    let c = Comment::default();
    assert_eq!(c.0, "");
  }

  #[test]
  fn comment_new_holds_content() {
    let c = Comment("% this is a comment".to_string());
    assert_eq!(c.0, "% this is a comment");
  }

  #[test]
  fn comment_display_is_always_empty_string() {
    // Display of a Comment is always "" (comments produce no visible
    // output).
    let c = Comment("visible".to_string());
    assert_eq!(format!("{c}"), "");
    let c2 = Comment::default();
    assert_eq!(format!("{c2}"), "");
  }

  #[test]
  fn comment_revert_yields_empty_tokens() {
    let c = Comment("any".to_string());
    let t = c.revert().unwrap();
    assert_eq!(t.len(), 0);
  }

  #[test]
  fn comment_is_empty_property() {
    // Perl: Comment->getProperty('isEmpty') returns 1.
    let c = Comment("anything".to_string());
    match c.get_property("isEmpty") {
      Some(Cow::Owned(Stored::Bool(true))) => {},
      other => panic!("expected Some(Bool(true)), got {other:?}"),
    }
    // Other keys return None.
    assert!(c.get_property("random_key").is_none());
  }

  #[test]
  fn comment_get_string_returns_content() {
    let c = Comment("hello".to_string());
    let s = c.get_string().unwrap();
    assert_eq!(s.as_ref(), "hello");
  }

  #[test]
  fn comment_equality() {
    let a = Comment("x".to_string());
    let b = Comment("x".to_string());
    let c = Comment("y".to_string());
    assert_eq!(a, b);
    assert_ne!(a, c);
  }

  #[test]
  fn comment_get_font_is_none() {
    let c = Comment::default();
    assert!(c.get_font().unwrap().is_none());
  }

  #[test]
  fn comment_get_width_is_zero() {
    let c = Comment::default();
    let w = c.get_width(None).unwrap();
    match w {
      Some(RegisterValue::Dimension(d)) => assert_eq!(d.value_of(), 0),
      other => panic!("expected Dimension(0), got {other:?}"),
    }
  }

  #[test]
  fn comment_compute_size_all_zero() {
    let c = Comment::default();
    let (w, h, d) = c.compute_size(HashMap::default()).unwrap();
    assert_eq!(w.value_of(), 0);
    assert_eq!(h.value_of(), 0);
    assert_eq!(d.value_of(), 0);
  }

  #[test]
  fn comment_set_property_noop() {
    let mut c = Comment("x".to_string());
    c.set_property("any", Stored::Bool(true));
    // Properties are always NO_PROPERTIES; nothing persists.
    assert_eq!(c.get_properties().len(), 0);
  }
}
