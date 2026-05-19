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
  RequirePackage!("revtex3_support");
  state::assign_value("\\text:locked", Stored::None, Some(Scope::Global));
  RequirePackage!("longtable");
  RequirePackage!("psfig");
  def_macro_noop("\\lefthead{}")?;
  def_macro_noop("\\righthead{}")?;

  // Perl aipproc.sty.ltxml does NOT define \references — Perl behaves
  // lossy-silent when papers use `\begin{references}…\bibitem` under
  // aipproc (it drops the whole bibliography, reporting "No obvious
  // problems"). Rust's stricter validator surfaces the malformation
  // as Error:malformed:ltx:bibitem "…isn't allowed in <ltx:section>".
  //
  // Rust-over-Perl improvement: alias `\references` / `\endreferences`
  // to `\thebibliography` / `\endthebibliography`. This routes the
  // `\bibitem`s through the established bibliography machinery
  // (beginBibliography, item autoclose, etc.) and preserves the
  // bibliography content in the output — better than Perl's silent
  // drop. Fixes 4 papers in the 10k sandbox Class D bibitem-aipproc
  // cluster: astro-ph9711070, cond-mat0109365, nucl-ex9706010,
  // nucl-th0010030. See docs/SANDBOX_TRIAGE.md Class D sub-pattern.
  DefMacro!("\\references", "\\thebibliography{}");
  Let!("\\endreferences", "\\endthebibliography");
  Let!("\\reference", "\\bibitem");
});
