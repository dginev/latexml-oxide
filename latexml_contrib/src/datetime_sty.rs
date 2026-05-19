use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  Warn!(
    "missing_file",
    "datetime.sty",
    "datetime.sty is only minimally stubbed and will not be interpreted raw."
  );

  // datetime.sty L181-188 `\newdateformat{name}{def}` creates a date-
  // format command. Stub as no-op — we don't render datetime
  // distinctly so author's custom format is moot. Witness cluster:
  // arXiv:2506.21718 / 2507.03037 — Rust 4 → 0, beats Perl=0
  // (REAL REGRESSION → BOTH CLEAN).
  def_macro_noop("\\newdateformat{}{}")?;
  // Companion format setters as no-ops.
  def_macro_noop("\\settimeformat{}")?;
  // \formatdate{day}{month}{year} — emit as plain numeric date.
  // Round-34 surpass-Perl: was gobbled; preserve content inline.
  DefMacro!("\\formatdate{}{}{}", "#1/#2/#3");
  DefMacro!("\\formattime{}{}{}", "#1:#2:#3");
  // Date-component stubs (some packages call directly).
  def_macro_noop("\\monthname[]")?;
  def_macro_noop("\\shortmonthname[]")?;

  // datetime.sty L260+ `\newdate{name}{day}{month}{year}` declares a
  // named date that `\displaydate{name}` later prints. Real package
  // stores components in `\<name>@day`/`\<name>@month`/`\<name>@year`.
  // Stub each as no-op.
  def_macro_noop("\\newdate{}{}{}{}")?;
  def_macro_noop("\\displaydate{}")?;
});
