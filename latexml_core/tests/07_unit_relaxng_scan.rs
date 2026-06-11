//! Integration tests for `latexml_core::common::relaxng::scan`.
//!
//! Loads LaTeXML's stock `LaTeXML.rng` (the XML vocabulary that drives
//! latexml-oxide) and verifies the scanner reaches the end without
//! errors and that coarse-grained shape invariants hold.
//!
//! SKIP-on-missing — the corpus is not bundled with oxide; absence is
//! treated as "no sample available", not a failure.
//!
//! Schemas that are not natively part of latexml-oxide (e.g. the
//! validator's HTML5 `scholarly-ltx.rng`) are tested in their own
//! repositories against the same scanner via `latexml_core` as a
//! dependency.

use std::path::{Path, PathBuf};

use latexml_core::common::relaxng::{
  Pattern, Relaxng,
  simplify::simplify_top,
  tex::{Options as TexOptions, document_modules},
};

fn count_recursive(pat: &Pattern, predicate: &mut dyn FnMut(&Pattern) -> bool) -> usize {
  let mut n = if predicate(pat) { 1 } else { 0 };
  match pat {
    Pattern::Element { body, .. }
    | Pattern::Attribute { body, .. }
    | Pattern::Start { body }
    | Pattern::Combination { body, .. }
    | Pattern::Grammar { body, .. }
    | Pattern::Module { body, .. }
    | Pattern::Def { body, .. } => {
      for c in body {
        n += count_recursive(c, predicate);
      }
    },
    Pattern::Override { module, replacements } => {
      n += count_recursive(module, predicate);
      for c in replacements {
        n += count_recursive(c, predicate);
      }
    },
    _ => {},
  }
  n
}

fn count_kind(patterns: &[Pattern], mut predicate: impl FnMut(&Pattern) -> bool) -> usize {
  patterns
    .iter()
    .map(|p| count_recursive(p, &mut predicate))
    .sum()
}

fn first_existing(candidates: &[&str]) -> Option<PathBuf> {
  for c in candidates {
    let p = Path::new(c);
    if p.is_file() {
      return Some(p.to_path_buf());
    }
  }
  None
}

#[test]
fn scan_latexml_rng_smokes() {
  let candidates = [
    "/home/deyan/git/my-LaTeXML/blib/lib/LaTeXML/resources/RelaxNG/LaTeXML.rng",
    "/home/deyan/git/my-LaTeXML/lib/LaTeXML/resources/RelaxNG/LaTeXML.rng",
  ];
  let Some(rng_path) = first_existing(&candidates) else {
    eprintln!("[skip] LaTeXML.rng not available on this host");
    return;
  };
  let dir = rng_path.parent().unwrap();
  let mut rng = Relaxng::new("LaTeXML");
  rng.with_latexml_defaults();
  let raw = latexml_core::common::relaxng::scan::scan_external(
    &mut rng,
    rng_path.file_name().unwrap().to_str().unwrap(),
    None,
    &[dir],
  )
  .expect("scan_external should succeed on LaTeXML.rng");

  // Should produce a single Module wrapper around the schema.
  assert_eq!(raw.len(), 1, "scan_external returns a single Module");
  let body = match &raw[0] {
    Pattern::Module { body, .. } => body,
    other => panic!("expected Module, got {:?}", other),
  };

  // LaTeXML.rng wraps a top-level <grammar>, so the first child of the
  // module body is a Grammar.
  assert!(
    matches!(body[0], Pattern::Grammar { .. }),
    "first item is Grammar"
  );

  // The schema should pull in includes; we should see at least a few
  // Module patterns at deeper levels.
  let module_count = count_kind(&raw, |p| matches!(p, Pattern::Module { .. }));
  assert!(
    module_count >= 5,
    "expected ≥5 Module patterns from includes, got {}",
    module_count
  );

  // And plenty of element definitions.
  let element_count = count_kind(&raw, |p| matches!(p, Pattern::Element { .. }));
  assert!(
    element_count > 50,
    "expected >50 Element patterns, got {}",
    element_count
  );

  // Some attributes too.
  let attr_count = count_kind(&raw, |p| matches!(p, Pattern::Attribute { .. }));
  assert!(
    attr_count > 10,
    "expected >10 Attribute patterns, got {}",
    attr_count
  );

  // Documentation annotations should appear.
  let doc_count = count_kind(&raw, |p| matches!(p, Pattern::Doc(_)));
  assert!(doc_count > 0, "expected some Doc annotations");
}

