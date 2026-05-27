use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: epstopdf.sty.ltxml
  // Nothing to do here!
  DefMacro!("\\epstopdfsetup{}", None);
  // `\epstopdfDeclareGraphicsRule{ext}{type}{ext-out}{cmd}` (epstopdf-base.sty)
  // registers an EPS→<other> conversion command for graphics inclusion.
  // We delegate graphics format conversion to mutool/gs at the post-
  // processing graphics phase, so author-supplied conversion rules
  // have no effect on our HTML output. No-op stub mirrors Perl's
  // "nothing to do" philosophy for the whole package.
  // Witness: 2 papers in R06-R09 emitting `Error:undefined:
  // \epstopdfDeclareGraphicsRule`.
  DefMacro!("\\epstopdfDeclareGraphicsRule{}{}{}{}", None);
  // `\epstopdfcall{cmd}` is the lower-level shell-out wrapper used by
  // \epstopdfDeclareGraphicsRule. Same rationale: no-op.
  DefMacro!("\\epstopdfcall{}", None);
});
