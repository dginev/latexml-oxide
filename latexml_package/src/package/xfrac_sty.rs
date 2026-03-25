use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: xfrac.sty.ltxml
  RequirePackage!("amstext");
  RequirePackage!("graphicx");
  RequirePackage!("nicefrac");

  DefMacro!("\\sfrac[]{} []{}", "\\ensuremath{\\@UnitsNiceFrac{#2}{#4}}");

  DefMacro!("\\DeclareInstance{}{}{}{}", None);
  DefMacro!("\\DeclareCollectionInstance{}{}{}{}{}", None);
  DefMacro!("\\UseCollection{}{}", None);
});
