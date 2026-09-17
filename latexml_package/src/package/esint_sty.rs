use crate::prelude::*;

LoadDefinitions!({
  DefMath!("\\iintop", None, "\u{222C}",
    role => "INTOP", meaning => "double-integral",
    variablesize_op => true);
  DefMath!("\\iint", None, "\u{222C}",
    role => "INTOP", meaning => "double-integral",
    variablesize_op => true);
  DefMath!("\\iiintop", None, "\u{222D}",
    role => "INTOP", meaning => "triple-integral",
    variablesize_op => true);
  DefMath!("\\iiint", None, "\u{222D}",
    role => "INTOP", meaning => "triple-integral",
    variablesize_op => true);
  DefMath!("\\iiiintop", None, "\u{2A0C}",
    role => "INTOP", meaning => "quadruple-integral",
    variablesize_op => true);
  DefMath!("\\iiiint", None, "\u{2A0C}",
    role => "INTOP", meaning => "quadruple-integral",
    variablesize_op => true);
  // dotsint: kludged composition of \int...\int
  DefPrimitive!(
    "\\lx@esint@dotsint",
    "\\lx@kludged{\\int\\lx@tweaked{width=0.4em,xoffset=-0.3em,yoffset=0.4ex}{\\ldots}\\int}"
  );
  DefMath!("\\dotsintop", None,
    "\\lx@kludged{\\int\\lx@tweaked{width=0.4em,xoffset=-0.3em,yoffset=0.4ex}{\\ldots}\\int}",
    role => "INTOP", meaning => "multiple-integral",
    variablesize_op => true);
  DefMath!("\\dotsint", None,
    "\\lx@kludged{\\int\\lx@tweaked{width=0.4em,xoffset=-0.3em,yoffset=0.4ex}{\\ldots}\\int}",
    role => "INTOP", meaning => "multiple-integral",
    variablesize_op => true);
  DefMath!("\\ointop", None, "\u{222E}",
    role => "INTOP", meaning => "contour-integral",
    variablesize_op => true);
  DefMath!("\\oint", None, "\u{222E}",
    role => "INTOP", meaning => "contour-integral",
    variablesize_op => true);
  DefMath!("\\oiintop", None, "\u{222F}",
    role => "INTOP", meaning => "surface-integral",
    variablesize_op => true);
  DefMath!("\\oiint", None, "\u{222F}",
    role => "INTOP", meaning => "surface-integral",
    variablesize_op => true);
  DefMath!("\\varoiintop", None, "\u{222F}",
    role => "INTOP", meaning => "surface-integral",
    variablesize_op => true);
  DefMath!("\\varoiint", None, "\u{222F}",
    role => "INTOP", meaning => "surface-integral",
    variablesize_op => true);
  DefMath!("\\sqintop", None, "\u{2A16}",
    role => "INTOP", meaning => "quaternion-integral",
    variablesize_op => true);
  DefMath!("\\sqint", None, "\u{2A16}",
    role => "INTOP", meaning => "quaternion-integral",
    variablesize_op => true);

  DefPrimitive!("\\lx@esint@box", "\u{25AD}");
  // sqiint: kludged composition of \iint with overlaid box
  DefMath!("\\sqiintop", None,
    "\\lx@kludged{\\iint\\mathchoice{\\lx@tweaked{width=0pt,xoffset=-1.2em}{\\lx@esint@box}}{\\lx@tweaked{width=0pt,xoffset=-1.0em}{\\lx@esint@box}}{\\lx@tweaked{width=0pt,xoffset=-1.0em}{\\lx@esint@box}}{\\lx@tweaked{width=0pt,xoffset=-1.0em}{\\lx@esint@box}}}",
    role => "INTOP", meaning => "quaternion-double-integral",
    variablesize_op => true);
  DefMath!("\\sqiint", None,
    "\\lx@kludged{\\iint\\mathchoice{\\lx@tweaked{width=0pt,xoffset=-1.2em}{\\lx@esint@box}}{\\lx@tweaked{width=0pt,xoffset=-1.0em}{\\lx@esint@box}}{\\lx@tweaked{width=0pt,xoffset=-1.0em}{\\lx@esint@box}}{\\lx@tweaked{width=0pt,xoffset=-1.0em}{\\lx@esint@box}}}",
    role => "INTOP", meaning => "quaternion-double-integral",
    variablesize_op => true);

  DefMath!("\\ointctrclockwiseop", None, "\u{2233}",
    role => "INTOP", meaning => "counterclockwise-contour-integral",
    variablesize_op => true);
  DefMath!("\\ointctrclockwise", None, "\u{2233}",
    role => "INTOP", meaning => "counterclockwise-contour-integral",
    variablesize_op => true);
  DefMath!("\\ointclockwiseop", None, "\u{2232}",
    role => "INTOP", meaning => "clockwise-contour-integral",
    variablesize_op => true);
  DefMath!("\\ointclockwise", None, "\u{2232}",
    role => "INTOP", meaning => "clockwise-contour-integral",
    variablesize_op => true);
  DefMath!("\\varointclockwiseop", None, "\u{2232}",
    role => "INTOP", meaning => "clockwise-contour-integral",
    variablesize_op => true);
  DefMath!("\\varointclockwise", None, "\u{2232}",
    role => "INTOP", meaning => "clockwise-contour-integral",
    variablesize_op => true);
  DefMath!("\\varointctrclockwiseop", None, "\u{2233}",
    role => "INTOP", meaning => "counterclockwise-contour-integral",
    variablesize_op => true);
  DefMath!("\\varointctrclockwise", None, "\u{2233}",
    role => "INTOP", meaning => "counterclockwise-contour-integral",
    variablesize_op => true);

  DefMath!("\\fintop", None, "\u{2A0F}",
    role => "INTOP", meaning => "average-integral",
    variablesize_op => true);
  DefMath!("\\fint", None, "\u{2A0F}",
    role => "INTOP", meaning => "average-integral",
    variablesize_op => true);

  // No unicode for these, guessing at meaning
  DefPrimitive!("\\lx@esint@landup", "-\u{25E0}-");
  DefPrimitive!("\\lx@esint@landdown", "-\u{25E1}-");
  DefMath!("\\landupintop", None,
    "\\lx@kludged{\\int{\\scriptscriptstyle\\mathchoice{\\lx@tweaked{width=0pt,xoffset=-1.1em,yoffset=0.2ex}{\\lx@esint@landup}}{\\lx@tweaked{width=0pt,xoffset=-0.9em,yoffset=0.2ex}{\\lx@esint@landup}}{\\lx@tweaked{width=0pt,xoffset=-0.9em,yoffset=0.2ex}{\\lx@esint@landup}}{\\lx@tweaked{width=0pt,xoffset=-0.9em,yoffset=0.2ex}{\\lx@esint@landup}}}}",
    role => "INTOP", meaning => "contour-integral-around-above",
    variablesize_op => true);
  DefMath!("\\landupint", None,
    "\\lx@kludged{\\int{\\scriptscriptstyle\\mathchoice{\\lx@tweaked{width=0pt,xoffset=-1.1em,yoffset=0.2ex}{\\lx@esint@landup}}{\\lx@tweaked{width=0pt,xoffset=-0.9em,yoffset=0.2ex}{\\lx@esint@landup}}{\\lx@tweaked{width=0pt,xoffset=-0.9em,yoffset=0.2ex}{\\lx@esint@landup}}{\\lx@tweaked{width=0pt,xoffset=-0.9em,yoffset=0.2ex}{\\lx@esint@landup}}}}",
    role => "INTOP", meaning => "contour-integral-around-above",
    variablesize_op => true);
  DefMath!("\\landdownintop", None,
    "\\lx@kludged{\\int{\\scriptscriptstyle\\mathchoice{\\lx@tweaked{width=0pt,xoffset=-1.1em,yoffset=0.2ex}{\\lx@esint@landdown}}{\\lx@tweaked{width=0pt,xoffset=-0.9em,yoffset=0.2ex}{\\lx@esint@landdown}}{\\lx@tweaked{width=0pt,xoffset=-0.9em,yoffset=0.2ex}{\\lx@esint@landdown}}{\\lx@tweaked{width=0pt,xoffset=-0.9em,yoffset=0.2ex}{\\lx@esint@landdown}}}}",
    role => "INTOP", meaning => "contour-integral-around-below",
    variablesize_op => true);
  DefMath!("\\landdownint", None,
    "\\lx@kludged{\\int{\\scriptscriptstyle\\mathchoice{\\lx@tweaked{width=0pt,xoffset=-1.1em,yoffset=0.2ex}{\\lx@esint@landdown}}{\\lx@tweaked{width=0pt,xoffset=-0.9em,yoffset=0.2ex}{\\lx@esint@landdown}}{\\lx@tweaked{width=0pt,xoffset=-0.9em,yoffset=0.2ex}{\\lx@esint@landdown}}{\\lx@tweaked{width=0pt,xoffset=-0.9em,yoffset=0.2ex}{\\lx@esint@landdown}}}}",
    role => "INTOP", meaning => "contour-integral-around-below",
    variablesize_op => true);
});
