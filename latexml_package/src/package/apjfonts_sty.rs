use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMath!("\\varGamma",   None, "\u{0393}", font => { shape => "italic" });
  DefMath!("\\varSigma",   None, "\u{03A3}", font => { shape => "italic" });
  DefMath!("\\varDelta",   None, "\u{0394}", font => { shape => "italic" });
  DefMath!("\\varUpsilon", None, "\u{03A5}", font => { shape => "italic" });
  DefMath!("\\varTheta",   None, "\u{0398}", font => { shape => "italic" });
  DefMath!("\\varPhi",     None, "\u{03A6}", font => { shape => "italic" });
  DefMath!("\\varLambda",  None, "\u{039B}", font => { shape => "italic" });
  DefMath!("\\varPsi",     None, "\u{03A8}", font => { shape => "italic" });
  DefMath!("\\varXi",      None, "\u{039E}", font => { shape => "italic" });
  DefMath!("\\varOmega",   None, "\u{03A9}", font => { shape => "italic" });
  DefMath!("\\varPi",      None, "\u{03A0}", font => { shape => "italic" });
  DefMath!("\\omicron", "\u{03BF}");
  DefMath!("\\barbar",  "\u{00AF}");
});