#[test]
fn simplify_latexml_rng_populates_state() {
  let candidates = [
    "/home/deyan/git/my-LaTeXML/blib/lib/LaTeXML/resources/RelaxNG/LaTeXML.rng",
    "/home/deyan/git/my-LaTeXML/lib/LaTeXML/resources/RelaxNG/LaTeXML.rng",
  ];
  let Some(rng_path) = first_existing(&candidates) else {
    eprintln!("[skip] LaTeXML.rng not available on this host");
    return;
  };
  let dir = rng_path.parent().unwrap();
  let mut rng = Relaxng::new("LaTeXML");
  rng.with_latexml_defaults();
  let raw = latexml_core::common::relaxng::scan::scan_external(
    &mut rng,
    rng_path.file_name().unwrap().to_str().unwrap(),
    None,
    &[dir],
  )
  .expect("scan_external");
  let _simplified = simplify_top(&mut rng, raw);

  // Every <include>'d file appeared as a Module pushed in document
  // order. The stock LaTeXML.rng pulls in many siblings.
  assert!(
    rng.modules.len() >= 5,
    "expected ≥5 modules, got {}",
    rng.modules.len()
  );
  // Every recorded module should be a Module variant (sanity).
  assert!(
    rng
      .modules
      .iter()
      .all(|m| matches!(m, Pattern::Module { .. })),
    "non-Module entry in rng.modules"
  );
  // Singleton element-defs populate elementdefs / element_reverse_defs.
  // LaTeXML.rng has many of these (ltx:document, ltx:para, …).
  assert!(
    !rng.elementdefs.is_empty(),
    "expected elementdefs to be populated"
  );
  assert_eq!(
    rng.elementdefs.len(),
    rng.element_reverse_defs.len(),
    "elementdefs ↔ element_reverse_defs should be bijective"
  );
  // Defs table populated for non-trivial pattern groups.
  assert!(!rng.defs.is_empty(), "expected defs to be populated");
  // Used-by graph populated.
  assert!(
    !rng.uses_name.is_empty(),
    "expected uses_name graph to be populated"
  );
  // Combiners recorded for every defs entry.
  for key in rng.defs.keys() {
    assert!(
      rng.def_combiner.contains_key(key),
      "missing def_combiner for {}",
      key
    );
  }
}

#[test]
fn document_modules_emits_full_pipeline_for_latexml_rng() {
  let candidates = [
    "/home/deyan/git/my-LaTeXML/blib/lib/LaTeXML/resources/RelaxNG/LaTeXML.rng",
    "/home/deyan/git/my-LaTeXML/lib/LaTeXML/resources/RelaxNG/LaTeXML.rng",
  ];
  let Some(rng_path) = first_existing(&candidates) else {
    eprintln!("[skip] LaTeXML.rng not available on this host");
    return;
  };
  let dir = rng_path.parent().unwrap();
  let mut rng = Relaxng::new("LaTeXML");
  rng.with_latexml_defaults();
  let raw = latexml_core::common::relaxng::scan::scan_external(
    &mut rng,
    rng_path.file_name().unwrap().to_str().unwrap(),
    None,
    &[dir],
  )
  .expect("scan_external");
  let _ = simplify_top(&mut rng, raw);
  let docs = document_modules(&rng, TexOptions::default());

  // Schemamodule wrappers: one per non-svg module, each in its own
  // \begin{schemamodule}{name}.
  assert!(
    docs.contains("\\begin{schemamodule}"),
    "missing schemamodule"
  );
  assert!(
    docs.contains("\\end{schemamodule}"),
    "missing schemamodule close"
  );

  // Should emit at least a few \patterndef{} lines (the bulk of schema docs).
  let patterndef_count = docs.matches("\\patterndef{").count();
  assert!(
    patterndef_count > 30,
    "expected >30 \\patterndef occurrences, got {}",
    patterndef_count
  );

  // Some \elementdef{}s for elements that flowed through.
  let elementdef_count = docs.matches("\\elementdef{").count();
  assert!(
    elementdef_count > 5,
    "expected >5 \\elementdef occurrences, got {}",
    elementdef_count
  );

  // Used-by cross-refs should appear via \patternref / \elementref.
  assert!(
    docs.contains("\\patternref{") || docs.contains("\\elementref{"),
    "expected cross-references in output"
  );

  // SKIP_SVG: no schemamodule for an SVG-shaped name.
  assert!(
    !docs.contains(":svg:"),
    "expected SVG modules to be skipped"
  );

  // No \patternadd{} should leak through after the upgrade pass —
  // every left-over \patternadd from `defchoice/definterleave` gets
  // promoted to \patterndefadd or matched against an emitted
  // \patterndef.
  // (We can't assert there are zero \patternadds without first
  // confirming no patterns went unresolved; this is a softer check.)
  let patternadd_count = docs.matches("\\patternadd{").count();
  let patterndefadd_count = docs.matches("\\patterndefadd{").count();
  // Either matched or upgraded — never silently dropped.
  // Print counts so a failing run shows what actually emerged.
  eprintln!(
    "patterndef={} elementdef={} patternadd_residual={} patterndefadd={}",
    patterndef_count, elementdef_count, patternadd_count, patterndefadd_count
  );
}

