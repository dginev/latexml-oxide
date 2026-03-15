// Structure tests — individually listed for per-test #[ignore] support.
use latexml::util::test::*;
use std::rc::Rc;
const DIR: &str = "tests/structure";

// --- Currently passing tests (28) ---

#[test]
fn abstract_test() {
  latexml_test_single("tests/structure/abstract.tex", "abstract", DIR, None, None);
}

#[test]
fn app_test() {
  latexml_test_single("tests/structure/app.tex", "app", DIR, None, None);
}

#[test]
fn apps_test() {
  latexml_test_single("tests/structure/apps.tex", "apps", DIR, None, None);
}

#[test]
fn article_test() {
  latexml_test_single("tests/structure/article.tex", "article", DIR, None, None);
}

#[test]
fn authors_test() {
  latexml_test_single("tests/structure/authors.tex", "authors", DIR, None, None);
}

#[test]
fn autoref_test() {
  latexml_test_single("tests/structure/autoref.tex", "autoref", DIR, None, None);
}

#[test]
fn badabstract_test() {
  latexml_test_single("tests/structure/badabstract.tex", "badabstract", DIR, None, None);
}

#[test]
fn beforeafter_test() {
  latexml_test_single("tests/structure/beforeafter.tex", "beforeafter", DIR, None, None);
}

#[test]
fn book_test() {
  latexml_test_single("tests/structure/book.tex", "book", DIR, None, None);
}

#[test]
fn changectr_test() {
  latexml_test_single("tests/structure/changectr.tex", "changectr", DIR, None, None);
}

#[test]
fn columns_test() {
  latexml_test_single("tests/structure/columns.tex", "columns", DIR, None, None);
}

#[test]
fn endnote_test() {
  latexml_test_single("tests/structure/endnote.tex", "endnote", DIR, None, None);
}

#[test]
fn epitest_test() {
  latexml_test_single("tests/structure/epitest.tex", "epitest", DIR, None, None);
}

#[test]
fn faketitlepage_test() {
  latexml_test_single("tests/structure/faketitlepage.tex", "faketitlepage", DIR, None, None);
}

#[test]
fn fancyhdr_test() {
  latexml_test_single("tests/structure/fancyhdr.tex", "fancyhdr", DIR, None, None);
}

#[test]
fn footnote_test() {
  latexml_test_single("tests/structure/footnote.tex", "footnote", DIR, None, None);
}

#[test]
fn hyperref_test() {
  latexml_test_single("tests/structure/hyperref.tex", "hyperref", DIR, None, None);
}

#[test]
fn itemize_test() {
  latexml_test_single("tests/structure/itemize.tex", "itemize", DIR, None, None);
}

#[test]
fn mainfile_test() {
  latexml_test_single("tests/structure/mainfile.tex", "mainfile", DIR, None, None);
}

#[test]
fn para_test() {
  latexml_test_single("tests/structure/para.tex", "para", DIR, None, None);
}

#[test]
fn plainsample_test() {
  latexml_test_single("tests/structure/plainsample.tex", "plainsample", DIR, None, None);
}

#[test]
fn report_test() {
  latexml_test_single("tests/structure/report.tex", "report", DIR, None, None);
}

#[test]
fn sec_test() {
  latexml_test_single("tests/structure/sec.tex", "sec", DIR, None, None);
}

#[test]
fn titlepage_test() {
  latexml_test_single("tests/structure/titlepage.tex", "titlepage", DIR, None, None);
}

#[test]
fn filelist_test() {
  latexml_test_single("tests/structure/filelist.tex", "filelist", DIR, None,
    Some(Rc::new(latexml_contrib::dispatch)));
}

#[test]
fn floatnames_test() {
  latexml_test_single("tests/structure/floatnames.tex", "floatnames", DIR, None, None);
}

// --- Newly added tests (need package/infrastructure work) ---

#[test]
fn acro_test() {
  latexml_test_single("tests/structure/acro.tex", "acro", DIR, None, None);
}

#[test]
#[ignore] // 825 diffs — needs \@add@to@frontmatter, MathFork
fn amsarticle_test() {
  latexml_test_single("tests/structure/amsarticle.tex", "amsarticle", DIR, None, None);
}

#[test]
fn bibsect_test() {
  latexml_test_single("tests/structure/bibsect.tex", "bibsect", DIR, None, None);
}

#[test]
fn crazybib_test() {
  latexml_test_single("tests/structure/crazybib.tex", "crazybib", DIR, None, None);
}

#[test]
fn csquotes_test() {
  latexml_test_single("tests/structure/csquotes.tex", "csquotes", DIR, None, None);
}

#[test]
fn enum_test() {
  latexml_test_single("tests/structure/enum.tex", "enum", DIR, None, None);
}

#[test]
#[ignore] // 362 diffs — needs MathFork infrastructure (afterConstruct rearrangement)
fn eqnums_test() {
  latexml_test_single("tests/structure/eqnums.tex", "eqnums", DIR, None, None);
}

#[test]
#[ignore] // needs graphicx figure grid support
fn figure_grids_test() {
  latexml_test_single("tests/structure/figure_grids.tex", "figure_grids", DIR, None, None);
}

#[test]
fn figures_test() {
  latexml_test_single("tests/structure/figures.tex", "figures", DIR, None, None);
}

#[test]
fn glossary_test() {
  latexml_test_single("tests/structure/glossary.tex", "glossary", DIR, None, None);
}

#[test]
#[ignore] // 979 diffs — eqnarray MathFork/MathBranch structure (math parser), QED symbol, equation numbering
fn ieee_test() {
  latexml_test_single("tests/structure/IEEE.tex", "IEEE", DIR, None, None);
}

#[test]
fn natbib_test() {
  latexml_test_single("tests/structure/natbib.tex", "natbib", DIR, None, None);
}

#[test]
fn options_test() {
  latexml_test_single("tests/structure/options.tex", "options", DIR, None,
    Some(Rc::new(latexml_contrib::dispatch)));
}

#[test]
fn paralists_test() {
  latexml_test_single("tests/structure/paralists.tex", "paralists", DIR, None, None);
}

#[test]
fn subcaption_test() {
  latexml_test_single("tests/structure/subcaption.tex", "subcaption", DIR, None, None);
}

#[test]
fn svabstract_test() {
  latexml_test_single("tests/structure/svabstract.tex", "svabstract", DIR, None, None);
}
