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

/// An author block's line break that opens a line of notes only ([`notes_only_line`]) taken out, with its `*` and
/// `[len]` and any spacer `\\` after it: the notes join the line above, as written without the break. Such a line
/// prints only its notes' marks; its notes are the names' above (`Ann Able$^{1,*}$, … and Cat Cole$^{1}$\\
/// \thanks{$^{1}$Ann Able and Cat Cole are with Univ A, Town.}…`: 2609.17106, 2411.05212, 2312.10807's `\\[3.0ex]`).
/// Read as a line of its own, it led with the mark inside its first note — `author_mark_position` (marks.rs) counts a
/// mark inside a `\thanks`, `has_author_marks` skips it, and the two disagree — and became one affiliation answering
/// mark 1 while its notes went to the document: Ann's and Cat's affiliation lost, all three notes left unowned (Perl
/// the same, Base_Utility.pool.ltxml:702-712; KNOWN_PERL_ERRORS #1206, OXIDIZED_DESIGN_DIVERGENCES #957). Only that
/// misreading: a block whose names carry marks (`has_author_marks`, outside its notes) and a line with a note whose
/// leading mark a name requests ([`notes_the_names_request`]: its other notes the document's, returned to be added after
/// the block's calls — review rev100c S1); a line of unmarked notes stays a line — in a marked block it continues the entry above as joined it would, and in an
/// unmarked block it is the document's notes, no author's by position (ieeetran_cls.rs `without_stop_after_grade`, batch
/// 86d: 2112.14209, 2409.11755, 2510.15773, 2605.22853). And only after a names line: its `\and` group's first, or one
/// whose marks follow its text — after an affiliation or an `Emails: …` line the notes stay a line of their own
/// (joined, 1911.03356's address list no longer read as one). Only a break at the block's top level, outside braces
/// and environments (a tabular's row end is no line), and only up to the next line break or `\and`.
pub(in crate::base_utilities) fn notes_line_joins_names(stuff: Tokens) -> (Tokens, Vec<Token>) {
  let v = stuff.unlist_ref();
  let brk = T_CS!("\\\\");
  if !v.contains(&brk) || !has_author_marks(&stuff) {
    return (stuff, Vec::new());
  }
  let ends_line = |t: &Token| {
    *t == brk
      || author_and_splits()
        .iter()
        .any(|split| matches!(split, SplitDelim::Token(cs) if cs == t))
  };
  let past_spaces = |mut at: usize| {
    while v.get(at) == Some(&T_SPACE!()) {
      at += 1;
    }
    at
  };
  // (past a `\\` at `at`: its `*` and `[…]`, the spaces before the `[` skipped as `\@ifnextchar` skips them)
  let past_break = |at: usize| {
    let mut j = at + 1;
    if v.get(j) == Some(&T_OTHER!("*")) {
      j += 1;
    }
    let open = past_spaces(j);
    if v.get(open) == Some(&T_OTHER!("["))
      && let Some(close) = v[open..].iter().position(|t| *t == T_OTHER!("]"))
    {
      j = open + close + 1;
    }
    j
  };
  // (the labels the block's names request, outside their notes)
  let requested: Vec<String> = author_mark_operands(v)
    .iter()
    .flat_map(|operand| mark_keys(operand))
    .collect();
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut document_notes: Vec<Token> = Vec::new();
  let mut changed = false;
  let (mut depth, mut environments) = (0usize, 0usize);
  // (where the line above starts in `out`, and whether it is its `\and` group's first)
  let (mut above_start, mut first_line) = (0usize, true);
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    if t == brk && depth == 0 && environments == 0 {
      // (the line above a names line: its group's first, or one whose marks follow its text — not an affiliation its
      // mark leads, nor an unmarked line, `Emails: …` whose address list a note joined would no longer read as one,
      // 1911.03356)
      let above = Tokens::new(out[above_start..].to_vec());
      let names_above = first_line || (has_author_marks(&above) && !marker_leads(&above));
      let mut j = past_break(i);
      // (a spacer `\\` right after it, `\\ \\`)
      while v.get(past_spaces(j)) == Some(&brk) {
        j = past_break(past_spaces(j));
      }
      let mut level = 0usize;
      let line_end = (j..v.len())
        .find(|&k| {
          match v[k].get_catcode() {
            Catcode::BEGIN => level += 1,
            Catcode::END => level = level.saturating_sub(1),
            _ => {},
          }
          level == 0 && ends_line(&v[k])
        })
        .unwrap_or(v.len());
      let line = &v[j..line_end];
      if names_above
        && let Some(notes) = line_notes(line)
        && let Some((joined, document)) = notes_the_names_request(line, &notes, &requested)
      {
        out.extend(joined);
        document_notes.extend(document);
        changed = true;
        i = line_end;
        continue;
      }
    }
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if depth == 0 && t == T_CS!("\\begin") => environments += 1,
      _ if depth == 0 && t == T_CS!("\\end") => environments = environments.saturating_sub(1),
      _ => {},
    }
    out.push(t);
    i += 1;
    if depth == 0 && environments == 0 && ends_line(&t) {
      above_start = out.len();
      first_line = t != brk;
    }
  }
  if changed {
    (Tokens::new(out), document_notes)
  } else {
    (stuff, document_notes)
  }
}

