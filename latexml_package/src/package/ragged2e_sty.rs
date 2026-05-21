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
  // ragged2e's CapitalCase env variants must alias the lowercase
  // LaTeX *envs* (which carry the correct `internal_vertical` mode
  // via DefEnvironment), NOT the bare command forms. Mapping to the
  // command (`\center`/`\flushright`) skips the env-mode push, so
  // `\end{FlushRight}` finds an unmatched mode and emits
  // "Attempt to end mode `internal_vertical` in `restricted_horizontal`".
  // Witness 2305.12077.
  DefMacro!(T_CS!("\\begin{Center}"),     None, "\\begin{center}");
  DefMacro!(T_CS!("\\end{Center}"),       None, "\\end{center}");
  DefMacro!(T_CS!("\\begin{FlushLeft}"),  None, "\\begin{flushleft}");
  DefMacro!(T_CS!("\\end{FlushLeft}"),    None, "\\end{flushleft}");
  DefMacro!(T_CS!("\\begin{FlushRight}"), None, "\\begin{flushright}");
  DefMacro!(T_CS!("\\end{FlushRight}"),   None, "\\end{flushright}");
});
