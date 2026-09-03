use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // A registered binding REPLACES the raw file, so the real schooldocs.sty
  // must be loaded here first: it `\RequirePackage{xcolor}` (:32), defines
  // `titlecolor` (:100), `\subject`/`\institute`/`\seprule`… — all of which
  // the former stub skipped (`\definecolor` undefined, schooldocs-examples
  // 17 errors; Perl raw-loads it clean). The patches below are applied on
  // top. Guard: `perfect_kernel_batch54::schooldocs_binding_loads_the_real_style`.
  InputDefinitions!("schooldocs", noltxml => true, extension => Some(Cow::Borrowed("sty")));
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
