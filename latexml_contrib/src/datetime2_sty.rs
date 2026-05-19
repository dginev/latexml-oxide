use latexml_package::prelude::*;


LoadDefinitions!({
  Warn!(
    "missing_file",
    "datetime2.sty",
    "datetime2.sty is only minimally stubbed and will not be interpreted raw."
  );
  // datetime2 L518: \DTMcurrenttime / \DTMnow / \DTMcurrentdate.
  // We don't render time placeholders; gobble.
  def_macro_noop("\\DTMcurrenttime")?;
  def_macro_noop("\\DTMnow")?;
  def_macro_noop("\\DTMcurrentdate")?;
  DefMacro!("\\DTMtoday", "\\today");
  def_macro_noop("\\DTMusemodule{}{}")?;
  def_macro_noop("\\DTMsetdatestyle{}")?;
  def_macro_noop("\\DTMsetstyle{}")?;
  def_macro_noop("\\DTMlangsetup[]{}")?;
  def_macro_noop("\\DTMnewstyle{}{}{}{}")?;
});
