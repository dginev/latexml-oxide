//! Guards for FX24 (RR1's gap 24): what a class or document appends to `\@author` after the author setter stored it —
//! more authors after `\and`, rows of the authors before it — is read at `\maketitle` as the author block's
//! continuation (frontmatter_stores.rs `author_tail_calls`), also where the binding's `\author` stores nothing in
//! `\@author` (authblk's: `seen_author_tail`). Each converts its repro and asserts every creator's name and contacts in
//! order and the exact Error and Warning counts. Perl reads only `\author`'s argument and loses the tail.

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
/// e-mails under the block (no `\and`). Every address is kept on the creators and no word goes uncounted. Red
/// (int158a): the line lost, 1 Warning. Which author holds which address is not pinned: the line is one author-block
/// row of every creator, not each address on its owner (RED `authblk_email_row_appended_to_author_goes_to_every_author_red.tex`).
/// Distinct: the binding route's rows.
#[test]
fn an_authblk_email_line_appended_to_author_is_kept() {
  let (log, xml) = run(
    "\\documentclass{article}\n\\usepackage{authblk}\n\\begin{document}\n\\title{A Title}\n\\author[1]{Alice A}\n\
     \\author[2]{Bob B}\n\\affil[1]{Inst X}\n\\affil[2]{Inst Y}\n\\makeatletter\\g@addto@macro\\@author{\\\\[0.5em]\\small\
     \\texttt{alice@x.org}, \\texttt{bob@y.org}}\\makeatother\n\\maketitle\nBody.\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let found = creators(&xml);
  let names: Vec<&str> = found.iter().map(|c| c[0].as_str()).collect();
  assert_eq!(names, ["Alice A", "Bob B"], "{xml}");
  for creator in &found {
    assert!(
      creator.contains(&"affiliation=Inst X".to_string())
        != creator.contains(&"affiliation=Inst Y".to_string()),
      "{xml}"
    );
  }
  for address in ["alice@x.org", "bob@y.org"] {
    assert!(
      found
        .iter()
        .any(|c| c[1..].iter().any(|contact| contact.contains(address))),
      "{address}: {xml}"
    );
  }
}

/// amsart's `\author` stores its argument in `\@author` and its title page prints `\authors`, not `\@author`: a document's
/// `\g@addto@macro\@author{\and Bob B}` prints nothing (pdflatex: "ALICE A" alone), so no creator is made of it — the
/// binding route reads only a `\@author` the setter left at the kernel's empty value. Distinct: the route's refusal.
#[test]
fn an_amsart_author_append_invents_no_creator() {
  let (log, xml) = run(
    "\\documentclass{amsart}\n\\begin{document}\n\\title{A Title}\n\\author{Alice A}\n\
     \\makeatletter\\g@addto@macro\\@author{\\and Bob B}\\makeatother\n\\maketitle\nBody.\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), owned(&[&["Alice A"]]), "{xml}");
}
