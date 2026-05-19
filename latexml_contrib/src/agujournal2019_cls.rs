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
  // Internal toggles — no content.
  def_macro_noop("\\draftfalse")?;
  def_macro_noop("\\drafttrue")?;
  DefConditional!("\\ifdraft");
  // Author-supplied metadata — preserve as ltx:note frontmatter.
  DefMacro!("\\journalname{}",
    "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
  DefMacro!("\\correspondingauthor{}{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1, #2}");

  // {keypoints} env — AGU title-page key-points list.
  DefEnvironment!(
    "{keypoints}",
    "<ltx:classification scheme='keypoints'>#body</ltx:classification>"
  );
  // AGU plot-axis explanation macros — pass through #2 / #1 so
  // the explanatory text appears in the output.
  DefMacro!("\\xexplain[]{}", "#2");
  DefMacro!("\\yexplain{}", "#1");
});
