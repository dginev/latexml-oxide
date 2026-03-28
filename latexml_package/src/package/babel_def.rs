//! babel.def — babel core definitions
//! Perl: babel.def.ltxml (34 lines)
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: Let('\bbl@opt@safe', '\@empty');
  // Inhibits some risky redefinitions in babel
  Let!("\\bbl@opt@safe", "\\@empty");

  // Load raw babel.def
  InputDefinitions!("babel", noltxml => true, extension => Some(Cow::Borrowed("def")));

  // Load babel support package (quote chars, language mapping, selectlanguage hook)
  RequirePackage!("babel_support");
});
