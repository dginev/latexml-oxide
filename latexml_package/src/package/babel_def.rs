//! babel.def — babel core definitions
//! Perl: babel.def.ltxml (34 lines)
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: Let('\bbl@opt@safe', '\@empty');
  // Inhibits some risky redefinitions in babel
  Let!("\\bbl@opt@safe", "\\@empty");

  // Load raw babel.def
  // (the format's, not a paper's copy: OXIDIZED_DESIGN_DIVERGENCES #1142)
  InputDefinitions!("babel", noltxml => true, extension => Some(Cow::Borrowed("def")),
    format_module => Some(FormatModule { marker: "\\bbl@version", tree_dir: "babel" }));

  // Load babel support package (quote chars, language mapping, selectlanguage hook)
  RequirePackage!("babel_support");
});
