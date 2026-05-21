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
  // datetime2.sty L642 / L668: \DTMnewdatestyle{<name>}{<body>} and
  // \DTMnewtimestyle{<name>}{<body>} — register a named date/time
  // style. Layout-only for HTML/XML, no-op. Witness 2404.13477.
  def_macro_noop("\\DTMnewdatestyle{}{}")?;
  def_macro_noop("\\DTMnewtimestyle{}{}")?;
  def_macro_noop("\\DTMnewzonestyle{}{}{}")?;
  def_macro_noop("\\DTMsettimestyle{}")?;
  def_macro_noop("\\DTMsetzonestyle{}")?;
  def_macro_noop("\\DTMshowstylesettings{}")?;
});
