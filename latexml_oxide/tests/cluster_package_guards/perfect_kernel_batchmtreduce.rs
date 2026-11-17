//! MTREDUCE (DG 2026-11-17, "reduce before the run"): the kernel asks only whether the class's own `\maketitle` runs
//! (`latex_constructs::class_maketitle_in_force`); a package's `\maketitle` prints the marks its binding declares
//! (`latex_constructs::declare_package_title_marks`), any other's are unknown. The class question's rows (a
//! `\NewCommandCopy` wrapper, a closing `\let\maketitle\relax`, a restore) are the FMRG suite's
//! (`perfect_kernel_batchfmmarks`); these are the bindings' paths the suite does not take.
use latexml::util::test::{
  convert_files_with, convert_with_setup_then, error_count, warning_count,
};

use super::perfect_kernel_batchfmmarks::{Row, assert_green, creator_lines, page_lines, page_of};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
  ($topic:literal, $name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/",
      $topic,
      "/",
      $name,
      ".tex"
    ))
  };
}

/// One row per binding path: acl.sty's `\maketitle` defined in its `\AtBeginDocument` code, captured there (the AAAI
/// rows of the FMRG suite capture the one the package's load defines), its marks the document's `\@fnsymbol`
/// (2502.19103, 1905.10033); AAAI's `\equalcontrib` from its binding across ";", " and " and a `\\` line end (2501.02471).
/// MTREDUCEb, one row per style family whose binding declares the marks its copy's `\maketitle` body sets
/// (`shipped_class::read_copy_title_marks`): the NIPS style's descendants ICLR (2303.07937), TMLR
/// (2502.12413), CoRL (2509.00328) and NIPS 2016-2018 (1706.03762), USENIX (2509.03117; its raw read takes the loader's
/// step after a raw package, so the copy's `\abstract` heading is no creator, 2301.04841) and siamproceedings
/// (2509.02371), each a new binding reading its copy raw; jmlr2e's stand-in copy read after its binding (2509.13653); the arxiv and PRIMEarxiv bindings' raw reads (2509.16272, 2509.03066); gtpart without its class,
/// whose metadata setters print nothing (1108.0474). MTREDUCEc, the body's line decides, one row per path: a fixed-glyph
/// macro read off the refused body (aaai2026.sty's `\fndag`, 2511.07192), an `\AtBeginDocument` body whose line the
/// binding does not know (acl, `\arabic`). MTREDUCEd: only a line before the body's first `\@maketitle` decides, so one
/// after it leaves the names unmarked (`[a]` for a shared note) and a reset after `\endgroup` keeps the marks; a line
/// under a conditional decides nothing (revMTREDUCEc S1, P1); and where no binding declares them, the marks of the
/// `\maketitle` that runs are read off its body the same way (DG 2026-11-17, `latex_constructs::read_title_marks_in_force`):
/// a renamed copy's (2509.10105's vv2.sty, its `\footnotemark[1]` linked again), an unbound package's (1503.02531's
/// nips14submit_e.sty, ltexpprt.sty), their typed marks linked; not under a raw class, whose title notes step no counter
/// here (siamart's, 2107.07572: no glyph).
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
  // (1 Warning: the stub's note written as its text, as below)
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
  Row {
    repro:      "mtreduceb_iclr_maketitle_reads_the_document_s_fnsymbol",
    tex:        repro!("mtreduceb_iclr_maketitle_reads_the_document_s_fnsymbol"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution."#,
      r#"Bob Baker ; thanks[*]=Equal contribution."#,
      r#"Cy Cole ; thanks[†]=Corresponding author."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole ; † plain"#,
      r#"margin: Corresponding author. ; Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreduceb_tmlr_maketitle_reads_the_document_s_fnsymbol",
    tex:        repro!("mtreduceb_tmlr_maketitle_reads_the_document_s_fnsymbol"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=First note."#,
      r#"Bob Baker ; thanks[†]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"margin: First note. ; Second note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreduceb_corl_maketitle_reads_the_document_s_fnsymbol",
    tex:        repro!("mtreduceb_corl_maketitle_reads_the_document_s_fnsymbol"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution. Correspondence to: ann@x.org"#,
      r#"Bob Baker ; thanks[*]=Equal contribution. Correspondence to: ann@x.org"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"margin: Equal contribution. Correspondence to: ann@x.org"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreduceb_nips_maketitle_reads_the_document_s_fnsymbol",
    tex:        repro!("mtreduceb_nips_maketitle_reads_the_document_s_fnsymbol"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution."#,
      r#"Bob Baker ; thanks[*]=Equal contribution."#,
      r#"Cy Cole ; thanks[†]=Work performed while at X."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole ; † plain"#,
      r#"margin: Equal contribution. ; Work performed while at X."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreduceb_usenix_maketitle_reads_the_document_s_fnsymbol",
    tex:        repro!("mtreduceb_usenix_maketitle_reads_the_document_s_fnsymbol"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; thanks[*]=Corresponding Author: Bob Baker (bob@y.org)."#,
      r#"Cy Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; * plain"#,
      r#"Cy Cole"#,
      r#"margin: Corresponding Author: Bob Baker (bob@y.org)."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreduceb_siamproceedings_maketitle_reads_the_document_s_fnsymbol",
    tex:        repro!("mtreduceb_siamproceedings_maketitle_reads_the_document_s_fnsymbol"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Univ A (ann@x.org)."#,
      r#"Bob Baker ; thanks[†]=Univ B (bob@y.org)."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"margin: Univ A (ann@x.org). ; Univ B (bob@y.org)."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreduceb_jmlr2e_stand_in_copy_reads_the_document_s_fnsymbol",
    tex:        repro!("mtreduceb_jmlr2e_stand_in_copy_reads_the_document_s_fnsymbol"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Corresponding author."#,
      r#"Bob Baker"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker"#,
      r#"margin: Corresponding author."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreduceb_primearxiv_maketitle_reads_the_document_s_fnsymbol",
    tex:        repro!("mtreduceb_primearxiv_maketitle_reads_the_document_s_fnsymbol"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=corresponding author"#,
      r#"Bob Baker"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker"#,
      r#"margin: corresponding author"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreduceb_arxiv_maketitle_reads_the_document_s_fnsymbol",
    tex:        repro!("mtreduceb_arxiv_maketitle_reads_the_document_s_fnsymbol"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution."#,
      r#"Bob Baker ; thanks[*]=Equal contribution."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreduceb_gtpart_metadata_setters_print_nothing",
    tex:        repro!(
      "class-bindings",
      "mtreduceb_gtpart_metadata_setters_print_nothing"
    ),
    warnings:   1,
    creators:   &[r#"Ann Able | address=University of Somewhere ; email=ann@x.org"#],
    moves:      r#""#,
    page:       &[r#"Ann Able"#],
    page_moves: r#""#,
  },
  // (1 Warning: the binding's `\equalcontrib` stub holds the note's text, no call argument the emitter can take its
  // tokens from, so it is written as its text — `emit::by_text`, "note no argument"; plain text, no markup lost)
  Row {
    repro:      "mtreducec_aaai_copy_fndag_line_prints_its_glyph",
    tex:        repro!("mtreducec_aaai_copy_fndag_line_prints_its_glyph"),
    warnings:   1,
    creators:   &[
      r#"Ann Able ; thanks[†]=These authors contributed equally."#,
      r#"Bob Baker ; thanks[†]=These authors contributed equally."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; †→note"#,
      r#"Bob Baker ; †→note"#,
      r#"margin: These authors contributed equally."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreducec_acl_begin_document_maketitle_arabic_prints_no_marks",
    tex:        repro!("mtreducec_acl_begin_document_maketitle_arabic_prints_no_marks"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=First note."#,
      r#"Bob Baker ; thanks[]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: First note."#,
      r#"Bob Baker | under: Second note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreduced_nips_fnsymbol_after_maketitle_leaves_names_unmarked",
    tex:        repro!("mtreduced_nips_fnsymbol_after_maketitle_leaves_names_unmarked"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=Equal contribution."#,
      r#"Bob Baker ; thanks[]=Equal contribution."#,
      r#"Cy Cole ; thanks[]=Work at X."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; [a]→note"#,
      r#"Bob Baker ; [a]→note"#,
      r#"Cy Cole | under: Work at X."#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreduced_nips_reset_after_endgroup_keeps_fnsymbol",
    tex:        repro!("mtreduced_nips_reset_after_endgroup_keeps_fnsymbol"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution."#,
      r#"Bob Baker ; thanks[*]=Equal contribution."#,
      r#"Cy Cole ; thanks[†]=Work at X."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole ; † plain"#,
      r#"margin: Equal contribution. ; Work at X."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreduced_nips_fnsymbol_under_false_conditional_leaves_names_unmarked",
    tex:        repro!("mtreduced_nips_fnsymbol_under_false_conditional_leaves_names_unmarked"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=Equal contribution."#,
      r#"Bob Baker ; thanks[]=Equal contribution."#,
      r#"Cy Cole ; thanks[]=Work at X."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; [a]→note"#,
      r#"Bob Baker ; [a]→note"#,
      r#"Cy Cole | under: Work at X."#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreduced_whole_block_footnotemark_links_its_own_note",
    tex:        repro!("mtreduced_whole_block_footnotemark_links_its_own_note"),
    warnings:   1,
    creators:   &[
      r#"Ann Able Bob Baker Dee Dunn Fay Fox ; footnotemark[*]= ; thanks[*]=Equal contribution"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able Bob Baker Dee Dunn Fay Fox ; *→note ; * plain"#,
      r#"margin: Equal contribution"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreduced_nips14submit_typed_dagger_links_its_thanks",
    tex:        repro!("mtreduced_nips14submit_typed_dagger_links_its_thanks"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Also at X. ; thanks[†]=Equal contribution."#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Cy Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain ; †→note"#,
      r#"Bob Baker ; †→note"#,
      r#"Cy Cole"#,
      r#"margin: Also at X. ; Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreduced_raw_copy_maketitle_typed_star_links_its_thanks",
    tex:        repro!("mtreduced_raw_copy_maketitle_typed_star_links_its_thanks"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Karlsruhe Institute of Technology."#,
      r#"Bob Baker ; footnotemark[*]="#,
      r#"Cy Cole ; footnotemark[*]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole ; *→note"#,
      r#"margin: Karlsruhe Institute of Technology."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreduced_raw_class_title_thanks_marks_unknown",
    tex:        repro!("mtreduced_raw_class_title_thanks_marks_unknown"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=Univ A."#,
      r#"Bob Baker ; thanks[]=Univ B."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: Univ A."#,
      r#"Bob Baker | under: Univ B."#,
    ],
    page_moves: r#""#,
  },
];

/// A package's `\maketitle` prints the marks its binding declares, wherever the package defines it.
#[test]
fn package_title_marks_are_their_binding_s() { assert_green(BINDINGS); }

/// MTREDUCEE: the kernel reads, where no binding declares them, the marks that hold where the names print
/// (`latex_constructs::kernel_title_marks`, revMTREDUCEd), each row a way they are not the body's `\fnsymbol` line, so
/// no glyph and no link: a definition whose group closes before `\@maketitle` (a brace group; `\begingroup`), the
/// `\@maketitle` in force setting its own `\arabic` before `\@author`, a body calling neither (B1); a fixed-glyph macro,
/// which stays a binding's (`\newcommand` `*` and `x`, `\def`, `\def\thefootnote`; B2); `\@author` before a `\@title`
/// holding a note (S1); a redefined `\thanks` and a `\let\thefootnote` (S2); a second `\maketitle`, the read ending
/// with the first (S3); IEEEtran, whose title notes step no counter here (N1). Each probe pdflatex-checked.
const KERNEL_READ: &[Row] = &[
  Row {
    repro:      "mtreducee_own_maketitle_fnsymbol_in_a_closed_brace_group_prints_no_glyph",
    tex:        repro!("mtreducee_own_maketitle_fnsymbol_in_a_closed_brace_group_prints_no_glyph"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=Univ A."#,
      r#"Bob Baker ; thanks[]=Univ B."#,
      r#"Cy Cole ; footnotemark[*]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: Univ A."#,
      r#"Bob Baker | under: Univ B."#,
      r#"Cy Cole ; * plain"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreducee_own_maketitle_fnsymbol_in_a_closed_begingroup_prints_no_glyph",
    tex:        repro!("mtreducee_own_maketitle_fnsymbol_in_a_closed_begingroup_prints_no_glyph"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=Univ A."#,
      r#"Bob Baker ; thanks[]=Univ B."#,
      r#"Cy Cole ; footnotemark[*]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: Univ A."#,
      r#"Bob Baker | under: Univ B."#,
      r#"Cy Cole ; * plain"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreducee_own_maketitle_inner_maketitle_arabic_prints_no_glyph",
    tex:        repro!("mtreducee_own_maketitle_inner_maketitle_arabic_prints_no_glyph"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=Univ A."#,
      r#"Bob Baker ; thanks[]=Univ B."#,
      r#"Cy Cole ; footnotemark[*]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: Univ A."#,
      r#"Bob Baker | under: Univ B."#,
      r#"Cy Cole ; * plain"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreducee_own_maketitle_without_maketitle_or_author_prints_no_glyph",
    tex:        repro!("mtreducee_own_maketitle_without_maketitle_or_author_prints_no_glyph"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=Univ A."#,
      r#"Bob Baker ; thanks[]=Univ B."#,
      r#"Cy Cole ; footnotemark[*]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: Univ A."#,
      r#"Bob Baker | under: Univ B."#,
      r#"Cy Cole ; * plain"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreducee_own_maketitle_fixed_glyph_macro_prints_no_glyph",
    tex:        repro!("mtreducee_own_maketitle_fixed_glyph_macro_prints_no_glyph"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=Univ A."#,
      r#"Bob Baker ; thanks[]=Univ B."#,
      r#"Cy Cole ; footnotemark[*]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: Univ A."#,
      r#"Bob Baker | under: Univ B."#,
      r#"Cy Cole ; * plain"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreducee_own_maketitle_fixed_letter_macro_prints_no_glyph",
    tex:        repro!("mtreducee_own_maketitle_fixed_letter_macro_prints_no_glyph"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=Univ A."#,
      r#"Bob Baker ; thanks[]=Univ B."#,
      r#"Cy Cole ; footnotemark[*]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: Univ A."#,
      r#"Bob Baker | under: Univ B."#,
      r#"Cy Cole ; * plain"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreducee_own_maketitle_fixed_letter_def_macro_prints_no_glyph",
    tex:        repro!("mtreducee_own_maketitle_fixed_letter_def_macro_prints_no_glyph"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=Univ A."#,
      r#"Bob Baker ; thanks[]=Univ B."#,
      r#"Cy Cole ; footnotemark[*]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: Univ A."#,
      r#"Bob Baker | under: Univ B."#,
      r#"Cy Cole ; * plain"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreducee_own_maketitle_def_thefootnote_fixed_letter_macro_prints_no_glyph",
    tex:        repro!(
      "mtreducee_own_maketitle_def_thefootnote_fixed_letter_macro_prints_no_glyph"
    ),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=Univ A."#,
      r#"Bob Baker ; thanks[]=Univ B."#,
      r#"Cy Cole ; footnotemark[*]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: Univ A."#,
      r#"Bob Baker | under: Univ B."#,
      r#"Cy Cole ; * plain"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreducee_own_maketitle_author_before_noted_title_prints_no_glyph",
    tex:        repro!("mtreducee_own_maketitle_author_before_noted_title_prints_no_glyph"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=Univ A."#,
      r#"Bob Baker ; thanks[]=Univ B."#,
      r#"Cy Cole ; footnotemark[†]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: Univ A."#,
      r#"Bob Baker | under: Univ B."#,
      r#"Cy Cole ; † plain"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreducee_own_maketitle_redefined_thanks_prints_no_glyph",
    tex:        repro!("mtreducee_own_maketitle_redefined_thanks_prints_no_glyph"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=Univ A."#,
      r#"Bob Baker ; thanks[]=Univ B."#,
      r#"Cy Cole ; footnotemark[*]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: Univ A."#,
      r#"Bob Baker | under: Univ B."#,
      r#"Cy Cole ; * plain"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreducee_own_maketitle_let_thefootnote_prints_no_glyph",
    tex:        repro!("mtreducee_own_maketitle_let_thefootnote_prints_no_glyph"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=Univ A."#,
      r#"Bob Baker ; thanks[]=Univ B."#,
      r#"Cy Cole ; footnotemark[*]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: Univ A."#,
      r#"Bob Baker | under: Univ B."#,
      r#"Cy Cole ; * plain"#,
    ],
    page_moves: r#""#,
  },
  // (1 Warning: the first block's names, which the second `\title` and `\author` replace, missing from the placed
  // frontmatter and counted — the base's too)
  Row {
    repro:      "mtreducee_own_maketitle_second_run_prints_no_glyph",
    tex:        repro!("mtreducee_own_maketitle_second_run_prints_no_glyph"),
    warnings:   1,
    creators:   &[
      r#"Dan Dee ; thanks[]=Univ D."#,
      r#"Eve Eck ; thanks[]=Univ E."#,
    ],
    moves:      r#""#,
    page:       &[r#"Dan Dee | under: Univ D."#, r#"Eve Eck | under: Univ E."#],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreducee_ieeetran_own_maketitle_prints_no_glyph",
    tex:        repro!("mtreducee_ieeetran_own_maketitle_prints_no_glyph"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker"#,
      r#"Cy Cole ; footnotemark[*]="#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#, r#"Cy Cole ; * plain"#],
    page_moves: r#""#,
  },
];

/// The kernel reads only what holds where the names print: else no glyph, no link.
#[test]
fn kernel_read_marks_are_what_holds_where_the_names_print() { assert_green(KERNEL_READ); }

/// A stand-in copy loaded from a subdirectory (`\usepackage{sub/neurips_2022}`; 44 of 120 NeurIPS papers) declares its
/// marks as one beside the document does: the binding looks the copy up by its basename, as it is recorded (the
/// fixture is neurips_2022.sty itself, comments stripped: a minimal stand-in is no stood-in file and is never read)
/// (revMTREDUCEc B1; 2210.05974, 2510.04543's `utils/neurips_2025` by `\input`).
#[test]
fn a_copy_in_a_subdirectory_keeps_its_marks() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/neurips_sub/neurips_copy_in_a_subdirectory_keeps_its_marks.tex"
    ),
    &[(
      "sub/neurips_2022.sty",
      include_str!(
        "../../../tools/perfect_kernel/repros/class-bindings/neurips_sub/sub/neurips_2022.sty"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creator_lines(&xml),
    [
      "Ann Able ; thanks[*]=Equal contribution.",
      "Bob Baker ; thanks[*]=Equal contribution.",
      "Cy Cole ; thanks[†]=Corr.",
    ],
    "{xml}"
  );
}

/// A stand-in copy read as the file its binding stands in for (neurips_2022.sty itself, comments stripped) whose
/// `\maketitle` sets no `\thefootnote`, its lines commented out, declares no marks: each note under its name, unmarked
/// (2305.15254; MTREDUCEb declared ∗ and †, which the PDF never prints). Its own test: the copy is a file beside the
/// document, which a row's single source cannot ship whole (MTREDUCEd).
#[test]
fn a_stand_in_copy_setting_no_marks_declares_none() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/neurips_commented/mtreducec_neurips_commented_maketitle_line_prints_no_marks.tex"
    ),
    &[(
      "neurips_2022.sty",
      include_str!(
        "../../../tools/perfect_kernel/repros/class-bindings/neurips_commented/neurips_2022.sty"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains("read as the file its binding stands in for"),
    "{log}"
  );
  assert_eq!(
    creator_lines(&xml),
    [
      "Ann Able ; thanks[]=Equal contribution.",
      "Bob Baker ; thanks[]=Corresponding: bob@x.org",
    ],
    "{xml}"
  );
  assert_eq!(
    page_lines(&page_of(&xml)),
    (
      vec![
        "Ann Able | under: Equal contribution.".to_string(),
        "Bob Baker | under: Corresponding: bob@x.org".to_string(),
      ],
      Vec::<String>::new()
    )
  );
}

/// A binding declares only the marks its copy's `\maketitle` in force prints: siamproceedings.sty's `titlepage` one
/// (:153-176) sets no `\thefootnote`, the title page's marks unknown here (row R TITLEPAGEMARKS; pdflatex prints article's
/// `\arabic`), so no glyph is written, never the other branch's `\@fnsymbol` (MTREDUCEb).
#[test]
fn siamproceedings_titlepage_declares_no_marks() {
  let tex = repro!("mtreduceb_siamproceedings_titlepage_marks_unknown");
  let (log, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
  assert_eq!(
    (error_count(&log), warning_count(&log), creator_lines(&xml)),
    (0, 0, vec![
      "Ann Able ; thanks[]=Univ A (ann@x.org).".to_string(),
      "Bob Baker ; thanks[]=Univ B (bob@y.org).".to_string(),
    ])
  );
}
