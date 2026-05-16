use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "datetime2.sty",
    "datetime2.sty is only minimally stubbed and will not be interpreted raw."
  );
  // datetime2 L518: \DTMcurrenttime / \DTMnow / \DTMcurrentdate.
  // We don't render time placeholders; gobble.
  DefMacro!("\\DTMcurrenttime", "");
  DefMacro!("\\DTMnow", "");
  DefMacro!("\\DTMcurrentdate", "");
  DefMacro!("\\DTMtoday", "\\today");
  DefMacro!("\\DTMusemodule{}{}", "");
  DefMacro!("\\DTMsetdatestyle{}", "");
  DefMacro!("\\DTMsetstyle{}", "");
  DefMacro!("\\DTMlangsetup[]{}", "");
  DefMacro!("\\DTMnewstyle{}{}{}{}", "");
});
