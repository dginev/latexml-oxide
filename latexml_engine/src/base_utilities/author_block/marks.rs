//! Author marks: answered marks, mark commands, `has_author_marks`, label marks.

use super::*;

/// Whether an author block holds the affiliations its marks request: a line after its first opening with a mark a
/// name before it requests (`Ann Able$^{1}$, Bob Baker$^{2}$\\ $^{1}$Univ A`), which the marked parse labels for the
/// names' marks to find. A line led by a mark no name requests is a name marked before it, or text (`${}^3$He …`), and
/// answers nothing; so does a name marked as the first, beside it on its printed line
/// ([`beside_prefix_marked_names`]: `$^{1}$Ann Able \quad $^{1}$Bob Baker`) or answered by a line below
/// ([`mark_answered_below`]).
pub(in crate::base_utilities) fn answers_its_marks(stuff: &Tokens) -> bool {
  let labels = |operands: Vec<Vec<Token>>| -> Vec<String> {
    operands
      .into_iter()
      .flat_map(|operand| {
        clean_frontmatter_labels(&Tokens::new(operand).to_string(), "affiliation")
      })
      .collect()
  };
  let pieces = split_tokens_delimited(stuff.clone(), author_affil_splits());
  let lines: Vec<Tokens> = pieces.iter().map(|(_, line)| line.clone()).collect();
  let mut requested: Vec<String> = Vec::new();
  let mut prefix_marked: Option<bool> = None;
  let mut names_line = true;
  for (index, (delimiter, line)) in pieces.iter().enumerate() {
    let beside =
      beside_prefix_marked_names(prefix_marked == Some(true), &mut names_line, delimiter);
    if line.is_empty() {
      continue;
    }
    let leads = marker_leads(line);
    let Some(prefix_marked) = prefix_marked else {
      prefix_marked = Some(leads);
      requested.extend(labels(author_mark_operands(line.unlist_ref())));
      continue;
    };
    // a line of names beside the first on its printed line (a per-author block of names alone,
    // `\IEEEauthorblockN{$^{1}$Ann Able \quad $^{1}$Bob Baker}`, splits at the `\quad` on the unmarked path), or on a
    // line of its own when its mark is new and an affiliation below answers it, as the marked path reads it
    let new_mark = || {
      labels(
        mark_operands(line.unlist_ref())
          .into_iter()
          .take(1)
          .collect(),
      )
      .iter()
      .any(|label| !requested.contains(label))
    };
    let a_name = prefix_marked
      && reads_as_names(line)
      && (beside || (new_mark() && mark_answered_below(line, &lines[index + 1..])));
    if leads && !a_name {
      let leading = mark_operands(line.unlist_ref())
        .into_iter()
        .take(1)
        .collect();
      if labels(leading)
        .iter()
        .any(|label| requested.contains(label))
      {
        return true;
      }
    } else {
      requested.extend(labels(author_mark_operands(line.unlist_ref())));
    }
  }
  false
}

/// Whether an affiliation below `line` (among `below`, the lines of an author block after it) answers the mark
/// leading `line`: the label a name marked before it requests (`$^{1}$Ann Able\\ $^{1}$Bob Baker\\ $^{1}$Univ A`). The
/// answering part must read as an affiliation ([`reads_as_affiliation`]): a line led by the same mark that is an email
/// or a place (`$^{1}$\texttt{ann@google.com}`, `$^{1}$Beijing, China`) answers nothing, so a name-shaped affiliation
/// above it (`$^{1}$Google DeepMind`) stays one.
pub(in crate::base_utilities) fn mark_answered_below(line: &Tokens, below: &[Tokens]) -> bool {
  let leading_label = |line: &Tokens| {
    mark_operands(line.unlist_ref())
      .into_iter()
      .next()
      .and_then(|operand| mark_label(&operand))
  };
  let Some(label) = leading_label(line) else {
    return false;
  };
  below
    .iter()
    .filter(|later| marker_leads(later))
    .any(|later| {
      marked_parts(later)
        .iter()
        .any(|part| leading_label(part).as_ref() == Some(&label) && reads_as_affiliation(part))
    })
}

/// The marked parts of a line (`split_wrapped_affiliation_marks`: `$^{1}$Univ A $^{2}$Univ B` → two), blank ones dropped.
fn marked_parts(line: &Tokens) -> Vec<Tokens> {
  split_wrapped_affiliation_marks(line.clone())
    .into_iter()
    .filter(|part| !part.unlist_ref().iter().all(|t| *t == T_SPACE!()))
    .collect()
}

