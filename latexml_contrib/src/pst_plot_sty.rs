use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "pst-plot.sty",
    "pst-plot.sty is not implemented and will not be interpreted raw."
  );
  // Forbid loading this package, even locally, until we get good enough at
  // reusing the internals for good SVG
});
