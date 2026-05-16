//! Stub for sagej.cls (SAGE journals).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // sagej frontmatter.
  DefMacro!("\\corrauth{}", "");
  DefMacro!("\\affiliation{}", "");
  DefMacro!("\\runninghead{}", "");

  // {acks}, {funding}, {dci} envs (sagej L456-470).
  DefEnvironment!("{acks}", "<ltx:acknowledgements>#body</ltx:acknowledgements>");
  DefEnvironment!("{funding}", "<ltx:acknowledgements name='funding'>#body</ltx:acknowledgements>");
  DefEnvironment!("{dci}", "<ltx:acknowledgements name='dci'>#body</ltx:acknowledgements>");
});
