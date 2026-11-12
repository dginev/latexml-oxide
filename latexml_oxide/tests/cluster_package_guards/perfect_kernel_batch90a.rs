//! Guards for 90a's glued-script rule, restored by ISO1 (OXIDIZED_DESIGN_DIVERGENCES #911): the lexer's
//! `glue_in_word_marks` and `events::mark_led_pieces`, one row per reading path.

use std::borrow::Cow;

use super::perfect_kernel_batchhg1::{Creators, Row, assert_rows};

/// A repro of `sectioning-frontmatter/`, its name the row's.
macro_rules! repro {
  ($name:literal) => {
    (
      $name,
      Cow::Borrowed(include_str!(concat!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
        $name,
        ".tex"
      ))),
    )
  };
}

/// Each row as pdflatex prints it (the repro headers name the oracle): a glued script is the formula's text unless a name
/// requests it or, none requesting, it opens its line (OXIDIZED_DESIGN_DIVERGENCES #911); one row per reading path.
#[test]
fn a_script_before_printed_ink_or_on_a_base_is_text() {
  assert_rows(&[
    Row {
      repro:    repro!("revtex_affiliation_isotope_prescript_keeps_its_line"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">1</text></sup></personname><contact name="Affiliation: " role="affiliation">Univ X</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">2</text></sup></personname><contact name="Affiliation: " role="affiliation">Univ Y, <Math mode="inline" tex="{\rm\lx@frontmatter@keepsup{3}He}" text="[3] * He"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMText><sup>3</sup></XMText><XMTok role="UNKNOWN">He</XMTok></XMApp></XMath></Math> Lab</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("revtex_affiliation_empty_base_prescript_keeps_its_line"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">1</text></sup></personname><contact name="Affiliation: " role="affiliation">Univ X</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">2</text></sup></personname><contact name="Affiliation: " role="affiliation">Univ Y, <Math mode="inline" tex="{}\lx@frontmatter@keepsup{87}{\rm Rb}" text="[87] * Rb"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMText><sup>87</sup></XMText><XMTok role="UNKNOWN">Rb</XMTok></XMApp></XMath></Math> Lab</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("kernel_author_line_isotope_prescript_keeps_its_entry"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><contact name="Affiliation: " role="affiliation">Univ X</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker</personname><contact name="Affiliation: " role="affiliation">Univ Y, <Math mode="inline" tex="{\rm{}^{3}He}" text="^ 3 He"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre2"/><XMTok role="UNKNOWN">He</XMTok><XMTok fontsize="70%" meaning="3" role="NUMBER">3</XMTok></XMApp></XMath></Math> Lab</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("elsarticle_address_isotope_prescript_keeps_its_line"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">1</text></sup></personname><contact name="Address: " role="address"><sup><text font="italic">1</text></sup>Univ X</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">2</text></sup></personname><contact name="Address: " role="address"><sup><text font="italic">2</text></sup>Univ Y, <Math mode="inline" tex="{\rm{}^{3}He}" text="^ 3 He"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre2"/><XMTok role="UNKNOWN">He</XMTok><XMTok fontsize="70%" meaning="3" role="NUMBER">3</XMTok></XMApp></XMath></Math> Lab</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("aa_institute_hand_marked_isotope_prescript_keeps_its_line"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">1</text></sup></personname><contact name="Affiliation: " role="affiliation"><sup><text font="italic">1</text></sup>Univ X</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">2</text></sup></personname><contact name="Affiliation: " role="affiliation"><sup><text font="italic">2</text></sup>Univ Y, <Math mode="inline" tex="{\rm{}^{3}He}" text="^ 3 He"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre2"/><XMTok role="UNKNOWN">He</XMTok><XMTok fontsize="70%" meaning="3" role="NUMBER">3</XMTok></XMApp></XMath></Math> Lab</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("revtex_affiliation_exponent_on_a_number_keeps_its_line"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">1</text></sup></personname><contact name="Affiliation: " role="affiliation">Univ X</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">2</text></sup></personname><contact name="Affiliation: " role="affiliation">Univ Y, Lab <Math mode="inline" tex="10\lx@frontmatter@keepsup{3}" text="10 * [3]"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok meaning="10" role="NUMBER">10</XMTok><XMText><sup>3</sup></XMText></XMApp></XMath></Math> Unit</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("revtex_affiliation_in_formula_leading_mark_labels_its_line"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">1</text></sup></personname><contact name="Affiliation: " role="affiliation"><Math mode="inline" tex="{\rm{}^{1}Dept}" text="^ 1 Dept"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre2"/><XMTok role="UNKNOWN">Dept</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math> of Physics, Univ X</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">2</text></sup></personname><contact name="Affiliation: " role="affiliation"><Math mode="inline" tex="{\rm{}^{2}Lab}" text="^ 2 Lab"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre2"/><XMTok role="UNKNOWN">Lab</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math> of Chemistry, Univ Y</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("revtex_affiliation_glued_and_spaced_marks_part_the_line"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">a</text></sup></personname><contact name="Affiliation: " role="affiliation">Univ X</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">b</text></sup></personname><contact name="Affiliation: " role="affiliation"> Univ Y</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("revtex_affiliation_requested_in_formula_marks_part_one_line"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">1</text></sup></personname><contact name="Affiliation: " role="affiliation"><Math mode="inline" tex="{\rm{}^{1}Dept}" text="^ 1 Dept"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre2"/><XMTok role="UNKNOWN">Dept</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math> of Physics, Univ X</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">2</text></sup></personname><contact name="Affiliation: " role="affiliation"><Math mode="inline" tex="{\rm{}^{2}Lab}" text="^ 2 Lab"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre2"/><XMTok role="UNKNOWN">Lab</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math> of Chemistry, Univ Y</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("elsarticle_address_requested_in_formula_marks_part_one_line"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">a</text></sup></personname><contact name="Address: " role="address"><Math mode="inline" tex="{\rm{}^{a}Dept}" text="^ a Dept"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre2"/><XMTok role="UNKNOWN">Dept</XMTok><XMTok fontsize="70%" role="UNKNOWN">a</XMTok></XMApp></XMath></Math> of Physics, Univ X</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">b</text></sup></personname><contact name="Address: " role="address"><Math mode="inline" tex="{\rm{}^{b}Lab}" text="^ b Lab"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre2"/><XMTok role="UNKNOWN">Lab</XMTok><XMTok fontsize="70%" role="UNKNOWN">b</XMTok></XMApp></XMath></Math> of Chemistry, Univ Y</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("kernel_author_line_requested_in_formula_marks_part_one_line"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><contact name="Affiliation: " role="affiliation"><Math mode="inline" tex="{\rm{}^{1}Dept}" text="^ 1 Dept"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre2"/><XMTok role="UNKNOWN">Dept</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math> of Physics, Univ X</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker</personname><contact name="Affiliation: " role="affiliation"><Math mode="inline" tex="{\rm{}^{2}Lab}" text="^ 2 Lab"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre2"/><XMTok role="UNKNOWN">Lab</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math> of Chemistry, Univ Y</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("authblk_affil_requested_in_formula_marks_part_one_line"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><contact name="Affiliation: " role="affiliation"><Math mode="inline" tex="{\rm{}^{1}Dept}" text="^ 1 Dept"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre2"/><XMTok role="UNKNOWN">Dept</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math> of Physics, Univ X</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker</personname><contact name="Affiliation: " role="affiliation"><Math mode="inline" tex="{\rm{}^{2}Lab}" text="^ 2 Lab"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre2"/><XMTok role="UNKNOWN">Lab</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math> of Chemistry, Univ Y</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("ieeetran_blocka_requested_in_formula_marks_part_one_line"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">1</text></sup></personname><contact name="Affiliation: " role="affiliation"><Math mode="inline" tex="{\rm{}^{1}Dept}" text="^ 1 Dept"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre2"/><XMTok role="UNKNOWN">Dept</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math> of Physics, Univ X</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">2</text></sup></personname><contact name="Affiliation: " role="affiliation"><Math mode="inline" tex="{\rm{}^{2}Lab}" text="^ 2 Lab"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre2"/><XMTok role="UNKNOWN">Lab</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math> of Chemistry, Univ Y</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("unknown_class_affiliation_requested_in_formula_marks_part_one_line"),
      files:    &[],
      counts:   (0, 1),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">1</text></sup></personname><contact name="Affiliation: " role="affiliation"><Math mode="inline" tex="{\rm{}^{1}Dept}" text="^ 1 Dept"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre2"/><XMTok role="UNKNOWN">Dept</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math> of Physics, Univ X</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">2</text></sup></personname><contact name="Affiliation: " role="affiliation"><Math mode="inline" tex="{\rm{}^{2}Lab}" text="^ 2 Lab"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre2"/><XMTok role="UNKNOWN">Lab</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math> of Chemistry, Univ Y</contact></creator>"#,
      ]),
    },
    // (pinned RED: the cut inside `$5^{2}$` writes the formula on both sides, its duplicated "5" counted)
    Row {
      repro:    repro!("revtex_affiliation_requested_exponent_on_a_number_parts_the_line_red"),
      files:    &[],
      counts:   (0, 1),
      lines:    &["the placed frontmatter more often than the author blocks print them: 5."],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">1</text></sup></personname><contact name="Affiliation: " role="affiliation"><sup><text font="italic">1</text></sup>Univ X, Lab <Math mode="inline" tex="5^{2}" text="5 ^ 2"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok meaning="5" role="NUMBER">5</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math></contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">2</text></sup></personname><contact name="Affiliation: " role="affiliation"><Math mode="inline" tex="5^{2}" text="5 ^ 2"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok meaning="5" role="NUMBER">5</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math>Univ Y</contact></creator>"#,
      ]),
    },
  ]);
}
