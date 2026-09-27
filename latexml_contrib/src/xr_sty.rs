use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "xr.sty",
    "xr.sty is not implemented and will not be interpreted raw."
  );
  // xr and xr-hyper read `\externaldocument[prefix][nocite]{file}[url]` (xr.sty:44-47,
  // xr-hyper.sty:38-41, `\XR@[#1][#2]#3{\@testopt{\XR@@{#1}{#2}{#3}}{#3.\XR@ext}}`; xr v5 read
  // `[prefix]{file}` only, xr-2023-07-04.sty:46-48). The file is Semiverbatim so a name with `_`
  // (`ex_supplement`) does not leak the underscore into text mode (witness 2402.12241); without
  // the trailing optional (the external document's address, a K13 stage-2 finding)
  // `[https://…]` was left as text.
  def_macro_noop("\\externaldocument[][] Semiverbatim OptionalSemiverbatim")?;
  def_macro_noop("\\externalcitedocument[][] Semiverbatim OptionalSemiverbatim")?;
});
