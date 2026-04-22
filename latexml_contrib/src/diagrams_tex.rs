use latexml_package::prelude::*;

use crate::discard_env::discard_env_body;

#[rustfmt::skip]
LoadDefinitions!({
  Warn!(
    "missing_file",
    "diagrams.tex",
    "diagrams.tex is not implemented and will not be interpreted raw."
  );
  // Perl ar5iv-bindings/diagrams.tex.ltxml: \begin{diagram}…\end{diagram}
  // body is discarded via discard_env_body, and the whole environment is
  // replaced by <ltx:ERROR>{diagram}</ltx:ERROR>. Single-shot error,
  // body-consuming group via bgroup/egroup.
  DefConstructor!(
    T_CS!("\\begin{diagram}"), None,
    "<ltx:ERROR>{diagram}</ltx:ERROR>",
    bounded => true,
    mode    => "text",
    locked  => true,
    before_digest => { discard_env_body("diagram", "diagrams.tex.ltxml")?; }
  );
  DefMacro!("\\enddiagram", "\\relax");
});
