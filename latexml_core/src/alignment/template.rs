//! Support for tabular/array environments
use super::cell::Cell;
use crate::Digested;
use crate::common::dimension::Dimension;
use crate::state::Stored;
use crate::token::Token;
use crate::tokens::Tokens;
use rustc_hash::FxHashMap as HashMap;

use std::collections::VecDeque;
use std::fmt::{self, Debug, Display};

// ??
pub type Row = Template;
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Align {
  #[default]
  Left,
  Center,
  Right,
  Justify,
  /// Perl: align => 'char:X' — decimal-aligned column (dcolumn.sty)
  Char(String),
}
impl Align {
  pub fn char_code(&self) -> char {
    match self {
      Align::Right => 'r',
      Align::Left => 'l',
      Align::Center => 'c',
      Align::Justify => 'p',
      Align::Char(_) => 'c', // fallback for sizing
    }
  }
  pub fn name(&self) -> String {
    match self {
      Align::Right => "right".to_string(),
      Align::Left => "left".to_string(),
      Align::Center => "center".to_string(),
      Align::Justify => "justify".to_string(),
      Align::Char(ch) => format!("char:{ch}"),
    }
  }
}
impl From<char> for Align {
  fn from(c: char) -> Align {
    match c {
      'l' => Align::Left,
      'r' => Align::Right,
      'c' => Align::Center,
      'p' => Align::Justify,
      _ => Align::default(), // fallback
    }
  }
}

