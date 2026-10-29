//! Guards for batch 90a (frontmatter refactor stage 1, `docs/parity/FRONTMATTER_DIGESTED_REFACTOR.md`;
//! OXIDIZED_DESIGN_DIVERGENCES #837, #911): whether a formula in a name or an affiliation is a mark is read from its
//! digested XMath (`observer::marks::read_formula`), the one reading the observer's writer, `ab_audit fmshadow`,
//! `relocate::shown_marks` and `name_text::is_name_mark` share — never from its `tex`. A formula printing nothing but
//! floating scripts (spacing aside, punctuation after) is a mark; a script with a letter or digit after it, or on a
//! printed base, is the item's text. Each guard converts its repros in the shipping Default mode, asserts every creator
//! whole (name and contacts, in order), no Error or Warning, and the writer's outcome. Every repro compiles clean with
//! pdflatex (TL 2025). The reader's shapes as digested and as parsed: `observer::marks::tests`, and
//! `ab_audit fmshadow`'s `a_parsed_prescript_on_a_word_is_text_a_script_on_a_base_the_base`.
use latexml::util::test::{convert_with_then, error_count, warning_count};
use latexml_engine::base_utilities::fm_shadow_json;
use serde_json::Value;

use super::perfect_kernel_batch84b::creators;

/// The repro's conversion in the Default mode, clean: its writer's outcome and its creators.
#[track_caller]
fn converted(tex: &str) -> (String, Vec<Vec<String>>) {
  let (log, xml, json) = convert_with_then(tex, Some("ar5iv.sty"), |_| fm_shadow_json());
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let json: Value = serde_json::from_str(&json).expect("the shadow JSON");
  (
    json["writer"].as_str().unwrap_or("none").to_string(),
    creators(&xml),
  )
}

fn owned(expected: &[&[&str]]) -> Vec<Vec<String>> {
  expected
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect()
}

/// A formula printing nothing but a floating script is a mark, whatever spells it: a font switch before the script
/// (2105.07958's `\address{$\rm ^a$Email: …}`; the names' `$\rm ^{2,a}$` and a group that is the whole formula,
/// `${\rm ^a}$`) is a token's font, spacing after it (`$^{1}\hspace{1mm}$`, review 88c r4 h2) an `XMHint`. So the
/// writer's view keys the item without its mark and writes what the reading gives. Red (int118a): the `tex` reading
/// kept the switched "a" in the key and the writer declined, the address every author's; the h2 row is green on
/// int118a and pins the hint (88c's `tex` reading counted `\hspace{1mm}`'s digits as printed and declined it). Baker's
/// contacts in today's order, the e-mail address first: a residual, not TeX's (RED repro
/// `revtex_marked_address_contacts_in_print_order`).
#[test]
fn a_formula_printing_only_floating_scripts_is_a_mark() {
  let switched = owned(&[&["Ann Able 1", "affiliation=Univ X"], &[
    "Bob Baker 2,a",
    "affiliation=a Email: bob.baker@y.org",
    "affiliation=Univ Y",
  ]]);
  for (tex, want) in [
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_font_switched_address_mark_keys_its_entry.tex"
      ),
      &switched,
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_font_switched_marks_on_names_and_braced.tex"
      ),
      &switched,
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/amsart_address_mark_spaced_after_its_script_is_a_mark.tex"
      ),
      &owned(&[&["Ann Able 1", "address=1 Univ X"], &[
        "Bob Baker 2",
        "address=2 Univ Y",
      ]]),
    ),
  ] {
    assert_eq!(
      converted(tex),
      ("written".to_string(), want.clone()),
      "{tex}"
    );
  }
}

