use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: showkeys.sty.ltxml
  // Nothing to do here.... but the `color` option: showkeys.sty:51-56 `\DeclareOption{color}{\AtEndOfPackage{%
  // \RequirePackage{color}…}}`, so a paper loading `[color]{showkeys}` has `\definecolor` and `\color` without
  // loading color itself (the key colours are page furniture here; Perl's binding declares no option, KNOWN_PERL_ERRORS
  // #692; witnesses 0712.3213, 1010.3777, 1103.2047).
  DeclareOption!("color", {
    AssignValue!("showkeys@color" => true);
  });
  DeclareOption!(None, None);
  ProcessOptions!();
  if lookup_bool("showkeys@color") {
    RequirePackage!("color");
  }
});
