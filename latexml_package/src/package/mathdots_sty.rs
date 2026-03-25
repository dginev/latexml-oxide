use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Main purpose is to track the current sizing.
  // \ddots, \vdots in TeX.pool
  DefMath!("\\iddots", None, "\u{22F0}", role => "ID");

  // Copied from amsmath
  DefMath!("\\dddot{}",  "\u{02D9}\u{02D9}\u{02D9}",                 operator_role => "OVERACCENT");
  DefMath!("\\ddddot{}", "\u{02D9}\u{02D9}\u{02D9}\u{02D9}",         operator_role => "OVERACCENT");

  Let!("\\fixedddots",    "\\ddots");
  Let!("\\fixedvdots",    "\\vdots");
  Let!("\\fixediddots",   "\\iddots");
  Let!("\\originalddots",  "\\ddots");
  Let!("\\originalvdots",  "\\vdots");
  Let!("\\originaliddots", "\\iddots");
  Let!("\\originaldddot",  "\\dddot");
  Let!("\\originalddddot", "\\ddddot");
  Let!("\\MDoddots",       "\\ddots");
  Let!("\\MDovdots",       "\\vdots");
  Let!("\\MDoiddots",      "\\iddots");
  Let!("\\MDodddot",       "\\dddot");
  Let!("\\MDoddddot",      "\\ddddot");

  DefRegister!("\\MDoprekern"  => MuDimension::new_spec("0mu"));
  DefRegister!("\\MDodotkern"  => MuDimension::new_spec("-1.3mu"));
  DefRegister!("\\MDopostkern" => MuDimension::new_spec("-1mu"));
});
