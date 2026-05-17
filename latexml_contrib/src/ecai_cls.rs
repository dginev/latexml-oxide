//! Stub for ecai.cls (ECAI conference class).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // ECAI frontmatter (ecai.cls L1290) — preserve paper ID as note.
  DefMacro!("\\paperid{}",
    "\\@add@frontmatter{ltx:note}[role=paperid]{#1}");
  DefMacro!("\\makepaperid", "");
  // ECAI authors use \orcid for ORCID identifier; preserve as note.
  // Witness 2501.02040 + 3 ecai papers.
  DefMacro!("\\orcid{}",
    "\\@add@frontmatter{ltx:note}[role=orcid]{#1}");
  // {ack} environment — acknowledgments block. Render content
  // inline (matches our neurips_sty pattern). Witness 2408.16081.
  DefEnvironment!("{ack}", "#body",
    before_digest => { gullet::unread_one(T_CS!("\\section*{Acknowledgments}")); });
});
