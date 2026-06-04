//! Stub for IEEEAerospaceCLS (IEEE Aerospace conference template).
//!
//! IEEEAerospaceCLS is an IEEEtran-derived class for the IEEE Aerospace
//! Conference. Route to IEEEtran which supplies \appendices, \PARstart
//! and similar IEEE-template macros. Witness 2408.05924, 2408.06274.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("IEEEtran");
  // IEEEAerospaceCLS.cls L290-292 defines its own sectioning
  // `\def\acknowledgments{\section*{Acknowledgments}\addcontentsline{toc}
  // {section}{Acknowledgments}}` — a NO-arg command that opens an
  // "Acknowledgments" section (content runs until the next section; there is
  // no `\endacknowledgments`, so this is NOT the bounded ltx:acknowledgements
  // environment form). Perl ships no IEEEAerospaceCLS binding → raw-loads the
  // bundled cls and gets this def. Our binding intercepts the cls (routing to
  // IEEEtran), so `\acknowledgments` was undefined where Perl is clean. Mirror
  // the real cls: an unnumbered "Acknowledgments" section (the `\addcontents
  // line` is TOC-only, moot in HTML). Witness 1610.07252. RUST 1 → 0.
  DefMacro!("\\acknowledgments", "\\section*{Acknowledgments}");
});
