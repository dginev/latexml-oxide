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
  // Perl: lineno.sty.ltxml — stub (line numbering not meaningful for XML)
  DefEnvironment!("{linenumbers*}[Number]",         "#body");
  DefEnvironment!("{runninglinenumbers*}[Number]",  "#body");
  DefEnvironment!("{pagewiselinenumbers*}[Number]", "#body");
  DefEnvironment!("{linenomath}",                   "#body");
  DefEnvironment!("{linenomath*}",                  "#body");
  // Real lineno.sty also defines control sequences `\linenomath`,
  // `\linenomathWithnumbers`, `\linenomathNonumbers` (raw-load
  // sees these as macros). Other packages — eccv.sty, journal templates —
  // test them with `\ifx\linenomath\linenomathWithnumbers` to switch
  // between AMS-math styles. Without explicit defs here, all three resolve
  // to `\relax` and the `\ifx` test is TRUE — the then-branch fires
  // `\patchcmd\linenomathAMS{...}` which is undefined → cascade of
  // `\else` / `\fi` mismatch (27 of 44 wp4 \else-error papers use eccv).
  // Make them three *distinct* no-op macros so the `\ifx` test picks the
  // else-branch reliably, matching the no-linenumbers default.
  // Don't redefine `\linenomath` / `\endlinenomath` — those are the
  // env-begin/env-end macros set up by DefEnvironment above. We DO
  // define the two "style switch" macros that real lineno provides,
  // with distinct bodies so journal-template `\ifx\linenomath\linenomathWithnumbers`
  // tests reliably pick the no-linenumbers branch.
  DefMacro!("\\linenomathWithnumbers", "\\relax");
  DefMacro!("\\linenomathNonumbers",   "\\@empty");
  // \internallinenumbers (lineno.sty) — adds line numbers inside the
  // environment body. Also gets a starred form auto-defined via
  // `\@namedef{internallinenumbers*}{\internallinenumbers*}` at lineno
  // L?? (cf. iclr2025_conference.sty L230 which calls it). Stub as
  // body-passthrough since line numbers are irrelevant in XML output.
  // Witness 52 papers with iclr2025_conference using this env.
  DefEnvironment!("{internallinenumbers}[Number]",  "#body");
  DefEnvironment!("{internallinenumbers*}[Number]", "#body");

  def_macro_noop("\\linenumbers OptionalMatch:* [Number]")?;
  def_macro_noop("\\nolinenumbers")?;
  def_macro_noop("\\runninglinenumbers OptionalMatch:* [Number]")?;
  def_macro_noop("\\pagewiselinenumbers")?;
  def_macro_noop("\\realpagewiselinenumbers")?;
  def_macro_noop("\\runningpagewiselinenumbers")?;

  def_macro_noop("\\leftlinenumbers  OptionalMatch:*")?;
  def_macro_noop("\\rightlinenumbers OptionalMatch:*")?;
  def_macro_noop("\\switchlinenumbers OptionalMatch:*")?;

  def_macro_noop("\\setrunninglinenumbers")?;
  def_macro_noop("\\setpagewiselinenumbers")?;

  def_macro_noop("\\resetlinenumber [Number]")?;
  def_macro_noop("\\modulolinenumbers [Number]")?;

  def_macro_noop("\\linenumberfont")?;
  DefRegister!("\\linenumbersep", Number(0));

  def_macro_noop("\\thelinenumber")?;

  def_macro_noop("\\makeLineNumber")?;
  def_macro_noop("\\makeLineNumberRunning")?;
  def_macro_noop("\\makeLineNumberOdd")?;
  def_macro_noop("\\makeLineNumberEven")?;
  def_macro_noop("\\makeLineNumberRight")?;
  def_macro_noop("\\makeLineNumberLeft")?;
  def_macro_noop("\\LineNumber")?;

  DefMacro!("\\numquote",        "\\quote");
  DefMacro!("\\endnumquote",     "\\endquote");
  DefMacro!("\\numquotation",    "\\quote");
  DefMacro!("\\endnumquotation", "\\endquote");

  def_macro_noop("\\quotelinenumberfont")?;
  DefRegister!("\\quotelinenumbersep", Number(0));
});
