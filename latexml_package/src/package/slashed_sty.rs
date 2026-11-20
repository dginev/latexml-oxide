use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: slashed.sty.ltxml
  // Unlikely we can do anything with the spacing fine-tuning,
  // so we'll just ignore this...
  DefMacro!("\\declareslashed{}{}{}{}{}", None);
  DefMacro!("\\sla@{}{}{}{}{}", "\\lx@slashed{#5}");
  // Let the \not handler in TeX.pool take care of this.... — through the kernel's `\not` saved here, not the name:
  // slashed.sty:112-118 never calls `\not`, so an author's `\def\not#1{\slashed{#1}}` closed a loop (2504.21590;
  // OXIDIZED_DESIGN_DIVERGENCES #1100).
  Let!("\\lx@slashed@not", "\\not");
  DefMacro!("\\lx@slashed{}", "\\lx@slashed@not{#1}");
  Let!("\\slashed", "\\lx@slashed");
});
