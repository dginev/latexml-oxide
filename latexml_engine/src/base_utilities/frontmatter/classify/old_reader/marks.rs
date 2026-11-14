//! The mark readers authblk's `\author` reads a name by (`authblk_sty.rs`): whether its marks end what it prints
//! (`author_marks_end_name`), the labels they request (`author_label_marks`), and a block without the marks that print
//! nothing (`drop_empty_marks`, which the API's entry cutter and the IJCAI chain call too). Moved unchanged by Z18m, which deleted `author_block/`: the first two, with
//! `reads_as_labels`, from `author_block/marks.rs`; `drop_empty_marks`, with `NOTE_TEXT_COMMANDS`, from
//! `author_block/email_lines.rs` (whose e-mail line readers and row starts Z18j moved to
//! `defs/frontmatter_api/email_lines.rs`).

use super::*;
use crate::frontmatter_stores::cs_in;

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
