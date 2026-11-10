//! The text rules on printed ink and the token primitives of `author_block/marks.rs`, moved unchanged (the plan's row
//! "Knowledge in, driver out, one pass", user ruling 2026-11-08).

use super::*;
use crate::frontmatter_stores::cs_in;

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

/// The 1-based position of the first author mark in `tokens`, as `position_of` (Perl's `positionOf`) counts: a superscript, `\textsuperscript`
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

/// How many distinct marks (by operand) `stuff` shows outside its notes. Two or more make an address a key the names'
/// marks index (aistats' marked address over a marked row, 2312.06786, 1901.02427); a single mark is a note's
/// (`$^*$equal contribution`, `Univ A$^*$` twice), and the address still pairs by column (77o r3 review: 1905.11190,
/// 2405.18281, 2011.03885, 2301.11862, 2410.15516; probes m_lead, m_star).
pub fn distinct_author_marks(stuff: &Tokens) -> usize {
  let mut seen: Vec<String> = Vec::new();
  for operand in author_mark_operands(stuff.unlist_ref()) {
    let key = Tokens::new(operand).to_string();
    if !seen.contains(&key) {
      seen.push(key);
    }
  }
  seen.len()
}

// (moved unchanged from `author_block/marks.rs` by Z18m, which deleted `author_block/`: Author marks: answered marks,
// mark commands, `has_author_marks`, label marks.)
/// Does a superscript's operand read as mark labels (`1`, `1,2`, `1, 3`, `a`, `\dagger`, `\mathrm{c}`, `*`), not as
/// text shown raised (`\textsuperscript{equal contrib.}`, `\textsuperscript{(*)}`, `$^{\rm (a)}$`,
/// `\textsuperscript{ (equal)}`)? Labels hold no word of three or more letters, and [`clean_frontmatter_labels`] gives
/// each back as it is (its font switches and braces aside): it drops parentheses and writes a space between the
/// letters of one as `_`, so a mark read from such text loses what it shows (77n r4 review probes d1, d3, d4, c4, d6).
fn reads_as_labels(operand: &[Token]) -> bool {
  let mut word = 0;
  for t in operand {
    word = if t.get_catcode() == Catcode::LETTER {
      word + 1
    } else {
      0
    };
    if word >= 3 {
      return false;
    }
  }
  let text = Tokens::new(
    operand
      .iter()
      .filter(|t| {
        !matches!(t.get_catcode(), Catcode::BEGIN | Catcode::END)
          && !(t.get_catcode() == Catcode::CS && t.with_str(|name| FONT_SWITCH.is_match(name)))
      })
      .copied()
      .collect(),
  )
  .to_string();
  let pieces: Vec<&str> = text
    .split(',')
    .map(str::trim)
    .filter(|piece| !piece.is_empty())
    .collect();
  clean_frontmatter_labels(&text, "") == pieces
}

