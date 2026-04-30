use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "xr.sty",
    "xr.sty is not implemented and will not be interpreted raw."
  );
  DefMacro!("\\externaldocument[]{}", "");
  DefMacro!("\\externalcitedocument[]{}", "");
});
