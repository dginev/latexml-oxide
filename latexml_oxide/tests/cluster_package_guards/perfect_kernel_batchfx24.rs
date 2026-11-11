//! Guards for FX24 (RR1's gap 24): what a class or document appends to `\@author` after the author setter stored it —
//! more authors after `\and`, rows of the authors before it — is read at `\maketitle` as the author block's
//! continuation (frontmatter_stores.rs `author_tail_calls`), also where the binding's `\author` stores nothing in
//! `\@author` and declares the class prints the tail (authblk's, llncs's: `seen_author_tail`) — only where the title page
//! in force prints `\@author` (`title_page_prints_author`, FX24b). Each converts its repro and asserts every creator's
//! name and contacts in order and the exact Error and Warning counts. Perl reads only `\author`'s argument and loses the
//! tail.

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $path
    ))
  };
}

fn owned(rows: &[&[&str]]) -> Vec<Vec<String>> {
  rows
    .iter()
    .map(|row| row.iter().map(|s| (*s).to_string()).collect())
    .collect()
}

/// `\newcommand\coauthor[1]{\g@addto@macro\@author{\and #1}}` after `\author{Alice A\\Inst X}`: the tail after the
/// stored argument holds a top-level `\and`, so what follows it is one more author block (`\lx@add@authors@append`),
/// which LaTeX's `\@maketitle` prints as a second column. Red (int158a): one creator Alice A, "Bob B" and "Inst Y" lost
/// with no Warning (witness 2511.15315's mechanism; jaa.cls/pramana.cls `\addauthor`). Distinct: the kernel route's
/// `\and` split with nothing before the `\and`.
#[test]
fn a_coauthor_appended_to_author_is_kept() {
  let (log, xml) = run(repro!("article_coauthor_appended_to_author_is_kept.tex"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators(&xml),
    owned(&[&["Alice A", "affiliation=Inst X"], &[
      "Bob B",
      "affiliation=Inst Y"
    ]]),
    "{xml}"
  );
}

/// A class's `\affil` appending `\\#1` and `\coauthor` appending `\and #1` after `\author{Alice A}`: the rows before the
/// first `\and` are the call's creator's (author-block rows, as 59d hands a tail), the rest Bob B with his line.
/// Red (int158a): Alice A alone, "Inst X", "Bob B", "Inst Y" lost, counted as 1 word. Distinct: rows and more authors
/// in one tail (`row_calls`, then the append), the rows' `annotate=all` reaching only the creators queued before.
#[test]
fn a_class_coauthor_after_its_rows_is_kept() {
  let (log, xml) = run(repro!(
    "class_coauthor_appended_to_author_keeps_the_column.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators(&xml),
    owned(&[&["Alice A", "authorblock=Inst X"], &[
      "Bob B",
      "affiliation=Inst Y"
    ]]),
    "{xml}"
  );
}

/// authblk: its `\author` stores nothing in the kernel `\@author`, so `\coauthor{Bob B}`'s `\and Bob B` follows the
/// value `\@author` held as the authors were queued (`seen_author_tail`); authblk prints it after the affiliations.
/// Red (int158a): Alice A alone, "Bob B" lost, 1 Warning. Distinct: the binding route.
#[test]
fn an_authblk_coauthor_appended_to_author_is_kept() {
  let (log, xml) = run(repro!(
    "authblk_coauthor_appended_to_author_keeps_the_name.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators(&xml),
    owned(
      &[&["Alice A", "affiliation=Inst X", "affiliation=Inst Y"], &[
        "Bob B"
      ]]
    ),
    "{xml}"
  );
}

/// 2511.15315's shape: authblk authors, then `\g@addto@macro\@author{\\[0.5em]\small\texttt{…}, \texttt{…}}`, a line of
/// e-mails under the block (no `\and`): an address the block's authors share (`row_calls` under the binding route,
/// `\lx@add@affiliation@shared`, `place_address_runs`), one address for each author in order, each on its author. Red (int158a): the line lost, 1
/// Warning; FX24: the line one author-block row of both creators (review revFX24 N3, S2). Distinct: the binding route's
/// rows, and their e-mail placement.
#[test]
fn an_authblk_email_line_appended_to_author_is_kept() {
  let (log, xml) = run(repro!(
    "authblk_email_row_appended_to_author_goes_to_its_owners.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators(&xml),
    owned(
      &[&["Alice A", "affiliation=Inst X", "email=alice@x.org"], &[
        "Bob B",
        "affiliation=Inst Y",
        "email=bob@y.org"
      ]]
    ),
    "{xml}"
  );
}

/// The title page in force prints only the title, so a tail appended to `\@author` prints nothing there and no creator
/// or row is made of it (`title_page_prints_author`; review revFX24 finding 1: FX24 invented "Bob B / Inst Y" in each).
/// One row per shape: a document's `\renewcommand\maketitle` (the `\and` tail and the 59d row tail, a Document-origin
/// `\lx@dropped@maketitle`), a raw class's own `\maketitle` and `\@maketitle`, a raw class's `\maketitle` and its
/// `\@maketitle` after `\LoadClass{article}` (File-origin, walked), elsarticle (a binding that does not declare
/// `author_tail`), authblk under a document's `\def\@maketitle` (declared, but its title page does not print
/// `\@author`). elsarticle's and authblk's 1 Warning is the title-page check's over-count of "bob" (its run prints
/// `\@author` there; pdflatex does not), as on int158a. Distinct: the gate of both routes.
#[test]
fn an_author_tail_the_title_page_does_not_print_is_not_read() {
  let alice_x: &[&[&str]] = &[&["Alice A", "affiliation=Inst X"]];
  let rows: [(&str, &str, usize, &[&[&str]]); 7] = [
    (
      "document \\maketitle, \\and tail",
      repro!("article_author_and_tail_unprinted_by_own_maketitle_makes_no_creator.tex"),
      0,
      alice_x,
    ),
    (
      "document \\maketitle, row tail",
      repro!("article_author_row_tail_unprinted_by_own_maketitle_makes_no_row.tex"),
      0,
      alice_x,
    ),
    (
      "raw class's own title page",
      repro!("raw_class_own_maketitle_author_tail_makes_no_creator.tex"),
      0,
      alice_x,
    ),
    (
      "raw class's \\maketitle over article",
      repro!("raw_class_renewed_maketitle_author_tail_makes_no_creator.tex"),
      0,
      alice_x,
    ),
    (
      "raw class's \\@maketitle over article",
      repro!("raw_class_renewed_at_maketitle_author_tail_makes_no_creator.tex"),
      0,
      alice_x,
    ),
    (
      "elsarticle",
      repro!("elsarticle_author_tail_outside_frontmatter_makes_no_creator.tex"),
      1,
      &[&["Alice A", "address=Inst X"]],
    ),
    (
      "authblk, document \\@maketitle",
      repro!("authblk_author_tail_unprinted_by_document_maketitle_makes_no_creator.tex"),
      1,
      alice_x,
    ),
  ];
  for (shape, tex, warnings, expected) in rows {
    let (log, xml) = run(tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, warnings),
      "{shape}: {log}"
    );
    assert_eq!(creators(&xml), owned(expected), "{shape}: {xml}");
  }
}

/// llncs's `\@maketitle` prints `\@author` (llncs.cls:1035) while its binding's `\author` stores nothing in it: the
/// binding declares `author_tail`, so `\and Bob B\\Inst Y` is one more author (pdflatex: "Alice A and Bob B",
/// "Inst Y", "Inst X" under the block). Red (int158a): Bob B lost, 1 Warning. Distinct: the declaration's positive side
/// beside authblk's, on a class binding.
#[test]
fn an_llncs_author_tail_is_kept() {
  let (log, xml) = run(repro!("llncs_author_tail_printed_keeps_the_author.tex"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators(&xml),
    owned(&[&["Alice A", "affiliation=Inst X"], &[
      "Bob B",
      "affiliation=Inst X",
      "affiliation=Inst Y"
    ]]),
    "{xml}"
  );
}

/// amsart's `\author` stores its argument in `\@author` and its title page prints `\authors`, not `\@author`: a document's
/// `\g@addto@macro\@author{\and Bob B}` prints nothing (pdflatex: "ALICE A" alone), so no creator is made of it — the
/// binding route reads a tail only under a binding declaring `author_tail` (authblk, llncs), which amsart's does
/// not. Distinct: the route's refusal for a setter that stores `\@author` itself. The 1 Warning is the title-page
/// check's over-count of "bob" (its stand-in prints `\@author` with the tail; amsart's title page does not), as for
/// elsarticle and authblk above: a safe-side false count, kept until the one title-page reading (TP1,
/// `agent_reports/2026-11-11_TP0_report.md`) runs the class's own title page. FX26/FX26b tried a binding declaration
/// and each round silenced a counted loss elsewhere (revFX26, revFX26b: titling over amsart), so it was parked for TP1.
#[test]
fn an_amsart_author_append_invents_no_creator() {
  let (log, xml) = run(
    "\\documentclass{amsart}\n\\begin{document}\n\\title{A Title}\n\\author{Alice A}\n\
     \\makeatletter\\g@addto@macro\\@author{\\and Bob B}\\makeatother\n\\maketitle\nBody.\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains("its output does not hold: bob"), "{log}");
  assert_eq!(creators(&xml), owned(&[&["Alice A"]]), "{xml}");
}
