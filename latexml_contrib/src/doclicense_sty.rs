//! doclicense.sty (Creative Commons and other license statements).
//!
//! The real package is loaded raw, with its options: `\doclicenseThis`
//! (doclicense.sty:222) typesets the license statement — "This work is licensed
//! under a Creative Commons … license" with the license image — and the
//! `\doclicenseName`/`\doclicenseURL`/`\doclicenseLongText`… accessors return the
//! license's text. The former stub made every one of them a no-op, so the license
//! statement never reached the XML (beautynote; repro
//! `repros/captions-floats/singleton_doclicenseThis.tex`, PDF-to-XML recall 0 %).
//! Guard: `perfect_kernel_batch56::doclicense_statement_reaches_the_xml`.
use latexml_package::prelude::*;

LoadDefinitions!({
  let opts: Vec<String> = lookup_vecdeque("opt@doclicense.sty")
    .map(|v| v.iter().map(|o| o.to_string()).collect())
    .unwrap_or_default();
  InputDefinitions!("doclicense", noltxml => true, extension => Some(Cow::Borrowed("sty")),
    handleoptions => true, options => opts);
});
