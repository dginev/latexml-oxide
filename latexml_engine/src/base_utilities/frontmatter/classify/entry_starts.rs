//! A list's lines cut where its entries start, the positions the digested reading gives (`argument_marks::list_marks`,
//! Option C P4b4a): the token surgery of `affiliations::entry_calls` — where a line stands in its list, a cut that
//! closes and reopens the groups and formula it falls in.

use super::*;

/// A line's entry starts (`list_marks`), from its first token, and whether it prints a mark at all.
#[derive(Clone)]
pub(in crate::base_utilities) struct LineStarts {
  pub(in crate::base_utilities) at:     Vec<usize>,
  pub(in crate::base_utilities) marked: bool,
}

/// Where each of `lines` stands in `stuff`, which [`split_tokens_delimited`] cut them from (each trimmed of its spaces);
/// `None` where one is not found.
pub(in crate::base_utilities) fn line_ranges(
  stuff: &[Token],
  lines: &DelimitedPieces,
) -> Option<Vec<std::ops::Range<usize>>> {
  let mut at = 0;
  let mut find = |part: &[Token]| -> Option<usize> {
    while !stuff[at..].starts_with(part) {
      at += 1;
      if at > stuff.len() {
        return None;
      }
    }
    let start = at;
    at += part.len();
    Some(start)
  };
  lines
    .iter()
    .map(|(delimiter, line)| {
      find(delimiter)?;
      let start = find(line.unlist_ref())?;
      Some(start..start + line.len())
    })
    .collect()
}

/// Each of `lines` (`split_tokens_delimited` of `stuff`) with its entry starts, as `marks` reads them; `None` for each
/// when `marks` is none or unread (counted) — the line's tokens decide then, `author_mark_position`. No line break or
/// cut stands inside an environment the list opens (`\begin{tabular}{c}\affmark{1} Univ A\\ \affmark{2} Univ B…`): its
/// lines are one, cut only outside it, as no piece may hold half of it (revP4b4a B1).
pub(in crate::base_utilities) fn lines_starts(
  stuff: &Tokens,
  lines: DelimitedPieces,
  marks: Option<&ListMarks>,
) -> (DelimitedPieces, Vec<Option<LineStarts>>) {
  let Some((Some(starts), Some(marked))) = marks.map(|marks| (&marks.starts, &marks.marked)) else {
    let n = lines.len();
    return (lines, vec![None; n]);
  };
  let v = stuff.unlist_ref();
  let Some(ranges) = line_ranges(v, &lines) else {
    count_unread_marks("affiliation_calls");
    let n = lines.len();
    return (lines, vec![None; n]);
  };
  let depth = environment_depths(v);
  let mut joined: DelimitedPieces = Vec::new();
  let mut joined_ranges: Vec<std::ops::Range<usize>> = Vec::new();
  for (line, range) in lines.into_iter().zip(ranges) {
    match (joined.last_mut(), joined_ranges.last_mut()) {
      (Some((_, last)), Some(last_range)) if depth[last_range.end] > 0 => {
        last_range.end = range.end;
        *last = Tokens::new(v[last_range.clone()].to_vec());
      },
      _ => {
        joined.push(line);
        joined_ranges.push(range);
      },
    }
  }
  joined
    .into_iter()
    .zip(joined_ranges)
    .map(|(line, range)| {
      let in_line = starts.iter().filter(|at| range.contains(at));
      // (the first opens the line's entry; a later one inside an environment cuts nothing)
      let at = in_line
        .enumerate()
        .filter(|&(n, &at)| n == 0 || depth[at] == 0)
        .map(|(_, at)| at - range.start)
        .collect();
      let marked = marked.iter().any(|at| range.contains(at));
      (line, Some(LineStarts { at, marked }))
    })
    .unzip()
}

/// The number of environments `v` opens that are open before each of its tokens, and past its end; an `\end` of one
/// opened before `v` is none of its own.
fn environment_depths(v: &[Token]) -> Vec<usize> {
  let mut depth = Vec::with_capacity(v.len() + 1);
  let mut open = 0usize;
  for t in v {
    depth.push(open);
    if *t == T_CS!("\\begin") {
      open += 1;
    } else if *t == T_CS!("\\end") {
      open = open.saturating_sub(1);
    }
  }
  depth.push(open);
  depth
}

