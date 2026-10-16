//! Red/green guards for batch 68f: three run-359 regressions — a dropped frontmatter environment's store read past an
//! `\expandafter` chain (2104.09383), the dependency scan honours a document's `\ver@<file>` marker (2201.11056), and
//! siunitx re-declares its column letter at begin-document (2410.09116): `S` under v3, `s` and `S` under the `[=v2]`
//! rollback. Each repro and inline fixture is checked against pdflatex.
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// rmmcart.cls:418's dropped `{abstract}` renewal `\expandafter\gdef\expandafter\abstracts@ve\expandafter{…}` records
/// `\abstracts@ve` for the `\maketitle` deposit, not `\expandafter` (which the deposit used to `\let` to `\@empty`,
/// leaking past its group: `\csname` construction then failed, 2104.09383 looped in a tikz-cd). Repro
/// sectioning-frontmatter/dropped_abstract_store_expandafter_gdef.
#[test]
fn dropped_store_skips_the_expandafter_chain_to_the_defined_name() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/dropped_abstract_store_expandafter_gdef.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Result: OK; [“expandafter]</p>"#,
  );
}

/// The store the `\expandafter` chain defines is the one the deposit reads given and empty: the class's
/// `\@maketitle` reading `\abs@ve` finds it defined (empty), and a `\csname`-built name is not recorded (nothing to
/// read, nothing nulled). Same mechanism as the repro above, red on 67int4 (`\abs@ve` undefined, the deposit dropped).
#[test]
fn dropped_store_behind_an_expandafter_chain_reads_given_and_empty() {
  let (log, xml) = convert_with(
    r"\begin{filecontents*}[overwrite]{rcy.cls}
\ProvidesClass{rcy}
\LoadClass{article}
\renewenvironment{abstract}{\expandafter\gdef\expandafter\abs@ve\expandafter{x}\expandafter\gdef\csname abs@cs\endcsname{y}}{}
\def\@maketitle{[\abs@ve]}
\end{filecontents*}
\documentclass{rcy}
\title{T}
\begin{document}
\maketitle
After: \expandafter\ifx\csname abs@cs\endcsname\relax undefined\else defined\fi.
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // The deposit keeps the class's `[…]` (it typesets something besides the frontmatter fields); on 67int4 the
  // undefined `\abs@ve` made it raise an error, so it was dropped.
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">[]After: undefined.</p>"#,
  );
}

/// sn-jnl-setup.sty's `\AddToHook{class/sn-jnl/before}` defines `\ver@program.sty`, so LaTeX skips sn-jnl.cls:311's
/// `\usepackage{program}` (latex.ltx:18750 `\@onefilewithoptions` → `\@ifl@aded`); the sn-jnl binding's dependency
/// scan, standing in for that line, skips it too, with an Info naming it — program.sty's active `|` otherwise broke
/// pgf (2201.11056 ran out of memory). Repro loader/scan_skips_document_marked_loaded_package.
#[test]
fn scan_skips_a_package_the_document_marked_loaded() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/loader/scan_skips_document_marked_loaded_package.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains(
      r"Skipping dependency program.sty: it is marked loaded (\ver@program.sty is defined)"
    ),
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Bar: <text font="typewriter" xml:id="p1.1.1">the character |</text>.</p>"#,
  );
}

/// siunitx owns `S` at begin-document (siunitx.sty:9348-9373), over cellspace's later `\newcolumntype{S}`
/// (cellspace.sty:174-177), with siunitx's one overwrite warning: each row's second cell is the siunitx number
/// (2410.09116's `{llSS}` tables). Repro package-bindings/siunitx_column_after_cellspace.
#[test]
fn siunitx_column_wins_over_a_later_cellspace_column() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/siunitx_column_after_cellspace.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains(r#"Tabular column type "S" overwritten with siunitx definition"#),
    "{log}"
  );
  assert_element(
    &xml,
    "td",
    &[r#"xml:id="p1.1.1.2""#],
    r#"<td align="left" xml:id="p1.1.1.2"><Math mode="inline" tex="62.0" text="62.0" xml:id="p1.m1">
         <XMath xml:id="p1.m1.1"><XMTok meaning="62.0" role="NUMBER">62.0</XMTok></XMath></Math></td>"#,
  );
  assert_element(
    &xml,
    "td",
    &[r#"xml:id="p1.1.2.2""#],
    r#"<td align="left" xml:id="p1.1.2.2"><Math mode="inline" tex="1.5" text="1.5" xml:id="p1.m2">
         <XMath xml:id="p1.m2.1"><XMTok meaning="1.5" role="NUMBER">1.5</XMTok></XMath></Math></td>"#,
  );
}

/// No column package after siunitx: no overwrite warning (the begin-document re-declaration finds its own `S`). `{lS}`:
/// v3 has no `s` column (pdflatex `Illegal pream-token (s)`).
#[test]
fn siunitx_column_alone_raises_no_overwrite_warning() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage{siunitx}
\begin{document}
\begin{tabular}{lS}
A & 62.0 \\
\end{tabular}
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "td",
    &[r#"xml:id="p1.1.1.2""#],
    r#"<td align="left" xml:id="p1.1.1.2"><Math mode="inline" tex="62.0" text="62.0" xml:id="p1.m1">
         <XMath xml:id="p1.m1.1"><XMTok meaning="62.0" role="NUMBER">62.0</XMTok></XMath></Math></td>"#,
  );
}

