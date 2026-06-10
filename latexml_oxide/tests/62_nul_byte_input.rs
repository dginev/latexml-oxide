//! A stray NUL byte in the input must not abort the conversion.
//!
//! Real-world `.bbl` files carry stray NULs from BibTeX `\"u`-mangling
//! (witness astro-ph0004127's spie4012-01a.bbl). Since commit 88f8bd44ce the
//! NUL default catcode is 12/OTHER (matching Perl, so `` `^^@ `` reads 0),
//! which lets the NUL survive tokenization — and a NUL inside math reaches
//! `Document::set_attribute` (the `tex=` reversion), where libxml's
//! `CString::new(value)` panics on the interior NUL (libxml node.rs:639),
//! killing the whole conversion (a process abort under the maxperf
//! `panic=abort` build). PR #249 review finding P0-1.
//!
//! The fix sanitizes XML-invalid characters at the serialization sinks, so
//! catcode-12 Perl parity is kept while serialization stays total.
//!
//! Dump-independent.
use latexml::util::test::convert_fixture;

#[test]
fn nul_byte_in_math_does_not_abort() {
  // The conversion runs in-process: a libxml CString panic would unwind
  // through (and fail) this test directly.
  let r = convert_fixture("tests/cluster_regressions/nul_byte_input.tex");

  let out = r
    .result
    .unwrap_or_else(|| {
      panic!(
        "conversion produced no result (status_code={}) — the NUL byte \
         likely aborted serialization",
        r.status_code
      )
    })
    .to_string();
  assert!(
    r.status_code < 3,
    "conversion hit a fatal (status_code={}) on a stray NUL byte",
    r.status_code
  );
  // The surrounding content must survive...
  assert!(
    out.contains("Before") && out.contains("after"),
    "document text around the NUL was lost"
  );
  // ...and no literal NUL may reach the XML (it is not a valid XML 1.0 char).
  assert!(
    !out.contains('\u{0000}'),
    "a literal NUL byte leaked into the XML output (invalid XML 1.0)"
  );
}
