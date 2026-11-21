//! Red/green guards for batch REGFIXS, frontmatter regressions of the v0.8.0 per-paper check: a title-code `\let` to a
//! command reading arguments is that command in the observer's rehearsal too (rims02e.sty's `\let\mail\thanks`), and a
//! note made from an EPL `\institute` legend line prints its `\inst` mark under the class's title meaning. Each guard
//! asserts every creator element whole and the exact Error and Warning counts.

use super::{
  perfect_kernel_batch46::{convert, error_count, warning_count},
  perfect_kernel_batcherr2::creator_elements,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// rims02e.sty:209's `\let\mail\thanks` in `\@maketitle` (math0408269): `\mail{…}` is the name's note in the rehearsal as
/// in its digestion, so the affiliation piece before it does not take it cut from its argument, whose `\thanks` then read
/// the author markup's cleanup tail (`\let\mail\@undefined`: `undefined:\@undefined`, "Supported by X." a second creator).
/// Distinct: `title_code::reads_arguments` on a `\let` (the PC3 guard covers a `\def` with parameters).
#[test]
fn a_title_code_let_to_a_reader_keeps_its_argument() {
  let (log, xml) = convert(repro!("title_code_let_reader_keeps_its_argument"), false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creator_elements(&xml), [
    "<creator role=\"author\">\n    <personname>Ann Bee</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><break/>Kobe University</contact>\n    <note class=\"ltx_note_frontmatter ltx_thanks_funding\" mark=\"*\" role=\"thanks\">Supported by X.</note>\n  </creator>",
  ]);
}

/// An EPL `\institute{… \inst{*} Corresponding author: ann@a.fr}` under `Ann Able\inst{1,*}` (1009.3152, 1907.04695): the
/// legend line's note is digested with `\inst` the class's title meaning (epl2.cls:692, the binding's `\lx@title@inst`),
/// so it holds no undefined `\inst` and its printed `*` is the note's mark. Distinct: `\lx@frontmatter@plainsups`
/// (defs/author_macros.rs), which every note body runs.
#[test]
fn an_epl_institute_legend_note_prints_its_mark() {
  let (log, xml) = convert(repro!("epl_institute_legend_note_prints_its_mark"), false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creator_elements(&xml), [
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, Paris</contact>\n    <note class=\"ltx_note_frontmatter ltx_thanks_correspondence\" mark=\"*\" role=\"thanks\">Corresponding author: ann@a.fr</note>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B, Lyon</contact>\n  </creator>",
  ]);
}

/// An unbraced `\newcommand\affil[1]\relax` in the title code reads one argument (`[1]`), as pdflatex digests it: the
/// rehearsal holds it defined, so `\affil{Univ X}` is no name text (red, REGFIXS: taken for a `\let` by "no braced
/// body", 2 content-check Warnings). Distinct: `title_code::reads_arguments`'s `\let` shape (spaces and one `=` between
/// the definee and the target). Synthetic (the REGFIXS review).
#[test]
fn an_unbraced_newcommand_count_is_no_let() {
  let (log, xml) = convert(
    repro!("title_code_newcommand_count_without_braces_reads"),
    false,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let creators = creator_elements(&xml);
  assert_eq!(creators.len(), 2, "{xml}");
  assert!(
    creators[0].contains("<personname>Ann Bee</personname>"),
    "{xml}"
  );
  assert!(
    creators[1].contains("<personname>Bob Cee</personname>"),
    "{xml}"
  );
  assert!(!xml.contains("Univ X"), "{xml}");
}
