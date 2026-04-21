use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("tikz");
  RequirePackage!("etoolbox");
  Warn!(
    "missing_file",
    "forest.sty",
    "forest.sty is not implemented and will not be interpreted raw."
  );
  // TODO: Perl has a discard_env_body closure that reads and discards
  // the {forest} environment body, emitting an <ltx:ERROR> element.
  // For now, stub the environment and macros.
  DefMacro!("\\endforest", "\\relax");
  DefMacro!("\\forestset{}", "\\relax");
  DefMacro!("\\forestoption{}", "\\relax");
  DefMacro!("\\foresteoption{}", "\\relax");
  DefMacro!("\\forestregister{}", "\\relax");
  DefMacro!("\\foresteregister{}", "\\relax");
});
