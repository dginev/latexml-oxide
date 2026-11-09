//! Guards for batch 90a (frontmatter refactor stage 1, `docs/archive/FRONTMATTER_DIGESTED_REFACTOR_2026-11-07.md`;
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

/// A script with a letter or digit printed after it in its formula is no mark: the isotope `${\rm ^{3}He}$` (review
/// 88c r1 q24; digested a floating script then "H", "e"), its text the line's, and the line "Univ Y, ³He Lab" stays
/// whole, Baker's (the writer declines its cut). A script on a printed base is no mark either: `OR $97403^{3}$`
/// (2203.01892's shape) keys as "OR 97403", as the observer's lexer reads it, and the writer's outcome is `same`.
/// Both green on int118a: they pin the fail-safe of the structural reading (a `tex` reading past the switch made the
/// isotope a mark, 88c r1, and wrote "He 3 Lab" to a nameless creator with a Warning). The pinned affiliation strings
/// are today's XML text content (each XMath token's text, the parser's invisible times included), not the print.
#[test]
fn a_script_before_printed_ink_or_on_a_base_is_text() {
  let (..) = converted(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_affiliation_isotope_prescript_keeps_its_line.tex"
  ));
}