/// Do an author's marks end what it prints (`Ann Able$^{1}$`, `Ann$^{1}$\thanks{…}`): after the first of them nothing
/// but further marks, spacing, braces and math shifts (`${}^{1}$`), and notes with their arguments — `\thanks`,
/// `\footnote`, the note commands of `is_name_note_command`, a document's note macro, and the binding's `contacts`,
/// whose argument goes to the frontmatter (authblk's `\affil` inside `\author`, 2605.03796)? Not when a line or
/// anything visible follows them: `Ann Able$^{1}$ \\ Univ X`, `Ann Able$^{a}$ (ann@x.org)`, `Ann Able$^{1}$
/// \orcidlink{…}` (77n r3 review probes m2, m9, m8). A superscript is a mark only when its operand reads as labels
/// (`reads_as_labels`): `Ann Able$^{1}$\textsuperscript{equal contrib.}`, `\textsuperscript{(*)}`, `$^{\rm (a)}$` show
/// text, which the list path would have cut to a label or dropped (77n r4 review probes d1, d3, d4, c4, d6). Public for
/// authblk's `\author` (latexml_package), which reads a single marked name as a list only then: the list path shows a
/// mark nothing answers after the name's last text. (A line break reaches that path by its own,
/// `has_author_block_separator`, where a mark before it stays, 108a.)
pub fn author_marks_end_name(stuff: &Tokens, contacts: &[Token]) -> bool {
  let tokens = outside_notes(stuff.unlist_ref());
  // (the index past a mark at `i`: `$^{1}$`, `^{1}`, `\textsuperscript{1}` and the mark commands)
  let mark_end = |i: usize| -> Option<usize> {
    let t = tokens[i];
    if t == T_MATH!() && tokens.get(i + 1) == Some(&T_SUPER!()) {
      sup_operand_at(&tokens, i + 2)
        .filter(|(operand, next)| tokens.get(*next) == Some(&T_MATH!()) && reads_as_labels(operand))
        .map(|(_, next)| next + 1)
    } else if t == T_SUPER!() || is_mark_command(&t) {
      sup_operand_at(&tokens, i + 1)
        .filter(|(operand, _)| reads_as_labels(operand))
        .map(|(_, next)| next)
    } else {
      None
    }
  };
  let mut marked = false;
  let mut i = 0;
  while i < tokens.len() {
    let t = tokens[i];
    if let Some(next) = mark_end(i) {
      marked = true;
      i = next;
    } else if !marked
      || is_spacing(&t)
      || matches!(
        t.get_catcode(),
        Catcode::BEGIN | Catcode::END | Catcode::MATH
      )
    {
      // (the name before its first mark; after it, spacing, braces and math shifts)
      i += 1;
    } else if t.with_str(is_name_note_command) || is_note_macro(&t) || contacts.contains(&t) {
      // (its optional argument and its arguments)
      i += 1;
      if tokens.get(i) == Some(&T_OTHER!("["))
        && let Some(close) = tokens[i..].iter().position(|t| *t == T_OTHER!("]"))
      {
        i += close + 1;
      }
      while let Some(t) = tokens.get(i) {
        match t.get_catcode() {
          Catcode::SPACE => i += 1,
          Catcode::BEGIN => i = skip_group(&tokens, i),
          _ => break,
        }
      }
    } else {
      return false;
    }
  }
  marked
}

/// The marks an author's name shows that read as labels (`reads_as_labels`), outside its notes and not inside a word
/// (`rewrite_in_word_superscripts`: `Craig M$^{\mathrm{c}}$Rae` spells the name): what the name requests, kept as it
/// prints. For authblk's `\author` kept whole on `\lx@add@creator` because text follows its marks (`Bob
/// Baker$^{1}$\textsuperscript{equal contrib.}`, `Bob Baker$^{1}$ (bob@x.org)`), which still requests the affiliation
/// its mark picks, as a marked name read as a list does — it requested nothing, and lost `$^1$Univ X` to the author
/// before it (77n r5 review probes k1, k2, k5). The binding's `contacts` (authblk's `\affil` inside `\author`) are left
/// out with their arguments: their marks label them. Only the labels are requested (`label_pieces`): a footnote symbol
/// (`$^{*}$`, `$^{1,\dagger}$`'s dagger, the `\ast` glued to `1\ast`) is the name's own text, which a request nothing
/// answered showed a second time after it (`Bob Baker$^{*}$ (bob@x.org)` printed "Bob Baker∗ (bob@x.org)∗"; 2605.10201's
/// `$^{1\ast}$`, `$^{1\dagger}$` gained "∗,†" in its name; 77n r6 review probes s1, s3, s5, z1).
pub fn author_label_marks(stuff: &Tokens, contacts: &[Token]) -> Vec<Tokens> {
  let tokens = rewrite_in_word_superscripts(stuff.clone());
  let v = tokens.unlist_ref();
  let mut name = Vec::with_capacity(v.len());
  let mut i = 0;
  while i < v.len() {
    if contacts.contains(&v[i]) {
      i += 1;
      if v.get(i) == Some(&T_OTHER!("["))
        && let Some(close) = v[i..].iter().position(|t| *t == T_OTHER!("]"))
      {
        i += close + 1;
      }
      while let Some(t) = v.get(i) {
        match t.get_catcode() {
          Catcode::SPACE => i += 1,
          Catcode::BEGIN => i = skip_group(v, i),
          _ => break,
        }
      }
    } else {
      name.push(v[i]);
      i += 1;
    }
  }
  author_mark_operands(&name)
    .into_iter()
    .filter(|operand| reads_as_labels(operand))
    .filter_map(|operand| label_pieces(&operand))
    .map(Tokens::new)
    .collect()
}

