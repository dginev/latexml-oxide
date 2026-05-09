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

use latexml_core::common::relaxng::{Pattern, Relaxng};
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

