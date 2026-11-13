//! The kernel's peeking commands (`\@ifnextchar`, `\@ifstar`, `\@testopt`,
//! and a `\newcommand` optional argument) peek by `\futurelet` behind an
//! unexpandable `\let` in latex.ltx (1756-1775, 1259-1261, 1249), so a
//! number scan (tex.web §445) ends at them and the peek happens where they
//! are executed (`gullet::scan_stops_at_futurelet_peek`). Perl's closures
//! peeked during the scan, and the branch they chose was read into the
//! number, or its conditionals into a false branch's skip (egpeirce-doc: 51
//! errors, the document lost). Digestion a binding runs in mid-scan is no part
//! of the scan (`gullet::NumberScan::suspend`). Repros in
//! `tools/perfect_kernel/repros/expansion-primitives/`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert, error_count, warning_count};

/// `\ifodd2\x x\else y\fi` with `\x` = `\@ifnextchar*{\footrue}{\foofalse}`:
/// the scan ends at `\@ifnextchar`, the false branch skips it to the `\else`
/// (pdflatex "yz", 0 errors; Perl 2 errors, Rust ran off the end and lost the
/// document).
#[test]
fn a_number_scan_ends_at_ifnextchar() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/ifnextchar_number_scan.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para xml:id=\"p1\"><p>yz</p></para>",
  );
}

/// `\c=1\s*` with `\s` = `\@ifstar{7}{5}` assigns 1 and typesets 7; the same
/// for `\@testopt` with and without its `[`. The closures had read their
/// branch into the number (`\c=17`, `\c=23`, and `\c=4323` swallowing the
/// `\the\c` that follows).
#[test]
fn count_assignments_end_at_ifstar_and_testopt() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/ifstar_count_assignment.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para xml:id=\"p1\"><p>7 1.</p></para>",
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    "<para xml:id=\"p2\"><p>3 2.</p></para>",
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    "<para xml:id=\"p3\"><p>34.</p></para>",
  );
}

/// Executed by the main loop, the peeking commands still skip spaces, pick
/// their branch and leave the peeked token; `\section*` (raw article.cls:
/// `\@startsection` → `\@ifstar`) and a tabular's `\\[2pt]` read their
/// options; a robust command and an optional-argument `\newcommand` survive
/// `\protected@edef`. Same output with and without the raw class.
#[test]
fn peeking_commands_still_peek_where_executed() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/ifnextchar_controls.tex"
  );
  for raw in [true, false] {
    let (stderr, xml) = convert(tex, raw);
    assert_eq!(error_count(&stderr), 0, "raw={raw}\n{stderr}");
    assert_eq!(warning_count(&stderr), 0, "raw={raw}\n{stderr}");
    assert_element(
      &xml,
      "section",
      &[r#"xml:id="Sx1""#],
      "<section xml:id=\"Sx1\"><title>Starred</title></section>",
    );
    assert_element(
      &xml,
      "para",
      &[r#"xml:id="S1.p1""#],
      "<para xml:id=\"S1.p1\"><p>(a) (d)b S Nz (a)b</p></para>",
    );
    assert_element(
      &xml,
      "para",
      &[r#"xml:id="S1.p2""#],
      "<para xml:id=\"S1.p2\"><p>+e+ [e]</p></para>",
    );
    assert_element(
      &xml,
      "tabular",
      &[],
      r#"<tabular vattach="middle"><tbody>
        <tr><td align="left" cssstyle="padding-bottom: 2.0pt">a</td>
          <td align="left" cssstyle="padding-bottom: 2.0pt">b</td></tr>
        <tr><td align="left">c</td><td align="left">d</td></tr>
        <tr><td align="left">e</td><td align="left">f</td></tr>
      </tbody></tabular>"#,
    );
  }
}

/// Left behind by the scan, the peeking macros keep their branch intact: a
/// `##` branch defines with one `#` (pdflatex "*[a] 1."), `\@testopt`'s
/// default goes in braced (latex.ltx:1260 `#1[{#2}]`; "<a]b>", was "<a>b]"),
/// and LaTeXML's `\@ifnext@n` ends a scan too ("7xy5.", was "xy57.").
#[test]
fn scan_leaves_the_peeking_branch_intact() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/ifnextchar_scan_branches.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para xml:id=\"p1\"><p>*[a] 1.</p></para>",
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    "<para xml:id=\"p2\"><p>¡a]b¿</p></para>",
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    "<para xml:id=\"p3\"><p>7xy5.</p></para>",
  );
}

