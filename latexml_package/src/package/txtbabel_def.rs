//! txtbabel.def — babel core definitions (modern babel)
//! Perl: txtbabel.def.ltxml (34 lines)
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Inhibits some risky redefinitions in babel
  Let!("\\bbl@opt@safe", "\\@empty");

  // Load raw txtbabel.def
  InputDefinitions!("txtbabel", noltxml => true, extension => Some(Cow::Borrowed("def")));

  // Load babel support package
  RequirePackage!("babel_support");
});
