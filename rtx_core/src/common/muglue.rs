use super::glue::{glue_string, new_setup, spec_setup, FillCode};
use crate::common::dimension::attribute_format;
use crate::common::numeric_ops::NumericOps;
use crate::definition::register::RegisterType;
use crate::{Object};
use std::fmt;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct MuGlue {
  pub skip: i64,
  pub plus: Option<i64>,
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
}
impl Object for MuGlue {}

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
    let (skip, plus, pfill, minus, mfill) =
      spec_setup(spec, plus, pfill, minus, mfill, "mu");
    MuGlue {
      skip,
      plus,
      pfill,
      minus,
      mfill,
    }
  }

  pub fn to_attribute(&self) -> String {
    let u = "mu";
    let mut string = attribute_format(self.skip, Some(u));
    if let Some(plus) = self.plus {
      if plus != 0 {
        string.push_str(" plus ");
        let fill_u = if let Some(pfill) = self.pfill {
          pfill.to_str()
        } else {
          u
        };
        string.push_str(&attribute_format(plus, Some(fill_u)));
      }
    }
    if let Some(minus) = self.minus {
      if minus != 0 {
        string.push_str(" minus ");
        let mfill_u = if let Some(mfill) = self.mfill {
          mfill.to_str()
        } else {
          u
        };
        string.push_str(&attribute_format(minus, Some(mfill_u)));
      }
    }
    string
  }
}
