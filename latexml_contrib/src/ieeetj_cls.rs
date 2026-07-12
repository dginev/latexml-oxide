//! ieeetj.cls — IEEE Transactions journal template (author-bundled, ~4900 lines;
//! not raw-loaded). OmniBus renders the authors (`\author`, `\affil` via
//! inst_support) fine, but the class's late-defined journal-metadata frontmatter
//! macros are undefined and leak as literal text (witness 2405.01673, 2603.04284
//! → `\corresp \authornote \receiveddate …`). Bind just those frontmatter
//! macros; everything else falls through to OmniBus.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");

  // Corresponding-author byline and author/funding note — preserve as notes.
  DefMacro!(
    "\\corresp{}",
    "\\lx@add@frontmatter{ltx:note}[role=corresponding]{#1}"
  );
  DefMacro!(
    "\\authornote{}",
    "\\lx@add@frontmatter{ltx:note}[role=note]{#1}"
  );
  // `\affil{…}` — a numbered affiliation block. inst_support already defines
  // `\affil`; keep that. (Listed here only for the record.)

  // Editorial dates and identifiers. IEEE templates ship these as unfilled
  // "XX Month, XXXX" / "XXXX.2022.1234567" placeholders; rendering them would
  // surface template noise, so gobble them (they leaked as raw text before).
  def_macro_noop("\\receiveddate{}")?;
  def_macro_noop("\\reviseddate{}")?;
  def_macro_noop("\\accepteddate{}")?;
  def_macro_noop("\\publisheddate{}")?;
  def_macro_noop("\\currentdate{}")?;
  def_macro_noop("\\doiinfo{}")?;
  def_macro_noop("\\history{}")?;
  def_macro_noop("\\articletype{}")?;
  // Author-supplied funding acknowledgement — preserve rather than gobble.
  DefMacro!(
    "\\fundingtext{}",
    "\\lx@add@frontmatter{ltx:note}[role=funding]{#1}"
  );
});
