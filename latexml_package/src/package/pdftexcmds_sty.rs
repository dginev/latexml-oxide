//! pdftexcmds.sty — pdfTeX utility commands
//! Perl: pdftexcmds.sty.ltxml
//! Everything is in pdfTeX.pool already; just require iftex.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("iftex");
  // Stubs for the pdftex-primitive wrappers that bmpsize (TL L51-53)
  // and other oberdiek packages probe via `\ifx\csname pdf@filedump
  // \endcsname\relax`. The raw load of pdftexcmds.sty would create
  // these conditionally; we bind instead so define defensively.
  // Witness 2406.02536, 2406.03347 (bmpsize "pdfTeX 1.30 or newer").
  DefMacro!("\\pdf@filedump{}{}{}", "");
  DefMacro!("\\pdf@mdfivesum{}", "");
  DefMacro!("\\pdf@filemdfivesum{}", "");
  DefMacro!("\\pdf@filesize{}", "0");
  DefMacro!("\\pdf@filemoddate{}", "");
  DefMacro!("\\pdf@strcmp{}{}", "0");
});
