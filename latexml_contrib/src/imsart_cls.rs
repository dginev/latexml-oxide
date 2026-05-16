//! Stub for IMS (Institute of Mathematical Statistics) `imsart` class.
//!
//! imsart.cls loads `article` + requires `imsart.sty` (support file with
//! \startlocaldefs, \endlocaldefs, etc.). We fall back to OmniBus and raw-
//! load imsart.sty so most user macros become available.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  // imsart.cls L149: \RequirePackage{imsart}.
  InputDefinitions!("imsart", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // Frontmatter primitives commonly used in imsart papers but not
  // always defined by imsart.sty (some are journal-driver dependent).
  // \startlocaldefs / \endlocaldefs are defined in imsart.sty L657-660;
  // these are belt-and-suspenders in case the raw load is short-circuited.
  DefMacro!("\\startlocaldefs", "\\makeatletter");
  DefMacro!("\\endlocaldefs", "\\makeatother");
  // imsart.sty L2268, L2360: \let\kwd@sep\relax inside conditionals
  // we may not fully replay. Define defensively. Witness 2406.17390.
  Let!("\\kwd@sep", "\\relax");
});