/// Whether a marked line reads as names: each of its marked parts a name or names ([`name_count`]), so that
/// `$^{1}$Google DeepMind $^{2}$Huawei Technologies` is two parts, not one four-word name.
pub(in crate::base_utilities) fn reads_as_names(line: &Tokens) -> bool {
  let parts = marked_parts(line);
  !parts.is_empty()
    && parts
      .iter()
      .all(|part| name_count(&visible_name_text(part.unlist_ref())) > 0)
}

/// Whether the line after `delimiter` stands beside the names of an author block whose names are marked before them
/// (`prefix_marked`: `$^{1}$Ann Able \quad $^{1}$Bob Baker \quad $^{2}$Cat Cole\\ $^{1}$Univ A`): on the printed line
/// of its first name, after `\quad`/`\qquad`, where a line break has not yet ended the names. `names_line` tracks it
/// across a block's lines: a line break ends it, the `\and` family opens the next author's.
pub(in crate::base_utilities) fn beside_prefix_marked_names(
  prefix_marked: bool,
  names_line: &mut bool,
  delimiter: &[Token],
) -> bool {
  let written = |delims: &[Token]| delimiter.iter().any(|t| delims.contains(t));
  if written(&[T_CS!("\\\\"), T_CS!("\\newline")]) {
    *names_line = false;
  } else if written(&[T_CS!("\\and"), T_CS!("\\And"), T_CS!("\\AND")]) {
    *names_line = true;
  }
  prefix_marked && *names_line && written(&[T_CS!("\\quad"), T_CS!("\\qquad")])
}

pub(in crate::base_utilities) fn affil_splits() -> Vec<SplitDelim> {
  vec![
    // The \and CONTROL-SEQUENCE family is added (vs upstream, which splits
    // affiliations only on \quad/\qquad/\\): it lets the shared affiliation
    // parser handle \and-separated institute/affiliation lists (e.g. LNCS
    // \institute{A \and B}). NOTE: the literal word " and " is deliberately NOT
    // included (unlike author_affil_splits) — institution names routinely
    // contain "and" ("Electrical and Computer Engineering"), so splitting on it
    // wrongly fragments them. Affiliations never use ',' as a separator either.
    T_CS!("\\and").into(),
    T_CS!("\\And").into(),
    T_CS!("\\AND").into(),
    T_CS!("\\quad").into(),
    T_CS!("\\qquad").into(),
    T_CS!("\\\\").into(),
    T_CS!("\\newline").into(),
  ]
}
/// `tokens` with every alias of a mark command written as the command it means ([`mark_command_meant`]), so the
/// readers of an author or affiliation block that match a command by name (`\textsuperscript`'s symbol and ordinal
/// rewrites) see the alias as the command.
pub(in crate::base_utilities) fn unalias_mark_commands(tokens: Tokens) -> Tokens {
  if !tokens.unlist_ref().iter().any(|t| {
    matches!(t.get_catcode(), Catcode::CS | Catcode::ACTIVE)
      && !mark_commands().contains(t)
      && is_mark_command(t)
  }) {
    return tokens;
  }
  Tokens::new(
    tokens
      .unlist()
      .into_iter()
      .map(|t| mark_command_meant(&t).unwrap_or(t))
      .collect(),
  )
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

/// Does an author block mark its names (`$^{a}$`, `\textsuperscript{1}`)? A superscript inside a `\footnote` or
/// `\thanks` is the note's own text, not a mark: 0911.0568's `LPT Orsay (CNRS)~\footnote{$^a$Laboratoire …}` line
/// made its PoS author block a marked list, its affiliation and email line read as a person (63k).
pub fn has_author_marks(stuff: &Tokens) -> bool {
  let tokens = stuff.unlist_ref();
  let mut i = 0;
  while i < tokens.len() {
    let t = &tokens[i];
    if *t == T_CS!("\\footnote") || *t == T_CS!("\\thanks") {
      i += 1;
      while i < tokens.len() && tokens[i] == T_SPACE!() {
        i += 1;
      }
      if i < tokens.len() && tokens[i] == T_OTHER!("[") {
        while i < tokens.len() && tokens[i] != T_OTHER!("]") {
          i += 1;
        }
        i += 1;
      }
      if i < tokens.len() && tokens[i].code == Catcode::BEGIN {
        let mut depth = 0usize;
        while i < tokens.len() {
          match tokens[i].code {
            Catcode::BEGIN => depth += 1,
            Catcode::END => {
              depth -= 1;
              if depth == 0 {
                break;
              }
            },
            _ => {},
          }
          i += 1;
        }
      }
    } else if is_author_mark(t) {
      return true;
    }
    i += 1;
  }
  false
}

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
