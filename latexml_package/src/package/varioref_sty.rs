use crate::prelude::*;

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

  // Perl noops `\labelformat{}{}` ("Should use this, but...."). Since
  // 2019-10-01 the macro is the KERNEL's (latex.ltx:14978, defined in
  // latex_constructs.rs) and the refnum formatter applies the `\p@<ctr>` it
  // sets up, so the binding leaves it alone (KPE #160).

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
  // varioref.sty:58-68 `\vref@addto\extras<lang>{<code>}`: run the code and
  // append it to the language hook, creating the hook when undefined. Classes
  // call it (univie-ling-expose/-paper/-thesis; class census 2026-09-24). Its
  // `\@onlypreamble` is dropped: those classes call it from `\AtBeginDocument`.
  RawTeX!(r"\def\vref@addto#1#2{#2\@temptokena{#2}\ifx#1\undefined\edef#1{\the\@temptokena}\else\toks@\expandafter{#1}\edef#1{\the\toks@\the\@temptokena}\fi\@temptokena{}\toks@\@temptokena}");
  // varioref.sty:799-802: the `space`/`nospace` switch, true by default
  // (`\ExecuteOptions{english,final,space}`). zref-vario.sty:486 clears it in
  // its varioref setup (tikz-cookingsymbols-doc `\zvref`: undefined
  // `\@vrefhandlespacefalse`; Perl varioref.sty.ltxml lacks it too). The
  // binding models no page-reference spacing, so only the switch is needed.
  // Guard: `binding_singletons_56::varioref_handlespace_switch_exists`.
  RawTeX!(r"\newif\if@vrefhandlespace\@vrefhandlespacetrue");
});
