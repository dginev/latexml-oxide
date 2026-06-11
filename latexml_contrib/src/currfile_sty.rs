use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "currfile.sty",
    "currfile.sty is only minimally stubbed and will not be interpreted raw."
  );
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
