use crate::package::*;
//======================================================================-
// C.6.1 Quotations and Verse
//======================================================================-
LoadDefinitions!({
  DefConstructor!("\\@block@cr[Dimension]", "<ltx:break/>\n",
  reversion => Tokens!(T_CS!("\\\\"), T_CR!()));
  DefEnvironment!("{quote}",
  "<ltx:quote>#body</ltx:quote>",
  before_digest => {
    Let!("\\\\", "\\@block@cr"); Let!("\\par", "\\@block@cr") },
  mode => "text");
  DefEnvironment!("{quotation}",
  "<ltx:quote>#body</ltx:quote>",
  before_digest => {
    Let!("\\\\", "\\@block@cr"); Let!("\\par", "\\@block@cr") },
  mode => "text");
  // NOTE: Handling of \\ within these environments?
  DefEnvironment!("{verse}",
  "<ltx:quote role='verse'>#body</ltx:quote>",
  before_digest => {
    Let!("\\\\", "\\@block@cr"); Let!("\\par", "\\@block@cr") },
  mode => "text");
});
