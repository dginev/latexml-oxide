use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("textcomp");
  RequirePackage!("eufrak");
  RequirePackage!("amsmath");
  Let!("\\slimits@", "\\nolimits");
  Warn!(
    "missing_file",
    "MnSymbol.sty",
    "MnSymbol.sty is only minimally stubbed and will not be interpreted raw."
  );

  // MnSymbol provides hundreds of math symbols via \Decl@Mn@Char.
  // Define the most commonly-needed ones explicitly here — papers
  // using \checkmark from MnSymbol without loading amsfonts hit
  // `\checkmark undefined` (e.g. arXiv:2508.12496). Witness for this
  // single CS is large enough to add it without bringing in the full
  // raw load. Extend the list as more arxmliv papers surface other
  // MnSymbol-only symbols.
  DefMath!("\\checkmark", "\u{2713}", role => "ID"); // CHECK MARK
});
