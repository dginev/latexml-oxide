use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: subfig.sty.ltxml — 118 lines
  // Subfigure/subtable support with counter management

  // Perl L28: \refstepcounter@noreset — steps counter without resetting subcounters
  // (simplified: just step the counter normally since we don't track reset chains)
  DefMacro!("\\refstepcounter@noreset{}", "\\refstepcounter{#1}");

  // Perl L31-32: \newsubfloat — creates subfloat machinery for a float type
  // We pre-define figure/table; generic \newsubfloat is a stub
  DefMacro!("\\newsubfloat[]{}", "");

  // \subfloat — Perl L69-79
  DefMacro!("\\subfloat",
    "\\ifx\\@captype\\@undefined\\expandafter\\@gobble\\else\\expandafter\\@firstofone\\fi{\\sf@subfloat}");
  DefMacro!("\\sf@subfloat",
    "\\csname lx@subfloat@\\@captype\\endcsname");

  // \subref — Perl L82-84
  DefMacro!("\\subref",       "\\@ifstar\\sf@@subref\\sf@subref");
  DefMacro!("\\sf@subref{}",  "\\ref{sub@#1}");
  DefMacro!("\\sf@@subref{}", "\\pageref{sub@#1}");

  // Caption setup stubs — Perl L86-90
  DefMacro!("\\DeclareCaptionListOfFormat{}{}", "");
  DefMacro!("\\DeclareSubrefFormat{}{}", "");
  DefMacro!("\\listsubcaptions", "");
  DefMacro!("\\captionsetup[]{}", "");
  DefMacro!("\\clearcaptionsetup{}", "");
  DefConditional!("\\ifmaincaptiontop");
  DefConditional!("\\iflx@donecaption");

  // Counter setup — Perl L91-93
  RawTeX!("\\@ifundefined{c@subfigure}{\\newcounter{subfigure}[figure]}{}");
  RawTeX!("\\@ifundefined{c@subtable}{\\newcounter{subtable}[table]}{}");
  NewCounter!("subfigure@save");
  NewCounter!("subtable@save");

  // Subfigure display macros
  DefMacro!("\\thesubfigure", "\\alph{subfigure}");
  DefMacro!("\\thesubtable", "\\alph{subtable}");
  DefMacro!("\\fnum@subfigure", "(\\thesubfigure)");
  DefMacro!("\\fnum@subtable", "(\\thesubtable)");
  DefMacro!("\\p@subfigure", "\\thefigure");
  DefMacro!("\\p@subtable", "\\thetable");

  // \lx@subfloat@figure — Perl L45-60
  DefMacro!("\\lx@subfloat@figure[][]{}",
    "\\iflx@donecaption\\else\\refstepcounter@noreset{\\@captype}\\fi\\begin{lx@subfloat@@figure}#3\\caption{#1}\\end{lx@subfloat@@figure}\\iflx@donecaption\\else\\addtocounter{\\@captype}{\\m@ne}\\fi");
  DefEnvironment!("{lx@subfloat@@figure}",
    "^ <ltx:figure xml:id='#id'>#tags#body</ltx:figure>",
    mode => "internal_vertical"
  );

  // \lx@subfloat@table — Perl L45-60 (table variant)
  DefMacro!("\\lx@subfloat@table[][]{}",
    "\\iflx@donecaption\\else\\refstepcounter@noreset{\\@captype}\\fi\\begin{lx@subfloat@@table}#3\\caption{#1}\\end{lx@subfloat@@table}\\iflx@donecaption\\else\\addtocounter{\\@captype}{\\m@ne}\\fi");
  DefEnvironment!("{lx@subfloat@@table}",
    "^ <ltx:table xml:id='#id'>#tags#body</ltx:table>",
    mode => "internal_vertical"
  );

  // \ContinuedFloat — Perl L98-102
  // Perl decrements the parent counter AND restores the sub-counter from
  // sub<captype>@save. Prior Rust only decremented the parent, leaving
  // the sub-counter at whatever value the prior float ended on, so a
  // \ContinuedFloat followed by a \subfloat would keep counting from the
  // stale sub index instead of rewinding.
  RawTeX!(r"\def\lx@subfig@continue@restore#1{\setcounter{sub#1}{\value{sub#1@save}}}");
  DefMacro!("\\ContinuedFloat",
    r"\addtocounter{\@captype}{\m@ne}\expandafter\lx@subfig@continue@restore\expandafter{\@captype}");
});
