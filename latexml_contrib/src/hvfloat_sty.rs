use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // hvfloat.sty — captions beside/after/around float objects.
  //
  // Raw-impossibility justification (perfect-kernel README protocol): raw
  // hvfloat BUILDS its floats manually out of (mini)boxes with `\@captype`
  // set (hvfloat.sty L545+ `\do@hvFloat` box assembly), so `\caption` runs
  // with NO float element open — our (and Perl's) `^^<ltx:caption>` float-up
  // finds no legal ancestor and every caption errors
  // `malformed:ltx:(toc)caption isn't allowed in <ltx:block>` (43 docs, the
  // largest single malformed cluster of sweep 16; real LaTeX only requires
  // `\@captype`). The box choreography is pure page layout; the SEMANTIC
  // content is exactly a float + object + caption + label — map it to the
  // real environment.
  //
  // \hvFloat*?[keys]{figure|table}{object}[shortcap]{caption}{label}
  // (hvfloat.sty L535-550). Placement keys (capPos, rotation, fullpage…)
  // are presentational — dropped.
  // hvfloat.sty's real dependency chain (caption/graphicx/…).
  RequirePackage!("caption");
  RequirePackage!("graphicx");
  DefMacro!(
    "\\hvFloat OptionalMatch:* []{}{}[]{}{}",
    "\\begin{#3}\\centering #4\\caption[#5]{#6}\\label{#7}\\end{#3}"
  );
  // Companion setup macros (presentational).
  def_macro_noop("\\hvFloatSet{}")?;
  def_macro_noop("\\hvFloatSetDefaults")?;
  def_macro_noop("\\hvSet{}")?;
  // Two-object variant \hvFloat with sub-floats is rare in the manuals;
  // the main form covers the corpus. Registers used by the demos:
  DefRegister!("\\hvObjectWidth" => Dimension!("0pt"));
  DefRegister!("\\hvCapWidth" => Dimension!("0pt"));
});
