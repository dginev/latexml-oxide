//! Stub for jair.sty (Journal of AI Research style).
//!
//! JAIR uses JMLR-style author lists with `\name Name \email addr \\
//! \addr Affiliation`. The raw jair.sty doesn't define these helpers
//! (it expects the user to load jair-conventions separately); without
//! them the author block lands as cascade of `undefined:\\name /\\email
//! /\\addr` per author. Witness 2309.16146 (52 errors from 5+ authors).
//!
//! Mirror jmlr2e_sty's approach: empty stubs so the inline-flow text
//! survives (the surrounding `\\` line-breaks structure the authors
//! visually even without explicit role markup).
use latexml_package::prelude::*;


LoadDefinitions!({
  // \name <text> — author-name marker (no arg in JAIR).
  def_macro_noop("\\name")?;
  // \addr <text> — affiliation marker.
  def_macro_noop("\\addr")?;
  // \email <text> — email marker.
  def_macro_noop("\\email")?;
  // \And — author separator (JMLR/JAIR convention).
  DefMacro!("\\And", " \\hskip 2em ");
  // \AND — same in caps (some templates).
  DefMacro!("\\AND", " \\hskip 2em ");

  // jair.sty L244: \jairheading{vol}{year}{pages}{submitted}{published}
  // 5-arg metadata setter for running header. Preserve as frontmatter note.
  DefMacro!("\\jairheading{}{}{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=jair-heading]{Vol. #1 (#2), #3 — sub: #4, pub: #5}");
  // jair.sty L260: \ShortHeadings{title}{authors} — running-page short forms.
  def_macro_noop("\\ShortHeadings{}{}")?;
  // jair.sty L256: \firstpageno{N} — page counter setter; no-op for HTML.
  def_macro_noop("\\firstpageno{}")?;
  // {acks} env — JAIR acknowledgements wrapper. Mirror sagej's pattern.
  DefEnvironment!("{acks}",
    "<ltx:acknowledgements>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  // \@BBN — internal bibliography helper used by JAIR's bbl. No-op.
  def_macro_noop("\\@BBN")?;
});
