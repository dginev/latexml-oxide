//! Citation labels filled in by the post cross-referencer from each bibref's
//! `show` pattern — Perl `CrossRef::make_bibcite` (CrossRef.pm:498-644). The
//! bibliography and the links are built by the post stage, so every guard
//! converts to HTML: in one process (`--dest t.html`) and through the split
//! post session the corpus uses (`t.tex → t.xml`, then `--whatsin=xml t.xml`).
use std::process::Command;

use super::perfect_kernel_batch46::{error_count, warning_count};

const RAW: &str = "--preload=[rawstyles,rawclasses]latexml.sty";

const CROSSREF_TEX: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bibref_show_crossref.tex");
const CROSSREF_BIB: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bibref_show_crossref.bib");
const NATBIB_TEX: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bibref_show_natbib.tex");
const NATBIB_BIB: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bibref_show_natbib.bib");
const NUMBERS_TEX: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bibref_show_natbib_numbers.tex");
const SUPER_TEX: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bibref_show_natbib_super.tex");
const NUMBERS_BIB: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bibref_show_natbib_numbers.bib");
const ONCE_TEX: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bibref_show_biblatex_bbl_once.tex");
const ONCE_BBL: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bibref_show_biblatex_bbl_once.bbl");
const BIBLATEX_TEX: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bibref_show_biblatex_bbl.tex");
const BIBLATEX_BBL: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bibref_show_biblatex_bbl.bbl");

/// Convert `t.tex` (with side `files`) to HTML in one process. Returns
/// (ANSI-stripped stderr, HTML).
fn convert_html(tex: &str, files: &[(&str, &str)]) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  for (name, content) in files {
    std::fs::write(workdir.path().join(name), content).expect("write side file");
  }
  let output = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--nocomments",
      "--timeout=110",
      RAW,
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let html = std::fs::read_to_string(workdir.path().join("t.html")).unwrap_or_default();
  (stderr, html)
}

