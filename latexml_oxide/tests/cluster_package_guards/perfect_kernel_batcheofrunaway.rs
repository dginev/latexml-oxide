//! Red/green guards for batch EOFRUNAWAY: at the TRUE end of all input a macro's first miss of a required argument
//! follows Perl's put-back (EOFPUTBACK, OD #304: "Missing argument", the call runs on the empty argument); a later
//! call missing the same argument there is a scanner re-requesting it from the tokens put back, which Perl runs to its
//! error cap. That call is TeX's runaway, "File ended while scanning use of", and is abandoned (tex.web §338-339,
//! §392); the put-back is read on (OD #304). Witnesses 2511.11682 (ifthen `\@tempc`), 2510.17429 (`\@iforloop`):
//! 502 and 525 errors to the cap of 500. Repros under `tools/perfect_kernel/repros/expansion-primitives/`.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The diagnostics of `log`, in order.
fn diagnostics(log: &str) -> Vec<&str> {
  log
    .lines()
    .filter(|l| l.starts_with("Error:") || l.starts_with("Fatal:") || l.starts_with("Warning:"))
    .collect()
}

/// A self-recursive scanner re-requesting its delimited argument at the true end: the ifthen `\@tempc#1\NOT#2` shape
/// that hands itself the same tokens again (2511.11682; a later call in the put-back abandoned too, an Info),
/// `\@iforloop` whose `,` misses while its `\@@` is found in the
/// put-back and whose body typesets `[]` each round (2510.17429), and a scanner whose put-back grows each round (the
/// shape of two pgf papers, which ended in `Fatal:Timeout:PushbackLimit` keeping nothing). Each: one "Missing
/// argument", one "File ended while scanning use of", no Fatal, the put-back typeset; Perl 101 errors and
/// too_many_errors for the first two, out of memory for the third; pdflatex "File ended while scanning use of" and the
/// emergency stop. Distinct: the re-request path of `gullet::note_missing_argument`.
#[test]
fn true_end_rerequest_is_abandoned() {
  let cases: [(&str, &str, &[&str], &[&str]); 3] = [
    (
      "until_true_end_rerequest_abandons",
      include_str!(
        "../../../tools/perfect_kernel/repros/expansion-primitives/until_true_end_rerequest_abandons.tex"
      ),
      &[
        r"Error:expected:Until: Missing argument Until:\NOT at end of input",
        r"Error:expected:\par File ended while scanning use of \scan",
      ],
      &[r#"<p xml:id="p1.1">before x y z</p>"#],
    ),
    (
      "until_true_end_for_loop_abandons",
      include_str!(
        "../../../tools/perfect_kernel/repros/expansion-primitives/until_true_end_for_loop_abandons.tex"
      ),
      &[
        "Error:expected:Until: Missing argument Until:, at end of input",
        r"Error:expected:\par File ended while scanning use of \@iforloop",
        r"Error:undefined:\@@ The token T_CS[\@@] is not defined.",
      ],
      &[r#"<p xml:id="p1.1">before []a<ERROR class="undefined" xml:id="p1.1.1">\@@</ERROR>[]</p>"#],
    ),
    (
      "until_true_end_pushback_growth",
      include_str!(
        "../../../tools/perfect_kernel/repros/expansion-primitives/until_true_end_pushback_growth.tex"
      ),
      &[
        r"Error:expected:Until: Missing argument Until:\stop at end of input",
        r"Error:expected:\par File ended while scanning use of \sc",
        "Error:expected:{ Expected opening '{'",
      ],
      &[
        r#"<p xml:id="p1.1">before</p>"#,
        r#"<p xml:id="p2.1">xx</p>"#,
      ],
    ),
  ];
  for (name, tex, expected, paragraphs) in cases {
    let (log, xml) = convert_files_with(tex, &[], Some("ar5iv.sty"));
    assert_eq!(diagnostics(&log), expected, "{name}: {log}");
    assert_eq!(error_count(&log), expected.len(), "{name}: {log}");
    assert_eq!(warning_count(&log), 0, "{name}: {log}");
    for p in paragraphs {
      assert!(xml.contains(p), "{name}: {p}\n{xml}");
    }
    let again = usize::from(name == "until_true_end_rerequest_abandons");
    assert_eq!(log.matches(" again\n").count(), again, "{name}: {log}");
  }
}

/// Two calls of one delimited macro that both run off the true end, input read between them: the second is a new
/// call (less input left than at the first miss), so both follow Perl's put-back and neither is abandoned. Perl the
/// same: 2 errors, "before []one []two". Distinct: the progress test that keeps a caption typeset after its measuring
/// (1301.5371: abandoning it lost 2193 words; 1404.5770).
#[test]
fn true_end_new_call_runs_as_perl() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/until_true_end_second_call_runs.tex"
    ),
    &[],
    Some("ar5iv.sty"),
  );
  let miss = r"Error:expected:Until: Missing argument Until:\stop at end of input";
  assert_eq!(diagnostics(&log), [miss, miss], "{log}");
  assert_eq!(error_count(&log), 2, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(
    xml.contains(r#"<p xml:id="p1.1">before []one []two</p>"#),
    "{xml}"
  );
}
