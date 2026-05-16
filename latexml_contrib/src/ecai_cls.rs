//! Stub for ecai.cls (ECAI conference class).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // ECAI frontmatter (ecai.cls L1290).
  DefMacro!("\\paperid{}", "");
  DefMacro!("\\makepaperid", "");
  // {ack} environment — acknowledgments block. Render content
  // inline (matches our neurips_sty pattern). Witness 2408.16081.
  DefEnvironment!("{ack}", "#body",
    before_digest => { gullet::unread_one(T_CS!("\\section*{Acknowledgments}")); });
});
