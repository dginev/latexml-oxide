use crate::prelude::*;
//======================================================================-
// C.6.1 Quotations and Verse
//======================================================================-
LoadDefinitions!({
  // Perl: Let('\@block@cr', '\lx@newline');  # Obsolete, but in case still used
  Let!("\\@block@cr", "\\lx@newline");
  DefEnvironment!("{quote}",
    "<ltx:quote>#body</ltx:quote>",
    mode => "internal_vertical");
  DefEnvironment!("{quotation}",
    "<ltx:quote>#body</ltx:quote>",
    mode => "internal_vertical");
  DefEnvironment!("{verse}",
    "<ltx:quote role='verse'>#body</ltx:quote>",
    mode => "internal_vertical");
});
