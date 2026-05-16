//! Stub for IEEE Access journal class (ieeeaccess.cls).
//!
//! ieeeaccess.cls is a derivative of IEEEtran. Route to the IEEEtran
//! binding and stub the IEEE Access-specific frontmatter macros.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("IEEEtran");
  RequirePackage!("ifpdf");

  // IEEE Access frontmatter (ieeeaccess.cls L270, L448-453, L839)
  DefMacro!("\\history{}", "");
  DefMacro!("\\corresp{}", "");
  DefMacro!("\\EOD", "");
  // \Figure[pos](opts)[scale]{file}{caption} — image inclusion.
  // Stub as a simple includegraphics-style construct rendering nothing
  // when called outside expected float context.
  DefMacro!("\\Figure", "\\@gobble");
  DefRegister!("\\titlepgskip" => Dimension!("0pt"));
  // Common metadata that ieeeaccess papers add via \address / \doi /
  // \tfootnote etc. — gobble cleanly.
  DefMacro!("\\doi{}", "");
  DefMacro!("\\address[]{}", "");
  DefMacro!("\\tfootnote{}", "");
});
