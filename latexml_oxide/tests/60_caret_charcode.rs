//! `` `^^@ `` (backtick + caret-notation char) must read the code point, like Perl.
//!
//! Root cause (found while triaging 2111.00584 → xintexpr/xinttrig): Rust set
//! NUL's (`\^^@`, U+0000) DEFAULT catcode to 9 (IGNORE, per the TeXbook),
//! whereas Perl LaTeXML uses 12 (OTHER). With IGNORE, the `^^@`-notation char
//! is *dropped* during tokenization, so the alphabetic constant `` `^^@ ``
//! skipped to the next token (e.g. `\relax`) and returned its code (114)
//! instead of 0. xint's `\romannumeral`&&@` expansion idiom (`&&@` is `^^@`
//! with `&` at catcode 7) relies on `` `^^@ `` == 0.
//!
//! Fix: NUL default catcode → 12 (OTHER), matching Perl. An explicit
//! `\catcode`^^Q=9` is still honored (only the default changes); stray raw NUL
//! bytes become harmless OTHER chars stripped at XML serialization (no bogus
//! `\uninger`-style CS, no invalid-XML NUL in output).
//!
//! Dump-independent.
use latexml::util::test::convert_fixture;

#[test]
fn backtick_caret_notation_reads_charcode() {
  let r = convert_fixture("tests/cluster_regressions/caret_charcode.tex");
  let out = r.result.expect("conversion produced no result");
  let xml = out.to_string();

  assert!(
    xml.contains("value is [0]"),
    "`` `^^@ `` must read code 0 (got output without `[0]`): NUL default catcode \
     regressed to IGNORE? — relevant excerpt: {:?}",
    xml.split("value is").nth(1).map(|s| &s[..s.len().min(40)])
  );
  assert!(
    xml.contains("Second [1]"),
    "`` `^^A `` must read code 1 (^^A == char 1)"
  );
}
