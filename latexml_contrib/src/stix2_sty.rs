use latexml_package::prelude::*;

LoadDefinitions!({
  // stix2.sty defines the AMS symbol set under its own names (`\nmid`
  // stix2.sty:1316, `\varnothing` :1294, `\twoheadrightarrow` :896, …), so a
  // document that loads stix2 instead of amssymb (rbt-mathnotes.sty:129)
  // needs the same math tokens: reuse amssymb's Unicode bindings, then add
  // the stix-only names. The former stub shadowed the raw file and left them
  // all undefined. Guard: `perfect_kernel_batch56::sweep46_single_name_gaps`.
  RequirePackage!("amssymb");
  DefMath!("\\twoheadrightarrowtail", "\u{2916}", role => "ARROW");
  DefMath!("\\twoheadleftarrowtail", "\u{2B3B}", role => "ARROW");
});
