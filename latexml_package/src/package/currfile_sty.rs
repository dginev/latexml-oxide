use crate::prelude::*;

LoadDefinitions!({
  // Real currfile.sty L30 `\RequirePackage{filehook}` — currfile is built on
  // filehook's input-file hooks, and downstream packages (e.g. sTeX 3.x's
  // `\AtEndOfPackageFile{…}`) rely on filehook being present transitively via
  // currfile. Even though the currfile macros below are only stubbed, the
  // filehook dependency is real and cheap (Rust ships a `filehook` binding).
  RequirePackage!("filehook");
  def_macro_noop("\\currfiledir")?;
  def_macro_noop("\\currfilebase")?;
  def_macro_noop("\\currfileext")?;
  def_macro_noop("\\currfilename")?;
  def_macro_noop("\\currfilepath")?;
  def_macro_noop("\\currfileabsdir")?;
  def_macro_noop("\\currfileabspath")?;
  def_macro_noop("\\getpwd")?;
  def_macro_noop("\\thepwd")?;
  DefConditional!("\\ifcurrfiledir");
  DefConditional!("\\ifcurrfilebase");
  DefConditional!("\\ifcurrfileext");
  DefConditional!("\\ifcurrfilename");
  DefConditional!("\\ifcurrfilepath");
  DefConditional!("\\ifcurrfile");
  DefConditional!("\\ifcurrfileabsdir");
  DefConditional!("\\ifcurrfileabspath");
});
