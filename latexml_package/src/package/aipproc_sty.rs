use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("revtex3_support");
  state::assign_value("\\text:locked", Stored::None, Some(Scope::Global));
  RequirePackage!("longtable");
  RequirePackage!("psfig");
  DefMacro!("\\lefthead{}",  "");
  DefMacro!("\\righthead{}", "");
});
