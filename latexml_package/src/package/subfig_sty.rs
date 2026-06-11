use crate::{
  engine::latex_constructs::{after_float, before_float},
  prelude::*,
};

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

  // Perl L30-62: \newsubfloat — installs subfloat machinery for a float type.
  //
  // For figure / table the machinery is pre-baked below (counters,
  // \thesub<name>, \fnum@sub<name>, \p@sub<name>, \lx@subfloat@<name>,
  // and the lx@subfloat@@<name> environment). Perl's RawTeX trailer
  // calls \newsubfloat{figure} and \newsubfloat{table} at end-of-load
  // guarded by \@ifundefined{c@subfigure}/\@ifundefined{c@subtable}, so
  // those calls are no-ops once our pre-bake has run — exact parity.
  //
  // For caller-defined float types (e.g. \newsubfloat{algorithm}),
  // Perl dynamically issues NewCounter + Let + DefMacroI + DefEnvironmentI
  // with $name substituted. The dynamic DefEnvironmentI requires a
  // runtime "install_environment" API that is not yet exposed in
  // latexml_core (state::install_definition handles macros/primitives
  // but not constructor-with-template environments). Documented as a
  // known missing-API task rather than masked as a stub: no paper in
  // the 7898-arxiv 2026-04-24 sandbox exercises this path.
  def_macro_noop("\\newsubfloat[]{}")?;

  // \subfloat — Perl L69-79
  DefMacro!("\\subfloat",
    "\\ifx\\@captype\\@undefined\\expandafter\\@gobble\\else\\expandafter\\@firstofone\\fi{\\sf@subfloat}");
  DefMacro!("\\sf@subfloat",
    "\\csname lx@subfloat@\\@captype\\endcsname");
  // subfig L73: \sidesubfloat — side-by-side subfloat variant. Real def
  // wraps \subfloat with a minipage and lineup arg. Stub as plain
  // \subfloat so the subfloat machinery still kicks in. Witness 2309.00194.
  DefMacro!("\\sidesubfloat", "\\subfloat");

  // \subref — Perl L82-84
  DefMacro!("\\subref",       "\\@ifstar\\sf@@subref\\sf@subref");
  DefMacro!("\\sf@subref{}",  "\\ref{sub@#1}");
  DefMacro!("\\sf@@subref{}", "\\pageref{sub@#1}");

  // Caption setup stubs — Perl L86-90
  def_macro_noop("\\DeclareCaptionListOfFormat{}{}")?;
  def_macro_noop("\\DeclareSubrefFormat{}{}")?;
  def_macro_noop("\\listsubcaptions")?;
  def_macro_noop("\\captionsetup[]{}")?;
  def_macro_noop("\\clearcaptionsetup{}")?;
  DefConditional!("\\ifmaincaptiontop");
  DefConditional!("\\iflx@donecaption");

  // Counter setup — Perl L36 uses NewCounter with `idprefix => 'sf'` and
  // `idwithin => $name` so subfigure/subtable get xml:ids like `F1.sf2`,
  // `T3.sf1`. The prior Rust port routed through `\newcounter{subfigure}
  // [figure]` via RawTeX, which skipped LaTeXML's id machinery entirely,
  // leaving subfigures with bare numeric ids that collided across floats.
  // Call NewCounter directly with the Perl options; the `\@ifundefined`
  // guard is dropped because NewCounter is itself idempotent (Perl L36
  // reads as a fresh-or-overwrite, mirroring `\newcounter` semantics).
  NewCounter!("subfigure", "figure", idprefix => "sf", idwithin => "figure");
  NewCounter!("subtable", "table", idprefix => "sf", idwithin => "table");
  NewCounter!("subfigure@save");
  NewCounter!("subtable@save");

  // Perl L37/Perl-tail RawTeX `\@ifundefined{c@subfigure}{\newsubfloat
  // {figure}}{}` (and the table variant) executes \newsubfloat which
  // among other things does `Let('\ext@sub' . $name, '\ext@' . $name)`.
  // Since the Rust `\newsubfloat` is a stub, the figure/table cases are
  // pre-baked here so `\caption` machinery resolving `\ext@subfigure`
  // (e.g. via `subfig` callers) finds the figure-extension list. Sandbox
  // 0911.3405 (subfig + eptcs) cluster: 119 papers undefined
  // `\ext@subfigure`.
  Let!("\\ext@subfigure", "\\ext@figure");
  Let!("\\ext@subtable", "\\ext@table");

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