// (moved unchanged from `author_block/email_lines.rs` by Z18m, which deleted `author_block/`: Author-block empty marks
// (the e-mail line readers and row starts moved to `defs/frontmatter_api/email_lines.rs`, Z18j).)
/// `tokens` without the marks that print nothing: a mark command's empty argument (`\textsuperscript{}`, `mark_commands`)
/// and an empty math superscript (`$^{}$`), spaces and `\relax` aside, when the block holds no other mark. An empty mark is no mark
/// (user ruling R11 2026-10-20): it makes no block a marked one (2604.19440's `\textsuperscript{}Univ. Grenoble Alpes,
/// …` under four unmarked names, which pdflatex prints with no mark, an affiliation of all four). In a block other marks
/// make a marked one it stays, as before: it labels nothing there, and each name it ends is an author's piece of its
/// own (2604.22129's `Robert Maier\textsuperscript{}~~~Aljaz Bozic\textsuperscript{}` beside `David
/// Recasens\textsuperscript{\rm 1}`, which a marked block's marker-less pieces would weld into one creator). A note's
/// text is the note's, not the block's marks: what a `\thanks`, `\footnote` or `\footnotetext` (or a document's note
/// command, `is_note_macro`) holds is kept as written, so the note is read on what the author wrote — by the XML's
/// no-ink test as by the frontmatter observer's rehearsal of the same block (`\thanks{\textsuperscript{}}`: taken out,
/// the XML's note was empty and dropped where the observer's kept it; OXIDIZED_DESIGN_DIVERGENCES #916).
pub fn drop_empty_marks(tokens: Tokens) -> Tokens {
  let original = tokens.clone();
  let v = tokens.unlist();
  // (a `\relax` in it prints nothing either: `Ann Able\textsuperscript{\relax}, Bob Baker`, the r7 re-review's n10)
  let blank = |range: &[Token]| {
    range.iter().all(|t| {
      matches!(
        t.get_catcode(),
        Catcode::BEGIN | Catcode::END | Catcode::SPACE
      ) || *t == T_CS!("\\relax")
    })
  };
  let mut out = Vec::with_capacity(v.len());
  let mut i = 0;
  while i < v.len() {
    if cs_in(&v[i], NOTE_TEXT_COMMANDS) || is_note_macro(&v[i]) {
      // (the command, an optional `[…]`, and its braced text, as written)
      let mut end = i + 1 + v[i + 1..].iter().take_while(|t| **t == T_SPACE!()).count();
      if v.get(end) == Some(&T_OTHER!("[")) {
        end = v[end..]
          .iter()
          .position(|t| *t == T_OTHER!("]"))
          .map_or(v.len(), |at| end + at + 1);
        end += v[end..].iter().take_while(|t| **t == T_SPACE!()).count();
      }
      let close = skip_group(&v, end).max(i + 1).min(v.len());
      out.extend_from_slice(&v[i..close]);
      i = close;
      continue;
    }
    let (operand, after) = if mark_commands().contains(&v[i]) {
      let open = i + 1 + v[i + 1..].iter().take_while(|t| **t == T_SPACE!()).count();
      (open, 0)
    } else if v[i].get_catcode() == Catcode::MATH && v.get(i + 1) == Some(&T_SUPER!()) {
      (i + 2, 1)
    } else {
      out.push(v[i]);
      i += 1;
      continue;
    };
    let close = skip_group(&v, operand);
    if v
      .get(operand)
      .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
      && blank(&v[operand..close.min(v.len())])
      && (after == 0
        || v
          .get(close)
          .is_some_and(|t| t.get_catcode() == Catcode::MATH))
    {
      i = close + after;
    } else {
      out.push(v[i]);
      i += 1;
    }
  }
  let out = Tokens::new(out);
  if has_author_marks(&rewrite_symbol_superscripts(out.clone())) {
    original
  } else {
    out
  }
}

/// The note commands whose braced argument is a note's text ([`drop_empty_marks`]).
const NOTE_TEXT_COMMANDS: &[&str] = &["\\thanks", "\\footnote", "\\footnotetext"];
