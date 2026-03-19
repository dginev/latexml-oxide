use crate::prelude::*;

// Perl: latexsym.sty.ltxml — "All symbols that latexsym defines are already defined"
// This is a no-op binding since all latexsym symbols are already in plain.rs
LoadDefinitions!({
  // All latexsym symbols (\lhd, \rhd, \unlhd, \unrhd, etc.) are already defined
  // in the TeX/plain kernel. This binding exists only to prevent "package not found" errors.
});
