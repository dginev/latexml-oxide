//! Guards for EOFBIND: four bindings whose delimited reading ran past the end of the file (`Fatal:Mouth:EoF`, row R
//! DIGFATAL-EOF), each now reading as the real package does. One row per binding:
//! - algpseudocodex loads algorithmicx and its own language (algpseudocodex.sty:25, 68-69), not algpseudocode, whose
//!   `\Call` nests `\ifthenelse` inside ifthen's `\equal` (2512.12052);
//! - amsppt's `\subheading` takes an optional `\nofrills` and one undelimited argument (amsppt.sty:563-569,
//!   KNOWN_PERL_ERRORS #1342; 1412.5856, alg-geom9709005);
//! - longtable's `\longtable` has the real prefix and an `\LT@output` that preamble patches split (1409.6572), and a
//!   two-column document's `\LT@err`, a TeX Error, is an Info (the error-boundary row);
//! - jmlr's `\addr` text ends at its group's `}` (jmlr.cls:337; 2409.18592).
//!
//! Each converts its repro once and asserts the exact Error and Warning counts, the creators where the class has a
//! frontmatter, and every element of the kinds the change shows in, whole (the listing's lines by their text: their
//! formulas are the parser's).

use latexml::util::test::{
  assert_all_elements, convert_files_with, error_count, warning_count, xml_elements,
};

use super::perfect_kernel_batch84b::{creators, text};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

struct Row {
  name:     &'static str,
  tex:      &'static str,
  counts:   (usize, usize),
  creators: &'static [&'static [&'static str]],
  /// Every element `tag` the output holds, in order.
  elements: &'static [(&'static str, &'static [&'static str])],
  /// The text of every element `tag`, in order.
  texts:    &'static [(&'static str, &'static [&'static str])],
  /// Lines the log holds.
  log:      &'static [&'static str],
}

const ROWS: &[Row] = &[
  Row {
    name:     "algpseudocodex: a nested \\Call, in a formula, in text and in \\Procedure's arguments",
    tex:      repro!("listings-algorithms/algpseudocodex_nested_call.tex"),
    counts:   (0, 0),
    creators: &[],
    elements: &[("p", &[r#"<p xml:id="p1.2">Text after.</p>"#])],
    texts:    &[("listingline", &[
      "1 \u{2062} f ( g )",
      "2 h ( k (x))",
      "3 procedure p ( q )",
    ])],
    log:      &[],
  },
  Row {
    name:     "amsppt: \\subheading\\nofrills{..}, \\subheading 1 {..}, \\subheading {..}",
    tex:      repro!("amstex/amsppt_subheading_reads_one_argument.tex"),
    counts:   (0, 0),
    creators: &[],
    elements: &[
      ("title", &[
        "<title>Title</title>",
        "<title>Intro</title>",
        "<title>1</title>",
        "<title>Third</title>",
      ]),
      ("p", &[
        r#"<p xml:id="C0.S0.SSx1.p1.1">Body one.</p>"#,
        "<p xml:id=\"C0.S0.SSx2.p1.1\"><text font=\"bold\" xml:id=\"C0.S0.SSx2.p1.1.1\">Second</text>.\nBody two.</p>",
        r#"<p xml:id="C0.S0.SSx3.p1.1">Body three.</p>"#,
      ]),
    ],
    texts:    &[],
    log:      &[],
  },
  Row {
    name:     "longtable: a preamble patch split at \\LT@err and at \\LT@output's \\@outputpage",
    tex:      repro!("package-bindings/longtable_patched_at_its_lt_err.tex"),
    counts:   (0, 0),
    creators: &[],
    elements: &[
      ("p", &[
        r#"<p xml:id="p1.1">Text before.</p>"#,
        r#"<p xml:id="p2.1">Text after.</p>"#,
      ]),
      ("td", &[r#"<td align="left" xml:id="S0.T1.1.1.1">A</td>"#]),
    ],
    texts:    &[],
    log:      &[],
  },
  Row {
    name:     "longtable in a two-column document: longtable.sty's Error is an Info, the table kept",
    tex:      repro!("package-bindings/longtable_in_two_columns_reports_an_info.tex"),
    counts:   (0, 0),
    creators: &[],
    elements: &[
      ("p", &[
        r#"<p xml:id="p1.1">Text before.</p>"#,
        r#"<p xml:id="p2.1">Text after.</p>"#,
      ]),
      ("td", &[r#"<td align="left" xml:id="S0.T1.1.1.1">A</td>"#]),
    ],
    texts:    &[],
    log:      &["Info:latex:(longtable)  Package longtable Info: longtable not in 1-column mode"],
  },
  Row {
    name:     "jmlr: two braced {\\addr ..} affiliations",
    tex:      repro!("class-bindings/jmlr_addr_inside_braces.tex"),
    counts:   (0, 0),
    creators: &[&["Ann Lee", "affiliation=One", "affiliation=Two"]],
    elements: &[("p", &[r#"<p xml:id="p1.1">Body text.</p>"#])],
    texts:    &[],
    log:      &[],
  },
];

#[test]
fn eofbind_rows() {
  for row in ROWS {
    let (log, xml) = convert_files_with(row.tex, &[], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      row.counts,
      "{}: counts\n{log}",
      row.name
    );
    let want: Vec<Vec<String>> = row
      .creators
      .iter()
      .map(|c| c.iter().map(|s| (*s).to_string()).collect())
      .collect();
    assert_eq!(creators(&xml), want, "{}: creators\n{xml}", row.name);
    for (tag, expected) in row.elements {
      assert_all_elements(&xml, tag, &[], expected);
    }
    for line in row.log {
      assert!(log.contains(line), "{}: log line {line:?}\n{log}", row.name);
    }
    for (tag, expected) in row.texts {
      let got: Vec<String> = xml_elements(&xml, tag, &[])
        .iter()
        .map(|e| text(e))
        .collect();
      assert_eq!(got, *expected, "{}: {tag} texts\n{xml}", row.name);
    }
  }
}
