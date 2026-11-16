//! Red/green rows for batch LOSTF (OD ruling 13, DG 2026-11-14): three frontmatter regressions that lost content
//! without a Warning, each red on main b72bf540ff. fcs's starred `\author*` keeps its corresponding mark "✉", a contact
//! printing no letter or digit (OXIDIZED_DESIGN_DIVERGENCES #905; 2410.08723, 2401.13011, 2408.05715); vmsta's name parts
//! print as its title page prints them, the observer's rehearsal reading the author content under the binding's markup
//! (#802; 1508.02842, 1411.4499, 1506.02940); the pazh/astl title code's `\address` and `\addresstext` are the marks and
//! notes they print there, so no name merges into an affiliation (#941; 1202.2889, 1108.4222, 1206.0451). Each row
//! converts its repro once, in-process, and asserts its exact Error/Warning counts and every creator's name and contacts.
use latexml::util::test::{convert_files_with, convert_with};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

const PRELOAD: &str = "ar5iv.sty";

/// One row: the repro, the files beside it, its (Error, Warning) counts, every creator (name, then `role=text` per
/// contact) and fragments its XML holds.
struct Row {
  tex:      &'static str,
  files:    &'static [(&'static str, &'static str)],
  counts:   (usize, usize),
  creators: &'static [&'static [&'static str]],
  holds:    &'static [&'static str],
}

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

const FCS_CREATORS: &[&[&str]] = &[
  &["Ann Author", "address=University of Witness"],
  &[
    "Bob Corresponding",
    "corresponding=\u{2709}",
    "address=University of Witness",
    "address=Institute of Repro",
  ],
  &["Carl Plain"],
];

const ROWS: &[Row] = &[
  // fcs.cls:268-290: with the document's `\maketitle`, and without one, as the papers write it (the class makes the title
  // itself, :768-771). Red: Bob without his corresponding contact.
  Row {
    tex:      repro!("sectioning-frontmatter/fcs_author_star_and_address_labels.tex"),
    files:    &[],
    counts:   (0, 0),
    creators: FCS_CREATORS,
    holds:    &[],
  },
  Row {
    tex:      repro!("sectioning-frontmatter/fcs_author_star_without_maketitle.tex"),
    files:    &[],
    counts:   (0, 0),
    creators: FCS_CREATORS,
    holds:    &[],
  },
  // vmsta.cls:278-291 `\name@m@front`: `\inits` dropped, `\fnm~\snm` printed. Red: "Yu.Yuliya Mishura", "I.Ivan Voronov".
  Row {
    tex:      repro!("class-bindings/vmsta/vmsta_title_page_name_parts.tex"),
    files:    &[("vmsta.cls", repro!("class-bindings/vmsta/vmsta.cls"))],
    counts:   (0, 0),
    creators: &[&["Yuliya Mishura"], &["Ivan Voronov"]],
    holds:    &[],
  },
  // astlb.cls:117-118: the names keep their marks and the institutes are notes. Red: five creators, "2 1Univ A" one of
  // them. The e-mail's mark as an e-mail contact of its own and both notes on the last name are wrong still
  // (mark_then_store_email_read_by_a_refused_abstract_red, post-merge); the 2 Warnings count the notes' lost italic.
  Row {
    tex:      repro!("sectioning-frontmatter/astlb_title_code_address_marks_keep_the_names.tex"),
    files:    &[],
    counts:   (0, 2),
    creators: &[&["Ann Able", "mark=1", "email=*", "email=ann@a.org"], &[
      "Bob Baker",
      "mark=1,2",
    ]],
    holds:    &[
      r#"role="thanks" xml:id="id4">Univ A, Moscow</note>"#,
      r#"role="thanks" xml:id="id5">Univ B, Boston</note>"#,
    ],
  },
];

#[test]
fn lost_frontmatter_content_is_kept() {
  for row in ROWS {
    let (log, xml) = if row.files.is_empty() {
      convert_with(row.tex, Some(PRELOAD))
    } else {
      convert_files_with(row.tex, row.files, Some(PRELOAD))
    };
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      row.counts,
      "{log}"
    );
    let want: Vec<Vec<String>> = row
      .creators
      .iter()
      .map(|c| c.iter().map(|s| (*s).to_string()).collect())
      .collect();
    assert_eq!(creators(&xml), want, "{xml}");
    for fragment in row.holds {
      assert!(xml.contains(fragment), "{fragment} in {xml}");
    }
  }
}
