//! Guards for FX24 (RR1's gap 24): what a class or document appends to `\@author` after the author setter stored it —
//! more authors after `\and`, rows of the authors before it — is read at `\maketitle` as the author block's
//! continuation (frontmatter_stores.rs `author_tail_calls`), also where the binding's `\author` stores nothing in
//! `\@author` and declares the class prints the tail (authblk's, llncs's: `seen_author_tail`) — only where the title page
//! in force prints the tail (the title-page reading, `latex_constructs::title_page_reading`, TP1; FX24b's walk before). Each converts its repro and asserts every creator's
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

/// A row of [`an_author_tail_the_title_page_does_not_print_is_not_read`]: the shape, its repro, its Warning count,
/// whether "bob" is annotated, the creators.
type Row = (
  &'static str,
  &'static str,
  usize,
  bool,
  &'static [&'static [&'static str]],
);

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
/// or row is made of it (the title-page reading's `tail_printed`; review revFX24 finding 1: FX24 invented "Bob B / Inst Y" in each).
/// One row per shape: a document's `\renewcommand\maketitle` (the `\and` tail and the 59d row tail, a Document-origin
/// `\lx@dropped@maketitle`), a raw class's own `\maketitle` and `\@maketitle`, a raw class's `\maketitle` and its
/// `\@maketitle` after `\LoadClass{article}` (File-origin, walked), elsarticle (a binding that does not declare
/// `author_tail`), authblk under a document's `\def\@maketitle` (declared, but its title page does not print
/// `\@author`), acmart, an unknown class (OmniBus; its one Warning the missing class's), article under a document's
/// title-only `\@maketitle` (TP0's p2), and amsart with a document's addition to `\@maketitle`. Under elsarticle,
/// acmart, OmniBus and amsart the title-page check's stand-in for article.cls's page prints "bob", which pdflatex does
/// not: the title page in force positively does not print the appended tail (elsarticle's `\elsauthors`, acmart's
/// `\addresses` and amsart's `\authors`, which their bindings declare for their class, `AuthorPage::Own`; OmniBus's
/// unknown page), so the check reports it as annotated, an Info, not a Warning (FX27/FX27b, the user's ruling of
/// 2026-11-11; int158a-int159a: 1 Warning "The title page prints …"). A document's `\@maketitle` runs as written in
/// the title-page reading's run (TP1), so under authblk and article it prints no "bob" at all, as pdflatex does
/// (int160a: the stand-in printed it — authblk's an Info, article's a false Warning) — but only where the class's
/// `\maketitle` runs `\@maketitle` (TP1b, `TitlePageDecl::maketitle_runs`): elsarticle's and acmart's never do, so a
/// document's `\@maketitle` printing `\@author` makes no creator of the tail there (TP1 invented "Bob B", review revTP1
/// B1), and one printing the title alone leaves the check reading the stand-in's words, not the title alone. report under
/// `notitlepage` runs `\@maketitle` (report.cls:175-225; TP1b's undeclared report invented "Bob B", review revTP1b), and
/// article runs it after a document's `\@titlepagetrue`, which comes after article.cls:169 read it
/// (`MaketitleRuns::WithoutTitlepage`, keyed on the class's `\if@titlepage` after its options, TP1c). Distinct: the gate
/// of both routes, the Info side of the check's evidence rule, the `\maketitle` that runs no `\@maketitle`, and the
/// keyed rule's `notitlepage` side and its time of reading.
#[test]
fn an_author_tail_the_title_page_does_not_print_is_not_read() {
  let alice_x: &[&[&str]] = &[&["Alice A", "affiliation=Inst X"]];
  let annotated = "Info:frontmatter:annotated The title page annotates 1 word(s) that its output does not hold: bob";
  let alice: &[&[&str]] = &[&["Alice A"]];
  let rows: [Row; 19] = [
    (
      "document \\maketitle, \\and tail",
      repro!("article_author_and_tail_unprinted_by_own_maketitle_makes_no_creator.tex"),
      0,
      false,
      alice_x,
    ),
    (
      "document \\maketitle, row tail",
      repro!("article_author_row_tail_unprinted_by_own_maketitle_makes_no_row.tex"),
      0,
      false,
      alice_x,
    ),
    (
      "raw class's own title page",
      repro!("raw_class_own_maketitle_author_tail_makes_no_creator.tex"),
      0,
      false,
      alice_x,
    ),
    (
      "raw class's \\maketitle over article",
      repro!("raw_class_renewed_maketitle_author_tail_makes_no_creator.tex"),
      0,
      false,
      alice_x,
    ),
    (
      "raw class's \\@maketitle over article",
      repro!("raw_class_renewed_at_maketitle_author_tail_makes_no_creator.tex"),
      0,
      false,
      alice_x,
    ),
    (
      "elsarticle",
      repro!("elsarticle_author_tail_outside_frontmatter_makes_no_creator.tex"),
      0,
      true,
      &[&["Alice A", "address=Inst X"]],
    ),
    (
      "authblk, document \\@maketitle",
      repro!("authblk_author_tail_unprinted_by_document_maketitle_makes_no_creator.tex"),
      0,
      false,
      alice_x,
    ),
    (
      "article, document title-only \\@maketitle",
      repro!("article_document_title_only_maketitle_prints_no_author_tail.tex"),
      0,
      false,
      alice,
    ),
    (
      "amsart, appended \\@maketitle",
      repro!("amsart_author_tail_under_appended_maketitle_is_annotated.tex"),
      0,
      true,
      alice,
    ),
    (
      "acmart",
      repro!("acmart_author_tail_unprinted_is_annotated.tex"),
      0,
      true,
      alice,
    ),
    (
      "OmniBus, an unknown class (its one Warning the missing class's)",
      repro!("omnibus_unknown_class_author_tail_is_annotated.tex"),
      1,
      true,
      alice,
    ),
    (
      "elsarticle, document \\@maketitle printing \\@author",
      repro!("elsarticle_author_tail_under_document_maketitle_is_annotated.tex"),
      0,
      true,
      alice,
    ),
    (
      "acmart, document \\@maketitle printing \\@author",
      repro!("acmart_author_tail_under_document_maketitle_is_annotated.tex"),
      0,
      true,
      alice,
    ),
    (
      "elsarticle, document title-only \\@maketitle",
      repro!("elsarticle_document_title_only_maketitle_tail_is_annotated.tex"),
      0,
      true,
      alice,
    ),
    (
      "report[notitlepage], document title-only \\@maketitle",
      repro!("report_notitlepage_document_title_only_maketitle_prints_no_author_tail.tex"),
      0,
      false,
      alice,
    ),
    (
      "article, \\@titlepagetrue after the class, document title-only \\@maketitle",
      repro!("article_titlepage_set_after_load_document_maketitle_prints_no_author_tail.tex"),
      0,
      false,
      alice,
    ),
    // extsizes' copies, declared as the standard classes (review revTP1c: undeclared, they invented "Bob B")
    (
      "extarticle, document title-only \\@maketitle",
      repro!("extarticle_document_title_only_maketitle_prints_no_author_tail.tex"),
      0,
      false,
      alice,
    ),
    (
      "extreport[notitlepage], document title-only \\@maketitle",
      repro!("extreport_notitlepage_document_title_only_maketitle_prints_no_author_tail.tex"),
      0,
      false,
      alice,
    ),
    (
      "extbook[notitlepage], document title-only \\@maketitle",
      repro!("extbook_notitlepage_document_title_only_maketitle_prints_no_author_tail.tex"),
      0,
      false,
      alice,
    ),
  ];
  for (shape, tex, warnings, annotates, expected) in rows {
    let (log, xml) = run(tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, warnings),
      "{shape}: {log}"
    );
    assert!(!log.contains("frontmatter:unclaimed"), "{shape}: {log}");
    assert_eq!(log.contains(annotated), annotates, "{shape}: {log}");
    assert_eq!(creators(&xml), owned(expected), "{shape}: {xml}");
  }
}

