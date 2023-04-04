use crate::package::*;

LoadDefinitions!(state, {
  //======================================================================

  // Special Characters.
  // Try to give them some sense in math...
  DefMacro!("\\#", "\\ifmmode\\lx@math@hash\\else\\lx@text@hash\\fi");
  DefMacro!("\\&", "\\ifmmode\\lx@math@amp\\else\\lx@text@amp\\fi");
  DefMacro!(
    "\\%",
    "\\ifmmode\\lx@math@percent\\else\\lx@text@percent\\fi"
  );
  DefMacro!("\\$", "\\ifmmode\\lx@math@dollar\\else\\lx@text@dollar\\fi");
  DefMacro!(
    "\\_",
    "\\ifmmode\\lx@math@underscore\\else\\lx@text@underscore\\fi"
  );
  DefMacro!(T_CS!("\\lx@text@hash"), None, T_OTHER!("#"),  alias => "\\#");
  DefMacro!(T_CS!("\\lx@text@amp"), None, T_OTHER!("&"),  alias => "\\&");
  DefMacro!(T_CS!("\\lx@text@percent"), None, T_OTHER!("%"),  alias => "\\%");
  DefMacro!(T_CS!("\\lx@text@dollar"), None,  T_OTHER!("$"), alias => "\\$");
  DefMacro!(T_CS!("\\lx@text@underscore"), None, T_OTHER!("_"),  alias => "\\_");

  DefMath!("\\lx@math@hash",  None, "#", alias => "\\#");
  DefMath!("\\lx@math@amp",   None, "&", role  => "ADDOP", meaning => "and", alias => "\\&");
  DefMath!("\\lx@math@percent", None, "%", role  => "POSTFIX", meaning => "percent", alias => "\\%");
  DefMath!("\\lx@math@dollar", None, "\\$", role => "OPERATOR", meaning => "currency-dollar",
    alias => "\\$");
  DefMath!("\\lx@math@underscore", None, "_", alias => "\\_");
});
