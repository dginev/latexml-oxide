//! jpconf.cls (Journal of Physics: Conference Series, IOP's conference class, 2004/05/04 v1.0): the papers ship the
//! class (it is not in TeX Live), and it is read raw (OXIDIZED_DESIGN_DIVERGENCES #444). Its address and e-mail blocks
//! (:104-110: `\eads`, `\ead`, `\mailto`, `\address`, each an `{indented}` list item where it stands) are iopart's
//! (iopart.cls:240-248, 363-364), so after the read they are the frontmatter iopart_support makes of them
//! (`iopart_support_sty::iop_address_and_email`): each `\address` an affiliation of the authors before it, each `\ead`
//! an e-mail placed with its author, where read raw they were bullet lists after the abstract (0705.0202, 1011.0523).
//! Perl has no binding: OmniBus's `\address` and `\email`. Guard
//! `perfect_kernel_batch78m::jpconf_address_and_email_are_frontmatter`.
use latexml_package::{
  prelude::*,
  shipped_class::{RawRead, read_raw},
};

const JPCONF: RawRead = RawRead {
  shipped: true,
  ..RawRead::new("jpconf", "cls")
};

LoadDefinitions!({
  if read_raw(&JPCONF)? {
    iopart_support_sty::iop_address_and_email()?;
  }
});