/// v3 declares only `S` at begin-document (siunitx.sty:9364-9373, `table-column-type = S`, :9101), so a document's own
/// `\newcolumntype{s}` after siunitx (tabularx's `>{\hsize=.5\hsize}X` idiom) stays the document's, silently: the `s`
/// cell is the paragraph "alpha beta", as pdflatex sets it (68f r1 re-declared `s` too, warned, and set the cell as
/// the math token "alphabeta"). Repro package-bindings/siunitx_v3_document_s_column.
#[test]
fn siunitx_v3_leaves_a_document_s_column_alone() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/siunitx_v3_document_s_column.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "td",
    &[r#"xml:id="p1.1.1.1""#],
    r#"<td align="left" xml:id="p1.1.1.1"><inline-block vattach="top" xml:id="p1.1.1.1.1">
         <p xml:id="p1.1.1.1.1.1">alpha beta</p></inline-block></td>"#,
  );
  assert_element(
    &xml,
    "td",
    &[r#"xml:id="p1.2.1.1""#],
    r#"<td align="left" xml:id="p1.2.1.1"><Math mode="inline" tex="1.5" text="1.5" xml:id="p1.m1">
         <XMath xml:id="p1.m1.1"><XMTok meaning="1.5" role="NUMBER">1.5</XMTok></XMath></Math></td>"#,
  );
}

/// v3 has no `s` column, so an `s` the document declared BEFORE siunitx loaded stays the document's: pdflatex sets
/// `\newcolumntype{s}{l}` + `\usepackage{siunitx}` + `{sS} kg & 1.5` as the text "kg" and the number 1.5, no warning;
/// the binding's Perl-era load-time `s` replaced it with a unit parse (`\mathrm{k}\mathrm{g}`; 68f r2 re-review).
#[test]
fn siunitx_v3_keeps_an_s_column_declared_before_it() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage{array}
\newcolumntype{s}{l}
\usepackage{siunitx}
\begin{document}
\begin{tabular}{sS}
kg & 1.5 \\
\end{tabular}
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "td",
    &[r#"xml:id="p1.1.1.1""#],
    r#"<td align="left" xml:id="p1.1.1.1">kg</td>"#,
  );
}

/// v3's `\__siunitx_declare_column:Nnn` (siunitx.sty:9322-9330) warns whenever another `S` exists at begin-document,
/// so one declared BEFORE siunitx loaded is overwritten with the warning too (pdflatex: `Package siunitx Warning:
/// Tabular column type "S" overwritten with siunitx definition.`); our load-time `DefColumnType` replaced it silently.
#[test]
fn siunitx_v3_warns_over_a_column_declared_before_it() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage{array}
\newcolumntype{S}{c}
\usepackage{siunitx}
\begin{document}
\begin{tabular}{lS}
A & 62.0 \\
\end{tabular}
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains(r#"Tabular column type "S" overwritten with siunitx definition."#),
    "{log}"
  );
  assert_element(
    &xml,
    "td",
    &[r#"xml:id="p1.1.1.2""#],
    r#"<td align="left" xml:id="p1.1.1.2"><Math mode="inline" tex="62.0" text="62.0" xml:id="p1.m1">
         <XMath xml:id="p1.m1.1"><XMTok meaning="62.0" role="NUMBER">62.0</XMTok></XMath></Math></td>"#,
  );
}

/// The `[=v2]` rollback (siunitx.sty:37-39) re-creates `s` and `S` at begin-document (siunitx-v2.sty:5624-5625
/// `\__siunitx_table_rewrite_create:N`), over the document's own `s`, with no siunitx warning: the `s` cell is the
/// siunitx unit, as pdflatex sets it (68f r1 warned for `s`). Repro package-bindings/siunitx_v2_rollback_columns.
#[test]
fn siunitx_v2_rollback_reinstates_both_columns_silently() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/siunitx_v2_rollback_columns.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "td",
    &[r#"xml:id="p1.1.1.2""#],
    r#"<td align="left" xml:id="p1.1.1.2"><Math mode="inline" tex="62.0" text="62.0" xml:id="p1.m1">
         <XMath xml:id="p1.m1.1"><XMTok meaning="62.0" role="NUMBER">62.0</XMTok></XMath></Math></td>"#,
  );
  assert_element(
    &xml,
    "td",
    &[r#"xml:id="p1.1.1.3""#],
    r#"<td align="left" xml:id="p1.1.1.3"><Math mode="inline" tex="\mathrm{m}" text="m" xml:id="p1.m2">
         <XMath xml:id="p1.m2.1"><XMTok role="UNKNOWN">m</XMTok></XMath></Math></td>"#,
  );
}
