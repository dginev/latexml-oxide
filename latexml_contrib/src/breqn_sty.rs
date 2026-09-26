use latexml_package::prelude::*;

LoadDefinitions!({
  // breqn.sty:56 `\RequirePackage{keyval,calc}`: a document can rely on them
  // (OXIDIZED_DESIGN #317).
  RequirePackage!("keyval");
  RequirePackage!("calc");
  Warn!(
    "missing_file",
    "breqn.sty",
    "breqn.sty is not implemented and will not be interpreted raw."
  );
  // Forbid loading this package, even locally, until we can implement it natively
  DefMacro!("\\condition", "\\text");
  DefMacro!("\\hiderel{}", "#1");
  DefMacro!(
    T_CS!("\\begin{dmath}"),
    "OptionalUndigested",
    "\\begin{equation}"
  );
  DefMacro!(T_CS!("\\end{dmath}"), None, "\\end{equation}");
  DefMacro!(
    T_CS!("\\begin{dmath*}"),
    "OptionalUndigested",
    "\\begin{equation*}"
  );
  DefMacro!(T_CS!("\\end{dmath*}"), None, "\\end{equation*}");
});