/// A script with a letter or digit printed after it in its formula is no mark: the isotope `${\rm ^{3}He}$` (review
/// 88c r1 q24; digested a floating script then "H", "e"), its text the line's, and the line "Univ Y, ³He Lab" stays
/// whole, Baker's (the writer declines its cut). A script on a printed base is no mark either: `OR $97403^{3}$`
/// (2203.01892's shape) keys as "OR 97403", as the observer's lexer reads it, and the writer's outcome is `same`.
/// Both green on int118a: they pin the fail-safe of the structural reading (a `tex` reading past the switch made the
/// isotope a mark, 88c r1, and wrote "He 3 Lab" to a nameless creator with a Warning). The pinned affiliation strings
/// are today's XML text content (each XMath token's text, the parser's invisible times included), not the print.
#[test]
fn a_script_before_printed_ink_or_on_a_base_is_text() {
  let (writer, people) = converted(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_affiliation_isotope_prescript_keeps_its_line.tex"
  ));
  assert!(writer.starts_with("declined: Affiliation "), "{writer}");
  assert_eq!(
    people,
    owned(&[&["Ann Able 1", "affiliation=Univ X"], &[
      "Bob Baker 2",
      "affiliation=Univ Y, \u{2062} 3 He Lab"
    ]])
  );
  assert_eq!(
    converted(include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_affiliation_number_base_superscript_is_its_text.tex"
    )),
    (
      "same".to_string(),
      owned(&[&["Ann Able 1", "affiliation=Univ X"], &[
        "Bob Baker 2",
        "affiliation=Univ Y, OR \u{2062} 97403 3"
      ]])
    )
  );
}

/// The marks a name shows (`relocate::shown_marks`: each marked affiliation answered by the authors showing its mark,
/// today's path, before the writer) are read from the formula's XMath: `Bob Baker${^2}$` (2002.12524's `Po-Hao
/// Chou${^2}$`, 2006.00536's `Radoslaw Ryblewski${^1}$`: a group that is the whole formula) and `Cid Cole$\rm ^{1}$`
/// (2105.07958's switch) show their marks and answer their affiliations. Red (int118a): their `tex` opened with no `^`,
/// they showed none, Bob and Cid had no affiliation and "Univ Y" was a nameless creator's.
#[test]
fn a_formula_mark_on_a_name_answers_its_affiliation() {
  assert_eq!(
    converted(include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_braced_and_switched_name_marks_answer_their_affiliations.tex"
    )),
    (
      "same".to_string(),
      owned(&[
        &["Ann Able 1", "affiliation=Univ X"],
        &["Bob Baker 2", "affiliation=Univ Y"],
        &["Cid Cole 1", "affiliation=Univ X"],
      ])
    )
  );
}

/// A script inside a two-dimensional layout is no script the formula ends on: the formula is one printed item, its
/// text kept (`observer::marks::row_layout`, by the operator's role and meaning as Perl `Post/MathML.pm` lays it out:
/// `mfrac`, `msqrt`, `mroot`, `mover`, `menclose`). So `$\frac{a}{b^{1}}$` ending a line's word (review 90a r3 probe
/// AFg10) keys "Lab a b 1 Lab" whole, as the observer's lexer does, and the address is cut at its marks; and so is
/// `$\boxed{x^{1}}$` (probe AFg02), digested an `XMArg[@enclose]`, the enclosure's own code path. Red (90a r3): the
/// reader descended into the fraction (r2 too into the box), read the script on a base "ab" (`OnBase`), the writer
/// dropped the "1" from the key and declined, Able had no contact and Baker the merged address. Green on int118a.
#[test]
fn a_script_inside_a_two_dimensional_layout_ends_no_line() {
  for (tex, want) in [
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/amsart_address_fraction_script_ends_no_line.tex"
      ),
      owned(&[&["Ann Able 1", "address=1 Lab a b 1 Lab"], &[
        "Bob Baker 2",
        "address=2 Lbb a b 1 Lbb",
      ]]),
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/amsart_address_boxed_script_ends_no_line.tex"
      ),
      owned(&[&["Ann Able 1", "address=1 Lab x 1"], &[
        "Bob Baker 2",
        "address=2 Lbb x 1",
      ]]),
    ),
  ] {
    assert_eq!(converted(tex), ("written".to_string(), want), "{tex}");
  }
}
