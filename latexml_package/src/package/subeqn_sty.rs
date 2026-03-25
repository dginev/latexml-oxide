//! subeqn.sty — sub-equation numbering
//! Perl: subeqn.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\subequations",    "\\lx@equationgroup@subnumbering@begin");
  DefMacro!("\\endsubequations", "\\lx@equationgroup@subnumbering@end");
});
