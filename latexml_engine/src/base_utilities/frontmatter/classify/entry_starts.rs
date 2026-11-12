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

/// `line` cut where its entries start (`starts`, [`LineStarts`]), each piece with whether a mark opens it; what it prints
/// before its first start stays with that entry (`Univ X \\ Present address: $^2$Univ Y`'s second line, as
/// `\lx@add@authors` keeps it). A wrapper holding several starts (`{\it $^1$A, $^2$B}`) is cut inside, repeated on each
/// piece with its opening declarations, as [`split_wrapped_affiliation_marks`] repeats it.
pub(in crate::base_utilities) fn line_at_starts(
  line: Tokens,
  starts: &LineStarts,
) -> Vec<(Tokens, Option<bool>)> {
  if starts.at.is_empty() {
    return vec![(line, Some(starts.marked))];
  }
  cut_at_starts(line.unlist_ref(), &starts.at[1..], starts.at[0])
    .into_iter()
    .map(|piece| (Tokens::new(piece), Some(true)))
    .collect()
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
