//! updatemarks.sty — synchronize TeX marks across boxes / split boxes.
//!
//! updatemarks patches TeX output routines and box-splitting environments
//! (minipage, multicol, tcolorbox) using etoolbox \patchcmd to extract and
//! update TeX marks across pages. In HTML/XML document models, marks are not
//! page-split, and latexml constructs are non-expandable (causing raw
//! \patchcmd to fail and invoke \ERROR).
//!
//! Perl LaTeXML has no binding and hits Fatal:too_many_errors (>100 errors)
//! on raw loading.
//!
//! Witness: updatemarks.tex
use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "updatemarks.sty",
    "updatemarks.sty is an out-of-scope stub — mark synchronization across split boxes is meaningless in HTML/XML output."
  );
});
