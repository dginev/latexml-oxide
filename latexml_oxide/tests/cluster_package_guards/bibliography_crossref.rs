//! A crossref'd `.bib` entry. BibTeX.pool.ltxml:111-167 runs an entry's
//! preparers BEFORE it reads the field list: `copyCrossrefFields` (:213-225)
//! adds the fields the child inherits, which then get their handlers (batch
//! 56ja). MakeBibliography then prints "See [parent]" and not the host's title
//! (MakeBibliography.pm:729-734, a `not()` predicate the post XPath evaluates
//! since batch 56jd). Repro `tools/perfect_kernel/repros/index-bib/bib_crossref_host_title`.
use std::process::Command;

use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

const TEX: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_crossref_host_title.tex");
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

/// Convert the repro to HTML in one process (the bibliography is built by
/// post). Returns (ANSI-stripped stderr, HTML).
fn convert_html() -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
  std::fs::write(workdir.path().join("bib_crossref_host_title.bib"), BIB).expect("write .bib");
  let output = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--nocomments",
      "--timeout=110",
      "--preload=[rawstyles,rawclasses]latexml.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let html = std::fs::read_to_string(workdir.path().join("t.html")).unwrap_or_default();
  (stderr, html)
}

/// The child prints its inherited year, "See" its parent by title and editor
/// (Perl's `do_crossref`, show="title, author"), and no host row: the host
/// title/editor rows (MakeBibliography.pm:732-734) and the Rust-only host
/// publisher/place rows (DIVERGENCES #286) all carry
/// `[not(../ltx:bib-related[@bibrefs])]`, as plain.bst's
/// `format.incoll.inproc.crossref` prints "In [1], pages 1–10".
#[test]
fn crossref_child_sees_its_parent_and_skips_the_host_title() {
  let (stderr, html) = convert_html();
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &html,
    "li",
    &[r#"id="bib.bib2""#],
    r##"<li id="bib.bib2" class="ltx_bibitem ltx_bib_incollection"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[1]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Bob Author</span><span class="ltx_text ltx_bib_year"> (2001)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">One chapter</span>.
</span>
<span class="ltx_bibblock">See <span class="ltx_text ltx_bib_crossref"><cite class="ltx_cite"><a href="#bib.bib1" title="The whole volume" class="ltx_ref">The whole volume, Editor</a></cite></span>,
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_pages">pp. 1–10</span>.
</span>
<span class="ltx_bibblock ltx_bib_cited">Cited by: <a href="#p1" title="" class="ltx_ref">p1</a>.
</span></li>"##,
  );
}
