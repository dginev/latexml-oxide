//! Stub for ifacconf.cls (IFAC conference proceedings).
//!
//! IFAC papers use Elsevier-style frontmatter (\author, \address, \ead,
//! \sep, \thanks, \fnref, \corref). Route to OmniBus and provide
//! content-preserving stubs identical to cas-* / ceurart patterns.
//!
//! Witness 2503.16455 (ifacconf paper).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("graphicx");
  RequirePackage!("natbib");

  // Elsevier-style separator + frontmatter helpers — preserve content.
  DefMacro!("\\sep", ",");
  DefMacro!("\\address[]{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#2}");
  DefMacro!("\\ead[]{}",
    "\\@add@frontmatter{ltx:note}[role=email]{#2}");
  DefMacro!("\\thanks{}",
    "\\@add@frontmatter{ltx:note}[role=thanks]{#1}");
  DefMacro!("\\fnref{}", "\\textsuperscript{#1}");
  DefMacro!("\\corref{}", "\\textsuperscript{*#1}");
  DefMacro!("\\fntext[]{}",
    "\\@add@frontmatter{ltx:note}[role=footnote]{#2}");
  DefMacro!("\\cortext[]{}",
    "\\@add@frontmatter{ltx:note}[role=corresp]{#2}");
});
