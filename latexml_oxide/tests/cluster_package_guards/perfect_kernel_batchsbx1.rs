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
