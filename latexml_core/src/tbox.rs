use libxml::tree::Node;
use std::borrow::Cow;
use std::fmt;
use std::rc::Rc;

use crate::common::arena::SymHashMap as HashMap;
use crate::common::arena::{self, SymStr};
use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::font::Font;
use crate::common::locator::Locator;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::document::Document;
use crate::gullet;
use crate::pin;
use crate::state::{lookup_font, with_value};
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
use crate::{BoxOps, Digested};

/// Box is a Rust keyword, so we use "Tbox" instead, as in "TeX Box"
#[derive(Debug, Clone)]
pub struct Tbox {
  /// plain-text content
  pub text:       SymStr,
  /// associated font for `text`
  pub font:       Rc<Font>,
  /// source location where the box originated
  pub locator:    Locator,
  /// misc properties, such as sizing information
  pub properties: HashMap<Stored>,
  /// a Tokens list containing the TeX that created (or could have) the Tbox.
  pub tokens:     Tokens,
}

impl Default for Tbox {
  fn default() -> Self {
    Tbox {
      text:       pin!(""),
      font:       Rc::new(Font::text_default()),
      locator:    Locator::default(),
      properties: HashMap::default(),
      tokens:     Tokens!(),
    }
  }
}

impl PartialEq for Tbox {
  // Should this compare fonts too?
  fn eq(&self, other: &Self) -> bool { self.text == other.text && *self.font == *other.font }
}

//======================================================================
// Exported constructors
impl fmt::Display for Tbox {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    arena::with(self.text, |text| write!(f, "{}", text))
  }
}
impl Object for Tbox {
  fn get_locator(&self) -> Locator { self.locator }
  fn revert(&self) -> Result<Tokens> { Ok(self.tokens.clone()) }
  fn stringify(&self) -> String { format!("{self:?}") }
}
impl Tbox {
  /// creates a new Tbox.
  /// If `font_opt` or `locator_opt` are None, they are obtained from the
  /// currently active state? Note that `text` can
  /// be empty, which contributes nothing to the generated document,
  /// but does record the TeX code (in the tokens).
  pub fn new(
    text: SymStr,
    font_opt: Option<Rc<Font>>,
    locator_opt: Option<Locator>,
    tokens_opt: Tokens,
    mut properties: HashMap<Stored>,
  ) -> Self {
    let locator = locator_opt.unwrap_or_else(gullet::get_locator);
    let mut font = match font_opt {
      Some(f) => f,
      None => lookup_font().unwrap(),
    };
    let empty_sym = pin!("");
    let tokens = if text != empty_sym && tokens_opt.is_empty() {
      Tokens!(Token { text, code: Catcode::OTHER })
    } else {
      tokens_opt
    };

    // Perl: if ((!defined $properties{isSpace}) && (defined $string) && ($string =~ /^\s*$/)) {
    //         $properties{isSpace} = 1; }
    // Auto-mark all-whitespace text as isSpace (matches Perl Box() behavior)
    if !properties.contains_key("isSpace") && text != empty_sym {
      let is_all_ws = arena::with(text, |s| {
        !s.is_empty() && s.chars().all(|c| c.is_whitespace())
      });
      if is_all_ws {
        properties.insert("isSpace", Stored::Bool(true));
      }
    }

    if properties.contains_key("isSpace")
      && (properties.contains_key("width")
        || properties.contains_key("height")
        || properties.contains_key("depth"))
    {
      properties
        .entry("width")
        .or_insert_with(|| Stored::Dimension(Dimension::default()));
      properties
        .entry("height")
        .or_insert_with(|| Stored::Dimension(Dimension::default()));
      properties
        .entry("depth")
        .or_insert_with(|| Stored::Dimension(Dimension::default()));
    }
    if crate::state::lookup_bool_sym(crate::pin!("IN_MATH")) {
      properties.insert("mode", Stored::String(pin!("math")));
      if text != empty_sym {
        with_value(
          &arena::with(text, |text_str| s!("math_token_attributes_{}", text_str)),
          |value_opt| {
            if let Some(Stored::HashString(attr)) = value_opt {
              for (key, value) in attr.iter() {
                properties
                  .entry(key)
                  .or_insert_with(|| Stored::String(arena::pin(value)));
              }
            }
          },
        );
      }
      font = Rc::new(arena::with(text, |text_str| font.specialize(text_str)));
    }
    Tbox {
      text,
      font,
      locator,
      properties,
      tokens,
    }
  }
  /// checks if the text content is empty
  pub fn is_empty(&self) -> bool {
    // 1. A space-like thing
    // 2. empty (or whitespace) text content
    self.get_property_bool("isEmpty")
      || self.get_property_bool("isSpace")
      || arena::with(self.text, |text| text.trim().is_empty())
  }

