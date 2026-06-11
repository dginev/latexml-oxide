use std::{fmt, fmt::Write as _};

use crate::{
  common::{
    arena::{self, SymStr},
    object::Object,
  },
  util::pathname,
};

// TODO: This will require a large refactor, but
// switching the source from an owned String to a &str reference
// could provide a noticeable performance (and memory allocation) boost
// (and especially if we also start adding locators to tokens)
// my current thoughts are that we can have the core/gullet own the sources of all mouths
// so that we can borrow them with the lifetime of the main convert_document loop...
// that's harder than it sounds, I've already tried unsuccessfully with the Token contents,
// but the mouth sources should be easier to manage.
// definitely something that can be tried after test milestone is achieved.

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Locator {
  pub source:      SymStr,
  pub from_line:   u32,
  pub to_line:     u32,
  pub from_column: u32,
  pub to_column:   u32,
}

impl Default for Locator {
  fn default() -> Self {
    Locator {
      source:      arena::pin(file!()),
      from_line:   line!(),
      to_line:     line!(),
      from_column: column!(),
      to_column:   column!(),
    }
  }
}

// elide Locator debugging until we get to implementing them faithfully
impl fmt::Debug for Locator {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "[...]") }
}

impl fmt::Display for Locator {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.get_short_source(""))?;
    if self.from_line > 0 {
      write!(f, "; line {}", self.from_line)?;
      if self.from_column > 0 {
        write!(f, " col {}", self.from_column)?;
      }
    }
    if self.to_line > 0 {
      write!(f, " - line {}", self.to_line)?;
      if self.to_column > 0 {
        write!(f, " col {}", self.to_column)?;
      }
    }
    Ok(())
  }
}

impl Locator {
  pub fn new<S: AsRef<str>>(
    source: S,
    from_line: u32,
    from_column: u32,
    to_line: u32,
    to_column: u32,
  ) -> Self {
    Locator {
      source: arena::pin(source.as_ref()),
      from_line,
      to_line,
      from_column,
      to_column,
    }
  }

  /// creates a new locator range from a given start and end
  pub fn new_range(from: Locator, to: Locator) -> Option<Locator> {
    // make sure that either parameters are defined
    // bail if we have different sources
    if from.source != to.source {
      return None;
    }
    // the end coordinates depend on
    let (to_line, to_column) = if to.is_range() {
      (to.to_line, to.to_column)
    } else {
      (to.from_line, to.from_column)
    };
    Some(Locator {
      source: from.source,
      from_line: from.from_line,
      from_column: from.from_column,
      to_line,
      to_column,
    })
  }

  pub fn is_range(&self) -> bool { self.to_line > 0 || self.to_column > 0 }

  pub fn get_short_source(&self, string_source: &str) -> String {
    arena::with(self.source, |source| {
      if source.is_empty() {
        if string_source.is_empty() {
          "String".to_string()
        } else {
          string_source.to_string()
        }
      } else if source.contains(':') {
        let (base, ext) = pathname::url_split(source);
        s!("{}.{}", base, ext)
      } else {
        let (_path, base, _ext) = pathname::split(source);
        base
      }
    })
  }
  pub fn get_source(&self) -> SymStr { self.source }

  pub fn get_from_locator(&self) -> Locator {
    Locator {
      source: self.source,
      from_line: self.from_line,
      from_column: self.from_column,
      ..Locator::default()
    }
  }

  pub fn get_to_locator(&self) -> Locator {
    Locator {
      source: self.source,
      from_line: self.to_line,
      from_column: self.to_column,
      ..Locator::default()
    }
  }
}
impl Object for Locator {
  fn stringify(&self) -> String {
    let mut loc = arena::to_string(self.source);
    if loc.is_empty() {
      loc = "Anonymous String".to_string()
    };
    let range_from = if self.is_range() { " from" } else { "" };
    if self.from_line > 0 {
      write!(loc, ";{} line {}", range_from, self.from_line).ok();
      if self.from_column > 0 {
        write!(loc, " col {}", self.from_column).ok();
      }
    }
    if self.to_line > 0 {
      write!(loc, " to line {}", self.to_line).ok();
      if self.to_column > 0 {
        write!(loc, " col {}", self.to_column).ok();
      }
    }
    loc
  }

  /// getting the locator of a locator should return itself
  fn get_locator(&self) -> Option<Locator> { Some(*self) }
}

impl Locator {
  pub fn to_attribute(&self) -> String {
    let mut loc = self.get_short_source("anonymous_string") + "#text";
    if self.is_range() {
      loc.push_str("range(from='");
      // if self.from_line > 0 {
      loc.push_str(&self.from_line.to_string());
      // }
      // if self.from_column > 0 {
      loc.push(';');
      loc.push_str(&self.from_column.to_string());
      // }
      loc.push_str(",to='");
      // if self.to_line > 0 {
      loc.push_str(&self.to_line.to_string());
      // }
      // if self.to_column > 0 {
      loc.push(';');
      loc.push_str(&self.to_column.to_string());
    } else {
      loc.push_str("point('");
      // if self.from_line > 0 {
      loc.push_str(&self.from_line.to_string());
      // }
      // if self.from_column > 0 {
      loc.push(';');
      loc.push_str(&self.from_column.to_string());
    }
    // }
    loc.push_str(")'");
    loc
  }

