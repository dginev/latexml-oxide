//! Red/green guards for batch REGFIXR2, the review round of REGFIXR: a package's `{gather}` over amsmath's
//! `\start@gather` keeps its own end (text after the display, a group it closes), and a sub-caption label under a
//! shipped caption3 v1.8h takes the format caption3 v1's option lists hold. Each guard converts its repro under
//! `tools/perfect_kernel/repros/` (pdflatex 0 errors) and asserts no Error or Warning and the content in whole elements.
use latexml::util::test::{check_all_requirements, convert_files_with, convert_with, xml_element};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// The counts `(errors, warnings)` of `log`, which must hold no Fatal.
#[track_caller]
fn counts(log: &str) -> (usize, usize) {
  assert!(!log.contains("Fatal:"), "{log}");
  (error_count(log), warning_count(log))
}

/// The `<p>` texts of `xml`, in document order, their spaces collapsed.
fn paragraphs(xml: &str) -> Vec<String> {
  xml
    .split("<p xml:id=")
    .skip(1)
    .filter_map(|rest| {
      rest
        .split_once('>')
        .and_then(|(_, p)| p.split("</p>").next())
    })
    .map(|text| text.split_whitespace().collect::<Vec<_>>().join(" "))
    .collect()
}

/// A package's end of an environment `\start@gather` began runs, its amsmath idiom (amsmath.sty:1552-1557
/// `\math@cr \black@\totwidth@ \egroup \dollardollar@end`, `$$` in linenoaa.sty:51-55's copy: 2407.04074 2503.08108
/// 2503.20535 2604.03365 2608.16665) read as the binding's `\endgather`: what it prints after the display stays (was
/// dropped silently), and the `\begingroup` its begin opened is closed (was "open groups" at `\end{document}`).
/// Distinct from `perfect_kernel_batchregfixr::start_gather_ends_a_package_gather` (an end with nothing after `$$`).
#[test]
fn start_gather_runs_the_package_end() {
  let (log, xml) = convert_with(
    repro!("alignment/start_gather_package_end_runs.tex"),
    Some(PRELOAD),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_eq!(
    xml
      .matches("<equationgroup class=\"ltx_eqn_gather\"")
      .count(),
    3,
    "{xml}"
  );
  assert_eq!(
    paragraphs(&xml),
    ["Before.", "AFTERTEXT next.", "then", "last."],
    "{xml}"
  );
  let numbers: Vec<&str> = xml
    .split("<tag role=\"refnum\">")
    .skip(1)
    .filter_map(|rest| rest.split("</tag>").next())
    .collect();
  assert_eq!(numbers, ["1", "2"], "{xml}");
  for text in [
    "text=\"a = b\"",
    "text=\"c = d\"",
    "text=\"e = f\"",
    "text=\"g = h\"",
  ] {
    assert!(xml.contains(text), "{text}\n{xml}");
  }
}

/// linenoaa.sty:51-55 verbatim (`{gather*}` ends `\endgather\endlinenomath`, its `\endgather` the package's own):
/// the same XML as plain amsmath's `{gather}`/`{gather*}`, but for lineno's processing instruction.
#[test]
fn start_gather_linenoaa_is_plain_amsmath() {
  if !check_all_requirements(&[&["lineno.sty"]]) {
    return;
  }
  let source = repro!("alignment/start_gather_linenoaa.tex");
  let (log, xml) = convert_with(source, Some(PRELOAD));
  assert_eq!(counts(&log), (0, 0), "{log}");
  let body = source
    .lines()
    .find(|line| line.starts_with("Text."))
    .unwrap();
  let plain = format!(
    "\\documentclass{{article}}\n\\usepackage{{amsmath}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n"
  );
  let (plain_log, plain_xml) = convert_with(&plain, Some(PRELOAD));
  assert_eq!(counts(&plain_log), (0, 0), "{plain_log}");
  let xml = xml.replace(
    "<?latexml package=\"lineno\" options=\"mathlines,switch\"?>\n",
    "",
  );
  assert_eq!(xml, plain_xml);
}

/// A shipped caption3 v1.8h under raw floatrow (2010.15148 2011.14129 2102.00283 2112.02137 2201.07802) keeps a
/// subfig option and `\captionsetup[sub<type>]` in its option lists `\caption@opt@<name>` (v1.8h:258-262), which the
/// binding's keys never see: the sub-caption labels are "a)" "b)" (`[labelformat=brace]{subfig}`) and "a" (the
/// `subtable` list over it), as pdflatex, not "(a)"; the lists used, no "Unused \captionsetup" Warning (v1.8h:333-346).
/// Distinct from `perfect_kernel_batchregfixr::subfig_label_under_an_old_caption3` (the default format only).
#[test]
fn subfig_label_from_caption3_v1_options() {
  if !check_all_requirements(&[&["floatrow.sty"]]) {
    return;
  }
  let (log, xml) = convert_files_with(
    repro!("captions-floats/caption3_v18h/subfig_v1_option_labelformat.tex"),
    &[(
      "caption3.sty",
      repro!("captions-floats/caption3_v18h/caption3.sty"),
    )],
    Some(PRELOAD),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  for (panel, tag, text) in [
    ("S0.F1.sf1", "a)", "A"),
    ("S0.F1.sf2", "b)", "B"),
    ("S0.T1.sf1", "a", "D"),
  ] {
    let kind = if panel.starts_with("S0.F") {
      "figure"
    } else {
      "table"
    };
    let found = xml_element(&xml, kind, &[&format!("xml:id=\"{panel}\"")]).unwrap_or_default();
    assert!(
      found.contains(&format!(
        "<caption><tag close=\" \">{tag}</tag>{text}</caption>"
      )),
      "{found}\n{xml}"
    );
  }
}
