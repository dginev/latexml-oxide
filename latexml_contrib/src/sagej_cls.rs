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

  // {acks}, {funding}, {dci} envs (sagej L456-470). Use
  // internal_vertical mode so block-level body (paragraphs, lists,
  // funding-statement prose) is accepted — restricted_horizontal
  // default tripped `Attempt to end mode restricted_horizontal in
  // internal_vertical` on multi-paragraph bodies.
  DefEnvironment!("{acks}", "<ltx:acknowledgements>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  DefEnvironment!("{funding}", "<ltx:acknowledgements name='funding'>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  DefEnvironment!("{dci}", "<ltx:acknowledgements name='dci'>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
});
