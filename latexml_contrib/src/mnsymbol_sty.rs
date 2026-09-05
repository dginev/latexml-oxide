use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("textcomp");
  RequirePackage!("eufrak");
  RequirePackage!("amsmath");
  // MnSymbol is a full replacement symbol font (`\Decl@Mn@Char`); its
  // symbol SET is the AMS one plus extras — the amssymb binding covers the
  // shared names (`\blacktriangleright` univie-ling, `\nmid`), the extras
  // are added below as they surface (`\bigcircle` MnSymbol.sty:1635,
  // atableau). Guard: `perfect_kernel_batch56::font_symbol_packages_carry_amssymb`.
  RequirePackage!("amssymb");
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
  DefMath!("\\bigcircle", "\u{25EF}", role => "OP"); // LARGE CIRCLE (\Decl@Mn@Op)
});
