use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: appendix.sty.ltxml — 103 lines; Rust mirrors the macro set
  // one-for-one (L32-100). Perl's own "INCOMPLETE IMPLEMENTATION" header
  // at L20-23 is inherited, not a Rust gap — no further entries exist
  // upstream. `\appendixpage` / `\addappheadtotoc` are commented out in
  // both files (L28-30 Perl, absent in Rust); `\phantomsection` and
  // `\restoreapp` likewise.

  DefMacro!("\\appendixname",     "Appendix");
  DefMacro!("\\appendixtocname",  "Appendices");
  DefMacro!("\\appendixpagename", "Appendices");

  // Whether the entry in toc gets page number; Ignorable
  def_macro_noop("\\appendicestocpagenum")?;
  def_macro_noop("\\noappendicestocpagenum")?;

  // Switches, mostly ignorable(?)
  DefConditional!("\\if@dotoc@pp");
  DefConditional!("\\if@dotitle@pp");
  DefConditional!("\\if@dotitletoc@pp");
  DefConditional!("\\if@dohead@pp");
  DefConditional!("\\if@dopage@pp");
  // appendix.sty:36-40 (whether the class has chapters) and the internals that classes copying its `appendices`
  // call when they re-dress it (sn-jnl.cls; wlscirep.cls:221-245, 2606.09094): `\clear@ppage` (:100-101, a page
  // break: nothing here) and `\@resets@pp`/`\@resets@ppsub` (:181-205, :246-264), the counter resets and letter
  // numbering `\appendices`/`\subappendices` make too; their saved-section and hyperref steps have no XML
  // counterpart.
  DefConditional!("\\if@chapter@pp");
  RawTeX!(r"\@ifundefined{chapter}{}{\@chapter@pptrue}
\newcounter{@pps}
\newcommand{\@resets@pp}{\par\stepcounter{@pps}\setcounter{section}{0}%
  \if@chapter@pp\setcounter{chapter}{0}\renewcommand\@chapapp{\appendixname}\renewcommand\thechapter{\@Alph\c@chapter}%
  \else\setcounter{subsection}{0}\renewcommand\thesection{\@Alph\c@section}\fi}
\newcommand{\@resets@ppsub}{\par\stepcounter{@pps}%
  \if@chapter@pp\setcounter{section}{0}\renewcommand{\thesection}{\setthesection}%
  \else\setcounter{subsection}{0}\renewcommand{\thesubsection}{\setthesubsection}\fi}");
  def_macro_noop("\\clear@ppage")?;

  DefMacro!("\\appendixtocon",       "\\@dotoc@pptrue");
  DefMacro!("\\appendixtocoff",      "\\@dotoc@ppfalse");
  DefMacro!("\\appendixpageon",      "\\@dopage@pptrue");
  DefMacro!("\\appendixpageoff",     "\\@dopage@ppfalse");
  DefMacro!("\\appendixtitleon",     "\\@dotitle@pptrue");
  DefMacro!("\\appendixtitleoff",    "\\@dotitle@ppfalse");
  DefMacro!("\\appendixtitletocon",  "\\@dotitletoc@pptrue");
  DefMacro!("\\appendixtitletocoff", "\\@dotitletoc@ppfalse");
  DefMacro!("\\appendixheaderon",    "\\@dohead@pptrue");
  DefMacro!("\\appendixheaderoff",   "\\@dohead@ppfalse");

  DefMacro!("\\setthesection",    "\\Alph{section}");
  DefMacro!("\\setthesubsection", "\\thesection.\\Alph{subsection}");

  // \appendixpage and \addappheadtotoc are layout-only — they
  // typeset a "page break" page-header in the printed paper. No XML
  // analog; no-op stubs. (Perl L28-30 leaves them commented out;
  // matching by stubbing keeps the diagnostics quiet.)
  // Witnesses 2406.01767, 2406.13839.
  def_macro_noop("\\appendixpage")?;
  def_macro_noop("\\addappheadtotoc")?;

  DefPrimitive!("\\lx@pp@appendix@begin", {
    if has_chapter_counter() {
      begin_appendices("chapter");
    } else {
      begin_appendices("section");
    }
  });

  DefConstructor!("\\lx@pp@appendix@end", sub[document, _args, _props] {
    document.maybe_close_element("ltx:appendix")?;
  },
    before_digest => {
      end_appendices();
    }
  );

  // Adjust numbering!!!
  DefPrimitive!("\\lx@pp@subappendix@begin", {
    if has_chapter_counter() {
      begin_appendices("section");
    } else {
      begin_appendices("subsection");
    }
  });

  DefMacro!("\\appendices",
    r"\lx@pp@appendix@begin\if@dotoc@pp\addappheadtotoc\fi\if@dopage@pp\appendixpage\fi\if@dotitle@pp\def\fnum@appendix{\lx@refnum@compose{\appendixname}{\lx@the@@{appendix}}}\fi\if@dotitle@pp\def\fnum@toc@appendix{\lx@refnum@compose{\appendixname}{\lx@the@@{appendix}}}\fi"
  );

  // These must END appendices!!!!
  // AND CLOSE an open appendix!
  DefMacro!("\\endappendices", "\\lx@pp@appendix@end");

  DefMacro!("\\subappendices",    "\\lx@pp@subappendix@begin");
  DefMacro!("\\endsubappendices", "\\lx@pp@appendix@end");
});