/// Two axes of tabular orientation
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Axis {
  Column,
  Row,
}
impl Axis {
  /// The string name of a tabular axis
  pub fn name(&self) -> &'static str {
    match self {
      Axis::Column => "column",
      Axis::Row => "row",
    }
  }
  /// Maybe these may have been better named as "horizontal_group" and "vertical_group" in latexml?
  pub fn marker_name(&self) -> &'static str {
    match self {
      Axis::Column => "row",
      Axis::Row => "column",
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ColumnSpec {
  Integer,   // 'i'
  Empty,     // '_'
  Unknown,   // '?'
  Text,      // 't'
  MultiText, // 'tt' — multiple text elements (e.g. colorbox + text)
  Math,      // 'm'
  /// Math *and* Text, alternating
  MathAltText, // 'mx'
  D,         // 'd'
  Graphics,  // 'g'
}
impl ColumnSpec {
  /// The cell comparator.
  pub fn difference_heuristic(&self, other: &ColumnSpec) -> f64 {
    use ColumnSpec::*;
    match self {
      Empty => match other {
        Empty => 0.0,
        Math => 0.05,
        Integer => 0.05,
        Text => 0.05,
        Unknown => 0.05,
        MathAltText => 0.05,
        _ => 0.75,
      },
      Math => match other {
        Empty => 0.05,
        Math => 0.0,
        Integer => 0.1,
        MathAltText => 0.2,
        _ => 0.75,
      },
      Integer => match other {
        Empty => 0.05,
        Math => 0.1,
        Integer => 0.0,
        MathAltText => 0.2,
        _ => 0.75,
      },
      Text => match other {
        Empty => 0.05,
        Text => 0.0,
        MathAltText => 0.2,
        _ => 0.75, // includes MultiText — Perl fallthrough 0.75
      },
      MultiText => match other {
        Empty => 0.05,
        MultiText => 0.0,
        MathAltText => 0.2,
        _ => 0.75, // Perl: "tt" not in diff table → 0.75 fallthrough
      },
      Unknown => match other {
        Empty => 0.05,
        Unknown => 0.0,
        MathAltText => 0.2,
        _ => 0.75,
      },
      MathAltText => match other {
        Empty => 0.05,
        Math => 0.2,
        Integer => 0.2,
        Text => 0.2,
        Unknown => 0.2,
        MathAltText => 0.0,
        _ => 0.75,
      },
      D => match other {
        D => 0.0,
        _ => 0.75,
      },
      Graphics => match other {
        Graphics => 0.0,
        _ => 0.75,
      },
    }
  }
}
impl Display for ColumnSpec {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ColumnSpec::Integer => write!(f, "i"),
      ColumnSpec::Empty => write!(f, "_"),
      ColumnSpec::Unknown => write!(f, "?"),
      ColumnSpec::Text => write!(f, "t"),
      ColumnSpec::MultiText => write!(f, "tt"),
      ColumnSpec::Math => write!(f, "m"),
      ColumnSpec::MathAltText => write!(f, "mx"),
      ColumnSpec::D => write!(f, "d"),
      ColumnSpec::Graphics => write!(f, "g"),
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BorderSpec {
  Top,
  Bottom,
  Left,
  Right,
}

#[derive(Debug, Clone, Default)]
pub struct TemplateConfig {
  pub repeating:     Option<bool>,
  pub pseudorow:     Option<bool>,
  pub non_repeating: usize,
  pub repeated:      Vec<Cell>,
  pub reversion:     Option<Tokens>,
  pub columns:       Option<Vec<Cell>>,
  pub tokens:        Option<Vec<Token>>,
  pub save_before:   Option<VecDeque<Token>>,
  pub save_between:  Option<VecDeque<Token>>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Template {
  repeating:            bool,
  pseudorow:            bool,
  non_repeating:        usize,
  repeated:             Vec<Cell>,
  reversion:            Option<Tokens>,
  columns:              Vec<Cell>,
  pub tokens:           Vec<Token>,
  padding:              Option<Dimension>,
  pub top_padding:      Option<Dimension>,
  pub bottom_padding:   Option<Dimension>,
  pub before:           VecDeque<Digested>,
  pub after:            VecDeque<Digested>,
  save_before:          VecDeque<Token>,
  save_between:         VecDeque<Token>,
  disabled_intercolumn: bool,
  pub cached_width:     Option<Dimension>,
  pub cached_height:    Option<Dimension>,
  pub cached_depth:     Option<Dimension>,
  pub x:                Option<Dimension>,
  pub y:                Option<Dimension>,
  /// Per-row properties (e.g. xml:id, tags) set during digestion
  /// and consumed during construction. Perl: $$row{id}, $$row{tags}.
  /// Uses Stored to preserve typed values (esp. Digested for tags).
  pub properties:       HashMap<String, Stored>,
}

impl Display for Template {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Alignment[]",) }
}
impl Template {
  pub fn new(config: TemplateConfig) -> Self {
    let repeating = config.repeating.unwrap_or(false) || !config.repeated.is_empty();
    let pseudorow = config.pseudorow.unwrap_or(false);
    let mut columns = config.columns.unwrap_or_default();
    let mut repeated = config.repeated;
    let non_repeating = columns.len();
    let save_before = config.save_before.unwrap_or_default();
    let save_between = config.save_between.unwrap_or_default(); // `between` comes before `before`!
    for column in columns.iter_mut() {
      column.empty = true;
    }
    for v in repeated.iter_mut() {
      v.empty = true;
    }

    Template {
      columns,
      pseudorow,
      repeating,
      repeated,
      non_repeating,
      save_before,
      save_between,
      disabled_intercolumn: false,
      before: VecDeque::new(),
      after: VecDeque::new(),
      padding: None,
      top_padding: None,
      bottom_padding: None,
      cached_width: None,
      cached_height: None,
      cached_depth: None,
      x: None,
      y: None,
      reversion: config.reversion,
      tokens: config.tokens.unwrap_or_default(),
      properties: rustc_hash::FxHashMap::default(),
    }
  }
  pub fn set_reversion(&mut self, tks: Tokens) { self.reversion = Some(tks); }
  pub fn set_repeating(&mut self) { self.repeating = true; }
  pub fn set_padding(&mut self, d: Dimension) { self.padding = Some(d); }
  pub fn get_padding(&self) -> Option<&Dimension> { self.padding.as_ref() }

  /// Perl Template.pm L76-80: disableIntercolumn
  /// Only sets the flag when there is a current column.
  /// Perl: `if (my $col = $$self{current_column}) { $$self{disabled_intercolumn} = 1; }`
  pub fn disable_intercolumn(&mut self) {
    if !self.columns.is_empty() || !self.repeated.is_empty() {
      self.disabled_intercolumn = true;
    }
  }

  /// Perl Template.pm L113-118: finish
  /// Appends \lx@intercol to last column's after unless disabled_intercolumn
  pub fn finish(&mut self) {
    let last = if self.repeating {
      self.repeated.last_mut()
    } else {
      self.columns.last_mut()
    };
    if let Some(prev) = last {
      if !self.disabled_intercolumn {
        // `take()` moves out the current Option<Tokens> (replacing with
        // None) — we immediately re-assign, so the clone in the old
        // `.clone().unwrap_or_default().unlist()` was redundant.
        let mut after = prev.after.take().unwrap_or_default().unlist();
        after.push(T_CS!("\\lx@intercol"));
        prev.after = Some(Tokens::new(after));
        prev.has_intercol_after = true;
      }
    }
  }

  // These add material before & after the current column
  pub fn add_before_column(&mut self, mut new: VecDeque<Token>) {
    let current_sb = self.save_before.drain(..);
    new.extend(current_sb);
    self.save_before = new; // NOTE: goes all the way to front!
  }
  // NOTE: \lx@column@trimright should ONLY be added to LaTeX tabular style templates!!!!
  // NOT \halign style templates!
  pub fn add_after_column(&mut self, new: Vec<Token>) {
    if let Some(current_column) = self.columns.last_mut() {
      let current_after = current_column.after.take().unwrap_or_default().unlist();
      current_column.after = Some(Tokens!(T_CS!("\\lx@column@trimright"), new, current_after));
    }
  }

  // Perl Template.pm L65-74: addBetweenColumn
  pub fn add_between_column(&mut self, tokens: Vec<Token>) {
    if let Some(current_column) = self.columns.last_mut() {
      let mut combined = Vec::new();
      let current_after = current_column.after.take().unwrap_or_default().unlist();
      combined.extend(current_after);
      // Perl L69-70: prepend \lx@intercol unless disabled_intercolumn
      if !self.disabled_intercolumn {
        combined.push(T_CS!("\\lx@intercol"));
      }
      combined.extend(tokens);
      current_column.after = Some(Tokens::new(combined));
    } else {
      self.save_between.extend(tokens);
    }
  }

  // Perl Template.pm L82-110: addColumn
  pub fn add_column(&mut self, mut col: Cell) {
    // Perl L85-87: append \lx@intercol to previous column's after unless disabled_intercolumn
    if let Some(prev) = if self.repeating {
      self.repeated.last_mut()
    } else {
      self.columns.last_mut()
    } {
      if !self.disabled_intercolumn {
        let mut after = prev.after.take().unwrap_or_default().unlist();
        after.push(T_CS!("\\lx@intercol"));
        prev.after = Some(Tokens::new(after));
        prev.has_intercol_after = true;
      }
    }
    // Perl L88-95: build before from save_between + \lx@intercol + properties before + save_before
    let mut before = Vec::new();
    if !self.save_between.is_empty() {
      before.extend(self.save_between.clone());
    }
    // Perl L90: push \lx@intercol unless disabled_intercolumn
    let has_intercol_before = !self.disabled_intercolumn;
    if has_intercol_before {
      before.push(T_CS!("\\lx@intercol"));
    }
    // Perl L91: delete disabled_intercolumn
    self.disabled_intercolumn = false;

    if let Some(prop_before) = col.before {
      before.extend(prop_before.unlist());
    }
    if !self.save_before.is_empty() {
      before.extend(self.save_before.clone());
    }
    col.before = if !before.is_empty() {
      Some(Tokens::new(before))
    } else {
      None
    };
    col.has_intercol_before = has_intercol_before;
    let mut after = vec![T_CS!("\\lx@column@trimright")];
    if let Some(prop_after) = col.after {
      after.extend(prop_after.unlist());
    }
    col.after = if after.is_empty() {
      None
    } else {
      Some(Tokens::new(after))
    };
    col.empty = true;
    self.save_between = VecDeque::new();
    self.save_before = VecDeque::new();

    if self.repeating {
      self.non_repeating = self.columns.len();
      self.repeated.push(col);
    } else {
      self.columns.push(col);
    }
  }

  pub fn get_column_mut(&mut self, n: usize) -> Option<&mut Cell> {
    let all_columns = self.columns.len();
    if (n > all_columns) && self.repeating {
      let rep = &self.repeated;
      let m = rep.len();
      if m > 0 {
        for i in all_columns..n {
          let dup = rep[(i - self.non_repeating) % m].clone();
          self.columns.push(dup);
        }
      }
    }
    if n > 0 {
      self.columns.get_mut(n - 1)
    } else {
      None
    }
  }

  pub fn get_columns(&self) -> &[Cell] { &self.columns }
  pub fn get_columns_mut(&mut self) -> &mut Vec<Cell> { &mut self.columns }
  pub fn get_repeated_mut(&mut self) -> &mut Vec<Cell> { &mut self.repeated }
  pub fn set_pseudo(&mut self) { self.pseudorow = true; }
  pub fn unset_pseudo(&mut self) { self.pseudorow = false; }
  pub fn is_pseudo(&self) -> bool { self.pseudorow }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn align_default_is_left() {
    assert_eq!(Align::default(), Align::Left);
  }

  #[test]
  fn align_char_code() {
    assert_eq!(Align::Left.char_code(), 'l');
    assert_eq!(Align::Right.char_code(), 'r');
    assert_eq!(Align::Center.char_code(), 'c');
    assert_eq!(Align::Justify.char_code(), 'p');
    // Char variant falls back to 'c' for sizing.
    assert_eq!(Align::Char(".".to_string()).char_code(), 'c');
  }

  #[test]
  fn align_name() {
    assert_eq!(Align::Left.name(), "left");
    assert_eq!(Align::Right.name(), "right");
    assert_eq!(Align::Center.name(), "center");
    assert_eq!(Align::Justify.name(), "justify");
    assert_eq!(Align::Char(".".to_string()).name(), "char:.");
  }

  #[test]
  fn align_from_char_basic() {
    assert_eq!(Align::from('l'), Align::Left);
    assert_eq!(Align::from('r'), Align::Right);
    assert_eq!(Align::from('c'), Align::Center);
    assert_eq!(Align::from('p'), Align::Justify);
  }

  #[test]
  fn align_from_char_unknown_is_default() {
    // Unknown char falls back to Default (Left).
    assert_eq!(Align::from('x'), Align::Left);
    assert_eq!(Align::from('?'), Align::Left);
  }

  #[test]
  fn axis_name() {
    assert_eq!(Axis::Column.name(), "column");
    assert_eq!(Axis::Row.name(), "row");
  }

  #[test]
  fn axis_marker_name_is_inverse() {
    // marker_name is intentionally "the other axis" — column's
    // marker is a row, row's marker is a column (possibly a naming
    // artifact from the Perl side).
    assert_eq!(Axis::Column.marker_name(), "row");
    assert_eq!(Axis::Row.marker_name(), "column");
  }

  #[test]
  fn column_spec_display_chars() {
    assert_eq!(format!("{}", ColumnSpec::Integer), "i");
    assert_eq!(format!("{}", ColumnSpec::Empty), "_");
    assert_eq!(format!("{}", ColumnSpec::Unknown), "?");
    assert_eq!(format!("{}", ColumnSpec::Text), "t");
    assert_eq!(format!("{}", ColumnSpec::MultiText), "tt");
    assert_eq!(format!("{}", ColumnSpec::Math), "m");
    assert_eq!(format!("{}", ColumnSpec::MathAltText), "mx");
    assert_eq!(format!("{}", ColumnSpec::D), "d");
    assert_eq!(format!("{}", ColumnSpec::Graphics), "g");
  }

  #[test]
  fn column_spec_difference_heuristic_self_is_zero() {
    // Like-to-like distances are 0 for each non-generic variant.
    assert_eq!(
      ColumnSpec::Empty.difference_heuristic(&ColumnSpec::Empty),
      0.0
    );
    assert_eq!(
      ColumnSpec::Math.difference_heuristic(&ColumnSpec::Math),
      0.0
    );
    assert_eq!(
      ColumnSpec::Integer.difference_heuristic(&ColumnSpec::Integer),
      0.0
    );
    assert_eq!(
      ColumnSpec::Text.difference_heuristic(&ColumnSpec::Text),
      0.0
    );
    assert_eq!(ColumnSpec::D.difference_heuristic(&ColumnSpec::D), 0.0);
    assert_eq!(
      ColumnSpec::Graphics.difference_heuristic(&ColumnSpec::Graphics),
      0.0
    );
  }

  #[test]
  fn column_spec_difference_heuristic_incompatible_is_large() {
    // Graphics vs anything else → 0.75 (Perl's "strong difference").
    assert_eq!(
      ColumnSpec::Graphics.difference_heuristic(&ColumnSpec::Math),
      0.75
    );
    assert_eq!(ColumnSpec::D.difference_heuristic(&ColumnSpec::Text), 0.75);
  }
}
