use crate::prelude::*;
//======================================================================-
// C.6.1 Quotations and Verse
//======================================================================-
LoadDefinitions!({
  // Perl: Let('\@block@cr', '\lx@newline');  # Obsolete, but in case still used
  Let!("\\@block@cr", "\\lx@newline");
  // Perl: mode => 'internal_vertical' — quote/quotation/verse don't override \\ or \par
  DefEnvironment!("{quote}",
    "<ltx:quote>#body</ltx:quote>",
    mode => "text");
  DefEnvironment!("{quotation}",
    "<ltx:quote>#body</ltx:quote>",
    mode => "text");
  DefEnvironment!("{verse}",
    "<ltx:quote role='verse'>#body</ltx:quote>",
    mode => "text");
});
