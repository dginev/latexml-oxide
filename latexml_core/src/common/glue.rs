use std::{cmp::Ordering, fmt};

use once_cell::sync::Lazy;
use regex::Regex;

use super::dimension::fixedformat;
use crate::{
  Object,
  common::{
    dimension::attribute_format,
    error::Result,
    numeric_ops::{EPSILON, NumericOps, fixpoint, kround},
  },
  definition::register::{RegisterType, RegisterValue},
  digested::Digested,
  state::*,
};

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
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.to_str()) }
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
  pub fn to_str(&self) -> &'static str {
    match self {
      FillCode::Fil => "fil",
      FillCode::Fill => "fill",
      FillCode::Filll => "filll",
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

macro_rules! plus_re_str {
  () => {
    concat!(r"\s+plus\s*(", num_re_str!(), ")(", fill_re_str!(), r")")
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

static _NUM_RE: Lazy<Regex> = Lazy::new(|| Regex::new(num_re_str!()).unwrap());
static UNIT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(unit_re_str!()).unwrap());
static _FILL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(fill_re_str!()).unwrap());
static _PLUS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(plus_re_str!()).unwrap());
static _MINUS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(minus_re_str!()).unwrap());
static GLUE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(GLUE_RE_STR).unwrap());

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct Glue {
  pub skip:  i64,
  pub plus:  Option<i64>,
  pub pfill: Option<FillCode>,
  pub minus: Option<i64>,
  pub mfill: Option<FillCode>,
}

impl NumericOps for Glue {
  fn value_of(self) -> i64 { self.skip }
  fn register_type(&self) -> RegisterType { RegisterType::Glue }
  // identity, used to type cast in runtime
  fn into_glue_type(self) -> Glue { self }
  fn add<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    if other.register_type() != RegisterType::Glue {
      Glue {
        skip:  self.skip + other.value_of(),
        plus:  self.plus,
        pfill: self.pfill,
        minus: self.minus,
        mfill: self.mfill,
      }
    } else {
      // Both glues, add
      self.add_glue(other.into_glue_type())
    }
  }
  fn new(skip: i64) -> Self {
    Glue {
      skip,
      plus: None,
      pfill: None,
      minus: None,
      mfill: None,
    }
  }
  fn new_f64(number: f64) -> Self {
    let (skip, plus, pfill, minus, mfill) = new_setup(number, None, None, None, None);
    Glue {
      skip,
      plus,
      pfill,
      minus,
      mfill,
    }
  }
  // Perl Glue.pm: multiply scales skip, plus, AND minus components
  fn multiply<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    let factor = other.value_f64();
    Glue {
      skip:  (self.skip as f64 * factor) as i64,
      plus:  self.plus.map(|p| (p as f64 * factor) as i64),
      pfill: self.pfill,
      minus: self.minus.map(|m| (m as f64 * factor) as i64),
      mfill: self.mfill,
    }
  }
  // Perl Glue.pm: divide scales skip, plus, AND minus components
  fn divide<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    let mut divisor = other.value_f64();
    if divisor == 0.0 {
      divisor = EPSILON;
    }
    Glue {
      skip:  (self.skip as f64 / divisor).trunc() as i64,
      plus:  self.plus.map(|p| (p as f64 / divisor).trunc() as i64),
      pfill: self.pfill,
      minus: self.minus.map(|m| (m as f64 / divisor).trunc() as i64),
      mfill: self.mfill,
    }
  }
  fn subtract<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    if other.register_type() != RegisterType::Glue {
      Glue {
        skip:  self.skip - other.value_of(),
        plus:  self.plus,
        pfill: self.pfill,
        minus: self.minus,
        mfill: self.mfill,
      }
    } else {
      let other_glue = other.into_glue_type();
      self.add_glue(Glue {
        skip:  -other_glue.skip,
        plus:  other_glue.plus.map(|p| -p),
        pfill: other_glue.pfill,
        minus: other_glue.minus.map(|m| -m),
        mfill: other_glue.mfill,
      })
    }
  }
  // Negate all components (skip, plus, minus)
  fn negate(self) -> Self {
    Glue {
      skip:  -self.skip,
      plus:  self.plus.map(|p| -p),
      pfill: self.pfill,
      minus: self.minus.map(|m| -m),
      mfill: self.mfill,
    }
  }
  fn smaller<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    let other_val = other.value_of();
    if self.skip <= other_val {
      self
    } else {
      Self::new(other_val)
    }
  }
  fn larger<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    let other_val = other.value_of();
    if self.skip >= other_val {
      self
    } else {
      Self::new(other_val)
    }
  }
}

