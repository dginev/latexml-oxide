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

use latexml_core::common::relaxng::{simplify::simplify_top, Pattern, Relaxng};
use std::path::{Path, PathBuf};

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
  patterns.iter().map(|p| count_recursive(p, &mut predicate)).sum()
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
  assert!(matches!(body[0], Pattern::Grammar { .. }), "first item is Grammar");

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
    rng.modules.iter().all(|m| matches!(m, Pattern::Module { .. })),
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

