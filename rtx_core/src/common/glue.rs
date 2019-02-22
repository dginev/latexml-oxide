use lazy_static::lazy_static;
use regex::Regex;

use crate::definition::register::NumericOps;

fn fillcode(ftype: &str) -> u8 {
  match ftype {
    "fil" => 1,
    "fill" => 2,
    "filll" => 3,
    _ => 0,
  }
}

static FILL: &[&str] = &["", "fil", "fill", "filll"];

lazy_static! {
  static ref NUM_RE: Regex = Regex::new(r"\d*\.?\d*").unwrap();
  static ref UNIT_RE: Regex = Regex::new(r"[a-zA-Z][a-zA-Z]").unwrap();
  static ref FILL_RE: Regex = Regex::new(r"fil|fill|filll|[a-zA-Z][a-zA-Z]").unwrap();
  static ref PLUS_RE: Regex = Regex::new(r"\s+plus\s*($1)($fill_re)").unwrap();
  static ref MINUS_RE: Regex = Regex::new(r"\s+minus\s*($num_re)($fill_re)").unwrap();
  static ref GLUE_RE: Regex = Regex::new(r"(\+?\-?$num_re)($unit_re)($plus_re)?($minus_re)?").unwrap();
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Glue {
  skip: f32,
  plus: Option<f32>,
  pfill: Option<f32>,
  minus: Option<f32>,
  mfill: Option<f32>,
}

impl Default for Glue {
  fn default() -> Self {
    Glue {
      skip: 0.0,
      plus: None,
      pfill: None,
      minus: None,
      mfill: None,
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MuGlue(pub f32);

impl Default for MuGlue {
  fn default() -> Self { MuGlue(0.0) }
}

impl NumericOps for Glue {
  fn value_of(self) -> f32 { self.skip }
  fn new<T: Into<f32>>(number: T) -> Self {
    Glue {
      skip: number.into(),
      ..Glue::default()
    }
  }
}

impl NumericOps for MuGlue {
  fn new<T: Into<f32>>(number: T) -> Self { MuGlue(number.into()) }
  fn value_of(self) -> f32 { self.0 }
}

impl Glue {
  pub fn spec_new(skip: f32, plus: Option<f32>, pfill: Option<f32>, minus: Option<f32>, mfill: Option<f32>) -> Self {
    Glue {
      skip,
      plus,
      pfill,
      minus,
      mfill,
    }
  }
}