/// Whether `line` starts no entry: it prints no mark at its level (`q2`'s `\affmark{1} Univ A\\ Street A`, the street);
/// unread, no mark in its tokens.
pub(in crate::base_utilities) fn line_unmarked(line: &Tokens, starts: Option<&LineStarts>) -> bool {
  starts.map_or_else(
    || author_mark_position(line).is_none(),
    |starts| !starts.marked,
  )
}

/// `line`'s entries (`$^1$Univ A; $^2$Univ B`: one label per affiliation, else the last mark labelled them all,
/// 2609.22690, 21347), each with whether a mark opens it, cut where they start (`events::entry_heads`); unread, a line
/// led by a mark is split at each whitespace-preceded one, as `\lx@add@authors` splits its marker-led lines.
pub(in crate::base_utilities) fn entry_pieces(
  line: Tokens,
  starts: Option<&LineStarts>,
) -> Result<Vec<(Tokens, Option<bool>)>> {
  let split = |line: Tokens, opens: Option<bool>| -> Result<Vec<(Tokens, Option<bool>)>> {
    Ok(
      split_wrapped_affiliation_marks(line)?
        .into_iter()
        .map(|seg| (seg, opens))
        .collect(),
    )
  };
  match starts {
    // (a line whose marks lead its notes, `\thanks{\IEEEauthorrefmark{1} Univ A; \IEEEauthorrefmark{2} Univ B}`, is cut
    // inside the note by its tokens: the `\thanks` idiom of `\lx@add@thanks`, 1504.01072, 2609.24896)
    Some(starts) if starts.at.len() < 2 && starts.marked && marker_leads(&line) => {
      split(line, Some(true))
    },
    Some(starts) => Ok(line_at_starts(line, starts)),
    None if marker_leads(&line) => split(line, None),
    None => Ok(vec![(line, None)]),
  }
}

/// `line` cut where its entries start (`starts`, [`LineStarts`]), each piece with whether a mark opens it; what it prints
/// before its first start stays with that entry (`Univ X \\ Present address: $^2$Univ Y`'s second line, as
/// `\lx@add@authors` keeps it). A wrapper holding several starts (`{\it $^1$A, $^2$B}`) is cut inside, repeated on each
/// piece with its opening declarations, as [`split_wrapped_affiliation_marks`] repeats it; a box the line is set in
/// (`\parbox{\linewidth}{$^1$Univ A\\ $^2$Univ B}`) is not: what it prints are its lines (revtex hg1 rows), each with the
/// declarations the body opens with.
fn line_at_starts(line: Tokens, starts: &LineStarts) -> Vec<(Tokens, Option<bool>)> {
  if starts.at.is_empty() {
    return vec![(line, Some(starts.marked))];
  }
  let v: &[Token] = line.unlist_ref();
  let (v, at) = match box_body(v) {
    Some(body) if starts.at.len() >= 2 && starts.at.iter().all(|at| body.contains(at)) => (
      &v[body.clone()],
      starts
        .at
        .iter()
        .map(|at| at - body.start)
        .collect::<Vec<_>>(),
    ),
    _ => (v, starts.at.clone()),
  };
  // (the declarations it opens with are in force on every piece, as the API Cut's `with_opening` keeps a list's:
  // `\parbox{…}{\centering\scriptsize \affmark{1} …}`, 2608.21425 — revP4b4a B2)
  let opening = list_lead_declarations(v, Some(at[0]));
  cut_at_starts(v, &at[1..], at[0])
    .into_iter()
    .enumerate()
    .map(|(n, piece)| {
      let piece = if n > 0 && !opening.is_empty() {
        [opening.as_slice(), piece.as_slice()].concat()
      } else {
        piece
      };
      (Tokens::new(piece), Some(true))
    })
    .collect()
}

