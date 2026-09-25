//! A crossref'd `.bib` entry (batch 56ja). BibTeX.pool.ltxml:111-167 runs an
//! entry's preparers BEFORE it reads the field list: `copyCrossrefFields`
//! (:213-225) adds the fields the child inherits, which then get their
//! handlers. Repro `tools/perfect_kernel/repros/index-bib/bib_crossref_host_title`.
use std::process::Command;

use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

const BIB: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_crossref_host_title.bib");

/// Convert `t.bib` to XML on its own. Returns (ANSI-stripped stderr, XML).
fn convert_bib(bib: &str) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.bib"), bib).expect("write t.bib");
  let output = Command::new(bin)
    .args(["t.bib", "--dest", "t.xml", "--nocomments", "--timeout=110"])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
  (stderr, xml)
}

/// The child `@incollection` inherits its parent's `year` and `publisher`: the
/// year prints as its publication date, the publisher joins the host book.
/// Perl 0.8.8 writes the same element.
#[test]
fn crossref_child_inherits_its_fields() {
  let (stderr, xml) = convert_bib(BIB);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "bibentry",
    &[r#"key="part""#],
    r#"<bibentry key="part" type="incollection" xml:id="bib.bib2">
        <bib-name role="author">
          <surname>Author</surname>
          <givenname>Bob</givenname>
        </bib-name>
        <bib-title>One chapter</bib-title>
        <bib-related role="host" type="book">
          <bib-title>The Whole Volume</bib-title>
          <bib-publisher>Pub</bib-publisher>
        </bib-related>
        <bib-related bibrefs="whole" role="host"/>
        <bib-part role="pages">1–10</bib-part>
        <bib-date role="publication">2001</bib-date>
        <bib-data role="self" type="BibTeX">@incollection{part,
    author = {Bob Author},
     title = {One Chapter},
 booktitle = {The Whole Volume},
  crossref = {whole},
     pages = {1–10},
 publisher = {Pub},
      year = {2001}}
</bib-data>
      </bibentry>"#,
  );
}
