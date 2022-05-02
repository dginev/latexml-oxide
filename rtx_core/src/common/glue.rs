use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::Ordering;
use std::fmt;

use crate::common::dimension::Dimension;
use crate::definition::register::{NumericOps, RegisterType};
use crate::state::State;

/// Positively silly enum, but it solves all kinds of issues with the Glue struct
/// most importantly allows us to keep deriving the Copy trait, and avoids storing
/// strings in Glue objects
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub enum FillCode {
  Fil,
  Fill,
  Filll,
}

impl fmt::Display for FillCode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      FillCode::Fil => write!(f, "fil"),
      FillCode::Fill => write!(f, "fill"),
      FillCode::Filll => write!(f, "filll"),
    }
  }
}

impl FillCode {
  pub fn new(index: usize) -> Option<FillCode> {
    match index {
      1 => Some(FillCode::Fil),
      2 => Some(FillCode::Fill),
      3 => Some(FillCode::Filll),
      _ => None,
    }
  }
  pub fn from(ftype: &str) -> Option<FillCode> {
    match ftype {
      "fil" => Some(FillCode::Fil),
      "fill" => Some(FillCode::Fill),
      "filll" => Some(FillCode::Filll),
      _ => None,
    }
  }
}

// Note: Regexes are not first-level objects in Rust, and neither are Strings
//       yet we would like to have some efficient
macro_rules! num_re_str {
  () => {
    r"\d*\.?\d*"
  };
}
macro_rules! unit_re_str {
  () => {
    r"[a-zA-Z][a-zA-Z]"
  };
}
macro_rules! fill_re_str {
  () => {
    r"fil|fill|filll|[a-zA-Z][a-zA-Z]"
  };
}

static NUM_EXACT_STR: &str = concat!(r"^", num_re_str!(), r"$");

macro_rules! plus_re_str {
  () => {
    concat!(r"\s+plus\s*($1)(", fill_re_str!(), r")")
  };
}
macro_rules! minus_re_str {
  () => {
    concat!(r"\s+minus\s*(", num_re_str!(), r")(", fill_re_str!(), r")")
  };
}

static GLUE_RE_STR: &str = concat!(
  r"^(\+?\-?",
  num_re_str!(),
  r")(",
  unit_re_str!(),
  r")(",
  plus_re_str!(),
  r")?(",
  minus_re_str!(),
  r")?$"
);

