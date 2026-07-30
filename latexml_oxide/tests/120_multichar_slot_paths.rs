//! Every decode path must honour a multi-character fontmap slot.
//!
//! Perl's `FontDecode` returns `$$map[$code]` whole, so a slot holding two
//! characters just works on every path. Rust splits the representation: the
//! array slot is a single `Option<char>` and multi-char entries live in a
//! `_fontmap_multichar` side table. Only `font::decode_str` consults that
//! table, so any caller of the single-`char` `font::decode` silently drops the
//! second character.
//!
//! T2B slot 128 is U+04F6 + U+0336 COMBINING LONG STROKE OVERLAY. Dropping the
//! overlay leaves U+04F6 — a *different* letter.
//!
//! There was also an ORDERING defect: `lookup_multichar_override` read the
//! table from state but ran *before* `decode`, which is what triggers the map
//! load. `\DeclareTextSymbol` decodes in the preamble and bakes the result
//! into a primitive body, so it hit the table before it existed and froze the
//! stripped letter for the whole document.
//!
//! Expectations verified against same-host Perl LaTeXML 0.8.8.
mod cluster;
use cluster::convert_to_xml;

fn marker_chars(xml: &str, name: &str) -> Vec<u32> {
  let open = format!("{name}[");
  let start = xml
    .find(&open)
    .unwrap_or_else(|| panic!("marker {name}[ absent"))
    + open.len();
  let rest = &xml[start..];
  let end = rest
    .find(']')
    .unwrap_or_else(|| panic!("marker {name}[ unclosed"));
  let mut out = String::new();
  let mut depth = 0usize;
  for c in rest[..end].chars() {
    match c {
      '<' => depth += 1,
      '>' => depth = depth.saturating_sub(1),
      _ if depth == 0 => out.push(c),
      _ => {},
    }
  }
  // Drop layout whitespace; only the glyph matters.
  out
    .chars()
    .map(|c| c as u32)
    .filter(|c| *c != 0x0A && *c != 0x20)
    .collect()
}

#[test]
fn declare_text_symbol_keeps_a_multichar_slot() {
  let xml = convert_to_xml("tests/cluster_regressions/multichar_slot_paths.tex");
  assert_eq!(
    marker_chars(&xml, "TS"),
    vec![0x04F6, 0x0336],
    "\\DeclareTextSymbol dropped the U+0336 overlay — either it is back on \
     font::decode instead of decode_str, or the multichar table is again being \
     consulted before the fontmap binding is loaded (it decodes in the \
     preamble, so it must preload the map itself)"
  );
}

#[test]
fn symbol_keeps_a_multichar_slot() {
  // Control: the same slot at document time, when the map is already loaded.
  // Worked before the fix; a failure here means the whole path broke.
  let xml = convert_to_xml("tests/cluster_regressions/multichar_slot_paths.tex");
  assert_eq!(
    marker_chars(&xml, "SYM"),
    vec![0x04F6, 0x0336],
    "\\symbol lost a multichar slot"
  );
}

/// KNOWN REMAINING GAP — pinned, not endorsed.
///
/// `\DeclareMathSymbol` routes through `mathchar.rs`, which calls the
/// single-`char` `font::decode` and stores the result in a
/// `glyph: Option<char>` field. Carrying a pair there needs that field to
/// become a string — a real type change through the math-char pipeline, so it
/// is deliberately out of scope here rather than done badly.
///
/// Perl gives `[U+04F6, U+0336]`. When this is fixed, THIS TEST WILL FAIL —
/// that is the intent. Update it to the Perl value and delete this note.
#[test]
fn declare_math_symbol_still_drops_the_overlay_known_gap() {
  let xml = convert_to_xml("tests/cluster_regressions/multichar_slot_paths.tex");
  assert_eq!(
    marker_chars(&xml, "MS"),
    vec![0x04F6],
    "the math-char decode path changed — if it now yields [0x04F6, 0x0336] \
     this gap is FIXED: update this test to expect the pair and remove it from \
     the known-gap list"
  );
}
