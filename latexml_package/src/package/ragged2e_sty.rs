use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Just copy the basic defns from LaTeX
  Let!("\\Centering",   "\\centering");
  Let!("\\RaggedRight", "\\raggedright");
  Let!("\\RaggedLeft",  "\\raggedleft");
  Let!("\\Center",      "\\center");
  Let!("\\endCenter",   "\\endcenter");
  Let!("\\FlushLeft",   "\\flushleft");
  Let!("\\FlushRight",  "\\flushright");
  DefMacro!("\\justifying", None);

  DefRegister!("\\CenteringLeftskip",      Dimension(0));
  DefRegister!("\\RaggedLeftLeftskip",     Dimension(0));
  DefRegister!("\\RaggedRightLeftskip",    Dimension(0));
  DefRegister!("\\CenteringRightskip",     Dimension(0));
  DefRegister!("\\RaggedLeftRightskip",    Dimension(0));
  DefRegister!("\\RaggedRightRightskip",   Dimension(0));
  DefRegister!("\\CenteringParfillskip",   Dimension(0));
  DefRegister!("\\RaggedLeftParfillskip",  Dimension(0));
  DefRegister!("\\RaggedRightParfillskip", Dimension(0));
  DefRegister!("\\JustifyingParfillskip",  Dimension(0));
  DefRegister!("\\CenteringParindent",     Dimension(0));
  DefRegister!("\\RaggedLeftParindent",    Dimension(0));
  DefRegister!("\\RaggedRightParindent",   Dimension(0));
  DefRegister!("\\JustifyingParindent",    Dimension(0));

  // ragged2e L292: \newenvironment{justify}{...}{...}
  // Witness 2406.15288.
  DefMacro!(T_CS!("\\begin{justify}"), None, "");
  DefMacro!(T_CS!("\\end{justify}"),   None, "");
  DefMacro!(T_CS!("\\begin{Center}"),  None, "\\center");
  DefMacro!(T_CS!("\\end{Center}"),    None, "\\endcenter");
  DefMacro!(T_CS!("\\begin{FlushLeft}"),  None, "\\flushleft");
  DefMacro!(T_CS!("\\end{FlushLeft}"),    None, "\\endflushleft");
  DefMacro!(T_CS!("\\begin{FlushRight}"), None, "\\flushright");
  DefMacro!(T_CS!("\\end{FlushRight}"),   None, "\\endflushright");
});
