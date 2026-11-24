//! Red/green guards for batch NATBIB: natbib's apalike label `\bibitem[Maes, 1995]{maes}` (no `(year)`) split at its
//! first `, ` into author and year, as natbib.sty:1005-1012 `\NAT@apalk` does, where Perl took the trailing digits as
//! the year ("(Maes,, 1995)", KNOWN_PERL_ERRORS #1332, OXIDIZED_DESIGN_DIVERGENCES #1161). Each guard converts its repro
//! under `tools/perfect_kernel/repros/index-bib/` to HTML with no Error or Warning and asserts the paragraph whole,
//! against pdflatex. Witness 2505.11584.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch78j::html_of;

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// The bare label's split (`\NAT@bare` → `\NAT@apalk`): a year with a suffix, two authors, "et al.", a braced author
/// holding its own `, `, a whole-braced label, a braced `(` before `, <year>` or `(year)` (2605.20003); the `(year)`
/// label as before; `\citealp`/`\citeauthor`/`\citeyear`/`\citeyearpar`/`\citealt`; and `\citet*`/`\citep*`, whose
/// full authors are empty (they printed the label). Red (MAIN999): "(Maes,, 1995)", "Maes, (1995)", "(Maes, 1995a, )",
/// "Maes, 1995 (1995)", "(Group et al., 2011, EBCTCG)". Repro index-bib/natbib_apalike_label_splits_at_comma.
#[test]
fn natbib_apalike_label_splits_at_comma() {
  let html = html_of(repro!("index-bib/natbib_apalike_label_splits_at_comma.tex"));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">A <cite class="ltx_cite ltx_citemacro_citep">(Maes, <a href="#bib.bib1" title="" class="ltx_ref">1995</a>)</cite>, <cite class="ltx_cite ltx_citemacro_citet">Maes (<a href="#bib.bib1" title="" class="ltx_ref">1995</a>)</cite>. B <cite class="ltx_cite ltx_citemacro_citep">(Maes, <a href="#bib.bib2" title="" class="ltx_ref">1995a</a>)</cite>, <cite class="ltx_cite ltx_citemacro_citet">Maes (<a href="#bib.bib2" title="" class="ltx_ref">1995a</a>)</cite>. C <cite class="ltx_cite ltx_citemacro_citep">(Smith and Jones, <a href="#bib.bib3" title="" class="ltx_ref">2001</a>)</cite>, <cite class="ltx_cite ltx_citemacro_citet">Smith and Jones (<a href="#bib.bib3" title="" class="ltx_ref">2001</a>)</cite>. D <cite class="ltx_cite ltx_citemacro_citep">(Smith et al., <a href="#bib.bib4" title="" class="ltx_ref">2001</a>)</cite>, <cite class="ltx_cite ltx_citemacro_citet">Smith et al. (<a href="#bib.bib4" title="" class="ltx_ref">2001</a>)</cite>.
E <cite class="ltx_cite ltx_citemacro_citep">(Maes, <a href="#bib.bib5" title="" class="ltx_ref">1995</a>)</cite>, <cite class="ltx_cite ltx_citemacro_citet">Maes (<a href="#bib.bib5" title="" class="ltx_ref">1995</a>)</cite>. F <cite class="ltx_cite ltx_citemacro_citep">Maes, <a href="#bib.bib1" title="" class="ltx_ref">1995</a></cite>; <cite class="ltx_cite ltx_citemacro_citeauthor"><a href="#bib.bib1" title="" class="ltx_ref">Maes</a></cite>; <cite class="ltx_cite ltx_citemacro_citeyear"><a href="#bib.bib1" title="" class="ltx_ref">1995</a></cite>; <cite class="ltx_cite ltx_citemacro_citeyearpar">(<a href="#bib.bib1" title="" class="ltx_ref">1995</a>)</cite>;
<cite class="ltx_cite ltx_citemacro_citet">Maes <a href="#bib.bib1" title="" class="ltx_ref">1995</a></cite>. G <cite class="ltx_cite ltx_citemacro_citep">(Smith, Jones, <a href="#bib.bib6" title="" class="ltx_ref">2002</a>)</cite>, <cite class="ltx_cite ltx_citemacro_citet">Smith, Jones (<a href="#bib.bib6" title="" class="ltx_ref">2002</a>)</cite>; <cite class="ltx_cite ltx_citemacro_citep">(Maes, <a href="#bib.bib7" title="" class="ltx_ref">1996</a>)</cite>. H <cite class="ltx_cite ltx_citemacro_citet">Maes (<a href="#bib.bib1" title="" class="ltx_ref">1995</a>)</cite>; <cite class="ltx_cite ltx_citemacro_citep">(Maes, <a href="#bib.bib1" title="" class="ltx_ref">1995</a>)</cite>. I <cite class="ltx_cite ltx_citemacro_citep">(Group (EBCTCG) et al., <a href="#bib.bib8" title="" class="ltx_ref">2011</a>)</cite>,
<cite class="ltx_cite ltx_citemacro_citet">Group (EBCTCG) et al. (<a href="#bib.bib8" title="" class="ltx_ref">2011</a>)</cite>; <cite class="ltx_cite ltx_citemacro_citep">(Group (EBCTCG) et al., <a href="#bib.bib9" title="" class="ltx_ref">2012</a>)</cite>, <cite class="ltx_cite ltx_citemacro_citet">Group (EBCTCG) et al. (<a href="#bib.bib9" title="" class="ltx_ref">2012</a>)</cite>.</p>"##,
  );
}

/// The `\citename{Jones et al., }1990` label (natbib.sty:1029 hands name and year to `\NAT@apalk`), and the apalike
/// label in numbers mode, both on the label's other code paths. Red (MAIN999): "Maes, [1]", "Jones et al., [2]", the
/// author "Jones et al.,". Repro index-bib/natbib_citename_label_splits_at_comma.
#[test]
fn natbib_citename_label_splits_at_comma() {
  let html = html_of(repro!(
    "index-bib/natbib_citename_label_splits_at_comma.tex"
  ));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">A <cite class="ltx_cite ltx_cite_numeric ltx_citemacro_citep">[<a href="#bib.bib1" title="" class="ltx_ref">1</a>]</cite>, <cite class="ltx_cite ltx_citemacro_citet">Maes [<a href="#bib.bib1" title="" class="ltx_ref">1</a>]</cite>; <cite class="ltx_cite ltx_citemacro_citeauthor"><a href="#bib.bib1" title="" class="ltx_ref">Maes</a></cite>; <cite class="ltx_cite ltx_citemacro_citeyear"><a href="#bib.bib1" title="" class="ltx_ref">1995</a></cite>. B <cite class="ltx_cite ltx_cite_numeric ltx_citemacro_citep">[<a href="#bib.bib2" title="" class="ltx_ref">2</a>]</cite>, <cite class="ltx_cite ltx_citemacro_citet">Jones et al. [<a href="#bib.bib2" title="" class="ltx_ref">2</a>]</cite>; <cite class="ltx_cite ltx_citemacro_citeauthor"><a href="#bib.bib2" title="" class="ltx_ref">Jones et al.</a></cite>;
<cite class="ltx_cite ltx_citemacro_citeyear"><a href="#bib.bib2" title="" class="ltx_ref">1990</a></cite>.</p>"##,
  );
}
