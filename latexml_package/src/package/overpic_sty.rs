use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: overpic.sty.ltxml
  RequirePackage!("graphicx");
  RequirePackage!("epic"); // we don't have binding yet.

  // Create an EMPTY picture environment, but with the tex attribute containing the
  // necessary code for LaTeX to generate an image.
  // TODO: DefEnvironment('{overpic} OptionalKeyVals:Gin Semiverbatim', ...) with afterDigestBody
  // The overpic environment requires complex afterDigestBody logic involving
  // \@includegraphicx, getSize, and setProperties. Stubbed as passthrough for now.
  DefEnvironment!("{overpic}[]{}", "<ltx:picture>#body</ltx:picture>");

  // Need {Overpic}, too, but it doesn't take an image, but random TeX
  // I suspect that will need an entirely different strategy!
  // And since it's used in only 3 papers on arXiv, it hardly seems worth it...
});