/// Convert `t.tex` to core XML, then post-process that XML alone to HTML in a
/// second process (`--whatsin=xml`, post_sweep.sh's split). Returns the
/// (ANSI-stripped) stderr of both runs and the HTML.
fn convert_split_html(tex: &str, files: &[(&str, &str)]) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  for (name, content) in files {
    std::fs::write(workdir.path().join(name), content).expect("write side file");
  }
  let core = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      RAW,
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn core");
  let post = Command::new(bin)
    .args([
      "--whatsin=xml",
      "t.xml",
      "--dest",
      "p.html",
      "--timeout=110",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn post");
  let stderr = format!(
    "{}{}",
    String::from_utf8_lossy(&core.stderr),
    String::from_utf8_lossy(&post.stderr)
  )
  .replace('\u{1b}', "");
  let html = std::fs::read_to_string(workdir.path().join("p.html")).unwrap_or_default();
  (stderr, html)
}

fn assert_crossref_cites(stderr: &str, html: &str) {
  assert_eq!(error_count(stderr), 0, "{stderr}");
  assert_eq!(warning_count(stderr), 0, "{stderr}");
  // (1) A plain `\cite` shows the refnum; its link carries the entry's title.
  latexml::util::test::assert_element(
    html,
    "p",
    &[r#"class="ltx_p""#],
    r##"<p class="ltx_p">Cite <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bib2" title="One chapter" class="ltx_ref">1</a>]</cite> and <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bib1" title="The whole volume" class="ltx_ref">2</a>]</cite>.</p>"##,
  );
  // (2) The bibliography's crossref bibref, `show="title, author"`.
  latexml::util::test::assert_element(
    html,
    "cite",
    &[r#"class="ltx_cite""#],
    r##"<cite class="ltx_cite"><a href="#bib.bib1" title="The whole volume" class="ltx_ref">The whole volume, Editor</a></cite>"##,
  );
}

/// Every citation link carries its entry's title as `title=` (Perl
/// CrossRef.pm:530-539 trims and collapses the bibitem's title tag, :555-557
/// puts it on the ref), and MakeBibliography's crossref bibref
/// (MakeBibliography.pm:639-642, `show => 'title, author'`) prints the parent's
/// title and authors: role words are case-folded with a trailing `s` stripped
/// (:585), `, ` passes through (:639), and the label is one link (:641-643). The
/// Rust fill only knew capitalized natbib keywords, so it printed the number
/// `2` and no title. Same-host Perl 0.8.8 renders both elements byte-identically.
#[test]
fn crossref_bibref_shows_title_and_author_and_links_carry_titles() {
  let files = [("bibref_show_crossref.bib", CROSSREF_BIB)];
  let (stderr, html) = convert_html(CROSSREF_TEX, &files);
  assert_crossref_cites(&stderr, &html);
  let (stderr, html) = convert_split_html(CROSSREF_TEX, &files);
  assert_crossref_cites(&stderr, &html);
}

/// (3) natbib author-year through the same walk: only the year is the link
/// (CrossRef.pm:608), authors and the bibrefphrase delimiters stay outside it,
/// and consecutive entries of the same authors share one author label, a
/// same-year run showing only the suffix (:610-616, "(Smith, 2001a, b)").
/// The whole paragraph is byte-identical to same-host Perl 0.8.8 (the previous
/// Rust linked the whole "Jones and Brown (1999)" and repeated "Smith").
#[test]
fn natbib_authoryear_links_the_year_and_merges_same_authors() {
  let (stderr, html) = convert_html(NATBIB_TEX, &[("bibref_show_natbib.bib", NATBIB_BIB)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &html,
    "p",
    &[r#"class="ltx_p""#],
    r##"<p class="ltx_p">Text <cite class="ltx_cite ltx_citemacro_citet">Jones and Brown (<a href="#bib.bib3" title="Joint work" class="ltx_ref">1999</a>)</cite> and <cite class="ltx_cite ltx_citemacro_citep">(Jones and Brown, <a href="#bib.bib3" title="Joint work" class="ltx_ref">1999</a>)</cite> and <cite class="ltx_cite ltx_citemacro_citep">(Smith, <a href="#bib.bib1" title="First paper" class="ltx_ref">2001a</a>, <a href="#bib.bib2" title="Second paper" class="ltx_ref">b</a>)</cite> and <cite class="ltx_cite ltx_citemacro_citet">Smith (<a href="#bib.bib1" title="First paper" class="ltx_ref">2001a</a>, <a href="#bib.bib2" title="Second paper" class="ltx_ref">b</a>)</cite>
and <cite class="ltx_cite ltx_citemacro_citeauthor"><a href="#bib.bib3" title="Joint work" class="ltx_ref">Jones and Brown</a></cite> and <cite class="ltx_cite ltx_citemacro_citeyear"><a href="#bib.bib3" title="Joint work" class="ltx_ref">1999</a></cite> and <cite class="ltx_cite ltx_citemacro_cite">Jones and Brown (<a href="#bib.bib3" title="Joint work" class="ltx_ref">1999</a>)</cite>.</p>"##,
  );
}

/// natbib numbers mode: `\citet`'s `number` role links each number and
/// absorbs the next same-author entry, "Smith [2, 3]" (CrossRef.pm:617-622),
/// and an entry with no authors, full authors or key demotes the whole
/// citation to refnum (:542). Byte-identical to same-host Perl 0.8.8; the tip
/// linked "Smith [2]" per entry.
#[test]
fn natbib_numbers_citet_merges_same_authors() {
  let (stderr, html) = convert_html(NUMBERS_TEX, &[(
    "bibref_show_natbib_numbers.bib",
    NUMBERS_BIB,
  )]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &html,
    "p",
    &[r#"class="ltx_p""#],
    r##"<p class="ltx_p">Text <cite class="ltx_cite ltx_citemacro_citet">Smith [<a href="#bib.bib1" title="First paper" class="ltx_ref">2</a>, <a href="#bib.bib2" title="Second paper" class="ltx_ref">3</a>]</cite> and <cite class="ltx_cite ltx_citemacro_citep">[<a href="#bib.bib1" title="First paper" class="ltx_ref">2</a>, <a href="#bib.bib2" title="Second paper" class="ltx_ref">3</a>]</cite> and <cite class="ltx_cite ltx_citemacro_citet"><a href="#bib.bib1" title="First paper" class="ltx_ref">2</a>, <a href="#bib.bib3" title="Anonymous notes" class="ltx_ref">1</a></cite>.</p>"##,
  );
}

/// natbib super mode: the `super` role wraps the number link in `<ltx:sup>`
/// (CrossRef.pm:623-631). Byte-identical to same-host Perl 0.8.8; the tip
/// had no `super` role and printed the literal word ("Smith Super").
#[test]
fn natbib_super_citet_superscripts_the_number() {
  let (stderr, html) = convert_html(SUPER_TEX, &[(
    "bibref_show_natbib_numbers.bib",
    NUMBERS_BIB,
  )]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &html,
    "p",
    &[r#"class="ltx_p""#],
    r##"<p class="ltx_p">Text <cite class="ltx_cite ltx_citemacro_citet">Smith <sup class="ltx_sup"><a href="#bib.bib1" title="First paper" class="ltx_ref">1</a></sup>; Smith <sup class="ltx_sup"><a href="#bib.bib2" title="Second paper" class="ltx_ref">2</a></sup></cite> and <cite class="ltx_cite ltx_citemacro_citep"><sup class="ltx_sup"><a href="#bib.bib1" title="First paper" class="ltx_ref">1</a></sup>; <sup class="ltx_sup"><a href="#bib.bib2" title="Second paper" class="ltx_ref">2</a></sup></cite>.</p>"##,
  );
}

/// A biblatex `.bbl` is read into bibentries and formatted by MakeBibliography
/// (batch 56jc, DIVERGENCES #306), which tags every bibitem with its title
/// (make_bibliography.rs, as MakeBibliography.pm:473-475 does), so the `title`
/// show role of `\citetitle` prints it — Perl CrossRef.pm:554 falls back only
/// to a key tag — and every link carries it as `title=`. The textual cite links
/// only the year, and apa's `\textcite` joins with "and" (apa.cbx:50-55).
#[test]
fn biblatex_bbl_citetitle_shows_the_title() {
  let (stderr, html) = convert_html(BIBLATEX_TEX, &[("t.bbl", BIBLATEX_BBL)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &html,
    "p",
    &[r#"class="ltx_p""#],
    r##"<p class="ltx_p">Title <cite class="ltx_cite ltx_citemacro_citetitle"><a href="#bib.bib1" title="A study of things" class="ltx_ref">A study of things</a></cite>; textual <cite class="ltx_cite ltx_citemacro_citet">Jones and Brown (<a href="#bib.bib2" title="On examples &amp; counterexamples" class="ltx_ref">2019</a>)</cite>.</p>"##,
  );
}

/// The `.bbl` title tag is not a second digest of the field: the `.bbl` reader
/// digests the field once into the bibentry, and MakeBibliography copies the
/// title into the tag (batch 56jc): a bibliography field is digested exactly
/// once (bib_field_digest_once.rs). `\hline` raises
/// `\noalign cannot be used here` on every digest — an undefined macro would
/// heal itself after the first — so a double digest counts 2. `\citetitle`
/// still shows the title (the `\hline` contributes no text).
#[test]
fn biblatex_bbl_title_is_digested_once() {
  let (stderr, html) = convert_html(ONCE_TEX, &[("t.bbl", ONCE_BBL)]);
  let n = stderr.matches("\\noalign cannot be used here").count();
  assert_eq!(n, 1, "the .bbl title was digested {n} times:\n{stderr}");
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &html,
    "p",
    &[r#"class="ltx_p""#],
    r##"<p class="ltx_p">Title <cite class="ltx_cite ltx_citemacro_citetitle"><a href="#bib.bib1" title="A study of things" class="ltx_ref">A study of things</a></cite>.</p>"##,
  );
}
