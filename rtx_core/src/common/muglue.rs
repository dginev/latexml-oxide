use super::glue::{glue_string, new_setup, spec_setup, FillCode};
use crate::common::dimension::attribute_format;
use crate::definition::register::{NumericOps, RegisterType};
use crate::state::State;
use crate::{Locator, Object};
use std::borrow::Cow;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MuGlue {
  pub skip: f32,
  pub plus: Option<f32>,
  pub pfill: Option<FillCode>,
  pub minus: Option<f32>,
  pub mfill: Option<FillCode>,
}
impl Default for MuGlue {
  fn default() -> Self {
    MuGlue {
      skip: 0.0,
      plus: None,
      pfill: None,
      minus: None,
      mfill: None,
    }
  }
}

impl fmt::Display for MuGlue {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let string = glue_string(self.skip, self.plus, self.pfill, self.minus, self.mfill, "mu");
    write!(f, "{}", string)
  }
}

impl NumericOps for MuGlue {
  fn value_of(self) -> f32 { self.skip }
  fn register_type(&self) -> RegisterType { RegisterType::MuGlue }
  fn add<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_of() + other.value_of())
  }
  fn subtract<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_of() - other.value_of())
  }
}
impl Object for MuGlue {
  fn get_locator(&self) -> Option<Cow<Locator>> { None }
}

impl MuGlue {
  pub fn new<T: Into<f32>>(number: T) -> Self {
    let (skip, plus, pfill, minus, mfill) = new_setup(number.into(), None, None, None, None);
    MuGlue {
      skip,
      plus,
      pfill,
      minus,
      mfill,
    }
  }

  pub fn new_full(skip: f32, plus: Option<f32>, pfill: Option<FillCode>, minus: Option<f32>, mfill: Option<FillCode>) -> Self {
    let (skip, plus, pfill, minus, mfill) = new_setup(skip, plus, pfill, minus, mfill);
    MuGlue {
      skip,
      plus,
      pfill,
      minus,
      mfill,
    }
  }

  pub fn new_spec(spec: &str, plus: Option<f32>, pfill: Option<FillCode>, minus: Option<f32>, mfill: Option<FillCode>, state: &State) -> Self {
    let (skip, plus, pfill, minus, mfill) = spec_setup(spec, plus, pfill, minus, mfill, "mu", state);
    MuGlue {
      skip,
      plus,
      pfill,
      minus,
      mfill,
    }
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

  pub fn to_attribute(&self) -> String {
    let u = "mu";
    let mut string = attribute_format(self.skip, Some(u));
    if let Some(plus) = self.plus {
      if plus != 0.0 {
        string.push_str(" plus ");
        let fill_u = if let Some(pfill) = self.pfill { pfill.to_str() } else { u };
        string.push_str(&attribute_format(plus, Some(fill_u)));
      }
    }
    if let Some(minus) = self.minus {
      if minus != 0.0 {
        string.push_str(" minus ");
        let mfill_u = if let Some(mfill) = self.mfill { mfill.to_str() } else { u };
        string.push_str(&attribute_format(minus, Some(mfill_u)));
      }
    }
    string
  }
}