pub fn glue_string(
  skip: i64,
  plus_opt: Option<i64>,
  pfill_opt: Option<FillCode>,
  minus_opt: Option<i64>,
  mfill_opt: Option<FillCode>,
  unit: &str,
) -> String {
  // ??? TODO: There seems to be some messy confusion about the types of the
  // pieces of glue/dimensions -- are we consistently using i64 or f64?
  let mut string = fixedformat(skip, Some(unit));
  if let Some(plus) = plus_opt
    && plus != 0
  {
    string.push_str(" plus ");
    let p_fill = if let Some(fill) = pfill_opt {
      fill.to_str()
    } else {
      unit
    };
    string.push_str(&fixedformat(plus, Some(p_fill)))
  }
  if let Some(minus) = minus_opt
    && minus != 0
  {
    string.push_str(" minus ");
    let p_fill = if let Some(fill) = mfill_opt {
      fill.to_str()
    } else {
      unit
    };
    string.push_str(&fixedformat(minus, Some(p_fill)))
  }
  string
}

impl fmt::Display for Glue {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let string = glue_string(
      self.skip, self.plus, self.pfill, self.minus, self.mfill, "pt",
    );
    write!(f, "{string}")
  }
}
impl Object for Glue {
  fn be_digested(self) -> Result<Digested> { Ok(RegisterValue::Glue(self).into()) }
}

pub fn new_setup(
  skip: f64,
  plus: Option<f64>,
  pfill: Option<FillCode>,
  minus: Option<f64>,
  mfill: Option<FillCode>,
) -> (
  i64,
  Option<i64>,
  Option<FillCode>,
  Option<i64>,
  Option<FillCode>,
) {
  // See comment in Dimension for why kround rather than int
  (
    kround(skip),
    plus.map(kround),
    pfill,
    minus.map(kround),
    mfill,
  )
}

pub fn spec_setup(
  spec: &str,
  plus: Option<f64>,
  mut pfill: Option<FillCode>,
  minus: Option<f64>,
  mut mfill: Option<FillCode>,
  unit: &str,
) -> (
  i64,
  Option<i64>,
  Option<FillCode>,
  Option<i64>,
  Option<FillCode>,
) {
  if !UNIT_RE.is_match(spec) {
    // If no units, expect fixedpoint values
    let skip: f64 = spec.parse::<f64>().unwrap_or_default();
    new_setup(skip, plus, pfill, minus, mfill)
  } else {
    let is_mu = unit == "mu";
    if plus.is_some() || pfill.is_some() || minus.is_some() || mfill.is_some() {
      let msg = s!(
        "You should not create {} with both units and stretch",
        if is_mu { "MuGlue" } else { "Glue" }
      );
      Warn!("unexpected", "fill", msg);
    }

    if let Some(cs) = GLUE_RE.captures(spec) {
      let (f, unit, p, punit, m, munit) = (
        cs.get(1)
          .map(|v| v.as_str().parse::<f64>().unwrap_or_default())
          .unwrap_or_default(),
        cs.get(2).map_or("", |m| m.as_str()),
        cs.get(4)
          .map(|v| v.as_str().parse::<f64>().unwrap_or_default())
          .unwrap_or_default(),
        cs.get(5).map_or("", |m| m.as_str()),
        cs.get(7)
          .map(|v| v.as_str().parse::<f64>().unwrap_or_default())
          .unwrap_or_default(),
        cs.get(8).map_or("", |m| m.as_str()),
      );
      let skip = if unit.is_empty() {
        f.trunc() as i64
      } else if is_mu {
        if unit != "mu" {
          Warn!("unexpected", unit, "Assumed mu");
        }
        fixpoint(f, None) // in mu
      } else {
        fixpoint(f, Some(convert_unit(unit)))
      };

      let mut plus = if punit.is_empty() {
        None // Some(0.0) ?
      // ? punit = "0";
      } else if let Some(code) = FillCode::from(punit) {
        pfill = Some(code);
        Some(fixpoint(p, None))
      } else if is_mu {
        pfill = None;
        if punit != "mu" {
          Warn!("unexpected", punit, "Assumed mu");
        }
        Some(fixpoint(p, None))
      } else {
        pfill = None; // ? 0
        Some(fixpoint(p, Some(convert_unit(punit))))
      };

      let mut minus = if munit.is_empty() {
        None // ? Some(0.0);
      // munit = 0;
      } else if let Some(code) = FillCode::from(munit) {
        mfill = Some(code);
        Some(fixpoint(m, None))
      } else if is_mu {
        mfill = None; // 0
        if munit != "mu" {
          Warn!("unexpected", munit, "Assumed mu");
        }
        Some(fixpoint(m, None))
      } else {
        mfill = None; // 0
        Some(fixpoint(m, Some(convert_unit(munit))))
      };

      if punit.is_empty() {
      } else if let Some(pfcode) = FillCode::from(punit) {
        plus = Some(fixpoint(p, None));
        pfill = Some(pfcode);
      } else {
        plus = Some(fixpoint(p, Some(convert_unit(punit))));
        pfill = None;
      }
      if munit.is_empty() {
      } else if let Some(mfcode) = FillCode::from(munit) {
        minus = Some(fixpoint(m, None));
        mfill = Some(mfcode);
      } else {
        minus = Some(fixpoint(m, Some(convert_unit(munit))));
        mfill = None;
      }
      (skip, plus, pfill, minus, mfill)
    } else {
      let msg = s!(
        "Missing {} specification assuming 0pt",
        if is_mu { "MuGlue" } else { "Glue" }
      );
      Warn!("unexpected", spec, msg);
      (0, None, None, None, None)
    }
  }
}

