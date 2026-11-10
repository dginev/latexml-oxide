//! Author-block empty marks (the e-mail line readers and row starts moved to `defs/frontmatter_api/email_lines.rs`, Z18j).

use super::*;
use crate::frontmatter_stores::cs_in;

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
