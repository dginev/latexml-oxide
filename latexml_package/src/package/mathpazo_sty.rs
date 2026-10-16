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

  // mathpazo.sty:63/67 declares the `upright` symbol font (its upright Greek), so `\symupright` exists: flexisym's
  // mathpazo.sym:49 reads it (`\cs_gset:cpx{mg@Greek}{\hexnumber@\symupright}`) under `\usepackage[mathpazo]{flexisym}`
  // (breqn). Perl's binding declares none (2605.15826, 2605.30950: `\symupright` undefined).
  RawTeX!(r"\DeclareSymbolFont{upright}{OT1}{zplm}{m}{n}\SetSymbolFont{upright}{bold}{OT1}{zplm}{b}{n}");
  // mathpazo.sty:72 `\DeclareMathAlphabet{\mathbold}{OML}{zplm}{b}{it}`: bold italic math letters (72 papers;
  // 0807.0088, 0905.0925). The `\PazoBB` alphabet (:101-103, the blackboard letters `\mathbb` is made to call,
  // unless `noBBpl`) is the binding's `\mathbb`. Perl's binding defines neither (KNOWN_PERL_ERRORS #761).
  RawTeX!(r"\DeclareMathAlphabet{\mathbold}{OML}{zplm}{b}{it}");
  Let!("\\PazoBB", "\\mathbb");
});
