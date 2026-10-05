use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // ragged2e.sty:48-54: `newcommands` clears `\if@raggedtwoe@originalcommands`, which is the default (:119).
  Digest!("\\newif\\if@raggedtwoe@originalcommands\\@raggedtwoe@originalcommandstrue")?;
  DeclareOption!("OriginalCommands", "\\@raggedtwoe@originalcommandstrue");
  DeclareOption!("originalcommands", "\\@raggedtwoe@originalcommandstrue");
  DeclareOption!("NewCommands",      "\\@raggedtwoe@originalcommandsfalse");
  DeclareOption!("newcommands",      "\\@raggedtwoe@originalcommandsfalse");
  // ragged2e.sty:93-110.
  for option in ["newparameters", "NewParameters", "originalparameters", "OriginalParameters", "raggedrightboxes",
    "footnotes", "document"] {
    DeclareOption!(option, None);
  }
  ProcessOptions!();
  // Under `newcommands` ragged2e.sty:298-311 saves LaTeX's own commands as `\LaTeXcentering` & co. before pointing the
  // lowercase names at its own; a class that loads it so restores them (sbc20.cls:518, 523 `\let\centering
  // \LaTeXcentering`). Here the lowercase commands stay LaTeX's, so the saves are those (2205.12270; Perl lacks them,
  // KPE #481).
  Digest!("\\if@raggedtwoe@originalcommands\\else
    \\let\\LaTeXcentering\\centering \\let\\LaTeXraggedleft\\raggedleft \\let\\LaTeXraggedright\\raggedright
    \\let\\LaTeXcenter\\center \\let\\endLaTeXcenter\\endcenter
    \\let\\LaTeXflushleft\\flushleft \\let\\endLaTeXflushleft\\endflushleft
    \\let\\LaTeXflushright\\flushright \\let\\endLaTeXflushright\\endflushright
  \\fi")?;
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

  // ragged2e.sty:292-297 `\newenvironment{justify}{\trivlist\justifying\item\relax}{\endtrivlist}`: its text is a
  // paragraph of its own, and `\begin`/`\end` group it (a `\begin{justify}` no-op ran it into the text before,
  // "text.More", and let its font changes leak out; witness 2406.15288). Papers also call `\justify` bare as a switch
  // (2204.13885, 1903.04078, 1909.10090's sigchi-ext.cls; "undefined \justify" in 16 papers of run 329).
  DefMacro!("\\justify",    "\\par");
  DefMacro!("\\endjustify", "\\par");
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
