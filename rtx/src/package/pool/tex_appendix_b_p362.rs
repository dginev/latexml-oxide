use package::*;

pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

  //======================================================================

  // Special Characters.
  // Try to give them some sense in math...
  // DefMacro!("\\#", "\ifmmode\lx@math@hash\else\lx@text@hash\fi");
  // DefMacro!("\\&", "\ifmmode\lx@math@amp\else\lx@text@amp\fi");
  // DefMacro!("\\%", "\ifmmode\lx@math@percent\else\lx@text@percent\fi");
  // DefMacro!("\\$", undef, "\ifmmode\lx@math@dollar\else\lx@text@dollar\fi");
  // DefMacro!("\\_", "\ifmmode\lx@math@underscore\else\lx@text@underscore\fi");
  // DefPrimitive!("\lx@text@hash",     "#",  alias => "\#");
  // DefPrimitive!("\lx@text@amp",      "&",  alias => "\&");
  // DefPrimitive!("\lx@text@percent",  "%",  alias => "\%");
  // DefPrimitive!("\lx@text@dollar",   "\$", alias => "\\\$");
  // DefPrimitive!("\lx@text@underscore", "_",  alias => "\_");

  DefMathI!("\\lx@math@hash",  None, "#", alias => "\\#");
  DefMathI!("\\lx@math@amp",   None, "&", role  => "ADDOP", meaning => "and", alias => "\\&");
  DefMathI!("\\lx@math@percent", None, "%", role  => "POSTFIX", meaning => "percent", alias => "\\%");
  DefMathI!("\\lx@math@dollar", None, "\\$", role => "OPERATOR", meaning => "currency-dollar",
    alias => "\\$");
  DefMathI!("\\lx@math@underscore", None, "_", alias => "\\_");

  Ok(())
}
