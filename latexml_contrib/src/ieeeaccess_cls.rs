//! Stub for IEEE Access journal class (ieeeaccess.cls).
//!
//! ieeeaccess.cls is a derivative of IEEEtran. Route to the IEEEtran
//! binding and stub the IEEE Access-specific frontmatter macros.
use latexml_package::prelude::*;

use crate::shipped_class::{class_option_given, shipped_class_loads_or};

LoadDefinitions!({
  LoadClass!("IEEEtran");
  RequirePackage!("ifpdf");
  // ieeeaccess.cls:121-131 (1901.05553's copy; 241, 251 in others): `\RequirePackage{color}`, then, unless the
  // `jtehm` option (:36), `\RequirePackage{spotcolor}` for the PANTONE blue, whose `\NewSpotColorSpace`/`\AddSpotColor`/
  // `\SetPageColorSpace` lines the template's papers copy into their preamble (72 papers of run 353 with them
  // undefined, 48 with `\textcolor`/`\definecolor`: 1901.05553, 1907.00310, 1712.00216, 1902.03817). spotcolor has no
  // binding and loads raw; a copy that dropped it (1 of 60 sampled) loads none.
  RequirePackage!("color");
  if !class_option_given("jtehm")? && shipped_class_loads_or("ieeeaccess", "spotcolor", true)? {
    RequirePackage!("spotcolor");
  }

  // IEEE Access frontmatter (ieeeaccess.cls L270, L448-453, L839) —
  // preserve author-supplied content as ltx:note frontmatter.
  DefMacro!(
    "\\history{}",
    "\\@add@frontmatter{ltx:note}[role=history]{#1}"
  );
  DefMacro!(
    "\\corresp{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}"
  );
  def_macro_noop("\\EOD")?;
  // \Figure[pos](opts)[scale]{file}{caption} — image inclusion.
  // Stub as a simple includegraphics-style construct rendering nothing
  // when called outside expected float context.
  DefMacro!("\\Figure", "\\@gobble");
  DefRegister!("\\titlepgskip" => Dimension!("0pt"));
  // Metadata — preserve author values.
  DefMacro!("\\doi{}", "\\@add@frontmatter{ltx:note}[role=doi]{#1}");
  DefMacro!(
    "\\address[]{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#2}"
  );
  DefMacro!(
    "\\tfootnote{}",
    "\\@add@frontmatter{ltx:note}[role=titlenote]{#1}"
  );
});
