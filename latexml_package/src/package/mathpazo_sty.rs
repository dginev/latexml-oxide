use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: mathpazo.sty.ltxml
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

  DefConstructor!("\\mathbb{}", "#1",
    bounded => true, require_math => true,
    font => {family => "blackboard", series => "medium", shape => "upright"});
});
