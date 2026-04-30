use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "l3draw.sty",
    "l3draw.sty is not supported and will not be interpreted raw."
  );
  // l3draw uses LaTeX3 \draw_begin: / \draw_end: which we cannot yet support.
  // The Perl binding defines a sub that reads until \draw_end: and returns an error.
  // We stub it as empty for now.
});
