//! Stub for morefloats.sty — raises the limit on the number of
//! *unprocessed* floats LaTeX can hold in its internal queue.
//!
//! Why a stub: morefloats.sty's entire body (L76+) is float-register-pool
//! capacity management. After option processing it computes the number of
//! "free" classic-TeX registers as `234 - max(\count10,\count11,\count12,
//! \count14)` (L271-274) and raises `Package morefloats Error: Too many
//! floats requested` when the requested float count exceeds it. That math
//! assumes the pre-eTeX 256-register pool; in our XML/HTML pipeline there
//! is NO fixed float-box register pool (floats become `<ltx:float>`
//! elements with no limit), and our `\count10` allocation high-water-mark
//! (~214 after the kernel) makes `234 - \count10` tiny, so any
//! `maxfloats=N` request spuriously trips the cap. The error is a pure
//! typesetting-capacity `\PackageError` — moot in our paradigm (WISDOM
//! #50): we never run out of float slots.
//!
//! Perl LaTeXML matches the EFFECTIVE skip: even under ar5iv it reports
//! morefloats as a missing-file (`Can't find binding for package
//! morefloats`) and only deps-scans it (kvoptions, etex/ifetex), never
//! executing the capacity body — so the paper converts cleanly with the
//! float limit simply unbounded. morefloats exports NO user macro (it is
//! all load-time register setup), and the 3 witness papers
//! (1504.06174, 1605.06159, 1607.05324) call none — so a no-op preserves
//! all content.
//!
//! We DO replicate morefloats.sty L63-74 — the kvoptions option-handling
//! prefix — so a paper's `\usepackage[maxfloats=120]{morefloats}` consumes
//! its options exactly as the real package does (stored in
//! `\morefloats@maxfloats` and ignored), rather than raising an
//! unknown-option warning. Only the moot capacity body (L76+) is omitted.
use latexml_package::prelude::*;

LoadDefinitions!({
  // morefloats.sty L63: \RequirePackage{kvoptions}
  RequirePackage!("kvoptions");
  // morefloats.sty L70-74: declare + process the string options as no-ops.
  RawTeX!(r"\SetupKeyvalOptions{family=morefloats,prefix=morefloats@}");
  RawTeX!(r"\DeclareStringOption{maxfloats}");
  RawTeX!(r"\DeclareStringOption{morefloats}");
  RawTeX!(r"\ProcessKeyvalOptions*");
  // Body (L76+: float-register capacity math + \PackageError) deliberately
  // omitted — moot for XML output and the source of the spurious cap.
});
