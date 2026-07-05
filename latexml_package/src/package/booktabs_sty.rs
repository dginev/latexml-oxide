use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: booktabs.sty.ltxml
  // Adjust thickness of rules? Currently no support for variable thickness.

  // \toprule[thickness]  doubled
  DefMacro!("\\toprule[Dimension]", "\\hline\\hline");
  // \midrule[thickness]
  DefMacro!("\\midrule[Dimension]", "\\hline");
  // \bottomrule[thickness] doubled
  DefMacro!("\\bottomrule[Dimension]", "\\hline\\hline");

  // \cmidrule[thickness](trim){col-col}
  DefMacro!("\\@afterfi Until:\\fi", "\\fi#1");
  DefMacro!("\\cmidrule[]",
    r"\@ifnextchar({\ifx.#1.\expandafter\ltx@@cmidrule\else\@afterfi\ltx@@cmidrule[#1]\fi}{\ifx.#1.\expandafter\ltx@cmidrule\else\@afterfi\ltx@cmidrule[#1]\fi}"
  );
  // The cmidrule helpers draw the partial rule via `\cline`. They route through
  // a PRIVATE saved copy (`\ltx@saved@cline`, captured at load below) rather than
  // the public `\cline`, so a document `\let\cline\cmidrule` (a common idiom to
  // make `\cline` render like a booktabs rule) does NOT create a
  // `\cmidrule`→`\cline`→`\cmidrule` infinite expansion. Real LaTeX avoids the
  // cycle because its `\cmidrule` draws the rule directly; LaTeXML's simplified
  // `\cmidrule`→`\cline` binding (shared with Perl, which hangs on this — see
  // KNOWN_PERL_ERRORS) would otherwise loop until the conditional limit.
  // Witnesses: arXiv 2506.23179, 2511.17056 (both `\let\cline\cmidrule`).
  // Output-neutral for ordinary `\cmidrule` (saved CS == `\cline` at load).
  DefMacro!("\\ltx@@cmidrule[Dimension] SkipMatch:( Until:){}", "\\ltx@saved@cline{#3}");
  DefMacro!("\\ltx@cmidrule[Dimension]{}", "\\ltx@saved@cline{#2}");

  // add vspace
  def_macro_noop("\\addlinespace[Dimension]")?;
  // adjust spacing to make double line
  def_macro_noop("\\morecmidrules")?;
  // \specialrule{thickness}{above}{below}
  DefMacro!("\\specialrule{Dimension}{Dimension}{Dimension}", "\\hline");

  // Capture the real `\cline` at load time (before any document redefinition)
  // so `\cmidrule` can draw its rule without depending on the live `\cline`.
  TeX!(r"\let\ltx@saved@cline\cline");

  TeX!(r"\newdimen\heavyrulewidth
\newdimen\lightrulewidth
\newdimen\cmidrulewidth
\newdimen\belowrulesep
\newdimen\belowbottomsep
\newdimen\aboverulesep
\newdimen\abovetopsep
\newdimen\cmidrulesep
\newdimen\cmidrulekern
\newdimen\defaultaddspace
\heavyrulewidth=.08em
\lightrulewidth=.05em
\cmidrulewidth=.03em
\belowrulesep=.65ex
\belowbottomsep=0pt
\aboverulesep=.4ex
\abovetopsep=0pt
\cmidrulesep=\doublerulesep
\cmidrulekern=.5em
\defaultaddspace=.5em
\newcount\@cmidla
\newcount\@cmidlb
\newdimen\@aboverulesep
\newdimen\@belowrulesep
\newcount\@thisruleclass
\newcount\@lastruleclass
");
});