/// The title page in force prints the appended tail and the reading makes no creator of it: a real loss, counted by the
/// title-page check's Warning, never as annotated (FX27). One row per shape: article with a document `\@maketitle` that
/// prints a `\let` copy of `\@author` (revFX24b's RED: the title-page reading's run marks `\@author`, not its copy); and
/// a binding declaring no page of its own (no `AuthorPage::Own`), whose page prints the tail — IEEEtran (an `\and` tail, and an affiliation and e-mail
/// line: 4 words), mnras, revtex4-1 (revFX27's blocker: FX27 made each an Info). pdflatex prints the tail in each (the repros'
/// headers). Distinct: the Warning side of the check's evidence rule.
#[test]
fn an_author_tail_printed_but_not_read_is_counted() {
  let alice_x: &[&[&str]] = &[&["Alice A", "affiliation=Inst X"]];
  let alice: &[&[&str]] = &[&["Alice A"]];
  let bob = "1 word(s) of the document's that its output does not hold: bob";
  // (shape, repro, the Warning's words, the creators)
  let rows: [(&str, &str, &str, &[&[&str]]); 5] = [
    (
      "article, a copy of \\@author",
      repro!("article_author_tail_printed_through_let_copy_of_author_is_lost_red.tex"),
      bob,
      alice_x,
    ),
    (
      "IEEEtran",
      repro!("ieeetran_author_tail_printed_is_lost_red.tex"),
      bob,
      alice,
    ),
    (
      "mnras",
      repro!("mnras_author_tail_printed_is_lost_red.tex"),
      bob,
      alice,
    ),
    (
      "revtex4-1",
      repro!("revtex4_1_author_tail_printed_is_lost_red.tex"),
      bob,
      alice,
    ),
    (
      "IEEEtran, an affiliation and e-mail tail",
      repro!("ieeetran_author_affiliation_email_tail_printed_is_lost_red.tex"),
      "4 word(s) of the document's that its output does not hold: lunar institute example org",
      alice,
    ),
  ];
  for (shape, tex, words, expected) in rows {
    let (log, xml) = run(tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 1),
      "{shape}: {log}"
    );
    assert!(
      log.contains(&format!(
        "Warning:frontmatter:unclaimed The title page prints {words}"
      )),
      "{shape}: {log}"
    );
    assert!(!log.contains("frontmatter:annotated"), "{shape}: {log}");
    assert_eq!(creators(&xml), owned(expected), "{shape}: {xml}");
  }
}

