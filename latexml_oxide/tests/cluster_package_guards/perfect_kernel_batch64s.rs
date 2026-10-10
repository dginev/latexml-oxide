//! Red/green guards for batch 64s: ar5iv's archival `\today` is the paper's own date — the newest modification time
//! among the TeX source files of its bundle (`latexml::source_date`), set as the job's clock — and stays Perl's empty
//! `\relax` only when the source carries no date (user ruling 2026-10-10). Only the bundle's date counts under ar5iv, so
//! these guards hold whatever `SOURCE_DATE_EPOCH` the environment exports: a dated source overwrites the clock that
//! variable seeded (before the format copies it into `\c_sys_*_int`), and an undated one keeps the `\relax` `\today`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// 2018-01-18T11:05:00Z: the date of the guards' source bundle.
const SOURCE_DATE: i64 = 1_516_273_500;

/// The XML of `tex`, from a source dated [`SOURCE_DATE`], converted under ar5iv: no error and no warning.
fn convert_dated(tex: &str) -> String {
  let (log, xml) =
    latexml::util::test::convert_dated_files_with(tex, &[], Some("ar5iv.sty"), SOURCE_DATE);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// ptapap.cls:322-325 (shipped with 1801.05985 and the class's other papers) splits the `\edef`'d `\today` with a
/// four-argument `\relax`-delimited macro to lowercase the month's first three letters. On ar5iv's `\relax` `\today`
/// the macro ran past its delimiter: "Paragraph ended before \next was complete".
#[test]
fn ptapap_splits_the_source_date() {
  let xml = convert_dated(
    r"\documentclass{article}
\edef\ptaJournalDate{\today}
\def\next#1#2#3#4\relax{\lowercase{\edef\next{#1#2#3}}}%
\expandafter\next\ptaJournalDate\relax
\begin{document}
\next
\end{document}
",
  );
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p xml:id="p1.1">jan</p></para>"#,
  );
}

/// `\date{\today}` in a title block shows the paper's date, as arXiv's PDF does (1811.05851: `\item[] \today` gives
/// "November 14, 2018").
#[test]
fn title_block_today_is_the_source_date() {
  let xml = convert_dated(
    r"\documentclass{article}
\title{On Dates}
\author{A. Author}
\date{\today}
\begin{document}
\maketitle
Body.
\end{document}
",
  );
  assert_element(
    &xml,
    "date",
    &[],
    r#"<date role="creation">January 18, 2018</date>"#,
  );
}

/// `\pdfcreationdate`, `\time` and the l3kernel's `\c_sys_*_int` (copied from the registers when the format loads)
/// keep the same clock as `\today`.
#[test]
fn pdf_creation_date_keeps_the_source_clock() {
  let xml = convert_dated(
    r"\documentclass{article}
\begin{document}
[\pdfcreationdate][\the\time][\the\year/\the\month/\the\day]
\ExplSyntaxOn [\int_use:N \c_sys_year_int/\int_use:N \c_sys_month_int/\int_use:N \c_sys_day_int] \ExplSyntaxOff
\end{document}
",
  );
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p xml:id="p1.1">[D:20180118110500Z][665][2018/1/18] [2018/1/18]</p></para>"#,
  );
}

/// A source with no date (a literal or stdin document, an archive of undated entries) keeps Perl's empty `\today`
/// (ar5iv.sty.ltxml:23-25): the conversion's own day is never stamped.
#[test]
fn undated_source_keeps_the_empty_today() {
  let (log, xml) = latexml::util::test::convert_with(
    r"\documentclass{article}
\begin{document}
[\today]
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p xml:id="p1.1">[]</p></para>"#,
  );
}
