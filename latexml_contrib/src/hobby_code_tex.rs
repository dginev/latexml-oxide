use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "hobby.code.tex",
    "hobby.code.tex is not implemented and will not be interpreted raw."
  );
  // Forbid loading this package, even locally, until we get our LaTeX 3
  // support ready to handle 2111.02755
});