  /// Serialise as a compact, web-facing `data-sourcepos` value:
  /// `tag:line:col-tag:line:col` for a range, `tag:line:col` for a point.
  ///
  /// This is the source-map feature's serialiser (issues #47/#92) — the
  /// brief, sibling-aligned form documented in `docs/SOURCE_PROVENANCE.md`
  /// §0/§0.1, deliberately *not* the XPointer `to_attribute()` above
  /// (which has zero web-platform support and is latent in the port).
  ///
  /// `tag` is the source's index in the document-level `sources` table
  /// (Source-Map-v3 style) — never an inlined path, so the markup stays
  /// tiny and is anonymisable. The file is first-class in *each* endpoint;
  /// a `Locator` currently carries a single `source` (so both endpoints
  /// share `tag`), but the endpoint-complete form future-proofs a
  /// per-endpoint-source `Locator`.
  pub fn to_sourcepos(&self, tag: u32) -> String {
    if self.is_range() {
      format!(
        "{tag}:{}:{}-{tag}:{}:{}",
        self.from_line, self.from_column, self.to_line, self.to_column
      )
    } else {
      format!("{tag}:{}:{}", self.from_line, self.from_column)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new_builds_from_parts() {
    let l = Locator::new("source.tex", 1, 2, 3, 4);
    assert_eq!(l.from_line, 1);
    assert_eq!(l.from_column, 2);
    assert_eq!(l.to_line, 3);
    assert_eq!(l.to_column, 4);
  }

  #[test]
  fn is_range_false_for_point() {
    // Point locator: to_line=0 AND to_column=0.
    let l = Locator::new("source", 1, 1, 0, 0);
    assert!(!l.is_range());
  }

  #[test]
  fn is_range_true_when_any_to_set() {
    let with_to_line = Locator::new("source", 1, 1, 5, 0);
    let with_to_col = Locator::new("source", 1, 1, 0, 5);
    assert!(with_to_line.is_range());
    assert!(with_to_col.is_range());
  }

  #[test]
  fn new_range_requires_same_source() {
    let a = Locator::new("a.tex", 1, 1, 0, 0);
    let b = Locator::new("b.tex", 5, 5, 0, 0);
    assert!(
      Locator::new_range(a, b).is_none(),
      "different sources must return None"
    );
  }

  #[test]
  fn new_range_takes_from_and_to() {
    let a = Locator::new("same.tex", 1, 2, 0, 0);
    let b = Locator::new("same.tex", 5, 6, 0, 0);
    let r = Locator::new_range(a, b).unwrap();
    assert_eq!(r.from_line, 1);
    assert_eq!(r.from_column, 2);
    // When `to` is a point (not a range), its from_line/col are used
    // as the to_line/col.
    assert_eq!(r.to_line, 5);
    assert_eq!(r.to_column, 6);
  }

  #[test]
  fn new_range_propagates_to_range() {
    // When `to` is itself a range, use its to_line/col (not its from_*)
    let a = Locator::new("same.tex", 1, 1, 0, 0);
    let b = Locator::new("same.tex", 5, 5, 10, 20);
    let r = Locator::new_range(a, b).unwrap();
    assert_eq!(r.from_line, 1);
    assert_eq!(r.to_line, 10);
    assert_eq!(r.to_column, 20);
  }

  #[test]
  fn display_includes_source_and_line() {
    let l = Locator::new("paper.tex", 42, 10, 0, 0);
    let s = format!("{l}");
    assert!(s.contains("paper"), "got {s:?}");
    assert!(s.contains("line 42"), "got {s:?}");
  }

  #[test]
  fn stringify_empty_source_is_anonymous_string() {
    let l = Locator::new("", 1, 1, 0, 0);
    let s = l.stringify();
    assert!(s.contains("Anonymous String"), "got {s:?}");
  }

  #[test]
  fn get_short_source_fallback_when_empty() {
    let l = Locator::new("", 0, 0, 0, 0);
    assert_eq!(l.get_short_source(""), "String");
    assert_eq!(l.get_short_source("inline"), "inline");
  }

  #[test]
  fn get_from_and_to_locators_preserve_source() {
    let l = Locator::new("paper.tex", 1, 2, 3, 4);
    let from = l.get_from_locator();
    let to = l.get_to_locator();
    assert_eq!(from.source, l.source);
    assert_eq!(to.source, l.source);
    // from_locator captures from_*, to_locator captures to_*.
    assert_eq!(from.from_line, 1);
    assert_eq!(from.from_column, 2);
    assert_eq!(to.from_line, 3);
    assert_eq!(to.from_column, 4);
  }

  #[test]
  fn object_get_locator_returns_self() {
    let l = Locator::new("paper.tex", 1, 2, 3, 4);
    let got = l.get_locator();
    assert_eq!(got, Some(l));
  }

  #[test]
  fn to_sourcepos_point() {
    // Point locator (to_line==0 && to_column==0): no `-` separator.
    let l = Locator::new("paper.tex", 12, 1, 0, 0);
    assert_eq!(l.to_sourcepos(0), "0:12:1");
  }

  #[test]
  fn to_sourcepos_range() {
    let l = Locator::new("paper.tex", 12, 1, 12, 240);
    assert_eq!(l.to_sourcepos(0), "0:12:1-0:12:240");
  }

  #[test]
  fn to_sourcepos_tag_is_per_endpoint() {
    // The integer file tag is first-class in *each* endpoint.
    let l = Locator::new("paper.tex", 3, 5, 7, 9);
    assert_eq!(l.to_sourcepos(2), "2:3:5-2:7:9");
  }
}
