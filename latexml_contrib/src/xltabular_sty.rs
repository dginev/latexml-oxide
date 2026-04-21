use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "xltabular.sty",
    "xltabular.sty is only minimally stubbed and will not be interpreted raw."
  );
  RequirePackage!("tabularx");
  RequirePackage!("longtable");
  DefMacro!("\\xltabular", "\\tabularx");
  DefMacro!("\\endxltabular", "\\endtabularx");
});
