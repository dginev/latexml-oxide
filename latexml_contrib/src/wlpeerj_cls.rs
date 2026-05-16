//! Stub for wlpeerj.cls (Wiley PeerJ template).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // Frontmatter
  DefMacro!("\\corrauthor[]{}{}", "");
  DefMacro!("\\authoraffiliation[]{}{}", "");
  DefMacro!("\\affil[]{}", "");
});
