//! A font selected by FAMILY must still decode through that family's fontmap.
//!
//! `bbding.sty` reaches its glyphs by family, not by encoding: `\dingfamily`
//! is `\fontencoding{U}\fontfamily{ding}\selectfont`, and neither Perl nor
//! Rust ships a `u.fontmap`. Perl's `\selectfont` handles this with an
//! explicit hack (latex_constructs.pool.ltxml L5207-5209) — if the family is
//! not a known typeface, try `LoadFontMap($family)` and, on success,
//! `MergeFont(encoding => $family)`. Rust was missing that branch, so every
//! `\@chooseSymbol{N}` fell through to the OT1 fallback and emitted OT1 slot
//! N's TEXT character: `\XSolidBrush` (`'045`) became a literal `%` and
//! `\Checkmark` (`'041`) became `!`. `ding_fontmap.rs` was dead code —
//! nothing ever set the `ding` encoding.
//!
//! Witness 2503.04421 (ICLR submission): 28 cells across its two main
//! results tables — the "pretrained?" column of both — rendered `%`/`!`
//! instead of ✗/✓, inverting the tables' meaning for a reader. The paper
//! converted with status 0 and zero `Error:` lines, so nothing but a
//! fidelity check catches this.
//!
//! Golden glyphs verified against Perl LaTeXML 0.8.8 on the same host.
//! Dump-independent (the fontmap bindings are compiled in).
use latexml::util::test::convert_fixture;

/// The text of the first paragraph starting with `label`, up to the next tag.
fn paragraph_text(out: &str, label: &str) -> String {
  let start = out
    .find(label)
    .unwrap_or_else(|| panic!("no {label:?} paragraph in the conversion output"))
    + label.len();
  let rest = &out[start..];
  let end = rest.find('<').unwrap_or(rest.len());
  rest[..end].trim().to_string()
}

#[test]
fn bbding_glyphs_decode_through_the_ding_family_fontmap() {
  let r = convert_fixture("tests/fonts/bbding.tex");
  let out = r
    .result
    .unwrap_or_else(|| {
      panic!(
        "conversion produced no result (status_code={})",
        r.status_code
      )
    })
    .to_string();

  // \Checkmark \CheckmarkBold \XSolid \XSolidBold \XSolidBrush
  // = ding slots '041 '042 '043 '044 '045 = U+2713..U+2717.
  // Pre-fix this read "! \" # $ %" — OT1 slots 33..37.
  assert_eq!(
    paragraph_text(&out, "Marks:"),
    "\u{2713} \u{2714} \u{2715} \u{2716} \u{2717}",
    "bbding marks did not decode through ding.fontmap — the \\selectfont \
     family-as-encoding branch regressed, and \\XSolidBrush is silently a \
     literal OT1 character again (witness 2503.04421)"
  );

  // Slots far from the ASCII range, so a fallback cannot coincidentally match:
  // \ScissorRight \Phone \Envelope \HandRight \PencilRight = '001 '010 '014 '021 '027.
  assert_eq!(
    paragraph_text(&out, "Slots:"),
    "\u{2702} \u{260E} \u{2709} \u{1F599} \u{270F}",
    "low ding slots did not decode through ding.fontmap"
  );

  // The family-derived encoding must survive a nested font switch: the
  // witness reaches these glyphs from inside \textbf{} in table cells, and a
  // `MergeFont` that the inner switch discards would decode to OT1 again.
  // Asserted on the surrounding text, not on element names, so the check
  // does not also pin \textbf/\emph markup.
  for (context, glyph) in [("bold", '\u{2713}'), ("italic", '\u{2717}')] {
    let expected = format!("{context} {glyph}");
    assert!(
      out.contains(&expected),
      "expected {expected:?} — ding glyph U+{:04X} was lost inside a nested \
       font switch",
      glyph as u32
    );
  }
}
