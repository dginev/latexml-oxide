use std::fmt;
use super::glue::{FillCode,Glue};
use crate::definition::register::{NumericOps, RegisterType};
use crate::state::State;


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
    unimplemented!();
  }
  // sub toString {
  // my ($self) = @_;
  // my ($sp, $plus, $pfill, $minus, $mfill) = @$self;
  // my $string = LaTeXML::Common::Float::floatformat($sp / 65536 * 1.8) . 'mu ';
  // $string .= 'plus ' . ($pfill
  //   ? $plus . $LaTeXML::Common::Glue::FILL[$pfill]
  //   : LaTeXML::Common::Float::floatformat($plus / 65536 * 1.8) . 'mu ') if $plus != 0;
  // $string .= 'minus ' . ($mfill
  //   ? $minus . $LaTeXML::Common::Glue::FILL[$mfill]
  //   : LaTeXML::Common::Float::floatformat($minus / 65536 * 1.8) . 'mu ') if $minus != 0;
  // return $string; }
}

impl NumericOps for MuGlue {
  fn value_of(self) -> f32 { self.skip }
  fn register_type(&self) -> RegisterType { RegisterType::MuGlue }
}

impl MuGlue {
  pub fn new<T: Into<f32>>(number: T) -> Self {
    let (skip,plus,pfill,minus,mfill) = Glue::new_setup(number.into(),None,None,None,None);
    MuGlue { skip,plus,pfill,minus,mfill }
  }

  pub fn new_full(skip:f32,plus:Option<f32>,pfill:Option<FillCode>,minus:Option<f32>, mfill:Option<FillCode>) -> Self {
    let (skip,plus,pfill,minus,mfill) = Glue::new_setup(skip,plus,pfill,minus,mfill);
    MuGlue { skip,plus,pfill,minus,mfill }
  }

  pub fn add<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_of() + other.value_of())
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

}