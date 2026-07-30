//! A fontmap slot holding TWO characters must survive into the output.
//!
//! Perl fontmaps may store a multi-character string in a slot — the six `msbm`
//! negated relations are base + U+0338 COMBINING LONG SOLIDUS OVERLAY
//! (`amsb.fontmap.ltxml` L20, L21, L23). Rust's slot is a single
//! `Option<char>`, so such entries live in a companion
//! `DeclareFontMapMultichar!` table that `font::decode_str` consults first.
//! `AMSb` had no such table, so these six slots decoded to the EMPTY STRING.
//!
//! Scope, checked rather than assumed: the familiar spellings `\nleqslant`,
//! `\ngeqslant`, `\nleqq`, `\ngeqq`, `\nsubseteqq`, `\nsupseteqq` are bound by
//! `amssymb` directly to explicit Unicode and never reach the fontmap — they
//! were correct all along and are NOT a guard for this. Only direct slot
//! access under the encoding exercises it.
//!
//! Expectations verified against same-host Perl LaTeXML 0.8.8.
mod cluster;
use cluster::convert_to_xml;

/// Text inside `NAME[...]`, markup stripped, as codepoints.
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
  out.chars().map(|c| c as u32).collect()
}

#[test]
fn amsb_multichar_slots_keep_their_combining_overlay() {
  let xml = convert_to_xml("tests/cluster_regressions/amsb_multichar_slots.tex");

  // Control first: a single-char slot on the same path. If this fails the
  // whole decode path broke, not the multichar table.
  assert_eq!(
    marker_chars(&xml, "OK"),
    vec![0x1D538],
    "the AMSb single-char path itself regressed (slot 65 should be 𝔸)"
  );

  for (marker, base) in [
    ("S10", 0x2A7Du32),
    ("S11", 0x2A7E),
    ("S20", 0x2266),
    ("S21", 0x2267),
    ("S34", 0x2AC5),
    ("S35", 0x2AC6),
  ] {
    assert_eq!(
      marker_chars(&xml, marker),
      vec![base, 0x0338],
      "AMSb slot behind {marker} lost its U+0338 overlay — the \
       DeclareFontMapMultichar! table is not being consulted, and the slot \
       decodes to the empty string (Perl gives the pair)"
    );
  }
}
