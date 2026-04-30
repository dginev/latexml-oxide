use crate::prelude::*;
LoadDefinitions!({
  // Perl: amstex.tex.ltxml L32 — this is the `pool' for AMSTeX (not AMS
  // LaTeX). Loaded by `\input amstex` from TeX mode, before LaTeX.pool
  // would be anticipated. Puts LaTeXML into "amstex mode".
  LoadPool!("AmSTeX");
});
