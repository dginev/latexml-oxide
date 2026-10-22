//! Author-block preprocessing: math-active scripts and environments read as commands.

use super::*;

/// An author block's math-active `^` and `_` (catcode 12, mathcode "8000: mathstyle.sty, which breqn loads through
/// flexisym.sty:418, sets them so — :233-236 its default `mathactivechars` option, `\AtBeginDocument{\catcode`\^=12
/// …}`, and :251-258 `\mathcode`\^="8000` with the active `^` `\let` to `\sp`; latexml_contrib's breqn_sty.rs:138-163) written as the
/// superscript and subscript they are in math, where TeX reads the active character's meaning, `\sp`/`\sb` (tex.web
/// §1151): the marks of `{Mohamed Benzaghta$^{\star}$, …, David
/// López-Pérez}$^{\sharp}$` (2504.02633) are marks, as they are with `^` of catcode 7.
pub(in crate::base_utilities) fn math_active_scripts(stuff: Tokens) -> Tokens {
  let v = stuff.unlist_ref();
  let active = |c: &str| lookup_mathcode(c) == Some(0x8000);
  let (sup, sub) = (
    v.contains(&T_OTHER!("^")) && active("^"),
    v.contains(&T_OTHER!("_")) && active("_"),
  );
  if !(sup || sub) {
    return stuff;
  }
  let mut in_math = false;
  Tokens::new(
    v.iter()
      .map(|t| {
        if t.get_catcode() == Catcode::MATH {
          in_math = !in_math;
        }
        if in_math && sup && *t == T_OTHER!("^") {
          T_SUPER!()
        } else if in_math && sub && *t == T_OTHER!("_") {
          T_SUB!()
        } else {
          *t
        }
      })
      .collect(),
  )
}

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
  if !operand.iter().any(|t| is_note(t) || makes_a_note(t, 0)) {
    return None;
  }
  let (mut mark, mut notes) = (Vec::new(), Vec::new());
  let mut i = 0;
  while i < operand.len() {
    let end = if is_note(&operand[i]) {
      note_command_end(operand, i)
    } else if makes_a_note(&operand[i], 0) {
      optional_arguments_end(operand, i + 1)
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
fn optional_arguments_end(tokens: &[Token], mut k: usize) -> usize {
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

/// Whether `t` is a macro of optional arguments only (none, or `\newcommand`'s `[<default>]`) whose expansion holds a
/// note command ([`MARK_NOTE_COMMANDS`]), directly or through another such macro (three deep at most).
fn makes_a_note(t: &Token, depth: usize) -> bool {
  if depth > 2
    || t.get_catcode() != Catcode::CS
    || MARK_NOTE_COMMANDS.iter().any(|cs| *t == T_CS!(*cs))
  {
    return false;
  }
  let Ok(Some(defn)) = lookup_definition(t) else {
    return false;
  };
  if !defn.is_expandable()
    || lookup_conditional(t).is_some()
    || defn
      .get_parameters()
      .is_some_and(|p| p.get_parameters().iter().any(|param| !param.optional))
  {
    return false;
  }
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return false;
  };
  body
    .unlist_ref()
    .iter()
    .any(|b| MARK_NOTE_COMMANDS.iter().any(|cs| *b == T_CS!(*cs)) || makes_a_note(b, depth + 1))
}

/// Whether a mark's operand, its notes taken out, prints nothing: braces, spaces, `\relax` and font switches only
/// (`^{\thanks{…}}`, `^{\mathbf{}}`).
pub(in crate::base_utilities) fn mark_prints_nothing(mark: &[Token]) -> bool {
  mark.iter().all(|t| {
    matches!(
      t.get_catcode(),
      Catcode::BEGIN | Catcode::END | Catcode::SPACE
    ) || *t == T_CS!("\\relax")
      || (t.get_catcode() == Catcode::CS && t.with_str(|name| FONT_SWITCH.is_match(name)))
  })
}

/// An author block's notes written inside a mark (`Nataliia Babina\textsuperscript{\rm 1,\rm 2\thanks{Work done …}}`,
/// `Ann$^{1,\thanks{Corresponding author.}}$`, `$^{1\footnotemark[1]}$`) taken out after it: LaTeX prints the labels
/// and the note's own mark in the superscript, its text as a footnote, so the note is the author's as one written
/// after the mark is (`\textsuperscript{\rm 1,\rm 2}\thanks{…}`), and the mark's labels are read without the note's
/// text in them — the request was the label `2\thanks{…}`, which no list answers and which printed nothing, so the
/// mark `2` and the note were lost (2512.05464, 1911.03118, 1912.00833, 2211.09622, 2305.16614, 2306.17472,
/// 2309.12862, 2402.18447, 2403.04164, 2410.12564, 2501.01423, 2502.15412, 2503.05905, 2603.17529, 2605.17958). A
/// mark holding nothing else (`$^{\thanks{Topical Group Conveners}}$`, 2212.00809) goes, its note kept. The mark
/// readers keep any note left in an operand all the same ([`split_mark_notes`], `\lx@sup@request@affiliation`). A
/// macro the mark holds that makes a note is taken out too (`\rm 1,2,\equalcontrib`, 2603.17529; `1\samethanks`,
/// 2309.12862). Perl reads the operand as the label too (Base_Utility.pool.ltxml:729-734; KNOWN_PERL_ERRORS #1136,
/// OXIDIZED_DESIGN_DIVERGENCES #881).
pub(in crate::base_utilities) fn notes_out_of_marks(stuff: Tokens) -> Tokens {
  let v = stuff.unlist_ref();
  if !v
    .iter()
    .any(|t| t.get_catcode() == Catcode::CS && !is_author_mark(t))
    || !v.iter().any(is_author_mark)
  {
    return stuff;
  }
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut changed = false;
  let mut i = 0;
  while i < v.len() {
    if is_author_mark(&v[i]) {
      let open = i + 1 + v[i + 1..].iter().take_while(|t| **t == T_SPACE!()).count();
      if v
        .get(open)
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
      {
        let close = skip_group(v, open);
        if close <= v.len()
          && v[close - 1].get_catcode() == Catcode::END
          && let Some((mark, notes)) = split_mark_notes(&v[open + 1..close - 1])
        {
          if !mark_prints_nothing(&mark) {
            out.push(v[i]);
            out.push(T_BEGIN!());
            out.extend(mark);
            out.push(T_END!());
          }
          out.extend(notes);
          i = close;
          changed = true;
          continue;
        }
      }
    }
    out.push(v[i]);
    i += 1;
  }
  if changed { Tokens::new(out) } else { stuff }
}

/// An author block's `\begin{footnote}{text}…\end{footnote}` as LaTeX reads it: `\begin{footnote}` runs `\footnote`,
/// which takes the braced text after it as its argument, so what follows up to `\end{footnote}` is the author line's
/// (`Velibor \v{Z}eli\begin{footnote}{Linn\'{e} FLOW Centre, …},\end{footnote} Du\v{s}an Zorica\begin{footnote}…`,
/// 1703.02032: the comma printed after the first note's mark parts the names). Written `\footnote{text}` and the rest,
/// the note is the name's and its comma a separator; read as one unit, the environment hid it. Only a `{footnote}` whose
/// braced text and `\end{footnote}` follow. The environment's group (`\begin` is `\begingroup`, `\end` its
/// `\endgroup`, latex.ltx) is not kept: it would open in one name and close in the next. So a declaration between the
/// note and `\end{footnote}` is no longer confined to it; in an author block that span is the separator after the mark.
pub(in crate::base_utilities) fn note_environments_as_commands(stuff: Tokens) -> Tokens {
  let v = stuff.unlist_ref();
  let name_is = |at: usize, name: &str| -> Option<usize> {
    if v.get(at)?.get_catcode() != Catcode::BEGIN {
      return None;
    }
    let close = (at + 1..v.len()).find(|&k| v[k].get_catcode() == Catcode::END)?;
    let read: String = v[at + 1..close].iter().map(ToString::to_string).collect();
    (read == name).then_some(close + 1)
  };
  if !v.iter().any(|t| *t == T_CS!("\\begin")) {
    return stuff;
  }
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut changed = false;
  let mut i = 0;
  while i < v.len() {
    if v[i] == T_CS!("\\begin")
      && let Some(mut j) = name_is(i + 1, "footnote")
    {
      while v.get(j) == Some(&T_SPACE!()) {
        j += 1;
      }
      let text_end =
        (v.get(j).is_some_and(|t| t.get_catcode() == Catcode::BEGIN)).then(|| skip_group(v, j));
      let end = text_end.and_then(|from| {
        (from..v.len()).find_map(|k| {
          (v[k] == T_CS!("\\end"))
            .then(|| name_is(k + 1, "footnote").map(|e| (k, e)))
            .flatten()
        })
      });
      if let (Some(text_end), Some((end_at, past_end))) = (text_end, end)
        && text_end <= v.len()
        && !v[text_end..end_at].iter().any(|t| *t == T_CS!("\\begin"))
      {
        out.push(T_CS!("\\footnote"));
        out.extend_from_slice(&v[j..text_end]);
        out.extend_from_slice(&v[text_end..end_at]);
        i = past_end;
        changed = true;
        continue;
      }
    }
    out.push(v[i]);
    i += 1;
  }
  if changed { Tokens::new(out) } else { stuff }
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
