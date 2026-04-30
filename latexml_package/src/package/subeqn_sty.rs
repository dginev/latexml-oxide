//! subeqn.sty — sub-equation numbering
//! Perl: subeqn.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl subeqn.sty.ltxml L21-22 locks both `\subequations` and
  // `\endsubequations`. arXiv papers commonly load subeqn.sty alongside
  // amsmath (which also defines these names) — the lock keeps our
  // \lx@equationgroup@subnumbering@* trampolines from being shadowed.
  DefMacro!("\\subequations",    "\\lx@equationgroup@subnumbering@begin",
    locked => true);
  DefMacro!("\\endsubequations", "\\lx@equationgroup@subnumbering@end",
    locked => true);
});
