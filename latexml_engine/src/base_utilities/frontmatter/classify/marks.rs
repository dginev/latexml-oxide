//! The text rules on printed ink and the token primitives of `author_block/marks.rs`, moved unchanged (the plan's row
//! "Knowledge in, driver out, one pass", user ruling 2026-11-08).

use super::*;

/// Whether a line reads as an affiliation: no name, no address (`@`), and a word that names an institution
/// (`non_name_word`: `Univ A`, `Department of Physics`, `MIT`).
pub fn reads_as_affiliation(line: &Tokens) -> bool {
  let text = visible_name_text(line.unlist_ref());
  name_count(&text) == 0 && !text.contains('@') && text.split_whitespace().any(non_name_word)
}

/// The line breaks an author block's lines are split at: `\\` and `\newline` (latex.ltx:9253-9256).
pub(in crate::base_utilities) fn author_line_breaks() -> Vec<SplitDelim> {
  vec![T_CS!("\\\\").into(), T_CS!("\\newline").into()]
}

/// The commands that print a mark raised, its operand their argument: `\textsuperscript`, and IEEEtran's author
/// reference mark (`Ann Able\IEEEauthorrefmark{1}` over `\IEEEauthorblockA{\IEEEauthorrefmark{1}Univ A}`, IEEEtran.cls's
/// `\IEEEauthorrefmark` a superscript symbol; 2406.11437, 2104.02493). Every reader of an author block's marks counts
/// them alike: as labels, in the operands an address is placed by, and where a line splits into its affiliations
/// (`\IEEEauthorrefmark{2}Google, \IEEEauthorrefmark{3}Google DeepMind`, 2310.02368).
pub(in crate::base_utilities) fn mark_commands() -> [Token; 3] {
  [
    T_CS!("\\textsuperscript"),
    T_CS!("\\IEEEauthorrefmark"),
    T_CS!("\\authorrefmark"),
  ]
}

/// The mark command `t` means, by TeX's `\ifx` (`Token::defined_as`, which follows `\let`): one of [`mark_commands`]
/// itself, or an alias a document made of it (`\let\up\textsuperscript`, 2209.14855's macros/custom.tex:23, `Ann
/// Able\up{1}`). The one test every reader of an author block's marks makes: a literal comparison took the alias for
/// name text, so the block read as unmarked and its `\up{2}Univ B` line became an author (batch 77p, HF14 M2(b)).
pub(in crate::base_utilities) fn mark_command_meant(t: &Token) -> Option<Token> {
  if !matches!(t.get_catcode(), Catcode::CS | Catcode::ACTIVE) {
    return None;
  }
  mark_commands().into_iter().find(|mark| t.defined_as(mark))
}

/// Is `t` a mark command, or an alias of one ([`mark_command_meant`])?
pub(in crate::base_utilities) fn is_mark_command(t: &Token) -> bool {
  mark_command_meant(t).is_some()
}

/// Is `t` one of the marks an author line links its names and affiliations by: a superscript, or a mark command
/// ([`is_mark_command`])?
pub(in crate::base_utilities) fn is_author_mark(t: &Token) -> bool {
  *t == T_SUPER!() || is_mark_command(t)
}

/// The 1-based position of the first author mark in `tokens`, as [`position_of`] counts: a superscript, `\textsuperscript`
/// or an author reference mark (`\IEEEauthorrefmark`, `\authorrefmark`), or a `\let` alias of one
/// (`is_author_mark`).
pub fn author_mark_position(tokens: &Tokens) -> Option<usize> {
  tokens
    .unlist_ref()
    .iter()
    .position(is_author_mark)
    .map(|i| i + 1)
}

/// The name annotations ([`NAME_ANNOTATIONS`]) printed beside the name, not taken into a note: an address, an ORCID
/// link, a membership grade, a citation mark, a superscript (a mark or text). The one classification of those commands
/// ([`is_name_note_command`]): `author_marks_end_name` reads what follows a name's mark by it, the email split of a
/// name line (`adds_to_frontmatter`) the markup of an address (77n r4 review, which found the two lists apart).
pub(in crate::base_utilities) const PRINTED_IN_PLACE: [&str; 8] = [
  "\\email",
  "\\textsuperscript",
  "\\orcidlink",
  "\\orcidID",
  "\\lx@aas@checkorcid",
  "\\IEEEmembership",
  "\\lx@IEEE@membership",
  "\\cite",
];

/// A name annotation command ([`NAME_ANNOTATIONS`]) whose argument goes into a note, not printed beside the name: all
/// but [`PRINTED_IN_PLACE`] (`\thanks`, `\footnote`, `\inst`, `\altaffilmark`, `\corref`, ecai's `\institute`, …).
pub(in crate::base_utilities) fn is_name_note_command(name: &str) -> bool {
  NAME_ANNOTATIONS.contains(&name) && !PRINTED_IN_PLACE.contains(&name)
}

/// In a superscript-labeled author block, decide whether a line reads
/// "Name\textsuperscript{n}" (an author — name TEXT precedes the marker) or
/// "\textsuperscript{n}Affil" (an affiliation — the marker LEADS the line).
/// Returns true iff a letter token precedes the first marker at `marker_pos`
/// (1-based, from `position_of`). Replaces an earlier `position < 8`
/// token-count proxy that misread short author names: "Min Xu" is 7 tokens, so
/// its trailing `\textsuperscript{1}` fell under the threshold and the author
/// was reclassified as an affiliation (html_feedback#6614, arXiv:2606.08234).
/// Keying on "is there a name before the marker" is length-independent.
/// OXIDIZED_DESIGN #52.
pub(in crate::base_utilities) fn name_precedes_marker(line: &Tokens, marker_pos: usize) -> bool {
  line.unlist_ref()[..marker_pos.saturating_sub(1)]
    .iter()
    .any(|t| t.code == Catcode::LETTER)
}
