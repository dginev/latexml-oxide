use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\subfloat",
    "\\ifx\\@captype\\@undefined\\expandafter\\@gobble\\else\\expandafter\\@firstofone\\fi{\\sf@subfloat}");
  DefMacro!("\\sf@subfloat",
    "\\csname lx@subfloat@\\@captype\\endcsname");
  DefMacro!("\\subref",       "\\@ifstar\\sf@@subref\\sf@subref");
  DefMacro!("\\sf@subref{}",  "\\ref{sub@#1}");
  DefMacro!("\\sf@@subref{}", "\\pageref{sub@#1}");
  DefMacro!("\\DeclareCaptionListOfFormat{}{}", "");
  DefMacro!("\\DeclareSubrefFormat{}{}", "");
  DefMacro!("\\listsubcaptions", "");
  DefMacro!("\\captionsetup[]{}", "");
  DefMacro!("\\clearcaptionsetup{}", "");
  DefConditional!("\\ifmaincaptiontop");
  DefConditional!("\\iflx@donecaption");
  RawTeX!("\\@ifundefined{c@subfigure}{\\newcounter{subfigure}[figure]}{}");
  RawTeX!("\\@ifundefined{c@subtable}{\\newcounter{subtable}[table]}{}");

  // Perl L30-70: \newsubfloat and subfloat environments
  // Pre-define subfloat environments for figure and table (the two common cases)
  DefMacro!("\\thesubfigure", "\\alph{subfigure}");
  DefMacro!("\\thesubtable", "\\alph{subtable}");
  DefMacro!("\\fnum@subfigure", "(\\thesubfigure)");
  DefMacro!("\\fnum@subtable", "(\\thesubtable)");

  // Perl L45-60: \lx@subfloat@figure — creates a sub-figure with optional caption
  DefMacro!("\\lx@subfloat@figure[][]{}",
    "\\begin{lx@subfloat@@figure}#3\\caption{#1}\\end{lx@subfloat@@figure}");
  DefEnvironment!("{lx@subfloat@@figure}",
    "^ <ltx:figure xml:id='#id'>#tags#body</ltx:figure>",
    mode => "internal_vertical"
  );

  DefMacro!("\\lx@subfloat@table[][]{}",
    "\\begin{lx@subfloat@@table}#3\\caption{#1}\\end{lx@subfloat@@table}");
  DefEnvironment!("{lx@subfloat@@table}",
    "^ <ltx:table xml:id='#id'>#tags#body</ltx:table>",
    mode => "internal_vertical"
  );

  // \ContinuedFloat — Perl L77-87
  DefMacro!("\\ContinuedFloat", "");

  // Perl L91-93: \newsubfloat creates dynamic subfloat envs
  // We pre-define figure/table above; for others, stub it
  DefMacro!("\\newsubfloat[]{}", "");
});