/// The body of the box `v` is wholly set in, `\parbox[t]{\linewidth}{…}`, `\mbox{…}`, `\hbox{…}` ([`LINE_BOXES`]).
fn box_body(v: &[Token]) -> Option<std::ops::Range<usize>> {
  let start = v.iter().position(|t| *t != T_SPACE!())?;
  let end = v.iter().rposition(|t| *t != T_SPACE!())? + 1;
  let past_spaces = |mut i: usize| {
    while v.get(i) == Some(&T_SPACE!()) {
      i += 1;
    }
    i
  };
  let mut i = start + 1;
  if v[start] == T_CS!("\\parbox") {
    // (its `[pos][height][inner-pos]` and its width)
    loop {
      i = past_spaces(i);
      if v.get(i) != Some(&T_OTHER!("[")) {
        break;
      }
      i += v[i..].iter().position(|t| *t == T_OTHER!("]"))? + 1;
    }
    i = skip_group(v, i);
  } else if !LINE_BOXES.iter().any(|cs| v[start] == T_CS!(*cs)) {
    return None;
  }
  i = past_spaces(i);
  (v.get(i)?.get_catcode() == Catcode::BEGIN && skip_group(v, i) == end).then(|| i + 1..end - 1)
}

/// `v` cut before each of `at` (positions in it, ascending): a cut inside a group closes it and opens it again, with
/// its wrapper command and the declarations it opens with (`notes::lead_declarations`) before the mark opening the first
/// entry (`first`: a macro printing it, `\tsec` with `\newcommand\tsec{\textsuperscript{\S}}`, is none; 2410.03583),
/// in the next piece.
fn cut_at_starts(v: &[Token], at: &[usize], first: usize) -> Vec<Vec<Token>> {
  let Some(&cut) = at.first() else {
    return vec![v.to_vec()];
  };
  // (from the braces opening it and a `\protect`, which print nothing: `{$\rm ^{3}$}`, `\protect\( ^{5} \protect\)`)
  let cut = v[..cut]
    .iter()
    .rposition(|t| t.get_catcode() != Catcode::BEGIN && *t != T_CS!("\\protect"))
    .map_or(0, |i| i + 1);
  // (the groups open at the cut, outermost first)
  let mut open: Vec<usize> = Vec::new();
  for (i, t) in v[..cut].iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => open.push(i),
      Catcode::END => {
        open.pop();
      },
      _ => {},
    }
  }
  // (the separator before the cut is none of the entry's, inside the wrapper too: `{\it $^1$A, $^2$B}`)
  // (a cut inside a formula, at a mark on a base, `Lab $5^{2}$Univ Y` under `Bob Baker$^{2}$`, closes the formula and
  // opens it again: the base the entry's before it, the script the next's label — revISO1 S1)
  let formula = v[..cut]
    .iter()
    .rposition(|t| t.get_catcode() == Catcode::MATH)
    .filter(|_| {
      v[..cut]
        .iter()
        .filter(|t| t.get_catcode() == Catcode::MATH)
        .count()
        % 2
        == 1
    })
    .filter(|shift| open.iter().all(|group| group < shift));
  let mut head = trim_trailing_separator(v[..cut].to_vec());
  head.extend(formula.map(|shift| v[shift]));
  head.extend(std::iter::repeat_n(T_END!(), open.len()));
  let mut rest: Vec<Token> = Vec::new();
  for &group in &open {
    if let Some(cmd) = group
      .checked_sub(1)
      .map(|c| v[c])
      .filter(|t| t.get_catcode() == Catcode::CS)
    {
      rest.push(cmd);
    }
    rest.push(v[group]);
    let end = if first > group { first.min(cut) } else { cut };
    rest.extend(lead_declarations(&v[group + 1..end]));
  }
  rest.extend(formula.map(|shift| v[shift]));
  let reopened = rest.len();
  rest.extend_from_slice(&v[cut..]);
  let later: Vec<usize> = at[1..].iter().map(|&a| a - cut + reopened).collect();
  let mut pieces = vec![head];
  pieces.extend(cut_at_starts(&rest, &later, reopened));
  pieces
}