impl Glue {
  pub fn new_full(
    skip: i64,
    plus: Option<i64>,
    pfill: Option<FillCode>,
    minus: Option<i64>,
    mfill: Option<FillCode>,
  ) -> Self {
    Glue {
      skip,
      plus,
      pfill,
      minus,
      mfill,
    }
  }
  pub fn new_full_f64(
    skip: f64,
    plus: Option<f64>,
    pfill: Option<FillCode>,
    minus: Option<f64>,
    mfill: Option<FillCode>,
  ) -> Self {
    let (skip, plus, pfill, minus, mfill) = new_setup(skip, plus, pfill, minus, mfill);
    Glue {
      skip,
      plus,
      pfill,
      minus,
      mfill,
    }
  }
  pub fn new_spec(
    spec: &str,
    plus: Option<f64>,
    pfill: Option<FillCode>,
    minus: Option<f64>,
    mfill: Option<FillCode>,
  ) -> Self {
    let (skip, plus, pfill, minus, mfill) = spec_setup(spec, plus, pfill, minus, mfill, "pt");
    Glue {
      skip,
      plus,
      pfill,
      minus,
      mfill,
    }
  }

  pub fn add_glue(self, other: Glue) -> Glue {
    // (pts, p, pf, m, mf) = @$self;
    // if (ref $other eq 'LaTeXML::Common::Glue') {
    // my ($pts2, $p2, $pf2, $m2, $mf2) = @$other;
    let skip = self.skip + other.skip;
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

  pub fn to_attribute(&self) -> String {
    let u = "pt";
    let mut string = attribute_format(self.skip, Some(u));
    if let Some(plus) = self.plus
      && plus != 0
    {
      string.push_str(" plus ");
      let fill_u = if let Some(pfill) = self.pfill {
        pfill.to_str()
      } else {
        u
      };
      string.push_str(&attribute_format(plus, Some(fill_u)));
    }
    if let Some(minus) = self.minus
      && minus != 0
    {
      string.push_str(" minus ");
      let mfill_u = if let Some(mfill) = self.mfill {
        mfill.to_str()
      } else {
        u
      };
      string.push_str(&attribute_format(minus, Some(mfill_u)));
    }
    string
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fillcode_new_from_index() {
    assert_eq!(FillCode::new(1), Some(FillCode::Fil));
    assert_eq!(FillCode::new(2), Some(FillCode::Fill));
    assert_eq!(FillCode::new(3), Some(FillCode::Filll));
    assert_eq!(FillCode::new(0), None);
    assert_eq!(FillCode::new(4), None);
    assert_eq!(FillCode::new(100), None);
  }

  #[test]
  fn fillcode_from_str_case_sensitive() {
    assert_eq!(FillCode::from("fil"), Some(FillCode::Fil));
    assert_eq!(FillCode::from("fill"), Some(FillCode::Fill));
    assert_eq!(FillCode::from("filll"), Some(FillCode::Filll));
    // Case-sensitive: uppercase is rejected.
    assert_eq!(FillCode::from("FIL"), None);
    assert_eq!(FillCode::from("Fil"), None);
    assert_eq!(FillCode::from(""), None);
    assert_eq!(FillCode::from("other"), None);
  }

  #[test]
  fn fillcode_to_str_roundtrip() {
    // from(to_str(c)) == c for all variants.
    for code in [FillCode::Fil, FillCode::Fill, FillCode::Filll] {
      let s = code.to_str();
      assert_eq!(
        FillCode::from(s),
        Some(code),
        "roundtrip broke at {code:?} via {s:?}"
      );
    }
  }

  #[test]
  fn fillcode_display_matches_to_str() {
    assert_eq!(format!("{}", FillCode::Fil), "fil");
    assert_eq!(format!("{}", FillCode::Fill), "fill");
    assert_eq!(format!("{}", FillCode::Filll), "filll");
  }

  #[test]
  fn fillcode_ord_fil_lt_fill_lt_filll() {
    // Derived Ord follows variant declaration order.
    assert!(FillCode::Fil < FillCode::Fill);
    assert!(FillCode::Fill < FillCode::Filll);
  }

  #[test]
  fn glue_default_is_zero_skip_no_stretch() {
    let g = Glue::default();
    assert_eq!(g.skip, 0);
    assert_eq!(g.plus, None);
    assert_eq!(g.minus, None);
    assert_eq!(g.pfill, None);
    assert_eq!(g.mfill, None);
  }

  #[test]
  fn glue_new_builds_skip_only() {
    let g = <Glue as NumericOps>::new(65536);
    assert_eq!(g.skip, 65536);
    assert_eq!(g.plus, None);
    assert_eq!(g.minus, None);
  }

  #[test]
  fn glue_value_of_returns_skip() {
    let g = Glue {
      skip:  1234,
      plus:  Some(10),
      pfill: None,
      minus: None,
      mfill: None,
    };
    assert_eq!(
      g.value_of(),
      1234,
      "value_of returns the skip, not the stretch"
    );
  }
}