/// `\count@=\@ifstar{1}{2}*`: the scan finds no number at all; the
/// "Missing number" diagnostic names `\@ifstar` itself, and the starred
/// branch is typeset (pdflatex "10.", its error our Warning).
#[test]
fn a_scan_meeting_ifstar_first_reports_a_missing_number() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/ifstar_missing_number.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains("Warning:expected:<number> Missing number, treated as zero"),
    "{stderr}"
  );
  assert!(stderr.contains(r"Next token is T_CS[\@ifstar]"), "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para xml:id=\"p1\"><p>10.</p></para>",
  );
}

/// `\newcommand\foo[1][7]` + `\cc=1\foo \the\cc.`: `\@protected@testopt`'s
/// peek ends the scan at `\foo` (pdflatex "71."; was ".", `\cc`=17).
#[test]
fn a_number_scan_ends_at_a_newcommand_optional_argument() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/newcommand_optional_number_scan.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para xml:id=\"p1\"><p>71.</p></para>",
  );
}

/// `\ifnum\x=1\cmidrule{1-2}\fi` at a row head: the scan ends at the rule's
/// `\@ifnextchar(`, the row-head peek expands it on to its `\noalign`, and
/// the rule is the next row's top border (pdflatex clean).
#[test]
fn a_rule_behind_ifnum_at_a_row_head_keeps_its_noalign() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/cmidrule_behind_ifnum.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular vattach="middle"><tbody>
      <tr><td align="left" border="tt">a</td><td align="left" border="tt">b</td></tr>
      <tr><td align="left" border="bb t">c</td><td align="left" border="bb t">d</td></tr>
    </tbody></tabular>"#,
  );
}

/// calc's `\widthof` digests a tabular while `\makebox[…]`'s width is being
/// scanned: the tabular's tabs act and its `\cmidrule` peeks, so the width
/// is the table's (pdflatex 34.56pt; was 22.6pt with the tabs inert).
#[test]
fn a_box_digested_in_mid_scan_is_outside_the_scan() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/widthof_tabular_in_makebox_width.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p><text align="center" width="34.6pt">X</text></p></para>"#,
  );
}

/// A cell's `\count@=4\@ifnextchar\bgroup{A}{B}{x}`: the peeked `{` goes back
/// with its brace count retracted, so the cell and the rows stay intact
/// (pdflatex "Ax4").
#[test]
fn a_peeked_brace_in_a_tabular_cell_keeps_the_row() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/ifnextchar_brace_in_tabular_cell.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular vattach="middle"><tbody>
      <tr><td align="left">a</td><td align="left">b</td></tr>
      <tr><td align="left">g</td><td align="left">Ax4</td></tr>
      <tr><td align="left">h</td><td align="left">i</td></tr>
    </tbody></tabular>"#,
  );
}

/// Every `\newcommand`-family optional argument ends a number scan
/// (`Parameter::testopt`: `\DeclareRobustCommand`, `\providecommand`,
/// `\renewcommand`, `\newrobustcmd`, twoopt's `convert_twoopt_args`); one
/// without an optional argument expands in the scan; a braced `\@testopt`
/// default keeps its inner group (pdflatex "71." ×4, "781.", ".", "<a>" in OT1).
#[test]
fn every_newcommand_family_optional_ends_a_number_scan() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/newcommand_family_optional_number_scan.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // OT1 prints `<` and `>` as `¡` and `¿`, in pdflatex too.
  let expected = ["71.", "71.", "71.", "71.", "781.", ".", "\u{A1}a\u{BF}"];
  for (i, text) in expected.iter().enumerate() {
    let n = i + 1;
    assert_element(
      &xml,
      "para",
      &[&format!(r#"xml:id="p{n}""#)],
      &format!(r#"<para xml:id="p{n}"><p>{text}</p></para>"#),
    );
  }
}

/// `\expandafter` in a number scan leaves a peeking macro unexpanded, as
/// TeX's one level reaches its unexpandable `\let` head (pdflatex "71.",
/// "31.", "32."; before, the branch was read into the number).
#[test]
fn expandafter_in_a_scan_stops_at_a_peek() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/expandafter_peek_in_number_scan.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (i, text) in ["71.", "31.", "32."].iter().enumerate() {
    let n = i + 1;
    assert_element(
      &xml,
      "para",
      &[&format!(r#"xml:id="p{n}""#)],
      &format!(r#"<para xml:id="p{n}"><p>{text}</p></para>"#),
    );
  }
}
