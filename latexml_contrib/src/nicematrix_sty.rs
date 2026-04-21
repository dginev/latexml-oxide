use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("pgfcore");
  RequirePackage!("amsmath");
  RequirePackage!("array");
  Warn!(
    "missing_file",
    "nicematrix.sty",
    "nicematrix.sty is not implemented and will not be interpreted raw."
  );
  // TODO: Perl has a discard_env_body closure that reads and discards
  // environment bodies, emitting <ltx:ERROR> elements.
  // For now, stub all NiceMatrix environments with \relax end macros.
  DefMacro!("\\endNiceTabular", "\\relax");
  DefMacro!("\\endNiceArray", "\\relax");
  DefMacro!("\\endNiceMatrix", "\\relax");
  DefMacro!("\\endNiceArrayWithDelims", "\\relax");
  DefMacro!("\\endpNiceArray", "\\relax");
  DefMacro!("\\endpNiceMatrix", "\\relax");
  DefMacro!("\\endNiceTabularX", "\\relax");
  DefMacro!("\\endbNiceArray", "\\relax");
  DefMacro!("\\endbNiceMatrix", "\\relax");
  DefMacro!("\\endBNiceArray", "\\relax");
  DefMacro!("\\endBNiceMatrix", "\\relax");
  DefMacro!("\\endvNiceArray", "\\relax");
  DefMacro!("\\endvNiceMatrix", "\\relax");
  DefMacro!("\\endVNiceArray", "\\relax");
  DefMacro!("\\endVNiceMatrix", "\\relax");
});