/// The title page in force prints the appended tail and the title-page reading's run sees it print, so the frontmatter
/// reads it (TP1, `latex_constructs::title_page_reading`): article with a document's addition to LaTeX's `\@maketitle`
/// (`\g@addto@macro`) or a `\let` copy of it — the kernel's `\@maketitle` holds the marker where the run puts
/// article.cls's page — amsart under a document's `\@maketitle` printing `\@author`, and titling's binding
/// `\@maketitle` (titling.sty:166-180) over amsart and elsarticle, each run its own evidence over the class binding's
/// declaration (FX26's and revFX26b's REDs), and IEEEtran under a document's `\@maketitle` printing `\@author` (its
/// `\maketitle` runs `\@maketitle`, as its binding declares, `TitlePageDecl::maketitle_runs`: TP1b). Under `titlepage`
/// (article's option, report's and book's default) the class's `\maketitle` prints its own page with `\@author` and runs
/// no `\@maketitle` (article.cls:169-192, report.cls:175-198, book.cls:181-204), so the stand-in decides, with or
/// without a document's title-only `\@maketitle` (`MaketitleRuns::WithoutTitlepage`, TP1c; TP1b lost article's "Bob B"
/// silently, review revTP1b). pdflatex prints "Bob B" in each (the repros' headers). Red (int160a): Alice A alone,
/// 1 Warning "bob" each (article[titlepage] without a `\@maketitle` of the document's: green throughout). Distinct: the
/// run's evidence on both routes, kernel and binding, and the keyed rule's `titlepage` side for each class.
#[test]
fn an_author_tail_the_title_page_prints_is_read() {
  let alice_bob_x: &[&[&str]] = &[&["Alice A", "affiliation=Inst X"], &[
    "Bob B",
    "affiliation=Inst Y",
  ]];
  let alice_bob: &[&[&str]] = &[&["Alice A"], &["Bob B"]];
  let rows: [(&str, &str, &[&[&str]]); 11] = [
    (
      "article, appended \\@maketitle",
      repro!("article_author_tail_under_appended_maketitle_is_kept.tex"),
      alice_bob_x,
    ),
    (
      "article, \\let-copied \\@maketitle",
      repro!("article_author_tail_under_let_copied_maketitle_is_kept.tex"),
      alice_bob_x,
    ),
    (
      "amsart, document \\@maketitle printing \\@author",
      repro!("amsart_author_tail_printed_by_document_maketitle_is_kept.tex"),
      &[&["Alice A"], &["Bob B"]],
    ),
    (
      "amsart, titling",
      repro!("amsart_titling_author_tail_printed_is_kept.tex"),
      &[&["Alice A"], &["Bob B"]],
    ),
    (
      "elsarticle, titling",
      repro!("elsarticle_titling_author_tail_printed_is_kept.tex"),
      &[&["Alice A"], &["Bob B"]],
    ),
    (
      "IEEEtran, document \\@maketitle printing \\@author",
      repro!("ieeetran_author_tail_printed_by_document_maketitle_is_kept.tex"),
      &[&["Alice A"], &["Bob B"]],
    ),
    (
      "article[titlepage], document title-only \\@maketitle",
      repro!("article_titlepage_document_title_only_maketitle_author_tail_is_kept.tex"),
      alice_bob,
    ),
    (
      "article[titlepage]",
      repro!("article_titlepage_author_tail_is_kept.tex"),
      alice_bob,
    ),
    (
      "report (titlepage, its default), document title-only \\@maketitle",
      repro!("report_document_title_only_maketitle_author_tail_is_kept.tex"),
      alice_bob,
    ),
    (
      "book (titlepage, its default), document title-only \\@maketitle",
      repro!("book_document_title_only_maketitle_author_tail_is_kept.tex"),
      alice_bob,
    ),
    (
      "extreport (titlepage, its default), document title-only \\@maketitle",
      repro!("extreport_document_title_only_maketitle_author_tail_is_kept.tex"),
      alice_bob,
    ),
  ];
  for (shape, tex, expected) in rows {
    let (log, xml) = run(tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{shape}: {log}"
    );
    assert!(!log.contains("frontmatter:annotated"), "{shape}: {log}");
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
/// not. Distinct: the route's refusal for a setter that stores `\@author` itself. The title-page check's stand-in prints
/// `\@author` with the tail ("bob"), which amsart's title page does not: amsart's binding owns `\author` and declares
/// that amsart's page prints its own `\authors` (`AuthorPage::Own`, FX27b), so the check reports "bob" as annotated, an Info, not a Warning (FX27, the user's ruling of 2026-11-11:
/// the author may mean it hidden, no damage can be claimed). int159a pinned it at 1 Warning (`ff7473bd2c`); FX26/FX26b's
/// binding declaration, which dropped the tail from the check's print, is parked (revFX26, revFX26b: it silenced a
/// counted loss), its mechanism not merged. Its second conversion is one loss with words of both severities (FX27's
/// split). Distinct from the tables: the Info alone, then the Warning and the Info side by side.
#[test]
fn an_amsart_author_append_invents_no_creator() {
  let (log, xml) = run(
    "\\documentclass{amsart}\n\\begin{document}\n\\title{A Title}\n\\author{Alice A}\n\
     \\makeatletter\\g@addto@macro\\@author{\\and Bob B}\\makeatother\n\\maketitle\nBody.\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains(
      "Info:frontmatter:annotated The title page annotates 1 word(s) that its output does not hold: bob"
    ),
    "{log}"
  );
  assert_eq!(creators(&xml), owned(&[&["Alice A"]]), "{xml}");
  // (one loss with words of both severities: "zed", appended to `\@title`, amsart's page prints — a Warning, a real loss,
  // RED —; "bob" only the `\@author` tail prints — the Info. int159a: one Warning "zed bob")
  let (log, xml) = run(repro!(
    "amsart_title_tail_lost_beside_an_annotated_author_tail_red.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains(
      "Warning:frontmatter:unclaimed The title page prints 1 word(s) of the document's that its output does not hold: zed"
    ),
    "{log}"
  );
  assert!(
    log.contains(
      "Info:frontmatter:annotated The title page annotates 1 word(s) that its output does not hold: bob"
    ),
    "{log}"
  );
  assert_eq!(creators(&xml), owned(&[&["Alice A"]]), "{xml}");
}