  /// Whether this box is in math mode.
  /// Perl: Box.pm::isMath (L79-81): `($mode || 'restricted_horizontal') =~ /math$/`.
  /// Matches "math", "inline_math", "display_math".
  pub fn is_math(&self) -> bool {
    match self.properties.get("mode") {
      Some(Stored::String(s)) => arena::with(*s, |m| m.ends_with("math")),
      _ => false,
    }
  }

  /// Batch-insert properties. Equivalent to calling `set_property` for each
  /// entry, but avoids re-looking up the properties map per call.
  /// Perl: Box.pm::setProperties (L171-175).
  pub fn set_properties<I>(&mut self, entries: I)
  where I: IntoIterator<Item = (&'static str, Stored)> {
    for (key, value) in entries {
      self.properties.insert(key, value);
    }
  }

  /// height + depth of this box (the TeX "total height").
  /// Returns Dimension::default() (0pt) when either component is absent.
  /// Perl: Box.pm::getTotalHeight (L202-210).
  pub fn total_height(&self) -> Dimension {
    let h = match self.properties.get("height") {
      Some(Stored::Dimension(d)) => d.0,
      _ => 0,
    };
    let d = match self.properties.get("depth") {
      Some(Stored::Dimension(d)) => d.0,
      _ => 0,
    };
    Dimension(h + d)
  }
}

