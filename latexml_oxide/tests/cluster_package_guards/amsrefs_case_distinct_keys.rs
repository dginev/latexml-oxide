//! Two amsrefs `\bib` keys that differ only in case are one key in the bibliography post-processing (lowercased, as
//! Perl's MakeBibliography `lc` and bibtex's case-insensitive keys), so one entry is lost; amsrefs keys are
//! case-sensitive and pdflatex prints both. Until the keys keep their case through citation and bibliography, the loss
//! is an honest Error naming the replaced entry (make_bibliography.rs `scan_bibentries`; bibtex calls the same
//! collision "Repeated entry"). Repro `tools/perfect_kernel/repros/index-bib/amsrefs_case_distinct_keys_both_kept`.
use std::process::Command;

use super::perfect_kernel_batch46::error_count;

const TEX: &str = include_str!(
  "../../../tools/perfect_kernel/repros/index-bib/amsrefs_case_distinct_keys_both_kept.tex"
);

/// Convert the repro to HTML in one process (the bibliography is built by post). Returns (ANSI-stripped stderr, HTML).
fn convert_html() -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
  let output = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let html = std::fs::read_to_string(workdir.path().join("t.html")).unwrap_or_default();
  (stderr, html)
}

/// The collision is reported once, on purpose, naming both keys and the replaced entry; the surviving entry is the
/// later one, as before.
#[test]
fn case_distinct_amsrefs_keys_report_the_lost_entry() {
  let (log, html) = convert_html();
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.contains(
      "Bibliography entries 'Able' and 'ABLE' differ only in case: they share one key and the entry 'Able' is replaced"
    ),
    "{log}"
  );
  assert_eq!(html.matches(r#"class="ltx_bibitem"#).count(), 1, "{html}");
  assert!(html.contains("Other"), "{html}");
}
