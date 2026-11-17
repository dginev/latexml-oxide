//! MTREDUCE (DG 2026-11-17, "reduce before the run"): the kernel asks only whether the class's own `\maketitle` runs
//! (`latex_constructs::class_maketitle_in_force`); a package's `\maketitle` prints the marks its binding declares
//! (`latex_constructs::declare_package_title_marks`), any other's are unknown. The class question's rows (a
//! `\NewCommandCopy` wrapper, a closing `\let\maketitle\relax`, a restore) are the FMRG suite's
//! (`perfect_kernel_batchfmmarks`); these are the bindings' paths the suite does not take.
use super::perfect_kernel_batchfmmarks::{Row, assert_green};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// One row per binding path: acl.sty's `\maketitle` defined in its `\AtBeginDocument` code, captured there (the AAAI
/// rows of the FMRG suite capture the one the package's load defines), its marks the document's `\@fnsymbol`
/// (2502.19103, 1905.10033); AAAI's `\equalcontrib` from its binding across ";", " and " and a `\\` line end (2501.02471).
const BINDINGS: &[Row] = &[
  Row {
    repro:      "mtreduce_acl_maketitle_at_begin_document_reads_the_document_s_fnsymbol",
    tex:        repro!("mtreduce_acl_maketitle_at_begin_document_reads_the_document_s_fnsymbol"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[†]=First note."#,
      r#"Bob Baker ; thanks[‡]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; † plain"#,
      r#"Bob Baker ; ‡ plain"#,
      r#"margin: First note. ; Second note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreduce_aaai_equalcontrib_across_separators",
    tex:        repro!("mtreduce_aaai_equalcontrib_across_separators"),
    warnings:   1,
    creators:   &[
      r#"Ann Able ; thanks[*]=These authors contributed equally."#,
      r#"Bob Baker ; thanks[*]=These authors contributed equally."#,
      r#"Cy Cole ; thanks[*]=These authors contributed equally."#,
      r#"Dee Dunn ; thanks[†]=Corresponding author."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole ; *→note"#,
      r#"Dee Dunn ; † plain"#,
      r#"margin: Corresponding author. ; These authors contributed equally."#,
    ],
    page_moves: r#""#,
  },
];

/// A package's `\maketitle` prints the marks its binding declares, wherever the package defines it.
#[test]
fn package_title_marks_are_their_binding_s() { assert_green(BINDINGS); }
