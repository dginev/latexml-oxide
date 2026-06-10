//! Stub for IEEEoj.cls (IEEE Open Journal template).
//!
//! IEEEoj is IEEEtran-derived but uses authblk-style `\author{}`/`\affil{}` for
//! the author/affiliation block. It was previously dispatched to
//! `ieeeaerospace_cls` (IEEEtran + `\acknowledgments`), which omits authblk — so
//! `\affil` was undefined while Perl defines it. Give IEEEoj its OWN binding:
//! ieeeaerospace's provisions (IEEEtran base + `\acknowledgments`) PLUS authblk
//! (which supplies `\affil`/`\affilmark`/`\author`), mirroring the sibling
//! `ieeeojcsys_cls`. Kept separate from `ieeeaerospace_cls` so the IEEE Aerospace
//! conference papers (which use IEEEtran's own author style) are unaffected by
//! authblk's `\author` redefinition. Witness 2203.03906.
//!
//! NOTE: the OJ-family siblings IEEEapm/IEEEtai (still on ieeeaerospace) likely
//! share the same `\affil` gap; route them here too once witnessed.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("IEEEtran");
  RequirePackage!("authblk");
  // Mirror ieeeaerospace_cls: IEEE OJ/Aerospace templates define a no-arg
  // `\acknowledgments` opening an unnumbered "Acknowledgments" section.
  DefMacro!("\\acknowledgments", "\\section*{Acknowledgments}");
});
