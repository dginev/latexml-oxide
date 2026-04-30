use crate::prelude::*;

LoadDefinitions!({
  DefConditional!("\\ifcspex");
  DefConditional!("\\ifbbgreekl");

  DefConstructor!("\\lx@mbfont{}", "#1",
    bounded => true, require_math => true,
    font => {family => "blackboard", shape => "upright",
             forcefamily => true, forceshape => true});

  DefMath!("\\Eins", "\\lx@mbfont{1}");
  DefMath!("\\Langle",  "\\lx@mbfont{<}", role => "OPEN");
  DefMath!("\\Lbrack",  "\\lx@mbfont{[}", role => "OPEN");
  DefMath!("\\Lparen",  "\\lx@mbfont{(}", role => "OPEN");
  DefMath!("\\Rangle",  "\\lx@mbfont{>}", role => "CLOSE");
  DefMath!("\\Rbrack",  "\\lx@mbfont{]}", role => "CLOSE");
  DefMath!("\\Rparen",  "\\lx@mbfont{)}", role => "CLOSE");
  DefMath!("\\bbalpha", "\\lx@mbfont{\\alpha}");
  DefMath!("\\bbbeta", "\\lx@mbfont{\\beta}");
  DefMath!("\\bbchi", "\\lx@mbfont{\\chi}");
  DefMath!("\\bbdelta", "\\lx@mbfont{\\delta}");
  // Yes, espilon !
  DefMath!("\\bbespilon", "\\lx@mbfont{\\epsilon}");
  DefMath!("\\bbeta", "\\lx@mbfont{\\beta}");
  DefMath!("\\bbgamma", "\\lx@mbfont{\\gamma}");
  DefMath!("\\bbiota", "\\lx@mbfont{i}");
  DefMath!("\\bbkappa", "\\lx@mbfont{\\kappa}");
  DefMath!("\\bblambda", "\\lx@mbfont{\\lambda}");
  DefMath!("\\bbmu", "\\lx@mbfont{\\mu}");
  DefMath!("\\bbnu", "\\lx@mbfont{\\nu}");
  DefMath!("\\bbomega", "\\lx@mbfont{\\omega}");
  DefMath!("\\bbphi", "\\lx@mbfont{\\phi}");
  DefMath!("\\bbpi", "\\lx@mbfont{\\pi}");
  DefMath!("\\bbpsi", "\\lx@mbfont{\\psi}");
  DefMath!("\\bbrho", "\\lx@mbfont{\\rho}");
  DefMath!("\\bbsigma", "\\lx@mbfont{\\sigma}");
  DefMath!("\\bbtau", "\\lx@mbfont{\\tau}");
  DefMath!("\\bbtheta", "\\lx@mbfont{\\theta}");
  DefMath!("\\bbupsilon", "\\lx@mbfont{\\upsilon}");
  DefMath!("\\bbxi", "\\lx@mbfont{\\xi}");
  DefMath!("\\bbzeta", "\\lx@mbfont{\\zeta}");
});
