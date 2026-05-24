// Source-locator (`--source-map`) MVP scaffold — issues #47/#92.
//
// Step 1 of the engine substrate (see `docs/SOURCE_PROVENANCE.md` and the
// "Source-locator MVP" section of `docs/SYNC_STATUS.md`): prove the
// `source_map` switch threads end-to-end — CLI → `Config` → `CoreOptions`
// → `StateOptions` → `State.source_map` (read via
// `state::source_map_enabled()`) — while remaining strictly additive and
// OFF by default.
//
// Fixture: the structural `tests/structure/article.tex`
// (sections / lists / tables / light math) — the ar5iv-editor preview-sync
// MVP target.
//
// Invariants pinned here:
//   * OFF (default): the core ltx XML carries no `data-sourcepos` attribute.
//   * ON (`source_map`): conversion still succeeds and is currently
//     byte-identical to OFF — the gate is wired but emission is not yet
//     implemented. When absorb-hook stamping lands (SOURCE_PROVENANCE §2),
//     flip `source_map_on_currently_inert` into a pinned `data-sourcepos` golden.
use latexml::converter::Converter;
use latexml_core::common::{Config, OutputFormat};

const ARTICLE: &str = "tests/structure/article.tex";

/// Convert the fixture to core ltx XML with the source-map switch in the
/// requested state. Returns the serialized XML (pre-post-processing).
fn convert_xml(source_map: bool) -> String {
  let config = Config {
    format: OutputFormat::XML,
    source_map: if source_map { Some(true) } else { None },
    ..Config::default()
  };
  let mut converter = Converter::from_config(config);
  converter
    .initialize_session()
    .expect("can initialize session");
  converter
    .convert(ARTICLE.to_string())
    .result
    .expect("conversion produced XML output")
}

#[test]
fn source_map_off_by_default_has_no_data_sourcepos() {
  let xml = convert_xml(false);
  assert!(
    !xml.contains("data-sourcepos"),
    "a default conversion must not emit any source-locator attribute"
  );
}

#[test]
fn source_map_on_emits_data_sourcepos() {
  // With the switch ON, `open_element` stamps each element (math kept
  // opaque) with a `data-sourcepos="tag:line:col[-tag:line:col]"`
  // attribute (SOURCE_PROVENANCE §0/§2). OFF must emit none.
  let off = convert_xml(false);
  let on = convert_xml(true);
  assert!(
    !off.contains("data-sourcepos"),
    "OFF (default) must emit no source-locator attribute"
  );
  assert!(
    on.contains("data-sourcepos=\""),
    "ON must stamp data-sourcepos attributes"
  );
  // Value shape sanity: at least one `tag:line:col` triple is present.
  let re = regex::Regex::new(r#"data-sourcepos="\d+:\d+:\d+"#).unwrap();
  assert!(
    re.is_match(&on),
    "data-sourcepos value must be tag:line:col(-tag:line:col); got none matching in output"
  );

  // Eyeball: count + a sample of distinct stamped values.
  let val_re = regex::Regex::new(r#"data-sourcepos="([^"]+)""#).unwrap();
  let vals: Vec<&str> = val_re
    .captures_iter(&on)
    .filter_map(|c| c.get(1).map(|m| m.as_str()))
    .collect();
  eprintln!("data-sourcepos count: {}", vals.len());
  for v in vals.iter().take(15) {
    eprintln!("  sample: {v}");
  }
  // What file does each tag resolve to? (drives the user-vs-foreign rule)
  for (i, s) in latexml_core::state::source_table_snapshot().iter().enumerate() {
    eprintln!("  tag {i} = {:?}", latexml_core::common::arena::to_string(*s));
  }

  // Math opacity (MVP scope): the `ltx:Math` wrapper may be stamped, but no
  // math-internal `ltx:XM*` element should carry a locator (we don't descend
  // into the Marpa-built MathML — SOURCE_PROVENANCE §7 A.3).
  let xm_stamped = regex::Regex::new(r#"<ltx:XM[A-Za-z]*\b[^>]*\bdata-sourcepos="#).unwrap();
  assert!(
    !xm_stamped.is_match(&on),
    "math must stay opaque — no ltx:XM* element may carry data-sourcepos"
  );
  // KNOWN DEFERRED GAP (SOURCE_PROVENANCE §7 A.3): the `ltx:Math` wrapper is
  // stamped during digestion but the Marpa math parser rebuilds the subtree
  // afterward (`base_xmath.rs`), discarding the stamp — so equations currently
  // inherit their containing element's locator via client-side DOM walk-up.
  // In-equation/math-wrapper provenance is out of MVP scope (math is opaque).
  let math_stamped = regex::Regex::new(r#"<ltx:Math\b[^>]*\bdata-sourcepos="#).unwrap();
  eprintln!(
    "ltx:Math wrapper stamped: {} (deferred — see §7 A.3)",
    math_stamped.is_match(&on)
  );
}
