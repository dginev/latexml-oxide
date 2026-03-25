use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: pgfplots.sty.ltxml (34 lines)
  // TODO: Full port requires InputDefinitions("pgfplots", noltxml => true) which loads
  // the raw TeX pgfplots package. This needs the pgf infrastructure (pgfsys-latexml.def).
  // For now, stub the package to prevent "not found" errors.
  //
  // Perl source: LaTeXML/lib/LaTeXML/Package/pgfplots.sty.ltxml
  DefMacro!("\\pgfplots@iffileexists", "\\IfFileExists");
  InputDefinitions!("pgfplots", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
