//! The text rules on printed ink and the token primitives of `author_block/preprocess.rs`, moved unchanged (the plan's row
//! "Knowledge in, driver out, one pass", user ruling 2026-11-08).

use super::*;

/// The note commands a mark's operand may carry (`Ann\textsuperscript{1,2\thanks{Work done at X.}}`): each prints
/// its own mark inside the superscript, after the labels, and its text as a footnote (latex.ltx:17215 `\thanks`,
/// :17649 `\footnote`, :17677 `\footnotemark`).
const MARK_NOTE_COMMANDS: [&str; 3] = ["\\thanks", "\\footnote", "\\footnotemark"];

/// The index after the note command at `i` ([`MARK_NOTE_COMMANDS`]): its `[<n>]`, and the braced text of a `\thanks`
/// or `\footnote` (`\footnotemark` has none).
fn note_command_end(tokens: &[Token], i: usize) -> usize {
  let skip_spaces = |mut k: usize| {
    while tokens.get(k) == Some(&T_SPACE!()) {
      k += 1;
    }
    k
  };
  let mut k = skip_spaces(i + 1);
  if tokens.get(k) == Some(&T_OTHER!("[")) {
    let mut depth = 0i32;
    while k < tokens.len() {
      match tokens[k].get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => depth -= 1,
        _ if depth == 0 && tokens[k] == T_OTHER!("]") => break,
        _ => {},
      }
      k += 1;
    }
    k = (k + 1).min(tokens.len());
  }
  if tokens[i] == T_CS!("\\footnotemark") {
    return k;
  }
  let text = skip_spaces(k);
  if text < tokens.len() {
    skip_group(tokens, text).min(tokens.len())
  } else {
    k
  }
}

/// A mark's operand (`1,\rm 2\thanks{…}`, `\mathbf{1,\thanks{…}}`) parted into the labels it prints and the notes it
/// holds, at any depth, with their arguments ([`MARK_NOTE_COMMANDS`]), or a macro that makes one, with its `[…]`
/// arguments ([`makes_a_note`]: the AAAI stub `\equalcontrib` in `\textsuperscript{\rm 1,2,\equalcontrib}`, 2603.17529;
/// a document's `\newcommand*\samethanks[1][\value{footnote}]{\footnotemark[#1]}` in `\textsuperscript{1\samethanks}`,
/// 2309.12862); `None` when it holds none.
pub(in crate::base_utilities) fn split_mark_notes(
  operand: &[Token],
) -> Option<(Vec<Token>, Vec<Token>)> {
  let is_note = |t: &Token| MARK_NOTE_COMMANDS.iter().any(|cs| *t == T_CS!(*cs));
  // (a macro that prints text as well as a note, `\newcommand\fnA{1\footnotemark}`, is read as TeX reads it, its body
  // in its place: the "1" a label, the note the author's; pdflatex "Ann Alpha1∗")
  let expanded;
  let operand = if operand.iter().any(|t| prints_beside_a_note(t, 0)) {
    expanded = note_macros_expanded(operand, 0);
    expanded.as_slice()
  } else {
    operand
  };
  if !operand.iter().any(|t| is_note(t) || makes_a_note(t, 0)) {
    return None;
  }
  let (mut mark, mut notes) = (Vec::new(), Vec::new());
  let mut i = 0;
  while i < operand.len() {
    // (a `\protect` right before a note command is the note's, `\textsuperscript{1\protect\thanks{…}}`: left
    // behind, the label read `1\protect`, which no list answers; pdflatex "Ann Alpha1∗")
    let at = if operand[i] == T_CS!("\\protect")
      && operand
        .get(i + 1)
        .is_some_and(|t| is_note(t) || makes_a_note(t, 0))
    {
      i + 1
    } else {
      i
    };
    let end = if is_note(&operand[at]) {
      note_command_end(operand, at)
    } else if makes_a_note(&operand[at], 0) {
      optional_arguments_end(operand, at + 1)
    } else {
      mark.push(operand[i]);
      i += 1;
      continue;
    };
    notes.extend_from_slice(&operand[i..end]);
    i = end;
  }
  Some((mark, notes))
}

