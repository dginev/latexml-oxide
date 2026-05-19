use latexml_package::prelude::*;


LoadDefinitions!({
  // Perl: apackage.sty.ltxml
  def_macro_noop("\\my@package@stuff")?;
  DeclareOption!(
    "acommonoption",
    "\\xdef\\my@package@stuff{\\my@package@stuff, acommonoption}"
  );
  DeclareOption!(
    "apackageoption",
    "\\xdef\\my@package@stuff{\\my@package@stuff, apackageoption}"
  );
  DeclareOption!(
    "anotherpackageoption",
    "\\xdef\\my@package@stuff{\\my@package@stuff, anotherpackageoption}"
  );
  ProcessOptions!();
  DefMacro!(
    "\\showpackagestuff",
    "\\par\\noindent Package options: \\my@package@stuff"
  );
});
