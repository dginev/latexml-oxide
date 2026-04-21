use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("textcomp");
  RequirePackage!("eufrak");
  RequirePackage!("amsmath");
  Let!("\\slimits@", "\\nolimits");
  Warn!(
    "missing_file",
    "MnSymbol.sty",
    "MnSymbol.sty is not implemented and will not be interpreted raw."
  );
});
