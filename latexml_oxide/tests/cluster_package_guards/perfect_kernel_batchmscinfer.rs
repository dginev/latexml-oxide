//! Guards for MSCINFER: the MSC edition a classification is stamped with follows its codes (`util::msc`,
//! `\lx@msc@infer`; OXIDIZED_DESIGN_DIVERGENCES #1168), under the amsart family's `\subjclass` and amsppt's. One row per
//! case of the rule: the call's year (C1), the author's discarded label (C2), a single fitting edition (C3), the
//! document's year and the nearest edition for a refuted year (R), the class's print, no codes, an edition the class
//! does not know, one whose text names an edition (`[msc2010]`), a code in no edition, and amsppt's yearless and
//! discarded years. A module of its own: the rows count Infos by name and convert dated sources, which the amsart MSC
//! rows of `perfect_kernel_batchthmedpsres` do not.

use latexml::util::test::{
  assert_all_elements, convert_dated_files_with, convert_with, error_count, warning_count,
};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

const MSC: &str = "Mathematics Subject Classification";

/// 2022-06-01, the source date of the dated rows.
const DATED: Option<i64> = Some(1_654_041_600);

struct Row {
  tex:     &'static str,
  /// The source's date (Unix seconds), none for an undated source.
  date:    Option<i64>,
  /// Errors and Warnings.
  counts:  (usize, usize),
  /// Every `Info:frontmatter:msc_…` line, in order.
  infos:   &'static [&'static str],
  /// The edition named, and the label's separator (amsart's colon, amsppt's period).
  edition: &'static str,
  sep:     &'static str,
  codes:   &'static str,
}

