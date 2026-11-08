//! Author-block preprocessing: math-active scripts and environments read as commands.

use super::*;

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
/// it — digested in a contact, `\thanks` is `\lx@add@pubnote@thanks` and `\footnote` `\lx@add@pubnote` (queue_digest.rs,
/// [`document_note_call`]) —, not the last name's (FIDELITY_CHARTER R1: 2609.17106's funding note on Markus Vincze, review rev100c S1); taken
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
    document.extend(document_note_call(&line[start..end])?);
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
