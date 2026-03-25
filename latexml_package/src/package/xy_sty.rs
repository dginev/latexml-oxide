use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: xy.sty.ltxml (57 lines)
  // TODO: Full port requires:
  // 1. AssignCatcode('@' => CC_OTHER) before loading
  // 2. InputDefinitions('xy', type => 'tex') — load raw TeX xy package
  // 3. DeclareOption for driver options (dvips, pdftex, etc.)
  // 4. \xyoption{} — load xy extension modules
  // 5. ProcessOptions
  //
  // Perl source: LaTeXML/lib/LaTeXML/Package/xy.sty.ltxml
  DefMacro!("\\xystycatcode", "12"); // catcode of @
  RequirePackage!("ifpdf");
  InputDefinitions!("xy", noltxml => true, extension => Some(Cow::Borrowed("tex")));

  // Driver options (all mapped to \xyoption)
  for option in [
    "cmactex", "dvips", "dvitops", "emtex", "ln", "oztex",
    "textures", "xdvi", "pdftex", "dvipdfm", "dvipdfmx",
  ].iter() {
    DeclareOption!(*option, None);
  }
  DeclareOption!("colour", None);
  DeclareOption!("cmtip", None);
  DeclareOption!(None, None);
  ProcessOptions!();
});
