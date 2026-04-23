use crate::engine::latex_constructs::{after_float, before_float};
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: subfig.sty.ltxml — 118 lines
  // Subfigure/subtable support with counter management

  // Perl L26-27: \refstepcounter@noreset passes noreset=1 to RefStepCounter,
  // which steps the counter but skips the usual subcounter reset. Rust
  // previously aliased it to plain \refstepcounter (which DOES reset
  // subcounters), so a \subfloat inside an uncaptioned float temporarily
  // stepping the parent counter would zero out the subcounter the next
  // \subfloat was trying to increment.
  DefPrimitive!("\\refstepcounter@noreset{}", sub[(cs)] {
    let cs_expanded = Expand!(cs).to_string();
    RefStepCounter!(&cs_expanded, true)?;
  });

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

  // \lx@subfloat@figure — Perl L45-60. Perl's afterDigest on the inner
  // environment copies the final subfigure counter into subfigure@save so
  // \ContinuedFloat can restore it. Rust embeds `\setcounter{subfigure@save}
  // {\value{subfigure}}` at the end of the expansion; runs at the same
  // moment (after caption digests, so sub counter is final).
  DefMacro!("\\lx@subfloat@figure[][]{}",
    "\\iflx@donecaption\\else\\refstepcounter@noreset{\\@captype}\\fi\\begin{lx@subfloat@@figure}#3\\caption{#1}\\end{lx@subfloat@@figure}\\iflx@donecaption\\else\\addtocounter{\\@captype}{\\m@ne}\\fi\\setcounter{subfigure@save}{\\value{subfigure}}");
  // Perl L58-60: beforeDigest=>{beforeFloat('subfigure')}, afterDigest=>
  // {afterFloat + SetCounter('subfigure@save', CounterValue('subfigure'))}.
  // beforeFloat sets \@captype='subfigure' so the nested \caption steps the
  // sub-counter and emits sub-id labels. afterFloat finalizes caption
  // properties on the whatsit. The subfigure@save counter sync is still
  // handled in the macro-body `\setcounter{subfigure@save}{\value{subfigure}}`.
  DefEnvironment!("{lx@subfloat@@figure}",
    "^ <ltx:figure xml:id='#id'>#tags#body</ltx:figure>",
    mode => "internal_vertical",
    before_digest => { before_float("subfigure", None); },
    after_digest  => sub[whatsit] { after_float(whatsit); }
  );

  // \lx@subfloat@table — Perl L45-60 (table variant)
  DefMacro!("\\lx@subfloat@table[][]{}",
    "\\iflx@donecaption\\else\\refstepcounter@noreset{\\@captype}\\fi\\begin{lx@subfloat@@table}#3\\caption{#1}\\end{lx@subfloat@@table}\\iflx@donecaption\\else\\addtocounter{\\@captype}{\\m@ne}\\fi\\setcounter{subtable@save}{\\value{subtable}}");
  DefEnvironment!("{lx@subfloat@@table}",
    "^ <ltx:table xml:id='#id'>#tags#body</ltx:table>",
    mode => "internal_vertical",
    before_digest => { before_float("subtable", None); },
    after_digest  => sub[whatsit] { after_float(whatsit); }
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
