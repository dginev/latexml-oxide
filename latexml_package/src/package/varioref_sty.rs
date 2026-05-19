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
  // Perl: varioref.sty.ltxml
  // INCOMPLETE IMPLEMENTATION (as in Perl)
  // Perl varioref.sty.ltxml L24-29: all five CSes pass `locked => 1`
  // so later packages (cleveref, revtex, etc.) or user \renewcommand
  // calls can't silently override these stubs — the tests for whether
  // varioref is loaded already read true once these are defined, so a
  // quiet override would leave the references broken.
  DefMacro!("\\vref OptionalMatch:* Semiverbatim", "\\ref{#2}", locked => true);
  DefMacro!("\\vpageref OptionalMatch:* Semiverbatim", "\\ref{#2}", locked => true);
  DefMacro!("\\vrefrange OptionalMatch:* Semiverbatim Semiverbatim",
    "\\vref{#2}--\\vref{#3}", locked => true);
  DefMacro!("\\vpagerefrange OptionalMatch:* Semiverbatim Semiverbatim",
    "\\vref{#2}--\\vref{#3}", locked => true);

  DefMacro!("\\vrefpagenum DefToken Semiverbatim", "\\def#1{\\ref{#2}}",
    locked => true);

  // Should use this, but....
  def_macro_noop("\\labelformat{}{}")?;

  Let!("\\Ref", "\\ref");
  Let!("\\Vref", "\\vref");

  def_macro_noop("\\refpagename")?;
  def_macro_noop("\\thevpagerefnum")?;

  // Ignorable?
  def_macro_noop("\\reftextafter")?;
  def_macro_noop("\\reftextbefore")?;
  def_macro_noop("\\reftextcurrent")?;
  def_macro_noop("\\reftextfaceafter")?;
  def_macro_noop("\\reftextfacebefore")?;
  def_macro_noop("\\reftextfaraway")?;

  DefMacro!("\\reftextpagerange Semiverbatim Semiverbatim", "\\vref{#2}--\\vref{#3}");
  DefMacro!("\\reftextlabelrange Semiverbatim Semiverbatim", "\\vref{#2}--\\vref{#3}");

  def_macro_noop("\\reftextvario{}{}")?;

  // Ignorable warnings stuff
  def_macro_noop("\\fullref")?;
  def_macro_noop("\\vrefshowerrors")?;
  def_macro_noop("\\vrefwarning")?;
});
