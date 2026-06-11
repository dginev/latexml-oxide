use std::fmt;

use super::glue::{FillCode, glue_string, new_setup, spec_setup};
use crate::{
  Object,
  common::{dimension::attribute_format, numeric_ops::NumericOps},
  definition::register::RegisterType,
  token::{Catcode, Token},
  tokens::Tokens,
};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct MuGlue {
  pub skip:  i64,
  pub plus:  Option<i64>,
  pub pfill: Option<FillCode>,
  pub minus: Option<i64>,
  pub mfill: Option<FillCode>,
}

impl fmt::Display for MuGlue {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let string = glue_string(
      self.skip, self.plus, self.pfill, self.minus, self.mfill, "mu",
    );
    write!(f, "{string}")
  }
}

impl NumericOps for MuGlue {
  fn value_of(self) -> i64 { self.skip }
  fn register_type(&self) -> RegisterType { RegisterType::MuGlue }
  fn new(skip: i64) -> Self {
    MuGlue {
      skip,
      plus: None,
      pfill: None,
      minus: None,
      mfill: None,
    }
  }
  fn new_f64(number: f64) -> Self {
    let (skip, plus, pfill, minus, mfill) = new_setup(number, None, None, None, None);
    MuGlue {
      skip,
      plus,
      pfill,
      minus,
      mfill,
    }
  }
  // Negate all components (skip, plus, minus)
  fn negate(self) -> Self {
    MuGlue {
      skip:  -self.skip,
      plus:  self.plus.map(|p| -p),
      pfill: self.pfill,
      minus: self.minus.map(|m| -m),
      mfill: self.mfill,
    }
  }
}
impl Object for MuGlue {
  fn revert(&self) -> crate::Result<Tokens> { Ok(Tokens::new(Explode!(self.to_string()))) }
}

impl MuGlue {
  pub fn new_full(
    skip: i64,
    plus: Option<i64>,
    pfill: Option<FillCode>,
    minus: Option<i64>,
    mfill: Option<FillCode>,
  ) -> Self {
    MuGlue {
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
    MuGlue {
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
    let (skip, plus, pfill, minus, mfill) = spec_setup(spec, plus, pfill, minus, mfill, "mu");
    MuGlue {
      skip,
      plus,
      pfill,
      minus,
      mfill,
    }
  }

  pub fn to_attribute(&self) -> String {
    // XML attribute output is pt-typed by convention (Perl
    // `Common/Dimension::attributeformat`). Convert mu→pt via the
    // shared `mu_to_pt` helper so XMHint width / lpadding /
    // rpadding values come out as `1.66663pt` not `3.0mu`.
    // Mirrors Perl `Common/MuGlue::ptValue` flow.
    let pt_skip = mu_to_pt(self.skip);
    let mut string = attribute_format(pt_skip, Some("pt"));
    if let Some(plus) = self.plus
      && plus != 0
    {
      string.push_str(" plus ");
      let fill_u = if let Some(pfill) = self.pfill {
        pfill.to_str()
      } else {
        "pt"
      };
      let plus_pt = if fill_u == "pt" { mu_to_pt(plus) } else { plus };
      string.push_str(&attribute_format(plus_pt, Some(fill_u)));
    }
    if let Some(minus) = self.minus
      && minus != 0
    {
      string.push_str(" minus ");
      let mfill_u = if let Some(mfill) = self.mfill {
        mfill.to_str()
      } else {
        "pt"
      };
      let minus_pt = if mfill_u == "pt" {
        mu_to_pt(minus)
      } else {
        minus
      };
      string.push_str(&attribute_format(minus_pt, Some(mfill_u)));
    }
    string
  }
}

fn mu_to_pt(mu_val: i64) -> i64 {
  let fs = crate::state::lookup_font()
    .and_then(|f| f.get_size())
    .unwrap_or(10.0);
  let unity = crate::common::numeric_ops::UNITY_F64;
  let muwidth = (fs * unity / 18.0) as i64;
  ((mu_val as f64 * muwidth as f64 / unity).trunc()) as i64
}

impl From<MuGlue> for Option<Tokens> {
  fn from(v: MuGlue) -> Option<Tokens> { Some(v.into()) }
}
impl From<MuGlue> for Tokens {
  fn from(v: MuGlue) -> Tokens {
    v.revert()
      .expect("MuGlue should always be revertable to Tokens.")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn muglue_default_is_zero() {
    let m = MuGlue::default();
    assert_eq!(m.skip, 0);
    assert_eq!(m.plus, None);
    assert_eq!(m.minus, None);
    assert_eq!(m.pfill, None);
    assert_eq!(m.mfill, None);
  }

  #[test]
  fn muglue_new_builds_skip_only() {
    let m = <MuGlue as NumericOps>::new(65536);
    assert_eq!(m.skip, 65536);
    assert_eq!(m.plus, None);
    assert_eq!(m.minus, None);
  }

  #[test]
  fn muglue_value_of_returns_skip() {
    let m = MuGlue {
      skip:  1234,
      plus:  Some(10),
      pfill: None,
      minus: None,
      mfill: None,
    };
    assert_eq!(m.value_of(), 1234);
  }

  #[test]
  fn muglue_register_type_is_muglue() {
    let m = MuGlue::default();
    assert_eq!(m.register_type(), RegisterType::MuGlue);
  }

  #[test]
  fn muglue_negate_flips_all_components() {
    let m = MuGlue {
      skip:  100,
      plus:  Some(10),
      pfill: Some(FillCode::Fil),
      minus: Some(5),
      mfill: Some(FillCode::Fill),
    };
    let n = m.negate();
    assert_eq!(n.skip, -100);
    assert_eq!(n.plus, Some(-10));
    assert_eq!(n.minus, Some(-5));
    // FillCodes are not negated — they describe stretch/shrink kind.
    assert_eq!(n.pfill, Some(FillCode::Fil));
    assert_eq!(n.mfill, Some(FillCode::Fill));
  }

  #[test]
  fn muglue_new_full_roundtrip() {
    let m = MuGlue::new_full(
      100,
      Some(10),
      Some(FillCode::Fil),
      Some(5),
      Some(FillCode::Fill),
    );
    assert_eq!(m.skip, 100);
    assert_eq!(m.plus, Some(10));
    assert_eq!(m.pfill, Some(FillCode::Fil));
    assert_eq!(m.minus, Some(5));
    assert_eq!(m.mfill, Some(FillCode::Fill));
  }

  #[test]
  fn muglue_display_includes_mu_unit() {
    // Display uses "mu" as the unit.
    let m = <MuGlue as NumericOps>::new(65536); // 1mu
    let out = format!("{m}");
    assert!(
      out.contains("mu"),
      "MuGlue display should include 'mu' unit; got {out:?}"
    );
  }

  #[test]
  fn muglue_equality() {
    let a = MuGlue {
      skip:  1,
      plus:  None,
      pfill: None,
      minus: None,
      mfill: None,
    };
    let b = MuGlue {
      skip:  1,
      plus:  None,
      pfill: None,
      minus: None,
      mfill: None,
    };
    let c = MuGlue {
      skip:  2,
      plus:  None,
      pfill: None,
      minus: None,
      mfill: None,
    };
    assert_eq!(a, b);
    assert_ne!(a, c);
  }

  #[test]
  fn muglue_double_negate_is_identity() {
    let m = MuGlue {
      skip:  100,
      plus:  Some(10),
      pfill: Some(FillCode::Fil),
      minus: Some(5),
      mfill: Some(FillCode::Fill),
    };
    assert_eq!(m.negate().negate(), m);
  }
}
