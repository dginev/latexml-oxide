//! Guards for 90a's rule, restored by ISO1 (OXIDIZED_DESIGN_DIVERGENCES #911): a script a letter or digit prints after in
//! its own formula is a prescript or an exponent, the formula's text, not a mark — the lexer's `glue_in_word_marks`, one
//! place for every reader of the events —, and an exponent on a base its formula prints opens no entry
//! (`events::mark_led_pieces`). One row per reading path (REVTeX's `\affiliation`, the kernel's in-block line,
//! elsarticle's `\address`, aa's `\institute`) and two controls whose marks still part the line (one opening its line).

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

/// Each row clean, every creator whole (pdflatex prints each line whole; the repro headers name the oracle):
/// - `${\rm ^{3}He}$` after a comma in REVTeX's marked `\affiliation` (2105.07958's prescript class; review 88c r1 q24),
///   and `${}^{87}{\rm Rb}$`, a script on an empty base: Baker's line whole;
/// - the same isotope in a kernel `\author`'s in-block address line, in elsarticle's `\address`, in aa's `\institute`;
/// - `Lab $10^{3}$ Unit`, an exponent on a number (2203.01892's `OR $97403^{3}$`): Baker's line whole;
/// - a script opening its line is its label, letters after it in its formula or not (`${\rm ^1Dept}$ of …`, 1006.5052);
/// - the control: `$^{a}$Univ X, $^{b}$ Univ Y` part at both marks, glued or spaced.
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
  ]);
}