const ROWS: &[Row] = &[
  Row {
    tex:     repro!("sectioning-frontmatter/msc_edition_declared_year_fits.tex"),
    date:    DATED,
    counts:  (0, 0),
    infos:   &[],
    edition: "2000",
    sep:     ":\u{a0}",
    codes:   "Primary 35Q55; Secondary 35B40",
  },
  Row {
    tex:     repro!("sectioning-frontmatter/msc_edition_author_label_fits.tex"),
    date:    None,
    counts:  (0, 0),
    infos:   &[
      "msc_edition MSC 2020 (the class's print) -> 2010: the edition of the author's own label (codes fit 1991/2000/2010/2020)",
    ],
    edition: "2010",
    sep:     ":\u{a0}",
    codes:   "16R10; 17A70, 16R30",
  },
  Row {
    tex:     repro!("sectioning-frontmatter/msc_edition_codes_fit_one.tex"),
    date:    None,
    counts:  (0, 0),
    infos:   &[
      "msc_edition MSC 2010 (the class's print) -> 2020: the only edition the codes are in (codes fit 2020)",
    ],
    edition: "2020",
    sep:     ":\u{a0}",
    codes:   "57K12, 20F36",
  },
  Row {
    tex:     repro!("sectioning-frontmatter/msc_edition_refuted_year_takes_the_era.tex"),
    date:    DATED,
    counts:  (0, 0),
    infos:   &[
      "msc_edition MSC 2000 (the class's print) -> 2020: the edition current in the document's year 2022 (codes fit 2010/2020)",
    ],
    edition: "2020",
    sep:     ":\u{a0}",
    codes:   "35Q55, 35B51",
  },
  Row {
    tex:     repro!("sectioning-frontmatter/msc_edition_refuted_year_takes_the_era.tex"),
    date:    None,
    counts:  (0, 0),
    infos:   &[
      "msc_edition MSC 2000 (the class's print) -> 2010: the codes' edition nearest the class's print, the source undated (codes fit 2010/2020)",
    ],
    edition: "2010",
    sep:     ":\u{a0}",
    codes:   "35Q55, 35B51",
  },
  Row {
    tex:     repro!("sectioning-frontmatter/msc_edition_refuted_year_takes_the_nearest.tex"),
    date:    DATED,
    counts:  (0, 0),
    infos:   &[
      "msc_edition MSC 2020 (the class's print) -> 2010: the codes' edition nearest the document's year 2022 (codes fit 1991/2000/2010)",
    ],
    edition: "2010",
    sep:     ":\u{a0}",
    codes:   "18E30, 14F05; 14M15",
  },
  Row {
    tex:     repro!("sectioning-frontmatter/msc_edition_class_print_fits.tex"),
    date:    None,
    counts:  (0, 0),
    infos:   &[],
    edition: "2020",
    sep:     ":\u{a0}",
    codes:   "41A25, 94A12",
  },
  Row {
    tex:     repro!("sectioning-frontmatter/msc_edition_no_codes_acm.tex"),
    date:    None,
    counts:  (0, 0),
    infos:   &[],
    edition: "2020",
    sep:     ":\u{a0}",
    codes:   "F.4.1, D.3.2",
  },
  // (the Warning is amsart's "Unknown edition (2011)")
  Row {
    tex:     repro!("sectioning-frontmatter/msc_edition_unknown_edition_codes_correct.tex"),
    date:    None,
    counts:  (0, 1),
    infos:   &[
      "msc_edition MSC 2020 (the class's print) -> 2010: the only edition the codes are in (codes fit 2010)",
    ],
    edition: "2010",
    sep:     ":\u{a0}",
    codes:   "14T05, 14M25",
  },
  // (the Warning is amsart's "Unknown edition (msc2010)"; the word `msc` is counted by the Info)
  Row {
    tex:     repro!("sectioning-frontmatter/msc_edition_call_text_names_the_edition.tex"),
    date:    None,
    counts:  (0, 1),
    infos:   &[
      "msc_edition MSC 2020 (the class's print) -> 2010: the edition the call names (codes fit 1991/2000/2010/2020)",
    ],
    edition: "2010",
    sep:     ":\u{a0}",
    codes:   "54H15, 54E50",
  },
  Row {
    tex:     repro!("sectioning-frontmatter/msc_edition_unknown_code.tex"),
    date:    None,
    counts:  (0, 0),
    infos:   &["msc_unknown_code MSC code 35Z05 is in no edition (1991, 2000, 2010, 2020)"],
    edition: "2010",
    sep:     ":\u{a0}",
    codes:   "35Q55, 35Q99, 35Z05",
  },
  Row {
    tex:     repro!("amstex/amsppt_msc_edition_yearless_2010_codes.tex"),
    date:    None,
    counts:  (0, 0),
    infos:   &[
      "msc_edition MSC 1991 (the class's print) -> 2010: the only edition the codes are in (codes fit 2010)",
    ],
    edition: "2010",
    sep:     ". ",
    codes:   "14T05, 14M25",
  },
  Row {
    tex:     repro!("amstex/amsppt_msc_edition_discarded_year.tex"),
    date:    None,
    counts:  (0, 0),
    infos:   &[
      "msc_edition MSC 1991 (the class's print) -> 2010: the edition of the author's own label (codes fit 1991/2000/2010/2020)",
    ],
    edition: "2010",
    sep:     ". ",
    codes:   "16R10, 17A70",
  },
];

/// Every row's repro converted once: its counts, its MSC Infos, and its one classification whole.
#[test]
fn mscinfer_rows() {
  for row in ROWS {
    let (log, xml) = match row.date {
      Some(date) => convert_dated_files_with(row.tex, &[], Some("ar5iv.sty"), date),
      None => convert_with(row.tex, Some("ar5iv.sty")),
    };
    let first = row.tex.lines().next().unwrap_or_default();
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      row.counts,
      "{first}: Errors and Warnings\n{log}"
    );
    let infos: Vec<&str> = log
      .lines()
      .filter_map(|line| line.strip_prefix("Info:frontmatter:"))
      .filter(|line| line.starts_with("msc_"))
      .collect();
    assert_eq!(infos, row.infos, "{first}: the MSC Infos\n{log}");
    let (edition, sep, codes) = (row.edition, row.sep, row.codes);
    assert_all_elements(&xml, "classification", &[], &[format!(
      "<classification name=\"{edition} {MSC}{sep}\" scheme=\"{edition} {MSC}\">{codes}</classification>"
    )]);
  }
}
