//! `\char` must decode with Perl's `FontDecode` semantics — including in math.
//!
//! Perl's `\char` calls `FontDecode` (`TeX_Character.pool.ltxml` L32-36), which
//! defaults the encoding to OT1 when the current font carries none:
//! `$encoding = $font->getEncoding || 'OT1'` (`Package.pm` L2877). Its sibling
//! `FontDecodeString` (`Package.pm` L2906) deliberately has **no** such
//! default, and `font::decode_str` is the port of that one — so calling it for
//! `\char` inherited the wrong half of a deliberate Perl asymmetry.
//!
//! That only bites in math mode, because `Font::math_default()` sets
//! `encoding: None` on purpose: `$\char65$` produced the empty string where
//! Perl produces `A`. Text mode was unaffected, which is why it survived.
//!
//! Second defect in the same line: the code was cast `as u8`, so an
//! out-of-range `\char300` wrapped onto slot 300 & 0xFF = 44 and printed `,`.
//! Perl guards `$code < 0` and then indexes the map, so out-of-range yields no
//! glyph at all.
//!
//! All expectations verified against same-host Perl LaTeXML 0.8.8, which
//! renders this fixture as `MATH[A] TEXT[A]`, `OOBMATH[] OOBTEXT[]`,
//! `SYMBOL[A] ACCENT[´]`.
use latexml::util::test::convert_fixture;

/// The text between `NAME[` and its `]`, with all markup stripped.
fn marker(out: &str, name: &str) -> String {
  let open = format!("{name}[");
  let start = out
    .find(&open)
    .unwrap_or_else(|| panic!("marker {name}[ absent from the conversion output"))
    + open.len();
  let rest = &out[start..];
  let end = rest
    .find(']')
    .unwrap_or_else(|| panic!("marker {name}[ is never closed"));
  // Strip tags: math lands in <math>…</math> wrappers, text does not.
  let inner = regex_lite_strip_tags(&rest[..end]);
  inner.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Minimal tag stripper — avoids pulling a regex dep into a test.
fn regex_lite_strip_tags(s: &str) -> String {
  let mut out = String::new();
  let mut depth = 0usize;
  for c in s.chars() {
    match c {
      '<' => depth += 1,
      '>' => depth = depth.saturating_sub(1),
      _ if depth == 0 => out.push(c),
      _ => {},
    }
  }
  out
}

#[test]
fn char_decodes_through_ot1_in_math_and_does_not_wrap_out_of_range() {
  let r = convert_fixture("tests/cluster_regressions/char_font_decode.tex");
  let out = r
    .result
    .unwrap_or_else(|| {
      panic!(
        "conversion produced no result (status_code={})",
        r.status_code
      )
    })
    .to_string();

  // The regression: math mode fell through to an empty encoding.
  assert_eq!(
    marker(&out, "MATH"),
    "A",
    "`$\\char65$` did not decode — `\\char` is using FontDecodeString \
     semantics again (no OT1 default) and silently drops the glyph in math, \
     where Font::math_default() leaves the encoding unset"
  );
  assert_eq!(marker(&out, "TEXT"), "A", "`\\char65` broke in text mode");
  assert_eq!(
    marker(&out, "SYMBOL"),
    "A",
    "`$\\symbol{{65}}$` did not decode — \\symbol routes through \\char"
  );

  // Out of range must yield NOTHING, not a wrapped slot. `,` is the specific
  // pre-fix answer (300 & 0xFF == 44), so name it in the message.
  for m in ["OOBMATH", "OOBTEXT"] {
    let got = marker(&out, m);
    assert!(
      got.is_empty(),
      "`\\char300` produced {got:?} instead of nothing — the code is being \
       truncated to u8 again (300 & 0xFF = 44 = `,`)"
    );
  }

  // A slot whose OT1 entry is an accent still routes through decode_str, which
  // is what carries Perl's multi-char map entries (NBSP-prefixed combining
  // marks, T1's "SS"). Pins that the fix did not bypass that wrapper.
  assert_eq!(
    marker(&out, "ACCENT"),
    "\u{00B4}",
    "`\\char19` lost its OT1 acute accent — the multi-char/combining-mark \
     handling in font::decode_str is being bypassed"
  );
}
