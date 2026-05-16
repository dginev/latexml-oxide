//! Stub for interact.cls (Taylor & Francis interact class).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("booktabs");
  RequirePackage!("graphicx");

  // Author-block macros.
  DefMacro!("\\name{}", "#1");
  DefMacro!("\\affil{}", "");
  DefMacro!("\\affilskip", "");

  // {amscode} env — interact L507.
  DefEnvironment!(
    "{amscode}",
    "<ltx:classification scheme='AMS'>#body</ltx:classification>"
  );

  // Frontmatter metadata.
  DefMacro!("\\articletype{}", "");
  DefMacro!("\\authormark{}", "");
  DefMacro!("\\corres{}", "");
  DefMacro!("\\thanks{}", "");
  DefMacro!("\\journalname{}", "");
});
