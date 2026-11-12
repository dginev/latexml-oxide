//! Red/green guards for batch PSF1: a `\psfrag` fragment an EPS graphic would place (latex + dvips) is not rendered
//! (PSFR, post-merge), so every EPS graphic in a fragment's scope (`\includegraphics`, `\epsfbox`, `\epsffile`,
//! `\psfig`) is one Warning naming the graphic and every tag (and `\psfragscanon`'s embedded `\tex` string) it drops
//! (PSF1b); a fragment no EPS graphic reaches prints nothing in TeX either. One table; each row's oracle (latex + dvips
//! or pdflatex `!` count, the printed text) is in its repro header under `tools/perfect_kernel/repros/reversion/`.
//! Red (int166a): every row 0 warnings; the `.epsi`/`.epsf` and `.eps.gz` rows also on PSF1b (EPS by extension).
use std::io::Write;

use latexml::util::test::{convert_bytes_files_then, xml_element};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// One repro: its source, its exact (errors, warnings), the Warning messages in order, and every `<graphics>`
/// element whole (or, where its candidates name a host path, the attribute it holds); `gzip` files are written
/// beside it compressed.
struct Row {
  name:     &'static str,
  tex:      &'static str,
  gzip:     &'static [(&'static str, &'static str)],
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

/// The PostScript the `.eps.gz` row compresses: an "A" psfrag replaces.
const EPS_A: &str = "%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 72 36\n/Times-Roman findfont 12 scalefont \
                     setfont\n10 10 moveto (A) show\nshowpage\n";

const ROWS: &[Row] = &[
  // The RED (QA41): a fragment in a `figure` consumed by the graphic in it; rendering stays RED (PSFR).
  Row {
    name:     "psfrag_fragment_dropped_silently_red",
    gzip:     &[],
    tex:      repro!("psfrag_fragment_dropped_silently_red"),
    counts:   (0, 1),
    warnings: &["The psfrag replacement of 'A' in the graphic 'psfragfig.eps' is not rendered"],
    graphics: &[r#"<graphics graphic="psfragfig.eps" xml:id="S0.F1.g1"/>"#],
  },
  // Preamble fragments and a `\psfrag*` placement reach both graphics, the extensionless one too: one Warning each,
  // naming all three placements in order.
  Row {
    name:     "psfrag_star_and_preamble_fragments_per_graphic",
    gzip:     &[],
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
    gzip:     &[],
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
    gzip:     &[],
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
    gzip:     &[],
    tex:      repro!("psfrag_scan_embedded_tex_fragment"),
    counts:   (0, 2),
    warnings: &[
      r"The psfrag replacement of the file's own \tex labels '\tex[bl][bl]{$\alpha$}' in the graphic 'psfragscan.eps' is not rendered",
      r"The psfrag replacement of the file's own \tex labels '\tex[bl][bl]{$\alpha$}' in the graphic 'psfragscan.eps' is not rendered",
    ],
    graphics: &[
      r#"<graphics graphic="psfragscan.eps" xml:id="p1.g1"/>"#,
      r#"<graphics graphic="psfragscan.eps" xml:id="p2.g1"/>"#,
      r#"<graphics graphic="psfragscan.eps" xml:id="p3.g1"/>"#,
    ],
  },
  // EPS by its PostScript header, whatever its extension (dvips's `\Gin@rule@*`; epsf reads any file): an `.epsi`
  // through `\includegraphics` and `\psfig` (hep-ph0610439), an `.epsf` through `\epsfbox` and `\epsffile`.
  Row {
    name:     "psfrag_fragment_eps_by_header",
    gzip:     &[],
    tex:      repro!("psfrag_fragment_eps_by_header"),
    counts:   (0, 4),
    warnings: &[
      "The psfrag replacement of 'A' in the graphic 'psfragi.epsi' is not rendered",
      "The psfrag replacement of 'A' in the graphic 'psfragi.epsi' is not rendered",
      "The psfrag replacement of 'A' in the graphic 'psfragf.epsf' is not rendered",
      "The psfrag replacement of 'A' in the graphic 'psfragf.epsf' is not rendered",
    ],
    graphics: &[
      r#"<graphics graphic="psfragi.epsi" xml:id="p1.g1"/>"#,
      r#"<graphics graphic="psfragi.epsi" xml:id="p2.g1"/>"#,
      r#"<graphics graphic="psfragf.epsf" xml:id="p3.g1"/>"#,
      r#"<graphics graphic="psfragf.epsf" xml:id="p4.g1"/>"#,
    ],
  },
  // A compressed EPS on disk, no PostScript header: EPS by the declared `\Gin@rule@.eps.gz` of type eps.
  Row {
    name:     "psfrag_fragment_eps_gz_rule",
    gzip:     &[("psfraggz.eps.gz", EPS_A)],
    tex:      repro!("psfrag_fragment_eps_gz_rule"),
    counts:   (0, 1),
    warnings: &["The psfrag replacement of 'A' in the graphic 'psfraggz.eps.gz' is not rendered"],
    graphics: &[r#"candidates="psfraggz.eps.gz""#],
  },
  // Safe side: a PNG graphic (pdflatex) is no EPS, so nothing is placed or lost.
  Row {
    name:     "psfrag_fragment_non_eps_graphic",
    gzip:     &[],
    tex:      repro!("psfrag_fragment_non_eps_graphic"),
    counts:   (0, 0),
    warnings: &[],
    graphics: &[r#"graphic="example-image.png""#],
  },
];

#[test]
fn batchpsf1_dropped_fragments_are_counted() {
  for row in ROWS {
    let gzip: Vec<(&str, Vec<u8>)> = row
      .gzip
      .iter()
      .map(|(file, text)| {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(text.as_bytes()).unwrap();
        (*file, encoder.finish().unwrap())
      })
      .collect();
    let files: Vec<(&str, &[u8])> = gzip
      .iter()
      .map(|(file, bytes)| (*file, &bytes[..]))
      .collect();
    let (log, xml, ()) =
      convert_bytes_files_then(row.tex.as_bytes(), &files, Some("ar5iv.sty"), |_, _| ());
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
