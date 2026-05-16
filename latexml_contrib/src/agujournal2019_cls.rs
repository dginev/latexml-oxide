//! Stub for agujournal2019.cls (AGU journal template).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("graphicx");
  RequirePackage!("apacite");

  // AGU frontmatter (agujournal2019.cls L389+, L573-587).
  DefMacro!("\\draftfalse", "");
  DefMacro!("\\drafttrue", "");
  DefMacro!("\\journalname{}", "");
  DefMacro!("\\correspondingauthor{}{}", "");
  DefConditional!("\\ifdraft");

  // {keypoints} env — AGU title-page key-points list.
  DefEnvironment!(
    "{keypoints}",
    "<ltx:classification scheme='keypoints'>#body</ltx:classification>"
  );
  DefMacro!("\\xexplain[]{}", "");
  DefMacro!("\\yexplain{}", "");
});
