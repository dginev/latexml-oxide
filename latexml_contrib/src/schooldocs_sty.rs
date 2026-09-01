use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // schooldocs.sty defines \subtitle only INSIDE \fancypagestyle{exam}{...}
  // (schooldocs.sty L312-315), whose body LaTeXML discards (fancyhdr_sty.rs
  // \fancypagestyle noop, matching Perl fancyhdr.sty.ltxml L56 — shared
  // failure). Hoist the semantic part: route to the standard frontmatter
  // subtitle. Witness: doc/latex/schooldocs/schooldocs-examples.tex L230
  // (perfect-kernel corpus).
  DefMacro!("\\subtitle{}", "\\lx@add@subtitle{#1}");
  // Presentational style hooks (schooldocs.sty \*style family).
  def_macro_noop("\\titlestyle")?;
  def_macro_noop("\\subtitlestyle")?;
  def_macro_noop("\\sectionstyle")?;
});
