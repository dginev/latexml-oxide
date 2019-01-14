use crate::definition::register::NumericOps;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Glue {
  number: f32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MuGlue {
  number: f32,
}

impl NumericOps for Glue {
  fn value_of(self) -> f32 { self.number }
  fn new<T: Into<f32>>(number: T) -> Self { Glue { number: number.into() } }
}

impl NumericOps for MuGlue {
  fn new<T: Into<f32>>(number: T) -> Self { MuGlue { number: number.into() } }
  fn value_of(self) -> f32 { self.number }
}

impl Glue {
  pub fn new_str(spec: &str, plus: Option<&str>, pfill: Option<&str>, minus: Option<&str>, mfill: Option<&str>) -> Self {
    // let spec = if spec.is_empty() {
    //   "0"
    // } else {
    //   spec
    // }.to_string();

    // if plus.is_none() && pfill.is_none() && minus.is_none() && mfill.is_none() {
    //   if spec =~ /^(\d*\.?\d*)$/ {
    //   } else if ($spec =~ /^$GLUE_re$/) {
    //     my ($f, $u, $p, $pu, $m, $mu) = ($1, $2, $4, $5, $7, $8);
    //     $spec = $f * $STATE->convertUnit($u);
    //     if (!$pu) { }
    //     elsif ($fillcode{$pu}) { $plus = $p;                            $pfill = $pu; }
    //     else                   { $plus = $p * $STATE->convertUnit($pu); $pfill = 0; }
    //     if (!$mu) { }
    //     elsif ($fillcode{$mu}) { $minus = $m;                            $mfill = $mu; }
    //     else                   { $minus = $m * $STATE->convertUnit($mu); $mfill = 0; }
    //   }
    // }
    // TODO:
    // Glue {
    //   spec,
    //   plus: plus.unwrap_or("0"),
    //   pfill: pfill.unwrap_or(0),
    //   minus: minus.unwrap_or("0"),
    //   mfill: mfill.unwrap_or(0)
    // }
    Glue { number: 0.0 }
  }
}

#[macro_export]
macro_rules! Glue {
  ($spec:expr) => {
    Glue::new_str($spec, None, None, None, None)
  };
}
