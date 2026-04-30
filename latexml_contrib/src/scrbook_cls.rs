use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "scrbook.cls",
    "scrbook.cls is only minimally stubbed and will not be interpreted raw."
  );
  LoadClass!("OmniBus");
  DefMacro!("\\setkomafont{}{}", "");
  DefMacro!("\\setcapindent{}", "");
  DefMacro!("\\deffootnote[]{}{}{}", "");
  DefMacro!("\\deffootnotemark{}", "");
});
