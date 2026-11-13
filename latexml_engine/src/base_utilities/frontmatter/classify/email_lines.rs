//! The text rules on printed ink and the token primitives of `author_block/email_lines.rs`, moved unchanged (the plan's row
//! "Knowledge in, driver out, one pass", user ruling 2026-11-08).

use super::*;
use crate::base_utilities::frontmatter::printed_ink;

/// Whether `cmd` is an email command (`\email`, `\mailto`, `\emailaddr`): the contact `\lx@add@email` stands for.
pub(in crate::base_utilities) fn is_email_wrapper(cmd: &Token) -> bool {
  cmd.with_str(|name| name.to_lowercase().contains("mail"))
}

/// A class's numbered store (`class_numbered_institutes`) with each line of addresses wrapped in an e-mail command that
/// prints nothing in place unwrapped (`unprinted_email_unwrapped`), its lines as the entry cutter splits them;
/// `stuff` itself when none is. The addresses its entries' Kind step takes out (OD #956); cjaa.cls, astro-ph0508574.
pub fn store_unprinted_emails_unwrapped(stuff: Tokens) -> Result<Tokens> {
  let lines = split_tokens_delimited(
    strip_linebreak_options(stuff.clone()),
    author_affil_splits(),
  );
  let mut out: Vec<Token> = Vec::new();
  let mut unwrapped = false;
  for (delimiter, line) in lines {
    let kept = unprinted_email_unwrapped(line.clone())?;
    unwrapped |= kept != line;
    out.extend(delimiter);
    out.extend(kept.unlist());
  }
  Ok(if unwrapped { Tokens::new(out) } else { stuff })
}

/// `line` without its e-mail command where that command, digested with the meaning TeX runs (a raw file's rerouted
/// setter's own, as `\lx@class@contact@dispatch` reads it, OD #936), prints nothing in place: cjaa.cls's `\email` sets
/// its addresses in `\@thanks` (astro-ph0508574, 0706.3426; 0911.3853), a note sets them at the page's foot. Else `line`:
/// a print in place keeps its words (`Contact\footnote{#1}` its "Contact"), a binding's call makes its contacts where it
/// stands, an unread print stays.
fn unprinted_email_unwrapped(line: Tokens) -> Result<Tokens> {
  let Some((cmd, inner)) = whole_line_cs_wrapper(&line)
    .filter(|(cmd, inner)| is_email_wrapper(cmd) && email_addresses(inner).is_some())
  else {
    return Ok(line);
  };
  let name = cmd.with_str(|s| s.trim_start_matches('\\').to_string());
  let rerouted = T_CS!(&s!("\\lx@class@contact@rerouted@{name}"));
  let meaning = if lookup_meaning(&rerouted).is_some() && x_equals(&cmd, &rerouted) {
    T_CS!(&s!("\\lx@class@contact@{name}"))
  } else {
    cmd
  };
  if lookup_definition(&meaning)?.is_none_or(|d| d.get_origin().is_latexml_owned()) {
    return Ok(line);
  }
  let mut print = vec![meaning, T_BEGIN!()];
  print.extend(inner.unlist_ref().iter().copied());
  print.push(T_END!());
  let in_place = printed_ink(&Tokens::new(print))?
    .is_none_or(|ink| ink.outside_notes.is_none_or(|text| !text.trim().is_empty()));
  Ok(if in_place { line } else { inner })
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
