//! Support for tabular/array environments
use crate::token::Token;
use crate::tokens::Tokens;
use crate::Digested;
use crate::common::dimension::Dimension;

use std::collections::VecDeque;
use std::fmt::{self, Display, Debug};
use libxml::tree::Node;

// ??
pub type Row = Template;
#[derive(Debug,Copy,Clone,Default, PartialEq)]
pub enum Align {
  #[default]
  Left,
  Center,
  Right,
  Justify
}
impl Align {
  pub fn char_code(&self) -> char {
    match self {
      Align::Right => 'r',
      Align::Left => 'l',
      Align::Center => 'c',
      Align::Justify => 'p'
    }
  }
}

/// Two axes of tabular orientation
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum Axis {
  Column,
  Row
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

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Column {
  pub empty: bool,
  pub omitted: bool,
  pub skipped: bool,
  pub before: Option<Tokens>,
  pub after: Option<Tokens>,
  pub align: Option<Align>,
  pub width: Option<Dimension>,
  pub height: Option<Dimension>,
  pub depth: Option<Dimension>,
  pub colspan: Option<usize>,
  pub boxes: Option<Digested>,
  pub cell_type: Option<char>,
  pub content_class: Option<ColumnSpec>,
  pub content_length: Option<usize>,
  pub border: String,
  pub border_left: Option<usize>,
  pub border_right: Option<usize>,
  pub border_top: Option<usize>,
  pub border_bottom: Option<usize>,
  pub cell: Option<Node>,
  pub thead_in_row: bool,
  pub thead_in_column: bool,
}
impl Column {
  pub fn border_at(&self, side: BorderSpec) -> Option<usize> {
    match side {
      BorderSpec::Left => self.border_left,
      BorderSpec::Right => self.border_right,
      BorderSpec::Top => self.border_top,
      BorderSpec::Bottom => self.border_bottom,
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ColumnSpec {
  Integer, // 'i'
  Empty, // '_'
  Unknown, // '?'
  Text, // 't'
  Math, // 'm'
  /// Math *and* Text, alternating
  MathAltText, // 'mx'
  D, // 'd'
  Graphics, // 'g'
}
impl ColumnSpec {
  /// The cell comparator.
  pub fn difference_heuristic(&self, other: &ColumnSpec) -> f64 {
    use ColumnSpec::*;
    match self {
      Empty => match other {
        Empty => 0.0, Math  => 0.05, Integer => 0.05, Text => 0.05, Unknown => 0.05, MathAltText => 0.05, _ => 0.75 },
      Math   => match other {
        Empty => 0.05, Math => 0.0, Integer => 0.1, MathAltText => 0.2, _ => 0.75 },
      Integer   => match other {
        Empty => 0.05,Math  => 0.1,  Integer  => 0.0, MathAltText => 0.2, _ => 0.75 },
      Text => match other {
        Empty => 0.05, Text => 0.0, MathAltText => 0.2, _ => 0.75 },
      Unknown => match other {
        Empty => 0.05, Unknown => 0.0,  MathAltText => 0.2, _ => 0.75 },
      MathAltText => match other {
        Empty => 0.05,Math  => 0.2,  Integer  => 0.2, Text => 0.2, Unknown => 0.2, MathAltText => 0.0, _ => 0.75 }
      D => match other {
        D => 0.0, _ => 0.75 },
      Graphics => match other {
        Graphics => 0.0, _ => 0.75 }
    }
  }
}
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum BorderSpec {
  Top,
  Bottom,
  Left,
  Right
}

#[derive(Debug, Clone, Default)]
pub struct TemplateConfig {
  pub repeating: Option<bool>,
  pub pseudorow: Option<bool>,
  pub non_repeating: usize,
  pub repeated: Vec<Column>,
  pub reversion: Option<Tokens>,
  pub columns: Option<Vec<Column>>,
  pub tokens: Option<Vec<Token>>,
  pub save_before: Option<VecDeque<Token>>,
  pub save_between: Option<VecDeque<Token>>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Template {
  repeating: bool,
  pseudorow: bool,
  non_repeating: usize,
  repeated: Vec<Column>,
  reversion: Option<Tokens>,
  columns: Vec<Column>,
  current_column: Option<Column>,
  pub tokens: Vec<Token>,
  padding: Option<Dimension>,
  pub top_padding: Option<Dimension>,
  pub bottom_padding: Option<Dimension>,
  pub before: VecDeque<Digested>,
  pub after: VecDeque<Digested>,
  save_before: VecDeque<Token>,
  save_between: VecDeque<Token>,
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
      before: VecDeque::new(),
      after: VecDeque::new(),
      current_column: None,
      padding: None,
      top_padding: None,
      bottom_padding: None,
      reversion: config.reversion,
      tokens: config.tokens.unwrap_or_default(),
    }
  }
  pub fn set_reversion(&mut self, tks: Tokens) { self.reversion = Some(tks); }
  pub fn set_repeating(&mut self) { self.repeating = true; }
  pub fn set_padding(&mut self, d: Dimension) { self.padding = Some(d); }
  pub fn get_padding(&self) -> Option<&Dimension> { self.padding.as_ref() }
  // These add material before & after the current column
  pub fn add_before_column(&mut self, mut new: VecDeque<Token>) {
    let current_sb = self.save_before.drain(..);
    new.extend(current_sb);
    self.save_before = new; // NOTE: goes all the way to front!
  }
  // NOTE: \@@eat@space should ONLY be added to LaTeX tabular style templates!!!!
  // NOT \halign style templates!
  pub fn add_after_column(&mut self, new: Vec<Token>) {
    if let Some(current_column) = &mut self.current_column {
      let current_after = current_column.after.take().unwrap_or_default().unlist();
      current_column.after = Some(Tokens!(T_CS!("\\@@eat@space"), new, current_after));
    }
  }

  // Or between this column & next...
  pub fn add_between_column(&mut self, tokens: Vec<Token>) {
    if let Some(current_column) = &mut self.current_column {
      let current_after = current_column.after.take().unwrap_or_default().unlist();
      current_column.after = Some(Tokens!(current_after, tokens));
    } else {
      self.save_between.extend(tokens);
    }
  }

  pub fn add_column(&mut self, mut col:Column) {
    let mut before = Vec::new();
    if !self.save_between.is_empty() {
      before.extend(self.save_between.clone());
    }
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
    let mut after = vec![T_CS!("\\@@eat@space")];
    if let Some(prop_after) = col.after {
      after.extend(prop_after.unlist());
    }
    col.after = if after.is_empty() {
      None
    } else {
      Some(Tokens::new(after))
    };
    col.empty           = true;
    self.save_between   = VecDeque::new();
    self.save_before    = VecDeque::new();
    self.current_column = Some(col.clone());

    if self.repeating {
      self.non_repeating = self.columns.len();
      self.repeated.push(col);
    } else {
      self.columns.push(col);
    }
  }

  pub fn get_column_mut(&mut self, n: usize) -> Option<&mut Column> {
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

  pub fn get_columns(&self) -> &[Column] { &self.columns }
  pub fn get_columns_mut(&mut self) -> &mut Vec<Column> { &mut self.columns }
  pub fn set_pseudo(&mut self) { self.pseudorow = true; }
  pub fn unset_pseudo(&mut self) { self.pseudorow = false; }
  pub fn is_pseudo(&self) -> bool { self.pseudorow }
}
