//! nag.sty — "Notification About Going's-ons" lint package
//! by Ulrich M. Schwarz.
//!
//! nag flags use of obsolete LaTeX 2.09 commands (`\bf`, `\it`, etc.),
//! deprecated packages (`epsfig`, `a4`, `psfig`), and other "thou
//! shalt not" patterns. It does this by redefining the flagged CSes
//! to first emit a Warning then call the original.
//!
//! Raw-loading nag.sty + nag-l2tabu.cfg adds attack surface to our
//! engine without providing any HTML-relevant value: nag's
//! `\ObsoleteCS`/`\ObsoletePackage`/`\ObsoleteClass` redefinitions
//! can confuse mode tracking in subsequent papers (witness 1411.3836:
//! amsart + nag + amsrefs paper → cascading `_` in text mode errors
//! after nag's redefinition chain). Perl LaTeXML has no nag binding;
//! with default `INCLUDE_STYLES=false` raw nag.sty is not loaded.
//!
//! Match Perl: stub the public API as no-ops. We lose nag's
//! lint warnings (which are diagnostic for the LaTeX author, not
//! useful in HTML output), but the kernel commands remain intact.
use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "nag.sty",
    "nag.sty is minimally stubbed — \\ObsoleteCS/\\ObsoletePackage/\\ObsoleteClass are no-ops; kernel commands are not redefined."
  );
  // The public API: silently consume their args.
  DefMacro!("\\ObsoletePackage[]{}{}", "");
  DefMacro!("\\ObsoleteClass{}{}", "");
  DefMacro!("\\ObsoleteCS[]{}{}", "");
  DefMacro!("\\nag[]{}", "");
  def_macro_noop("\\nagrequirepackage{}")?;
  def_macro_noop("\\nagshow{}")?;
});
