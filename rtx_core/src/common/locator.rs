use crate::common::object::Object;
use crate::common::arena;
use crate::util::pathname;
use std::fmt;
use std::fmt::Write as _;
use string_interner::symbol::SymbolU32;

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
  pub source: SymbolU32,
  pub from_line: u32,
  pub to_line: u32,
  pub from_column: u32,
  pub to_column: u32,
}

impl Default for Locator {
  fn default() -> Self {
    Locator {
      source: arena::pin(file!()),
      from_line: line!(),
      to_line: line!(),
      from_column: column!(),
      to_column: column!()
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
      to_column
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
  pub fn get_source(&self) -> SymbolU32 { self.source }

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
  fn get_locator(&self) -> Locator { *self }
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
}
