use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: tikz.sty.ltxml (57 lines)
  // TODO: Full port requires InputDefinitions("tikz", noltxml => true) which loads
  // the raw TeX tikz package. Also needs:
  // 1. \use@@tikzlibrary{} — DefPrimitive that loads tikzlibrary*.code.tex files
  // 2. \tikzcdset — redirect to pgfqkeys
  // 3. pgf infrastructure (pgfsys-latexml.def)
  //
  // Perl source: LaTeXML/lib/LaTeXML/Package/tikz.sty.ltxml
  // TikZ documents generate many warnings from unported pgf primitives.
  // Increase MAX_ERRORS to allow processing to complete.
  AssignValue!("MAX_ERRORS" => Stored::Int(1000));

  DefMacro!("\\pgfmathresult", "0.0");
  DefMacro!("\\tikz@align@temp", "\\pgfmathresult");
  InputDefinitions!("tikz", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  DefMacro!("\\tikzcdset", "\\pgfqkeys{/tikz/commutative diagrams}");
});