/// A line of notes only ([`line_notes`]: its `notes`, their spans) as it joins the names above
/// ([`notes_line_joins_names`]) — the line's tokens to join, and the notes taken out of the block as the document's —, or
/// `None` when it stays a line: a note joins when the mark leading it is one the names request (`requested`,
/// [`mark_keys`]); a note no name requests (`\thanks{This work was funded by …}`, a symbol only a note prints,
/// 2312.10807's `$^{*}$Equal contribution` under `\IEEEauthorrefmark{1}`) is the document's, as the line read whole made
/// it — digested in a contact, `\thanks` is `\lx@add@pubnote@thanks` and `\footnote` `\lx@add@pubnote` (queue_digest.rs)
/// —, not the last name's (FIDELITY_CHARTER R1: 2609.17106's funding note on Markus Vincze, review rev100c S1); taken
/// out of the block, so no mark its text prints is read as a name's (`$^{\dagger}$Corresponding author`). No note
/// requested, or one of the document's that is no plain `\thanks{…}` or `\footnote{…}`: the line stays.
fn notes_the_names_request(
  line: &[Token],
  notes: &[(usize, usize)],
  requested: &[String],
) -> Option<(Vec<Token>, Vec<Token>)> {
  let asked: Vec<bool> = notes
    .iter()
    .map(|&(start, end)| {
      leading_note_mark(&line[start..end]).is_some_and(|operand| {
        mark_keys(&operand)
          .iter()
          .any(|key| requested.contains(key))
      })
    })
    .collect();
  if !asked.contains(&true) {
    return None;
  }
  let (mut joined, mut document) = (Vec::with_capacity(line.len()), Vec::new());
  let mut at = 0;
  for (&(start, end), asked) in notes.iter().zip(asked) {
    joined.extend_from_slice(&line[at..start]);
    at = end;
    if asked {
      joined.extend_from_slice(&line[start..end]);
      continue;
    }
    // (no `[…]` before its text: an optional argument is no pubnote's)
    let plain = !line[start..end]
      .iter()
      .take_while(|t| t.get_catcode() != Catcode::BEGIN)
      .any(|t| *t == T_OTHER!("["));
    document.push(match line[start] {
      t if plain && t == T_CS!("\\thanks") => T_CS!("\\lx@add@pubnote@thanks"),
      t if plain && t == T_CS!("\\footnote") => T_CS!("\\lx@add@pubnote"),
      _ => return None,
    });
    document.extend_from_slice(&line[start + 1..end]);
  }
  joined.extend_from_slice(&line[at..]);
  Some((joined, document))
}

/// The operand of the mark leading a note's text (`\thanks{$^{1}$Univ A}`, `\thanks{$*$ Equal contribution}`): a mark
/// before any letter of it, or a math group of no letter it opens with (`$*$`, `$\dagger$`).
fn leading_note_mark(note: &[Token]) -> Option<Vec<Token>> {
  let open = note
    .iter()
    .position(|t| t.get_catcode() == Catcode::BEGIN)?;
  let text = &note[open + 1..note.len().saturating_sub(1)];
  let text = &text[text.iter().take_while(|t| **t == T_SPACE!()).count()..];
  let tokens = Tokens::new(text.to_vec());
  if let Some(at) = author_mark_position(&tokens)
    && !name_precedes_marker(&tokens, at)
  {
    return mark_operands(text).into_iter().next();
  }
  let close = text
    .iter()
    .skip(1)
    .position(|t| t.get_catcode() == Catcode::MATH)?
    + 1;
  let inner = &text[1..close];
  (text.first()?.get_catcode() == Catcode::MATH
    && !inner.is_empty()
    && inner.iter().all(|t| t.get_catcode() != Catcode::LETTER))
  .then(|| inner.to_vec())
}

/// The labels a mark's operand keys (`clean_frontmatter_labels`, no prefix), each comma piece's, and a piece's number
/// or letter and its symbol apart as well when they are glued ([`split_glued_symbol`]: `$^{1*}$` keys `1*`, `1` and
/// `*`, as 2411.05212's `$^*$Work done …` asks).
fn mark_keys(operand: &[Token]) -> Vec<String> {
  let mut keys = Vec::new();
  for piece in operand.split(|t| *t == T_OTHER!(",")) {
    let piece: Vec<Token> = piece
      .iter()
      .filter(|t| **t != T_SPACE!())
      .copied()
      .collect();
    let mut parts = vec![piece.clone()];
    if let Some((lead, symbol)) = split_glued_symbol(&piece) {
      parts.extend([lead, symbol]);
    }
    for part in parts {
      keys.extend(clean_frontmatter_labels(&Tokens::new(part).to_string(), ""));
    }
  }
  keys
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
