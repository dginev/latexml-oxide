use crate::prelude::*;

/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}


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
  DefMacro!("\\ltx@@cmidrule[Dimension] SkipMatch:( Until:){}", "\\cline{#3}");
  DefMacro!("\\ltx@cmidrule[Dimension]{}", "\\cline{#2}");

  // add vspace
  def_macro_noop("\\addlinespace[Dimension]")?;
  // adjust spacing to make double line
  def_macro_noop("\\morecmidrules")?;
  // \specialrule{thickness}{above}{below}
  DefMacro!("\\specialrule{Dimension}{Dimension}{Dimension}", "\\hline");

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
