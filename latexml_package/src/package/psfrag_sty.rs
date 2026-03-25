use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DeclareOption!("209mode", {});
  DeclareOption!("2emode",  {});
  DeclareOption!("scanall", {});
  ProcessOptions!();
  DefPrimitive!("\\psfrag OptionalMatch:* Semiverbatim [][][][]{}", None);
  DefMacro!("\\psfragscanon", "");
  DefMacro!("\\psfragscanoff", "");
});
