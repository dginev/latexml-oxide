//! `\DeclareMathAlphabet` maps NFSS codes to abstract font properties.
//!
//! Perl runs its three arguments through `lookupTeXFont`
//! (`Common/Font.pm` L230-239) — the same family/series/shape tables
//! `\selectfont` consults — before storing them (`latex_constructs.pool.ltxml`
//! L2677-2687). Rust stored the raw NFSS codes, so
//! `\DeclareMathAlphabet{\mysf}{OT1}{cmss}{m}{n}` emitted `font="cmss m n"`
//! where Perl emits `font="sansserif"`, and the MathML then carried
//! `mathvariant="normal"` for every declared alphabet — a sansserif, bold or
//! italic alphabet rendered upright.
//!
//! `font::lookup_tex_font` was already a faithful port of `lookupTeXFont`
//! with **no callers** — the same dead-helper shape as `ding_fontmap.rs`
//! before the `\selectfont` fix.
//!
//! Blast radius is document- and package-declared alphabets only: the stock
//! `\mathsf`/`\mathbf`/`\mathit`/`\mathrm` are bound directly in the pool and
//! are unaffected (asserted below, so a future change cannot quietly route
//! them through the broken path). Verified against same-host Perl 0.8.8,
//! which emits `sansserif`, `bold`, `italic` for the declared trio.
mod cluster;
use cluster::convert_to_xml;

/// The `font=` attribute of every `<XMTok>`, in document order.
fn tok_fonts(xml: &str) -> Vec<String> {
  let mut out = Vec::new();
  for seg in xml.split("<XMTok").skip(1) {
    let Some(end) = seg.find('>') else { continue };
    let tag = &seg[..end];
    if let Some(i) = tag.find("font=\"") {
      let rest = &tag[i + 6..];
      if let Some(j) = rest.find('"') {
        out.push(rest[..j].to_string());
      }
    }
  }
  out
}

#[test]
fn declared_math_alphabets_carry_abstract_font_properties() {
  let xml = convert_to_xml("tests/cluster_regressions/declaremathalphabet_props.tex");
  assert_eq!(
    tok_fonts(&xml),
    vec!["sansserif", "bold", "italic"],
    "\\DeclareMathAlphabet stored raw NFSS codes again — Perl maps them \
     through lookupTeXFont, and the raw form leaks into the XML as e.g. \
     `font=\"cmss m n\"`, which costs the alphabet its MathML variant"
  );
}

#[test]
fn stock_math_alphabets_are_unaffected() {
  // \mathsf/\mathbf/\mathit are bound directly in the pool, so they were
  // already correct — pinned so a future change cannot quietly reroute them
  // through the declaration path this test's sibling protects.
  let xml = convert_to_xml("tests/cluster_regressions/stock_math_alphabets.tex");
  assert_eq!(
    tok_fonts(&xml),
    vec!["sansserif", "bold", "italic"],
    "a stock math alphabet lost its abstract font property"
  );
}
