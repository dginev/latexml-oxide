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

  // Should use this, but....
  DefMacro!("\\labelformat{}{}", None);

  Let!("\\Ref", "\\ref");
  Let!("\\Vref", "\\vref");

  DefMacro!("\\refpagename", None);
  DefMacro!("\\thevpagerefnum", None);

  // Ignorable?
  DefMacro!("\\reftextafter", None);
  DefMacro!("\\reftextbefore", None);
  DefMacro!("\\reftextcurrent", None);
  DefMacro!("\\reftextfaceafter", None);
  DefMacro!("\\reftextfacebefore", None);
  DefMacro!("\\reftextfaraway", None);

  DefMacro!("\\reftextpagerange Semiverbatim Semiverbatim", "\\vref{#2}--\\vref{#3}");
  DefMacro!("\\reftextlabelrange Semiverbatim Semiverbatim", "\\vref{#2}--\\vref{#3}");

  DefMacro!("\\reftextvario{}{}", None);

  // Ignorable warnings stuff
  DefMacro!("\\fullref", None);
  DefMacro!("\\vrefshowerrors", None);
  DefMacro!("\\vrefwarning", None);
});