/// The index after the `[…]` arguments from `k` (spaces before each skipped).
pub(in crate::base_utilities) fn optional_arguments_end(tokens: &[Token], mut k: usize) -> usize {
  loop {
    let open = k
      + tokens[k.min(tokens.len())..]
        .iter()
        .take_while(|t| **t == T_SPACE!())
        .count();
    if tokens.get(open) != Some(&T_OTHER!("[")) {
      return k;
    }
    let mut depth = 0i32;
    let mut close = open;
    while close < tokens.len() {
      match tokens[close].get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => depth -= 1,
        _ if depth == 0 && tokens[close] == T_OTHER!("]") => break,
        _ => {},
      }
      close += 1;
    }
    if close >= tokens.len() {
      return k;
    }
    k = close + 1;
  }
}

/// The body of `t` when it is a macro of optional arguments only (none, or `\newcommand`'s `[<default>]`; none at all
/// when `parameterless`), not a conditional, and is not a note command itself ([`MARK_NOTE_COMMANDS`]).
fn note_macro_body(t: &Token, parameterless: bool) -> Option<Tokens> {
  if t.get_catcode() != Catcode::CS || MARK_NOTE_COMMANDS.iter().any(|cs| *t == T_CS!(*cs)) {
    return None;
  }
  let defn = lookup_definition(t).ok()??;
  if !defn.is_expandable()
    || lookup_conditional(t).is_some()
    || defn.get_parameters().is_some_and(|p| {
      p.get_parameters()
        .iter()
        .any(|param| parameterless || !param.optional)
    })
  {
    return None;
  }
  match defn.get_expansion() {
    Some(ExpansionBody::Tokens(body)) => Some(body.clone()),
    _ => None,
  }
}

/// Whether `t` is a macro of optional arguments only ([`note_macro_body`]) whose expansion holds a note command
/// ([`MARK_NOTE_COMMANDS`]) anywhere, directly or through another such macro (three deep at most): it is taken out of
/// the mark whole, unless it is one read with its body in place ([`prints_beside_a_note`]).
fn makes_a_note(t: &Token, depth: usize) -> bool {
  depth <= 2
    && note_macro_body(t, false).is_some_and(|body| {
      body
        .unlist_ref()
        .iter()
        .any(|b| MARK_NOTE_COMMANDS.iter().any(|cs| *b == T_CS!(*cs)) || makes_a_note(b, depth + 1))
    })
}

/// Whether `t` is a macro of no arguments whose expansion, read in the mark in its place, prints a label beside a
/// note (`\newcommand\fnA{1\footnotemark}`): no conditional in it, and, its notes taken out (note commands with
/// their arguments, a macro that [`makes_a_note`]), only label-safe tokens — characters, font switches
/// ([`FONT_SWITCH`]), spaces, a `\protect` right before a note command (the note's, as [`split_mark_notes`] reads it,
/// `{7\protect\thanks{…}}`) — or another such macro (three deep at most), with at least one character. Any other macro
/// holding a note (`{\thanks{…}\unskip}`, `{\ifshow\footnotemark\fi}`, one of optional arguments
/// `[1][x]{6\footnotemark}`, and one with a `\relax` or another `\protect` beside its label, `{\relax 5\footnotemark}`,
/// `{\protect 5\footnotemark}`, a `\DeclareRobustCommand`'s `\protect\fnS␣`, which the label would keep and no list
/// answers: review rev78v2 finding 1) is taken out whole, as [`makes_a_note`] reads it: what its body does beside the
/// note is TeX's to do, not a label's.
fn prints_beside_a_note(t: &Token, depth: usize) -> bool {
  if depth > 2 {
    return false;
  }
  let Some(body) = note_macro_body(t, true) else {
    return false;
  };
  let body = body.unlist_ref();
  if body.iter().any(|b| lookup_conditional(b).is_some()) {
    return false;
  }
  let (mut notes, mut prints) = (false, false);
  let mut i = 0;
  while i < body.len() {
    let b = &body[i];
    if MARK_NOTE_COMMANDS.iter().any(|cs| *b == T_CS!(*cs)) {
      i = note_command_end(body, i);
      notes = true;
      continue;
    }
    if matches!(b.get_catcode(), Catcode::LETTER | Catcode::OTHER) {
      prints = true;
    } else if prints_beside_a_note(b, depth + 1) {
      (notes, prints) = (true, true);
    } else if makes_a_note(b, depth + 1) {
      notes = true;
      i = optional_arguments_end(body, i + 1);
      continue;
    } else if !(*b == T_SPACE!()
      || (*b == T_CS!("\\protect")
        && body
          .get(i + 1)
          .is_some_and(|n| MARK_NOTE_COMMANDS.iter().any(|cs| *n == T_CS!(*cs))))
      || (b.get_catcode() == Catcode::CS && b.with_str(|name| FONT_SWITCH.is_match(name))))
    {
      return false;
    }
    i += 1;
  }
  notes && prints
}

