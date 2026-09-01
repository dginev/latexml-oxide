use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // tagpdf.sty — PDF/UA tagging layer. Tag-structure commands drive the
  // PDF backend's marked-content operators; our XML output IS the
  // accessible structure, so the surface is absorbed (sweep-16 tail:
  // tagpdf's own manual + tex-vpat).
  def_macro_noop("\\tagpdfsetup{}")?;
  def_macro_noop("\\tagstructbegin{}")?;
  def_macro_noop("\\tagstructend")?;
  def_macro_noop("\\tagmcbegin{}")?;
  def_macro_noop("\\tagmcend")?;
  def_macro_noop("\\tagpdfparaOn")?;
  def_macro_noop("\\tagpdfparaOff")?;
  DefEnvironment!("{tagpdfsuppress}", "#body");
  def_macro_noop("\\tagpdfsuppressmarks{}")?;
});