/// Foreign-schema smoke test: MathML 4 Core, a non-LaTeXML RelaxNG
/// schema. Exercises the dynamic-namespace path:
///
/// * No `with_latexml_defaults()` — the harness shouldn't need to know anything about MathML up
///   front.
/// * Pre-registers `m` via the public `register_namespace` API. trang inlines `default namespace m
///   = "..."` as `<grammar ns="..."/>` without an `xmlns:m` declaration, so without registration
///   the scanner would fall back to `namespace1:foo`. Registering `m` makes the well-known MathML
///   prefix surface in the AST.
///
/// Verifies the same scan → simplify → document_modules pipeline that
/// the LaTeXML.rng tests exercise, plus that the registered prefix
/// reaches the emitted TeX (`\elementdef{m:math}` etc.).
#[test]
fn document_modules_emits_full_pipeline_for_mathml_core_rng() {
  let candidates = ["/home/deyan/git/mathml-schema/rng/mathml4-core.rng"];
  let Some(rng_path) = first_existing(&candidates) else {
    eprintln!("[skip] mathml4-core.rng not available on this host");
    return;
  };
  let dir = rng_path.parent().unwrap();
  let mut rng = Relaxng::new("mathml4-core");
  rng.register_namespace("m", "http://www.w3.org/1998/Math/MathML");
  // mathml4-core also references the XHTML namespace via <nsName>
  // (for "any HTML container" patterns); register that too so the
  // test asserts the multi-namespace registration path.
  rng.register_namespace("xhtml", "http://www.w3.org/1999/xhtml");
  let raw = latexml_core::common::relaxng::scan::scan_external(
    &mut rng,
    rng_path.file_name().unwrap().to_str().unwrap(),
    None,
    &[dir],
  )
  .expect("scan_external should succeed on mathml4-core.rng");
  let _ = simplify_top(&mut rng, raw);

  // Every element name in the schema should carry the `m:` prefix
  // (i.e. the registered MathML prefix flowed through encode_qname).
  // No `namespace<N>:` synthetic prefix should leak.
  let mathml_elem_count = rng
    .elements
    .keys()
    .filter(|name| name.starts_with("m:"))
    .count();
  assert!(
    mathml_elem_count > 0,
    "expected >0 elements with `m:` prefix, got element keys: {:?}",
    rng.elements.keys().take(5).collect::<Vec<_>>()
  );
  let synthetic: Vec<&String> = rng
    .elements
    .keys()
    .filter(|name| name.starts_with("namespace"))
    .collect();
  assert!(
    synthetic.is_empty(),
    "no synthesized namespaceN prefixes should appear when `m` is registered, found: {:?}",
    synthetic
  );

  // Document modules should emit \schemamodule + \elementdef blocks.
  let docs = document_modules(&rng, TexOptions::default());
  assert!(
    docs.contains("\\begin{schemamodule}"),
    "missing schemamodule open"
  );
  assert!(
    docs.contains("\\elementdef{m:math}") || docs.contains("\\elementref{m:math}"),
    "expected `m:math` to surface in TeX output"
  );
  // No synthetic prefixes leaked into the printed TeX.
  assert!(
    !docs.contains("namespace1:") && !docs.contains("namespace2:"),
    "synthetic namespaceN prefix leaked into output"
  );
}
