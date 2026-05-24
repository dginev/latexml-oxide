// Source-locator (`--source-map`) MVP — issues #47/#92.
//
// Engine substrate (see `docs/SOURCE_PROVENANCE.md` + the "Source-locator
// MVP" section of `docs/SYNC_STATUS.md`), OFF by default. With the switch on,
// `Document::open_element_at` stamps each element with its construct's source
// range in LaTeXML's `data:` namespace as `data:sourcepos="tag:l:c[-tag:l:c]"`
// (cmark-gfm-style value, file-first-class integer tag). The post XSLT's
// `copy_foreign_attributes` path then converts that to the HTML5
// `data-sourcepos` attribute — Perl LaTeXML's own foreign-attribute
// convention, so no XSLT change is needed.
//
// Fixture: the structural `tests/structure/article.tex`.
//
// Invariants pinned here:
//   * OFF (default): no source-locator attribute in core XML or HTML.
//   * ON: core XML carries `data:sourcepos` on user-source (`.tex`) elements
//     only (synthetic-default + foreign `.cls`/`.sty` sources skipped); math
//     internals (`ltx:XM*`) stay opaque; and it converts through to HTML
//     `data-sourcepos`.
use latexml::converter::Converter;
use latexml::post::PostOptions;
use latexml_core::common::{Config, OutputFormat};

const ARTICLE: &str = "tests/structure/article.tex";

/// Convert the fixture to core ltx XML with the source-map switch in the
/// requested state. Returns `(serialized XML pre-post-processing, conversion log)`.
fn convert_response(source_map: bool) -> (String, String) {
  let config = Config {
    format: OutputFormat::XML,
    source_map: if source_map { Some(true) } else { None },
    ..Config::default()
  };
  let mut converter = Converter::from_config(config);
  converter
    .initialize_session()
    .expect("can initialize session");
  let resp = converter.convert(ARTICLE.to_string());
  (
    resp.result.expect("conversion produced XML output"),
    resp.log,
  )
}

/// Convert the fixture to core ltx XML with the source-map switch in the
/// requested state. Returns the serialized XML (pre-post-processing).
fn convert_xml(source_map: bool) -> String { convert_response(source_map).0 }

/// Post-process core ltx XML into HTML5 (exercises the XSLT attribute path).
fn html_from(xml: &str) -> String {
  let opts = PostOptions {
    pmml:                      true,
    cmml:                      false,
    keep_xmath:                false,
    stylesheet:                Some("resources/XSLT/LaTeXML-html5.xsl"),
    destination:               None,
    source_directory:          Some("tests/structure"),
    nodefaultresources:        true,
    css_files:                 &[],
    js_files:                  &[],
    noinvisibletimes:          false,
    mathtex:                   false,
    navigationtoc:             None,
    schemadocs:                false,
    split:                     false,
    split_xpath:               None,
    split_naming:              None,
    xslt_parameters:           &[],
    graphics_svg_threshold_kb: 0,
    whatsout:                  latexml_post::extract::Whatsout::default(),
  };
  latexml::post::run_post_processing(xml, &opts)
}

#[test]
fn source_map_off_by_default_has_no_locator() {
  let xml = convert_xml(false);
  assert!(
    !xml.contains("data:sourcepos") && !xml.contains("data-sourcepos"),
    "a default conversion must not emit any source-locator attribute"
  );
  // Nor any sources-table / decoder artifact.
  assert!(
    !xml.contains("data:sources") && !xml.contains("sourceMappingURL"),
    "a default conversion must not emit any source-map table"
  );
}

