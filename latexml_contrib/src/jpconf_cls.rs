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

/// This binding's frontmatter conventions (the observer's, OXIDIZED_DESIGN_DIVERGENCES #837), declared at its load
/// (`declare_frontmatter_conventions`, user ruling 2026-10-28: each binding declares its own):
/// jpconf: its notes print the body's arabic `\thefootnote` as a superscript (jpconf.cls:139, 153 `\@thefnmark`,
/// `\def\@fnsymbol#1{…^{\thefootnote}…}`; no `\maketitle` of its own changes `\thefootnote`): 1105.2659's
/// `\footnote[3]{Speaker}` shows "3" — the kernel's `\@fnsymbol\c@footnote` there recurses through `\thefootnote`
/// (review revfms2a2_7 finding 4); its `\address`/`\ead` are iopart's
pub const FRONTMATTER_CONVENTIONS: FmConventions = FmConventions {
  name: "jpconf",
  binding: "jpconf.cls",
  email_owner: FmEmailOwner::SpelledElseOrder,
  call_is_group: true,
  title_marks: FmTitleMarks::Body,
  cite: "jpconf.cls:104-110 (iopart's `\\address`/`\\ead`), :139, 153 (its notes' marks); OD #841 (0705.0202, \
         1011.0523)",
  ..FM_KERNEL_CONVENTIONS
};

LoadDefinitions!({
  // (this binding's frontmatter conventions, first: OXIDIZED_DESIGN_DIVERGENCES #837)
  declare_frontmatter_conventions(&FRONTMATTER_CONVENTIONS);
  if read_raw(&JPCONF)? {
    iopart_support_sty::iop_address_and_email()?;
  }
});
