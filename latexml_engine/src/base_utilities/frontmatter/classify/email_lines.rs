//! The text rules on printed ink and the token primitives of `author_block/email_lines.rs`, moved unchanged (the plan's row
//! "Knowledge in, driver out, one pass", user ruling 2026-11-08).

use super::*;

/// Whether `cmd` is an email command (`\email`, `\mailto`, `\emailaddr`): the contact `\lx@add@email` stands for.
pub(in crate::base_utilities) fn is_email_wrapper(cmd: &Token) -> bool {
  cmd.with_str(|name| name.to_lowercase().contains("mail"))
}

/// `line` without the e-mail command it is wrapped in where the command is a raw file's or the document's (cjaa.cls's
/// `\email`, which sets its address in `\@thanks`, printed nowhere in the entry): the addresses it prints, which a
/// marked list's entry takes out by its Kind step (`kind_entry_addresses`, OD #956); a binding's makes its contacts in
/// place. astro-ph0508574, 0911.3853.
pub(in crate::base_utilities) fn raw_email_unwrapped(line: Tokens) -> Tokens {
  match whole_line_cs_wrapper(&line) {
    Some((cmd, inner))
      if is_email_wrapper(&cmd)
        && lookup_definition(&cmd)
          .ok()
          .flatten()
          .is_some_and(|d| !d.get_origin().is_latexml_owned()) =>
    {
      inner
    },
    _ => line,
  }
}

/// The end of `tokens` before the line break and spacing they end with, which print nothing: spaces, `\\`, `\hfill`,
/// `\smallskip`…, `\protect`, `\vspace{…}`, `\hspace*{…}` (2409.06765's `\email vye@berkeley.edu\\ \vspace{-0.6cm}`,
/// read as one line; 2609.08102's `\author[Mech]{\protect\\Johann Guilleminot}`); 0 for a line of only those.
pub(in crate::base_utilities) fn spacing_end(tokens: &[Token]) -> usize {
  let spacing = |t: &Token, names: &[&str]| {
    t.get_catcode() == Catcode::CS && t.with_str(|s| names.contains(&s))
  };
  let mut end = tokens.len();
  loop {
    while end > 0 && tokens[end - 1] == T_SPACE!() {
      end -= 1;
    }
    if end > 0
      && spacing(&tokens[end - 1], &[
        "\\\\",
        "\\hfill",
        "\\smallskip",
        "\\medskip",
        "\\bigskip",
        "\\newline",
        "\\protect",
      ])
    {
      end -= 1;
      continue;
    }
    if end > 0 && tokens[end - 1].get_catcode() == Catcode::END {
      let mut depth = 0usize;
      let mut open = None;
      for j in (0..end).rev() {
        match tokens[j].get_catcode() {
          Catcode::END => depth += 1,
          Catcode::BEGIN => {
            depth -= 1;
            if depth == 0 {
              open = Some(j);
              break;
            }
          },
          _ => {},
        }
      }
      if let Some(open) = open {
        let mut cs = open;
        if cs > 0 && tokens[cs - 1] == T_OTHER!("*") {
          cs -= 1;
        }
        if cs > 0 && spacing(&tokens[cs - 1], &["\\vspace", "\\hspace"]) {
          end = cs - 1;
          continue;
        }
      }
    }
    return end;
  }
}

/// The marks that request an affiliation label in an author (`\lx@author@withsup`, llncs `\inst`) and set it in an
/// affiliation (`\lx@affiliation@withsup`, `\lx@affiliation@withinst`): the author marks ([`is_author_mark`]), llncs's
/// `\inst` and the symbol mark.
pub(in crate::base_utilities) fn is_affiliation_mark(t: &Token) -> bool {
  is_author_mark(t) || *t == T_CS!("\\inst") || *t == T_CS!("\\lx@frontmatter@symbolmark")
}

/// The operand of each mark in `tokens`, in order, read as `read_frontmatter_sup_operand` reads it.
pub(in crate::base_utilities) fn mark_operands(tokens: &[Token]) -> Vec<Vec<Token>> {
  let mut operands = Vec::new();
  let mut i = 0;
  while i < tokens.len() {
    if is_affiliation_mark(&tokens[i])
      && let Some((operand, next)) = sup_operand_at(tokens, i + 1)
    {
      operands.push(operand);
      i = next;
    } else {
      i += 1;
    }
  }
  operands
}

/// An author's tokens without its `\thanks{...}` and `\footnote{...}` (and their optional label).
pub(in crate::base_utilities) fn outside_notes(tokens: &[Token]) -> Vec<Token> {
  let mut outside = Vec::with_capacity(tokens.len());
  let mut i = 0;
  while i < tokens.len() {
    if tokens[i] == T_CS!("\\thanks") || tokens[i] == T_CS!("\\footnote") {
      let mut j = i + 1;
      if tokens.get(j) == Some(&T_OTHER!("["))
        && let Some(close) = tokens[j..].iter().position(|t| *t == T_OTHER!("]"))
      {
        j += close + 1;
      }
      if let Some((_, next)) = sup_operand_at(tokens, j) {
        i = next;
        continue;
      }
    }
    outside.push(tokens[i]);
    i += 1;
  }
  outside
}

/// The operand of a mark at `tokens[i..]`, as `read_frontmatter_sup_operand` reads it from the input — a group's
/// content, a control sequence with the group after it, else one token — and the index after it.
pub(in crate::base_utilities) fn sup_operand_at(
  tokens: &[Token],
  mut i: usize,
) -> Option<(Vec<Token>, usize)> {
  while tokens.get(i) == Some(&T_SPACE!()) {
    i += 1;
  }
  let group_end = |start: usize| {
    let mut depth = 0usize;
    for (j, t) in tokens.iter().enumerate().skip(start) {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth = depth.saturating_sub(1);
          if depth == 0 {
            return Some(j);
          }
        },
        _ => {},
      }
    }
    None
  };
  let first = *tokens.get(i)?;
  if first.get_catcode() == Catcode::BEGIN {
    let end = group_end(i)?;
    return Some((tokens[i + 1..end].to_vec(), end + 1));
  }
  if first.get_catcode() == Catcode::CS
    && tokens
      .get(i + 1)
      .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
  {
    let end = group_end(i + 1)?;
    return Some((tokens[i..=end].to_vec(), end + 1));
  }
  Some((vec![first], i + 1))
}