#[test]
fn source_map_on_emits_data_sourcepos_in_core_xml() {
  let off = convert_xml(false);
  let on = convert_xml(true);
  assert!(
    !off.contains("data:sourcepos"),
    "OFF (default) must emit no source-locator attribute"
  );
  assert!(
    on.contains("data:sourcepos=\""),
    "ON must stamp data:sourcepos attributes"
  );
  // Value shape: at least one `tag:line:col` triple.
  let shape = regex::Regex::new(r#"data:sourcepos="\d+:\d+:\d+"#).unwrap();
  assert!(shape.is_match(&on), "value must be tag:line:col(-tag:line:col)");

  // Eyeball: count, a sample of values, and the tag→file table.
  let val_re = regex::Regex::new(r#"data:sourcepos="([^"]+)""#).unwrap();
  let vals: Vec<&str> = val_re
    .captures_iter(&on)
    .filter_map(|c| c.get(1).map(|m| m.as_str()))
    .collect();
  eprintln!("core data:sourcepos count: {}", vals.len());
  for v in vals.iter().take(12) {
    eprintln!("  sample: {v}");
  }
  for (i, s) in latexml_core::state::source_table_snapshot().iter().enumerate() {
    eprintln!("  tag {i} = {:?}", latexml_core::common::arena::to_string(*s));
  }

  // Math opacity (MVP scope, §7 A.3): no math-internal `ltx:XM*` element may
  // carry a locator. (The `ltx:Math` wrapper itself is currently unstamped —
  // the Marpa math parser rebuilds the subtree and discards the stamp; that
  // is a documented deferred gap, equations inherit the container's locator.)
  // Elements serialize under the default `ltx` namespace (no prefix), so match
  // the unprefixed XMath-family element names (`<XMTok …>` etc.).
  let xm = regex::Regex::new(r#"<XM[A-Za-z]*\b[^>]*\bdata:sourcepos="#).unwrap();
  assert!(
    !xm.is_match(&on),
    "math must stay opaque — no XMath-family element may carry data:sourcepos"
  );
}

/// Pinned golden: key structural elements of `article.tex` → their exact
/// `data:sourcepos`. Guards the locator pipeline (constructor capture,
/// user-source filter, the `get_locator` `from` heuristic) against coverage or
/// accuracy regressions. Values are **line-accurate**; column precision is a
/// Tier-B refinement (Bruce brucemiller/LaTeXML#101 — accurate construct-start
/// needs expansion-provenance, see SYNC_STATUS). Update deliberately if the
/// conversion legitimately changes.
#[test]
fn source_map_pins_key_structural_locators() {
  let on = convert_xml(true);
  // (element tag, exact data:sourcepos) — the FIRST occurrence of each tag,
  // cross-checked against tests/structure/article.tex line numbers.
  let golden: &[(&str, &str)] = &[
    ("section", "0:12:1-0:12:24"),        // \section{First Section}  (line 12)
    ("equation", "0:14:1-0:14:17"),       // \begin{equation}         (line 14)
    ("itemize", "0:40:1-0:40:16"),        // \begin{itemize}          (line 40)
    ("item", "0:41:9-0:41:9"),            // \item one                (line 41)
    ("enumerate", "0:49:1-0:49:18"),      // \begin{enumerate}        (line 49)
    ("description", "0:58:1-0:58:20"),    // \begin{description}      (line 58)
    ("subsection", "0:65:1-0:65:26"),     // \subsection{A Subsection}(line 65)
    ("subsubsection", "0:70:1-0:70:32"),  // \subsubsection{...}      (line 70)
  ];
  for (tag, expected) in golden {
    let re =
      regex::Regex::new(&format!(r#"<{}\b[^>]*?\bdata:sourcepos="([^"]+)""#, tag)).unwrap();
    let actual = re.captures(&on).and_then(|c| c.get(1)).map(|m| m.as_str());
    assert_eq!(
      actual,
      Some(*expected),
      "golden mismatch for first <{tag}> data:sourcepos (line-accurate; update if intended)"
    );
  }
}

// The `data:` namespace is promoted to a document namespace on first use
// (Perl `getDocumentNamespacePrefix($ns,1)` parity, in `Document::set_attribute`),
// so finalize declares `xmlns:data` on the root and the `data:sourcepos`
// attributes resolve into it. The post XSLT's `copy_foreign_attributes` then
// converts `data:sourcepos` → HTML5 `data-sourcepos` — no XSLT change, same
// path `aria:` already uses. See `docs/SOURCE_PROVENANCE.md` §7 A.5.
#[test]
fn source_map_passes_through_xslt_to_html() {
  let html_off = html_from(&convert_xml(false));
  let html_on = html_from(&convert_xml(true));
  assert!(
    !html_off.contains("data-sourcepos") && !html_off.contains("data:sourcepos"),
    "OFF: HTML must carry no source-locator attribute"
  );
  assert!(
    html_on.contains("data-sourcepos=\""),
    "ON: data:sourcepos must convert to HTML data-sourcepos via copy_foreign_attributes"
  );
}

/// The source table is conversion *metadata*: it is serialised to the `.log`
/// (the decoder ring, where the array index *is* the integer `tag`), NOT
/// inlined into the output. The XML/HTML carry only the anonymous tag (in
/// `data:sourcepos` / `data-sourcepos`) — never a `sources` table or a source
/// filename — so the output stays anonymisable for a consumer that lacks the
/// source files. In-process embedders (the ar5iv-editor server) read the same
/// table programmatically via `state::source_table_snapshot()`. See
/// `docs/SOURCE_PROVENANCE.md` §0.1.
#[test]
fn source_map_table_goes_to_log_not_output() {
  // Install the capture logger so the `.log` buffer receives Info records
  // (no-op if a logger is already installed in this test process). Without a
  // logger the global `log` sink is a no-op and nothing is captured.
  let _ = latexml_core::util::logger::init(log::LevelFilter::Info);
  let (xml, log) = convert_response(true);

  // Decoder ring lives in the log: a `source-map` record naming the user source.
  assert!(
    log.contains("source-map"),
    "the .log must carry the source-map decoder table"
  );
  assert!(
    log.contains("article.tex"),
    "the source-map log table must name the user source file; got log:\n{log}"
  );

  // Anonymity of the output: the decoder is NOT inlined into the core XML …
  assert!(
    !xml.contains("data:sources") && !xml.contains("sourceMappingURL"),
    "the sources table must not be inlined into the core XML (anonymity)"
  );
  // … nor into the HTML after the XSLT.
  let html = html_from(&xml);
  assert!(
    !html.contains("data-sources") && !html.contains("data:sources"),
    "the sources table must not be inlined into the HTML (anonymity)"
  );
  // The output still carries the anonymous per-element tags.
  assert!(
    xml.contains("data:sourcepos=\""),
    "core XML must still carry the anonymous data:sourcepos tags"
  );
}
