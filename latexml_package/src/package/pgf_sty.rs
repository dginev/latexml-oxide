use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: pgf.sty.ltxml (54 lines)
  // TODO: Full port requires InputDefinitions("pgf", noltxml => true) which loads
  // the raw TeX pgf package. Also needs pgfsys-latexml.def driver, \lxSVG@picture
  // wrapper, and pgfsetcolor integration with font color system.
  //
  // Key Perl logic:
  // 1. DefMacro('\pgfsysdriver', 'pgfsys-latexml.def') — driver selection
  // 2. InputDefinitions('pgf', type => 'sty', noltxml => 1) — load raw TeX
  // 3. Let('\pgfutil@IfFileExists', '\IfFileExists')
  // 4. AtBeginDocument wraps pgfpicture with lxSVG@picture
  //
  // Perl source: LaTeXML/lib/LaTeXML/Package/pgf.sty.ltxml
  DefMacro!("\\pgfsysdriver", "pgfsys-latexml.def");
  InputDefinitions!("pgf", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  Let!("\\pgfutil@IfFileExists", "\\IfFileExists");
});
