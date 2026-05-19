use latexml_package::prelude::*;


LoadDefinitions!({
  Warn!(
    "missing_file",
    "xr.sty",
    "xr.sty is not implemented and will not be interpreted raw."
  );
  // xr-hyper.sty supports `\externaldocument[prefix][nocite]{file}[ext]`
  // with TWO optional args (prefix + nocite-mode) and an optional
  // extension. xr.sty original is just `[prefix]{file}`. Cover the
  // broader xr-hyper signature so a file with `_` in its name (e.g.
  // `\externaldocument[][nocite]{ex_supplement}`) doesn't leak the
  // underscore into text mode. Witness 2402.12241.
  def_macro_noop("\\externaldocument[][] Semiverbatim")?;
  def_macro_noop("\\externalcitedocument[][] Semiverbatim")?;
});
