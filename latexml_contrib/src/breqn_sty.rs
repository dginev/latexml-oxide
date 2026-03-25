use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!("missing_file", "breqn.sty",
    "breqn.sty is not implemented and will not be interpreted raw.");
  // Forbid loading this package, even locally, until we can implement it natively
  DefMacro!("\\condition", "\\text");
  DefMacro!("\\hiderel{}", "#1");
  DefMacro!("\\begin{dmath} OptionalUndigested", "\\begin{equation}");
  DefMacro!("\\end{dmath}", "\\end{equation}");
  DefMacro!("\\begin{dmath*} OptionalUndigested", "\\begin{equation*}");
  DefMacro!("\\end{dmath*}", "\\end{equation*}");
});
