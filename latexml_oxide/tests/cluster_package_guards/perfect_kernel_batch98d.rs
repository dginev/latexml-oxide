//! Red/green guards for batch 98d: `\end{X}` over a `\begingroup` opened inside X (raa.cls:322-349 `\institutename`,
//! `\begin{flushleft}\begingroup … \end{flushleft}\endgroup`; OXIDIZED_DESIGN_DIVERGENCES #965, KNOWN_PERL_ERRORS
//! #1207). Legal TeX: groups are untyped between `\begingroup` and `\endgroup` (tex.web §1063-1069), so `\end{X}`'s
//! `\endgroup` (latex.ltx:15384-15393) closes the innermost group and the class's own `\endgroup` closes X's. Ours, as
//! Perl, raised "Attempt to end mode" (Perl Stomach.pm:544-550), and the error held in the speculative title-page
//! deposit dropped raa's abstract and keywords. One code path (stomach.rs `end_environment_over_plain_groups`, called
//! by every DefEnvironment's `\end{X}`): the innermost plain group popped, X's mode and box ended, X's frame kept as
//! the plain group the next `\endgroup` closes. Distinct guards: the shape in the main galley and in a box; TeX's
//! scoping across it (one inner group, X's frame on top after the pop, and two, X's frame below another); the refusals
//! (another environment's `\end`, an `\end` with no `\begin` below); and the raw class's title page it unblocks.
//! Witnesses 1404.3644, 1304.5354, 1508.02561, 1509.00139, 2004.06992, 2110.11117, 2306.16795, 1604.05586,
//! 2003.13998, 2102.01352 (raa), 2511.10805 (raa_twocolumn). Repros under `tools/perfect_kernel/repros/`.
use latexml::util::test::{assert_element, convert_with};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch99b::converted,
};

/// The XML of `tex` converted under ar5iv, with `errors` Errors and `warnings` Warnings.
#[track_caller]
fn convert_counted(tex: &str, errors: usize, warnings: usize) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (errors, warnings),
    "{log}"
  );
  xml
}

/// raa's `\institutename` shape, in the main galley and inside a minipage: `\end{flushleft}` closes the class's
/// `\begingroup` and the flushleft, its words one left-aligned paragraph; the class's `\endgroup` closes the
/// environment's group, and the words after it are a paragraph of their own, not left-aligned.
#[test]
fn end_closes_a_begingroup_opened_inside_the_environment() {
  let xml = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/end_flushleft_closes_a_begingroup_opened_inside_it.tex"
    ),
    0,
    0,
  );
  assert_element(
    &xml,
    "document",
    &[],
    r#"<document xmlns="http://dlmf.nist.gov/LaTeXML">
      <para xml:id="p1"><p align="left" xml:id="p1.1">Institute words here.</p></para>
      <para xml:id="p2"><p xml:id="p2.1">Abstract words after.</p></para>
      <para xml:id="p3">
        <p xml:id="p3.1"><inline-block class="ltx_minipage" vattach="middle" width="142.3pt" xml:id="p3.1.1">
          <p align="left" xml:id="p3.1.1.1">Institute words here.</p>
          <p xml:id="p3.1.1.2">Boxed words after.</p>
        </inline-block></p>
      </para>
    </document>"#,
  );
}

/// TeX's scoping: `\end{X}` undoes only the innermost group; what X set before the class's `\begingroup` lasts until
/// the `\endgroup` that closes X's group, and each further `\endgroup` closes one more — with one inner group (X's
/// frame on top once it is popped) and with two (X's frame below the outer one). The text is pdflatex's.
#[test]
fn end_over_begingroups_keeps_tex_scoping() {
  let xml = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/end_environment_over_begingroups_keeps_tex_scoping.tex"
    ),
    0,
    0,
  );
  assert_element(
    &xml,
    "document",
    &[],
    r#"<document xmlns="http://dlmf.nist.gov/LaTeXML">
      <para xml:id="p1"><p align="left" xml:id="p1.1">One.</p></para>
      <para xml:id="p2"><p xml:id="p2.1">Kept [yes][no].After [no].</p></para>
      <para xml:id="p3"><p align="center" xml:id="p3.1">Two.</p></para>
      <para xml:id="p4"><p xml:id="p4.1">Kept [yes][yes][no].Then [yes][no].Last [no][no].</p></para>
    </document>"#,
  );
}

/// The refusals keep today's diagnostics, both LaTeX's `\@badend` too: another environment's `\end{Y}` over the
/// class's group inside `\begin{X}` (pdflatex 1 error), and an `\end{X}` with no `\begin{X}` below the group
/// (pdflatex 2). The words stay.
#[test]
fn end_over_begingroup_without_its_environment_stays_an_error() {
  let xml = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/end_other_environment_over_begingroup_stays_an_error.tex"
    ),
    2,
    1,
  );
  assert!(
    xml.contains("Mismatched words.") && xml.contains("After words."),
    "{xml}"
  );
  let xml = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/end_without_its_begin_over_begingroup_stays_an_error.tex"
    ),
    2,
    0,
  );
  assert!(
    xml.contains("Bare words.") && xml.contains("After words."),
    "{xml}"
  );
}

/// raa's title page (raa.cls:322-349 copied in a class of its own): with `\institutename` clean, the speculative
/// `\@maketitle` deposit is kept, and the class's "Abstract" and "Key words" paragraphs with it; the title and each
/// author's institute stay as before. Eager and `--streaming` alike.
#[test]
fn a_raa_title_page_keeps_its_abstract_and_keywords() {
  assert_eq!(
    converted(
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/raw_class_institutes_in_flushleft_begingroup_keep_the_title_page.tex"
      ),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B, City</contact> </creator> <para class="ltx_noindent" xml:id="p1"> <p xml:id="p1.1"><text font="bold" xml:id="p1.1.1">Abstract</text> We present the abstract words here.</p> </para> <para class="ltx_noindent" xml:id="p2"> <p xml:id="p2.1"><text font="bold" xml:id="p2.1.1">Key words: </text> stars: general, galaxies: photometry</p> </para> <section inlist="toc" xml:id="S1"> <tags> <tag>1</tag> <tag role="refnum">1</tag> <tag role="typerefnum">§1</tag> </tags> <title><tag close=" ">1</tag>Intro</title> <para xml:id="S1.p1"> <p xml:id="S1.p1.1">Text.</p> </para> </section>"#
  );
}
