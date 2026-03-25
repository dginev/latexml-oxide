use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: varioref.sty.ltxml
  // INCOMPLETE IMPLEMENTATION (as in Perl)
  DefMacro!("\\vref OptionalMatch:* Semiverbatim", "\\ref{#2}");
  DefMacro!("\\vpageref OptionalMatch:* Semiverbatim", "\\ref{#2}");
  DefMacro!("\\vrefrange OptionalMatch:* Semiverbatim Semiverbatim", "\\vref{#2}--\\vref{#3}");
  DefMacro!("\\vpagerefrange OptionalMatch:* Semiverbatim Semiverbatim", "\\vref{#2}--\\vref{#3}");

  DefMacro!("\\vrefpagenum DefToken Semiverbatim", "\\def#1{\\ref{#2}}");

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
