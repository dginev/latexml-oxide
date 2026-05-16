//! Stub for sagej.cls (SAGE journals).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // sagej frontmatter — preserve author content.
  DefMacro!("\\corrauth{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}");
  DefMacro!("\\affiliation{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#1}");
  DefMacro!("\\runninghead{}",
    "\\@add@frontmatter{ltx:note}[role=runninghead]{#1}");

  // {acks}, {funding}, {dci} envs (sagej L456-470).
  DefEnvironment!("{acks}", "<ltx:acknowledgements>#body</ltx:acknowledgements>");
  DefEnvironment!("{funding}", "<ltx:acknowledgements name='funding'>#body</ltx:acknowledgements>");
  DefEnvironment!("{dci}", "<ltx:acknowledgements name='dci'>#body</ltx:acknowledgements>");
});
