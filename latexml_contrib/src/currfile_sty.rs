use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "currfile.sty",
    "currfile.sty is only minimally stubbed and will not be interpreted raw."
  );
  DefMacro!("\\currfiledir", "");
  DefMacro!("\\currfilebase", "");
  DefMacro!("\\currfileext", "");
  DefMacro!("\\currfilename", "");
  DefMacro!("\\currfilepath", "");
  DefMacro!("\\currfileabsdir", "");
  DefMacro!("\\currfileabspath", "");
  DefMacro!("\\getpwd", "");
  DefMacro!("\\thepwd", "");
  DefConditional!("\\ifcurrfiledir");
  DefConditional!("\\ifcurrfilebase");
  DefConditional!("\\ifcurrfileext");
  DefConditional!("\\ifcurrfilename");
  DefConditional!("\\ifcurrfilepath");
  DefConditional!("\\ifcurrfile");
  DefConditional!("\\ifcurrfileabsdir");
  DefConditional!("\\ifcurrfileabspath");
});
