//! Stub for IEEE Access journal class (ieeeaccess.cls).
//!
//! ieeeaccess.cls is a derivative of IEEEtran. Route to the IEEEtran
//! binding and stub the IEEE Access-specific frontmatter macros.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("IEEEtran");
  RequirePackage!("ifpdf");

  // IEEE Access frontmatter (ieeeaccess.cls L270, L448-453, L839) —
  // preserve author-supplied content as ltx:note frontmatter.
  DefMacro!("\\history{}",
    "\\@add@frontmatter{ltx:note}[role=history]{#1}");
  DefMacro!("\\corresp{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}");
  DefMacro!("\\EOD", "");
  // \Figure[pos](opts)[scale]{file}{caption} — image inclusion.
  // Stub as a simple includegraphics-style construct rendering nothing
  // when called outside expected float context.
  DefMacro!("\\Figure", "\\@gobble");
  DefRegister!("\\titlepgskip" => Dimension!("0pt"));
  // Metadata — preserve author values.
  DefMacro!("\\doi{}",
    "\\@add@frontmatter{ltx:note}[role=doi]{#1}");
  DefMacro!("\\address[]{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#2}");
  DefMacro!("\\tfootnote{}",
    "\\@add@frontmatter{ltx:note}[role=titlenote]{#1}");
});
