//! Red/green guards for batch PSF1: a `\psfrag` fragment an EPS graphic would place (latex + dvips) is not rendered
//! (PSFR, post-merge), so every EPS graphic in a fragment's scope (`\includegraphics`, `\epsfbox`, `\epsffile`,
//! `\psfig`) is one Warning naming the graphic and every tag (and `\psfragscanon`'s embedded `\tex` string) it drops
//! (PSF1b); a fragment no EPS graphic reaches prints nothing in TeX either. One table; each row's oracle (latex + dvips
//! or pdflatex `!` count, the printed text) is in its repro header under `tools/perfect_kernel/repros/reversion/`.
//! Red (int166a): every row 0 warnings.
use latexml::util::test::{convert_files_with, xml_element};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// One repro: its source, its exact (errors, warnings), the Warning messages in order, and every `<graphics>`
/// element whole (or, where its candidates name a host path, the attribute it holds).
struct Row {
  name:     &'static str,
  tex:      &'static str,
  counts:   (usize, usize),
  warnings: &'static [&'static str],
  graphics: &'static [&'static str],
}

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/reversion/",
      $name,
      ".tex"
    ))
  };
}

const MISSING: &str = "Warning:missing_feature:psfrag ";

const ROWS: &[Row] = &[
  // The RED (QA41): a fragment in a `figure` consumed by the graphic in it; rendering stays RED (PSFR).
  Row {
    name:     "psfrag_fragment_dropped_silently_red",
    tex:      repro!("psfrag_fragment_dropped_silently_red"),
    counts:   (0, 1),
    warnings: &["The psfrag replacement of 'A' in the graphic 'psfragfig.eps' is not rendered"],
    graphics: &[r#"<graphics graphic="psfragfig.eps" xml:id="S0.F1.g1"/>"#],
  },
  // Preamble fragments and a `\psfrag*` placement reach both graphics, the extensionless one too: one Warning each,
  // naming all three placements in order.
  Row {
    name:     "psfrag_star_and_preamble_fragments_per_graphic",
    tex:      repro!("psfrag_star_and_preamble_fragments_per_graphic"),
    counts:   (0, 2),
    warnings: &[
      "The psfrag replacements of 'A', 'A', 'B' in the graphic 'psfragstar.eps' are not rendered",
      "The psfrag replacements of 'A', 'A', 'B' in the graphic 'psfragstar' are not rendered",
    ],
    graphics: &[
      r#"<graphics graphic="psfragstar.eps" xml:id="p1.g1"/>"#,
      r#"<graphics graphic="psfragstar" options="width=56.9055pt,keepaspectratio=true" xml:id="p2.g1"/>"#,
    ],
  },
  // Scope: the figure's fragment ends with its group; "B" (a group, no graphic) and "C" (none after it) lose nothing.
  Row {
    name:     "psfrag_fragment_scope_and_unconsumed",
    tex:      repro!("psfrag_fragment_scope_and_unconsumed"),
    counts:   (0, 1),
    warnings: &["The psfrag replacement of 'A' in the graphic 'psfragscope.eps' is not rendered"],
    graphics: &[
      r#"<graphics graphic="psfragscope.eps" xml:id="S0.F1.g1"/>"#,
      r#"<graphics graphic="psfragscope.eps" xml:id="p1.g1"/>"#,
    ],
  },
  // The other including commands: epsfig's `\epsfbox`, its `\let` copy `\epsffile`, and `\psfig`.
  Row {
    name:     "psfrag_fragment_epsfbox_psfig",
    tex:      repro!("psfrag_fragment_epsfbox_psfig"),
    counts:   (0, 3),
    warnings: &[
      "The psfrag replacement of 'A' in the graphic 'psfragbox.eps' is not rendered",
      "The psfrag replacement of 'A' in the graphic 'psfragbox.eps' is not rendered",
      "The psfrag replacement of 'A' in the graphic 'psfragbox.eps' is not rendered",
    ],
    graphics: &[
      r#"<graphics graphic="psfragbox.eps" xml:id="p1.g1"/>"#,
      r#"<graphics graphic="psfragbox.eps" xml:id="p2.g1"/>"#,
      r#"<graphics graphic="psfragbox.eps" xml:id="p3.g1"/>"#,
    ],
  },
  // `\psfragscanon`: the EPS's embedded `(\\tex` fragment, named as TeX reads it, at each graphic until
  // `\psfragscanoff`.
  Row {
    name:     "psfrag_scan_embedded_tex_fragment",
    tex:      repro!("psfrag_scan_embedded_tex_fragment"),
    counts:   (0, 2),
    warnings: &[
      r"The psfrag replacement of the embedded (\psfragscanon) '\tex[bl][bl]{$\alpha$}' in the graphic 'psfragscan.eps' is not rendered",
      r"The psfrag replacement of the embedded (\psfragscanon) '\tex[bl][bl]{$\alpha$}' in the graphic 'psfragscan.eps' is not rendered",
    ],
    graphics: &[
      r#"<graphics graphic="psfragscan.eps" xml:id="p1.g1"/>"#,
      r#"<graphics graphic="psfragscan.eps" xml:id="p2.g1"/>"#,
      r#"<graphics graphic="psfragscan.eps" xml:id="p3.g1"/>"#,
    ],
  },
  // Safe side: a PNG graphic (pdflatex) is no EPS, so nothing is placed or lost.
  Row {
    name:     "psfrag_fragment_non_eps_graphic",
    tex:      repro!("psfrag_fragment_non_eps_graphic"),
    counts:   (0, 0),
    warnings: &[],
    graphics: &[r#"graphic="example-image.png""#],
  },
];

#[test]
fn batchpsf1_dropped_fragments_are_counted() {
  for row in ROWS {
    let (log, xml) = convert_files_with(row.tex, &[], Some("ar5iv.sty"));
    let name = row.name;
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      row.counts,
      "{name}: {log}"
    );
    let warnings: Vec<&str> = log
      .lines()
      .filter_map(|line| line.strip_prefix(MISSING))
      .collect();
    assert_eq!(warnings, row.warnings, "{name}: {log}");
    let mut rest = xml.as_str();
    let mut graphics = Vec::new();
    while let Some(element) = xml_element(rest, "graphics", &[]) {
      let at = rest.find("<graphics").unwrap();
      rest = &rest[at + 1..];
      graphics.push(element);
    }
    assert_eq!(graphics.len(), row.graphics.len(), "{name}: {xml}");
    for (found, expected) in graphics.iter().zip(row.graphics) {
      if expected.starts_with('<') {
        assert_eq!(found, expected, "{name}: {xml}");
      } else {
        assert!(found.contains(expected), "{name}: {found}");
      }
    }
  }
}
