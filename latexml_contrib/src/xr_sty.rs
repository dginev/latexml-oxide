use latexml_package::prelude::*;


LoadDefinitions!({
  Warn!(
    "missing_file",
    "xr.sty",
    "xr.sty is not implemented and will not be interpreted raw."
  );
  def_macro_noop("\\externaldocument[]{}")?;
  def_macro_noop("\\externalcitedocument[]{}")?;
});