lazy_static! {
  static ref NUM_RE: Regex = Regex::new(num_re_str!()).unwrap();
  static ref NUM_EXACT_RE: Regex = Regex::new(NUM_EXACT_STR).unwrap();
  static ref UNIT_RE: Regex = Regex::new(unit_re_str!()).unwrap();
  static ref FILL_RE: Regex = Regex::new(fill_re_str!()).unwrap();
  static ref PLUS_RE: Regex = Regex::new(plus_re_str!()).unwrap();
  static ref MINUS_RE: Regex = Regex::new(minus_re_str!()).unwrap();
  static ref GLUE_RE: Regex = Regex::new(GLUE_RE_STR).unwrap();
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Glue {
  pub skip: f32,
  pub plus: Option<f32>,
  pub pfill: Option<FillCode>,
  pub minus: Option<f32>,
  pub mfill: Option<FillCode>,
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


impl NumericOps for Glue {
  fn value_of(self) -> f32 { self.skip }

  fn register_type(&self) -> RegisterType { RegisterType::Glue }
  // identity, used to type cast in runtime
  fn into_glue_type(self) -> Glue { self }
}

impl fmt::Display for Glue {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // my ($sp, $plus, $pfill, $minus, $mfill) = @$self;
    write!(f, "{}", Dimension::point_format(self.skip))?;
    if let Some(plus) = self.plus {
      if plus != 0.0 {
        write!(f, " plus ")?;
        if let Some(pfill) = self.pfill {
          write!(f, "{}{}", plus, pfill)?;
        } else {
          write!(f, "{}", Dimension::point_format(plus))?;
        };
      }
    }
    if let Some(minus) = self.minus {
      if minus != 0.0 {
        write!(f, " minus ")?;
        if let Some(mfill) = self.mfill {
          write!(f, "{}{}", minus, mfill)?;
        } else {
          write!(f, "{}", Dimension::point_format(minus))?;
        }
      }
    }
    Ok(())
  }
}

impl Glue {
  pub fn new<T: Into<f32>>(number: T) -> Self {
    let (skip, plus, pfill, minus, mfill) = Glue::new_setup(number.into(), None,None,None,None);
    Glue { skip,plus,pfill,minus,mfill }
  }

  pub(crate) fn new_setup(skip:f32,plus:Option<f32>,pfill:Option<FillCode>, minus: Option<f32>, mfill: Option<FillCode>) -> (f32,Option<f32>,Option<FillCode>,Option<f32>,Option<FillCode>) {
    (skip,plus,pfill,minus,mfill)
  }
  pub fn new_full(skip:f32,plus:Option<f32>,pfill:Option<FillCode>,minus:Option<f32>, mfill:Option<FillCode>) -> Self {
    let (skip,plus,pfill,minus,mfill) = Glue::new_setup(skip,plus,pfill,minus,mfill);
    Glue { skip,plus,pfill,minus,mfill }
  }
  pub fn add<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    if other.register_type() != RegisterType::Glue {
      Glue {
        skip: self.skip + other.value_of(),
        plus: self.plus,
        pfill: self.pfill,
        minus: self.minus,
        mfill: self.mfill,
      }
    } else {
      // Both glues, add
      self.add_glue(other.into_glue_type())
    }
  }
  pub fn add_glue(self, other: Glue) -> Glue {
    // (pts, p, pf, m, mf) = @$self;
    // if (ref $other eq 'LaTeXML::Common::Glue') {
    // my ($pts2, $p2, $pf2, $m2, $mf2) = @$other;
    let mut skip = self.skip + other.skip;
    let mut plus = self.plus;
    let mut minus = self.minus;
    let mut pfill = self.pfill;
    let mut mfill = self.mfill;

    match self.pfill.cmp(&other.pfill) {
      Ordering::Equal => {
        if let Some(oplus) = other.plus {
          plus = match plus {
            Some(splus) => Some(splus + oplus),
            None => Some(oplus),
          };
        }
      },
      Ordering::Less => {
        plus = other.plus;
        pfill = other.pfill;
      },
      _ => {},
    };
    match self.mfill.cmp(&other.mfill) {
      Ordering::Equal => {
        if let Some(ominus) = other.minus {
          minus = match minus {
            Some(sminus) => Some(sminus + ominus),
            None => Some(ominus),
          };
        }
      },
      Ordering::Less => {
        minus = other.minus;
        mfill = other.mfill;
      },
      _ => {},
    };

    Glue {
      skip,
      plus,
      pfill,
      minus,
      mfill,
    }
    // else {
    // return (ref $self)->new($pts + $other->valueOf, $p, $pf, $m, $mf); }
  }

  pub fn negate(self) -> Self
  where Self: Sized {
    let value = self.value_of();
    if value > 0.0 {
      Self::new(-value)
    } else {
      Self::new(value)
    }
  }
  pub fn multiply<T: Into<f32>>(self, other: T) -> Self
  where Self: Sized {
    let other: f32 = other.into();
    Self::new((self.value_of() * other).floor())
  }
  pub fn divide<T: Into<f32>>(self, other: T) -> Self
  where Self: Sized {
    let other: f32 = other.into();
    Self::new((self.value_of() / other).floor())
  }

  pub fn spec_new(
    mut spec: &str,
    mut plus: Option<f32>,
    mut pfill: Option<FillCode>,
    mut minus: Option<f32>,
    mut mfill: Option<FillCode>,
    state: &State,
  ) -> Self {
    let mut skip: f32 = spec.parse::<f32>().unwrap_or_default();
    if plus.is_none() && pfill.is_none() && minus.is_none() && mfill.is_none() {
      if NUM_EXACT_RE.is_match(spec) {
        // nothing to do in the simple numeric case
      } else if let Some(cs) = GLUE_RE.captures(spec) {
        let (f, u, p, pu, m, mu) = (
          cs.get(1).unwrap().as_str().parse::<f32>().unwrap_or_default(),
          cs.get(2).unwrap().as_str(),
          cs.get(4).unwrap().as_str().parse::<f32>().unwrap_or_default(),
          cs.get(5).unwrap().as_str(),
          cs.get(7).unwrap().as_str().parse::<f32>().unwrap_or_default(),
          cs.get(8).unwrap().as_str(),
        );
        skip = f * state.convert_unit(u);
        if pu.is_empty() {
        } else if let Some(pfcode) = FillCode::from(pu) {
          plus = Some(p);
          pfill = Some(pfcode);
        } else {
          plus = Some(p * state.convert_unit(pu));
          pfill = None;
        }
        if mu.is_empty() {
        } else if let Some(mfcode) = FillCode::from(mu) {
          minus = Some(m);
          mfill = Some(mfcode);
        } else {
          minus = Some(m * state.convert_unit(mu));
          mfill = None;
        }
      }
    }

    Glue {
      skip,
      plus,
      pfill,
      minus,
      mfill,
    }
  }
}