impl BoxOps for Tbox {
  fn get_tokens(&self) -> Option<&Tokens> { Some(&self.tokens) }
  fn get_properties(&self) -> &HashMap<Stored> { &self.properties }
  fn get_property(&self, key: &str) -> Option<Cow<'_, Stored>> {
    let props = &self.properties;
    if key == "isSpace" {
      match props.get(key) {
        Some(value) => Some(Cow::Owned(value.clone())),
        None => {
          let tex = self
            .get_tokens()
            .map(|tks| tks.clone().untex())
            .unwrap_or_default(); // !
          if !tex.is_empty() && tex.chars().all(char::is_whitespace) {
            // Check the TeX code, not (just) the string!
            Some(Cow::Owned(Stored::Bool(true)))
          } else {
            None
          }
        },
      }
    } else {
      props.get(key).map(|v| Cow::Owned(v.clone()))
    }
  }
  fn with_properties<R, FnR>(&self, caller: FnR) -> R
  where FnR: FnOnce(&HashMap<Stored>) -> R {
    caller(&self.properties)
  }
  fn get_properties_mut(&mut self) -> &mut HashMap<Stored> { &mut self.properties }
  fn get_string(&self) -> Result<Cow<'_, str>> {
    // TODO: Should we switch these to symbols? are they used often?
    Ok(Cow::Owned(arena::with(self.text, |text| text.to_string())))
  }

  fn be_absorbed(&self, document: &mut Document) -> Result<Vec<Node>> {
    let text = self.get_string()?;
    let font = &self.font;
    let mode = match self.properties.get("mode") {
      Some(Stored::String(s)) => *s,
      _ => pin!("text"),
    };

    if !text.is_empty() {
      // Perl Box::isMath: `mode =~ /math$/` matches "math" / "inline_math" / "display_math".
      let mode_is_math = arena::with(mode, |m| m.ends_with("math"));
      if mode_is_math {
        // Perl: DefMath ?#isMath — in text context, produce plain text.
        // Check if we're inside a math element by walking up the DOM.
        // This handles \And in author frontmatter (text) while preserving
        // \times inside XMText within math.
        let in_math_context = {
          let mut node = document.node.clone();
          let mut found = false;
          loop {
            let qname = crate::document::get_node_qname(&node);
            let hit = arena::with(qname, |s| s.contains("XM") || s.contains("Math"));
            if hit {
              found = true;
              break;
            }
            match node.get_parent() {
              Some(parent) => node = parent,
              None => break,
            }
          }
          found
        };
        if in_math_context {
          Ok(vec![document.insert_math_token(
            &text,
            Stored::cast_to_string_hash(&self.properties),
            Some(font),
          )?])
        } else {
          // Text fallback: produce plain text (Perl's ?#isMath else branch)
          match document.open_text(&text, font)? {
            None => Ok(Vec::new()),
            Some(node) => Ok(vec![node]),
          }
        }
      } else {
        match document.open_text(&text, font)? {
          None => Ok(Vec::new()),
          Some(node) => Ok(vec![node]),
        }
      }
    } else {
      Ok(Vec::new())
    }
  }

  fn get_font(&self) -> Result<Option<Cow<'_, Font>>> { Ok(Some(Cow::Borrowed(&self.font))) }

  fn compute_size(&self, options: HashMap<Stored>) -> Result<(Dimension, Dimension, Dimension)> {
    if let Some(body_stored) = self.get_property("body") {
      if let Stored::Digested(ref body) = *body_stored {
        body.compute_size(options)
      } else {
        panic!("the stored 'body' property should always be a Stored::Digested enum case.");
      }
    } else {
      Ok(self.font.compute_string_size(&self.get_string()?, options))
    }
  }
}

impl From<Tbox> for Result<Vec<Digested>> {
  fn from(tbox: Tbox) -> Result<Vec<Digested>> { Ok(vec![Digested::from(tbox)]) }
}
impl From<Tbox> for Option<Digested> {
  fn from(tbox: Tbox) -> Option<Digested> { Some(Digested::from(tbox)) }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tbox_default_has_empty_text() {
    let t = Tbox::default();
    assert_eq!(arena::to_string(t.text), "");
    assert_eq!(t.properties.len(), 0);
    assert_eq!(t.tokens.len(), 0);
  }

  #[test]
  fn tbox_display_of_default_is_empty() {
    let t = Tbox::default();
    assert_eq!(format!("{t}"), "");
  }

  #[test]
  fn tbox_display_of_text_content() {
    let t = Tbox {
      text: arena::pin("hello"),
      ..Default::default()
    };
    assert_eq!(format!("{t}"), "hello");
  }

  #[test]
  fn tbox_partial_eq_same_text_same_font() {
    let a = Tbox::default();
    let b = Tbox::default();
    assert_eq!(
      a, b,
      "two default Tboxes have same text '' and same text_default font"
    );
  }

  #[test]
  fn tbox_partial_eq_different_text() {
    let a = Tbox::default();
    let b = Tbox {
      text: arena::pin("X"),
      ..Default::default()
    };
    assert_ne!(a, b);
  }

  #[test]
  fn tbox_default_font_is_text_default() {
    let t = Tbox::default();
    // Font::text_default is the Rc backing the default.
    assert_eq!(*t.font, Font::text_default());
  }

  #[test]
  fn tbox_default_locator_is_default() {
    let t = Tbox::default();
    // A Default locator points at the crate source file/line where
    // Default::default was called; just verify it's not nonsense.
    let _ = t.locator;
  }
}
