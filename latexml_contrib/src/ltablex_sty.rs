use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "ltablex.sty",
    "ltablex.sty is only minimally stubbed and will not be interpreted raw."
  );
  RequirePackage!("tabularx");
  RequirePackage!("longtable");
});
