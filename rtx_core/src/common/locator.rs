use std::borrow::Cow;
use crate::util::pathname;
use crate::common::object::Object;

#[derive(Debug, Clone, PartialEq)]
pub struct Locator {
  source: String,
  from_line: usize,
  to_line: usize,
  from_column: usize,
  to_column: usize
}

impl Default for Locator {
  fn default() -> Self {
    Locator {
      source: String::new(),
      from_line: 0,
      to_line: 0,
      from_column: 0,
      to_column: 0,
    }
  }
}

impl ToString for Locator {
  fn to_string(&self) -> String {
    let mut loc = self.get_short_source("");
    if self.from_line > 0 {
      loc.push_str(&s!("; line {}", self.from_line));
      if self.from_column > 0 {
        loc.push_str(&s!(" col {}",self.from_column));
      }
    }
    if self.to_line > 0 {
      loc.push_str(&s!(" - line {}", self.to_line));
      if self.to_column > 0 {
        loc.push_str(&s!(" col {}", self.to_column));
      }
    }
    loc
  }
}

impl Locator {
  pub fn new(source: String, from_line: usize, from_column: usize, to_line: usize, to_column: usize) -> Self {
    Locator {
      source,
      from_line,
      from_column,
      to_line,
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
    let (to_line,to_column) = if to.is_range() {
      (to.to_line, to.to_column)
    } else {
      (to.from_line, to.from_column)
    };
    Some(Locator::new(from.source, from.from_line, from.from_column, to_line, to_column))
  }

  pub fn is_range(&self) -> bool {
    self.to_line > 0 || self.to_column > 0
  }

  pub fn get_short_source(&self, string_source: &str) -> String {
    if self.source.is_empty() {
      if string_source.is_empty() {
        "String".to_string()
      } else {
        string_source.to_string()
      }
    } else {
      if self.source.contains(":") {
        let (base, ext) = pathname::url_split(&self.source);
        s!("{}.{}",base,ext)
      } else {
        let (path, base, ext) = pathname::split(&self.source);
        s!("{}.{}",base,ext)
      }
    }
  }
  pub fn get_source(&self) -> &str {
    &self.source
  }

  pub fn get_from_locator(&self) -> Locator { 
    Locator {
      source: self.source.clone(),
      from_line: self.from_line,
      from_column: self.from_column,
      .. Locator::default()
    }
  }

  pub fn get_to_locator(&self) -> Locator { 
    Locator {
      source: self.source.clone(),
      from_line: self.to_line,
      from_column: self.to_column,
      ..Locator::default()
    }
  }
}
impl Object for Locator {

  fn stringify(&self) -> String {
    let mut loc = if self.source.is_empty() { "Anonymous String".to_string() } else { self.source.to_string() };
    let range_from = if self.is_range() {" from"} else {""};
    if self.from_line > 0 {
      loc.push_str(&s!(";{} line {}", range_from, self.from_line));
      if self.from_column > 0 {
        loc.push_str(&s!(" col {}", self.from_column));
      }
    }
    if self.to_line > 0 {
      loc.push_str(&s!(" to line {}", self.to_line));
      if self.to_column > 0 {
        loc.push_str(&s!(" col {}", self.to_column));
      }
    }
    loc
  }

  fn to_attribute(&self) -> String {
    let mut loc = self.get_short_source("anonymous_string") + "#text";
    if self.is_range() {
      loc.push_str(&s!("range(from='"));
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
      // }
      loc.push_str(")'");
   } else {
      loc.push_str("point('");
      // if self.from_line > 0 {
      loc.push_str(&self.from_line.to_string());
      // }
      // if self.from_column > 0 {
      loc.push(';');
      loc.push_str(&self.from_column.to_string());
      // }
      loc.push_str(")'");
   }
   loc
  }

  /// getting the locator of a locator should return itself
  fn get_locator(&self) -> Cow<Locator> {
    Cow::Borrowed(self)
  }
}
