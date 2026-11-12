//! Guards for ST3: a document converted after others on one thread, as a persistent `cortex_worker` converts its papers
//! (a `Converter` per paper, no engine reset between them), is the document converted alone. A new module: the concern,
//! state one conversion leaves to the next, has no table (72a's two one-thread tests each pin one class-copy record).

use latexml::util::test::{GUARD_JOBNAME, convert_on_one_thread, error_count, warning_count};

/// A repro of `tools/perfect_kernel/repros/`, its path the row's name.
macro_rules! repro {
  ($path:literal) => {
    (
      $path,
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/",
        $path,
        ".tex"
      )),
    )
  };
}

/// Each row's documents written as `<GUARD_JOBNAME>.tex` in a directory of its own: its source path, and the directory
/// kept alive with it.
fn written(tex: &str) -> (String, tempfile::TempDir) {
  let dir = tempfile::tempdir().expect("a document directory");
  let path = dir.path().join(format!("{GUARD_JOBNAME}.tex"));
  std::fs::write(&path, tex).expect("the document");
  (path.to_string_lossy().into_owned(), dir)
}

/// The log's Error and Warning lines, without their locators (which name the document's directory).
fn diagnostics(log: &str) -> Vec<&str> {
  log
    .lines()
    .filter(|line| {
      line.starts_with("Error:") || line.starts_with("Warning:") || line.starts_with("Fatal:")
    })
    .collect()
}

/// Each row: the documents converted before, the document, its (Error, Warning) counts alone. Converted after them on one
/// thread, the document has its counts, its diagnostics and its XML byte for byte as alone:
/// - an unknown KeyVals key is reported in each conversion that has it (keyvals.rs `SEEN_MISSING`, reset by
///   `initialize_report`; it was the thread's: the second conversion had no Warning; ST3 census, 16 of 64 papers lost a
///   'Frontmatter' keyset Info in each order, 1702.05670, 2511.20937);
/// - 90a's elsarticle row after the row before it (P4c3's question: the in-process row matched the CLI on each tree —
///   right on pk/ISO1 a161bbf507, wrong on its merge 9d38af1d10 — so it was the merge, not state carried over).
#[test]
fn a_document_after_another_on_one_thread_is_the_document_alone() {
  let keyval = repro!("macro-state/unknown_keyval_key_warns_in_each_document");
  let rows = [
    (vec![keyval], keyval, (0, 1)),
    (
      vec![repro!(
        "sectioning-frontmatter/revtex_affiliation_requested_in_formula_marks_part_one_line"
      )],
      repro!("sectioning-frontmatter/elsarticle_address_requested_in_formula_marks_part_one_line"),
      (0, 0),
    ),
  ];
  let mut failures = Vec::new();
  for (before, (name, tex), counts) in rows {
    let (source, _dir) = written(tex);
    let alone = convert_on_one_thread(vec![source.clone()], Some("ar5iv.sty")).remove(0);
    let befores: Vec<_> = before.iter().map(|(_, tex)| written(tex)).collect();
    let mut sources: Vec<String> = befores.iter().map(|(path, _)| path.clone()).collect();
    sources.push(source);
    let after = convert_on_one_thread(sources, Some("ar5iv.sty"))
      .pop()
      .expect("the document");
    let found = |log: &str| (error_count(log), warning_count(log));
    if found(&alone.0) != counts || found(&after.0) != counts {
      failures.push(format!(
        "{name}: counts alone {:?}, after {:?}, want {counts:?}\n{}",
        found(&alone.0),
        found(&after.0),
        after.0
      ));
    }
    if diagnostics(&alone.0) != diagnostics(&after.0) {
      failures.push(format!(
        "{name}: diagnostics alone {:#?}\nafter {:#?}",
        diagnostics(&alone.0),
        diagnostics(&after.0)
      ));
    }
    if alone.1.is_empty() || alone.1 != after.1 {
      failures.push(format!(
        "{name}: XML alone\n{}\nafter\n{}",
        alone.1, after.1
      ));
    }
  }
  assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
