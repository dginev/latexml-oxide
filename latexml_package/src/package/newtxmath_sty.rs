//! newtxmath.sty — TX math fonts (delegates to other packages)
//! Perl: newtxmath.sty.ltxml
use crate::prelude::*;

LoadDefinitions!({
  // Perl newtxmath.sty.ltxml L22-31: explicitly require ifxetex+ifluatex
  // (both set identical ifcond false stubs in Rust). Rust previously
  // consolidated to `iftex`, which is a superset — same conditionals
  // available — but drifts from Perl parity. Match the Perl chain exactly.
  RequirePackage!("amsmath");
  RequirePackage!("ifthen");
  RequirePackage!("etoolbox");
  RequirePackage!("ifxetex");
  RequirePackage!("ifluatex");
  RequirePackage!("xkeyval");
  RequirePackage!("amssymb");
  RequirePackage!("txfonts");
  // newtxmath.sty:2466-2467 and :2577: its own `\varmathbb` (txfonts' above), `\vmathbb` the
  // same, and a third blackboard variant `\vvmathbb` (the same font here). Constructors of their
  // own, so the reversion keeps the author's name.
  DefConstructor!("\\vmathbb{}", "#1", bounded => true, require_math => true,
    font => { family => "blackboard", series => "medium", shape => "upright" });
  DefConstructor!("\\vvmathbb{}", "#1", bounded => true, require_math => true,
    font => { family => "blackboard", series => "medium", shape => "upright" });
});
