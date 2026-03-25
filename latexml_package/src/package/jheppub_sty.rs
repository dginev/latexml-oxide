use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: jheppub.sty.ltxml
  // TODO: Full translation commented out — causes pathological memory usage during compilation.
  // Bisecting to find the culprit macro.
  RequirePackage!("hyperref");
  RequirePackage!("color");
  RequirePackage!("natbib");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  // RequirePackage!("epsfig");  // TODO: might pull in pathological expansion
  RequirePackage!("graphicx");
  // RequirePackage!("inst_support");  // TODO: might pull in pathological expansion
});