/// `operand` with each macro that prints text beside a note ([`prints_beside_a_note`]) replaced by its body, the
/// bodies read the same way (three deep at most). One after a `\protect` (`\textsuperscript{\protect\fnX}`) stays,
/// to be taken out with it, the note's ([`split_mark_notes`]): replaced, its label would read `\protect 2`, which no
/// list answers (review rev78v2 finding 1).
fn note_macros_expanded(operand: &[Token], depth: usize) -> Vec<Token> {
  let mut out = Vec::with_capacity(operand.len());
  for (k, t) in operand.iter().enumerate() {
    match note_macro_body(t, true) {
      Some(body)
        if prints_beside_a_note(t, depth) && (k == 0 || operand[k - 1] != T_CS!("\\protect")) =>
      {
        out.extend(note_macros_expanded(body.unlist_ref(), depth + 1));
      },
      _ => out.push(*t),
    }
  }
  out
}

/// After a `\begin`: the length of `{name} ... \end{name}` at the front of `stream`, through the matching `\end{name}`
/// (nested environments of the same name counted), or None when it does not close.
pub(in crate::base_utilities) fn environment_closes_ahead(
  stream: &VecDeque<Token>,
) -> Option<usize> {
  let name_at = |i: usize| -> Option<(String, usize)> {
    if stream.get(i)?.get_catcode() != Catcode::BEGIN {
      return None;
    }
    let close = (i + 1..stream.len()).find(|&k| stream[k].get_catcode() == Catcode::END)?;
    let name: String = (i + 1..close).map(|k| stream[k].to_string()).collect();
    Some((name, close + 1))
  };
  let (name, mut i) = name_at(0)?;
  let mut depth = 1;
  while i < stream.len() {
    let t = stream[i];
    if (t == T_CS!("\\begin") || t == T_CS!("\\end"))
      && let Some((other, next)) = name_at(i + 1)
    {
      if other == name {
        depth += if t == T_CS!("\\begin") { 1 } else { -1 };
        if depth == 0 {
          return Some(next);
        }
      }
      i = next;
    } else {
      i += 1;
    }
  }
  None
}

/// If `tokens` — ignoring leading/trailing spaces — is exactly a single control
/// sequence applied to one brace group that spans to the end (`\cmd{ … }`),
/// return `(\cmd, inner)`. Used to see through a whole-line font wrapper when
/// splitting an author line.
pub(in crate::base_utilities) fn whole_line_cs_wrapper(tokens: &Tokens) -> Option<(Token, Tokens)> {
  let v = tokens.unlist_ref();
  let mut start = 0;
  let mut end = v.len();
  while start < end && v[start] == T_SPACE!() {
    start += 1;
  }
  while end > start && v[end - 1] == T_SPACE!() {
    end -= 1;
  }
  let s = &v[start..end];
  // Need at least `\cmd { }` (the group may be empty, but then there is nothing
  // to split, so the caller's `len > 1` guard makes it a no-op anyway).
  if s.len() < 3 || s[0].get_catcode() != Catcode::CS || s[1].get_catcode() != Catcode::BEGIN {
    return None;
  }
  let mut level = 0;
  for (i, t) in s.iter().enumerate().skip(1) {
    match t.get_catcode() {
      Catcode::BEGIN => level += 1,
      Catcode::END => {
        level -= 1;
        if level == 0 {
          // The group must close exactly on the last token for this to be a
          // whole-line wrapper (no trailing content after `}`).
          return (i == s.len() - 1).then(|| (s[0], Tokens::new(s[2..i].to_vec())));
        }
      },
      _ => {},
    }
  }
  None
}
