//! Guards for SBX1, the 2605/2606 sandbox reruns' regressions (one table, a row a repro): aa.cls v9's linenoaa renews
//! `{multline}` on amsmath's internals (2606.12741 2605.25644 2606.23815 2606.30286); fairmeta's one `\author{}` naming
//! 13 authors (2605.17829); group authors read as "a line under the names" (2605.10052 2606.17184 2605.24212). The
//! tcolorbox overflow rows (DG 2026-11-13) are in `error_boundary::perfect_kernel_error_boundary`.

use super::{
  perfect_kernel_batch46::{convert_with, error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

const REPROS: &str = concat!(
  env!("CARGO_MANIFEST_DIR"),
  "/../tools/perfect_kernel/repros/"
);

/// A row's document: a repro (its path under `tools/perfect_kernel/repros/`), or inline.
enum Source {
  Repro(&'static str),
  Inline(&'static str),
}

/// What a row's XML must hold: the creators (`perfect_kernel_batch84b::creators`: name, then `role=text` per contact),
/// or the XML of the same document without the given lines.
enum Expect {
  Creators(&'static [&'static [&'static str]]),
  SameAsWithout(&'static [&'static str]),
}

const ROWS: &[(Source, (usize, usize), Expect)] = &[
  (
    Source::Repro("ams-internals/amsmath_start_multline_internals.tex"),
    (0, 0),
    Expect::SameAsWithout(&[
      "\\renewenvironment{multline}",
      "  \\iftagsleft@",
      "  \\endlinenomath",
      "\\renewenvironment{multline*}",
    ]),
  ),
  (
    Source::Repro("sectioning-frontmatter/fairmeta_one_author_call_names_several.tex"),
    (0, 0),
    Expect::Creators(&[
      &["Ann Able", "affiliation=MIT"],
      &["Bob Baker", "affiliation=Stanford University"],
      &["Carl Cole", "affiliation=MIT"],
    ]),
  ),
  // the list's authors are the new ones an unmarked `\affiliation` after it reaches, each of them (the queue's reading
  // reads `annotate=new` written out, no conditional; pdflatex 0 `!` with a stand-in fairmeta.cls)
  (
    Source::Inline(
      "\\documentclass{fairmeta}\n\\title{T}\n\\author{Ann Able, Bob Baker}\n\\affiliation{MIT}\n\
       \\begin{document}\n\\maketitle\nText.\n\\end{document}\n",
    ),
    (0, 0),
    Expect::Creators(&[&["Ann Able", "affiliation=MIT"], &[
      "Bob Baker",
      "affiliation=MIT",
    ]]),
  ),
  (
    Source::Repro("sectioning-frontmatter/group_author_in_lower_case_is_a_creator.tex"),
    (0, 0),
    Expect::Creators(&[&[
      "openJiuwen Team",
      "affiliation=Gaoling School of Artificial Intelligence",
      "affiliation=Renmin University of China",
      "email=research@openjiuwen.com",
    ]]),
  ),
  (
    Source::Repro("sectioning-frontmatter/speaks_for_author_is_a_creator.tex"),
    (0, 0),
    Expect::Creators(&[
      &["Siqi Li", "affiliation=Duke-NUS Medical School, Singapore"],
      &["Chuan Hong", "affiliation=Duke University, Durham, NC, USA"],
      &[
        "for the PAROS Clinical Research Network",
        "affiliation=Duke-NUS Medical School, Singapore",
      ],
    ]),
  ),
  // 2606.17184's shape: a collaboration named as the author in lower case, its footnote a note
  (
    Source::Inline(
      "\\documentclass{article}\n\\title{T}\n\\author{sPHENIX Collaboration\\footnote{See the appendix for the \
       list of collaboration members}}\n\\begin{document}\n\\maketitle\nText.\n\\end{document}\n",
    ),
    (0, 0),
    Expect::Creators(&[&[
      "sPHENIX Collaboration",
      "note=See the appendix for the list of collaboration members",
    ]]),
  ),
  // the Error side: a legend's text named as the author, its team word no group's name (pdflatex 0 `!`, the output the same)
  (
    Source::Inline(
      "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{now at the Theory Group}\n\\maketitle\n\
       Body.\n\\end{document}\n",
    ),
    (1, 0),
    Expect::Creators(&[&["now at the Theory Group"]]),
  ),
];

#[test]
fn sandbox_2605_rows() {
  for (source, (errors, warnings), expect) in ROWS {
    let (name, tex) = match source {
      Source::Repro(path) => (
        *path,
        std::fs::read_to_string(format!("{REPROS}{path}")).expect("repro"),
      ),
      Source::Inline(tex) => ("inline", tex.to_string()),
    };
    let (log, xml) = convert_with(&tex, Some("ar5iv.sty"));
    assert_eq!(error_count(&log), *errors, "{name}:\n{log}");
    assert_eq!(warning_count(&log), *warnings, "{name}:\n{log}");
    match expect {
      Expect::Creators(want) => {
        let want: Vec<Vec<String>> = want
          .iter()
          .map(|c| c.iter().map(ToString::to_string).collect())
          .collect();
        assert_eq!(creators(&xml), want, "{name}:\n{xml}");
      },
      Expect::SameAsWithout(lines) => {
        let control: String = tex
          .lines()
          .filter(|l| !lines.iter().any(|p| l.starts_with(p)))
          .map(|l| format!("{l}\n"))
          .collect();
        assert_ne!(control, tex, "{name}: no line removed");
        let (control_log, control_xml) = convert_with(&control, Some("ar5iv.sty"));
        assert_eq!(
          error_count(&control_log),
          0,
          "{name} control:\n{control_log}"
        );
        assert_eq!(
          xml, control_xml,
          "{name}: differs from the document without the renewals"
        );
      },
    }
  }
}
