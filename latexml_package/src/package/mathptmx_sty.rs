//! mathptmx.sty — Times Roman math fonts
//! Perl: mathptmx.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMath!("\\omicron", "\u{03BF}");  // GREEK SMALL LETTER OMICRON
  Let!("\\upDelta",   "\\Delta");
  Let!("\\upGamma",   "\\Gamma");
  Let!("\\upLambda",  "\\Lambda");
  Let!("\\upOmega",   "\\Omega");
  Let!("\\upPhi",     "\\Phi");
  Let!("\\upPi",      "\\Pi");
  Let!("\\upPsi",     "\\Psi");
  Let!("\\upSigma",   "\\Sigma");
  Let!("\\upTheta",   "\\Theta");
  Let!("\\upUpsilon", "\\Upsilon");
  Let!("\\upXi",      "\\Xi");
});
