//! Support for tabular/array environments
//!
use crate::token::Token;
use crate::tokens::Tokens;

use std::fmt::{self,Display};

#[derive(Debug,Clone,Default)]
pub struct Pattern {
  pub empty: bool,
  pub before: Option<Tokens>,
  pub after: Option<Tokens>,
}
#[derive(Debug,Clone,Default)]
pub struct TemplateConfig {
  pub repeating: Option<bool>,
  pub non_repeating: Option<bool>,
  pub repeated: Vec<Pattern>,
  pub reversion: Option<Tokens>,
  pub columns: Option<Vec<Column>>,
  pub tokens: Option<Vec<Token>>,
  pub save_before: Option<Vec<Tokens>>,
  pub save_between: Option<Vec<Tokens>>
}

#[derive(Debug,Clone,Default)]
pub struct Column {
  empty: bool,
}

#[derive(Debug,Clone,Default)]
pub struct Template {
  repeating: bool,
  non_repeating: bool,
  repeated: Vec<Pattern>,
  reversion: Option<Tokens>,
  columns: Vec<Column>,
  tokens: Vec<Token>,
  save_before: Vec<Tokens>,
  save_between: Vec<Tokens>
}

impl Display for Template {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "Alignment[]",
    )
  }
}
impl Template {
  pub fn new(config: TemplateConfig) -> Self {
    let repeating     = config.repeating.unwrap_or(false) || !config.repeated.is_empty();
    let mut columns = config.columns.unwrap_or_default();
    let mut repeated      = config.repeated;
    let non_repeating = !columns.is_empty();
    let save_before   = config.save_before.unwrap_or_default();
    let save_between  = config.save_between.unwrap_or_default(); // `between` comes before `before`!
    for mut column in columns.iter_mut() {
      column.empty = true;
    }
    for mut v in repeated.iter_mut() {
      v.empty = true;
    }

    Template {
      columns,
      repeating,
      repeated,
      non_repeating,
      save_before,
      save_between,
      reversion: config.reversion,
      tokens: config.tokens.unwrap_or_default(),
    }
  }
  pub fn set_reversion(&mut self, tks: Tokens) {
    self.reversion = Some(tks);
  }
}
