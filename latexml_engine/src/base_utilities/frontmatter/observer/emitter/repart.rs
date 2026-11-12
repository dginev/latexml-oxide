//! The spans the emitter reads a reading creator's entries and names by: an entry's span in its block ([`entry_span`]),
//! the tokens before a name's first word ([`prefix_tokens`]), a name joined across a layout's cells
//! ([`joined_across_cells`]) and a span crossing a row's columns ([`crosses_columns`]). What is left of the S2b writer's
//! re-part (OXIDIZED_DESIGN_DIVERGENCES #928), which wrote placed creators again from the reading by DOM operations and
//! was deleted with the writer (DEL4; its witnesses 1909.05311, 2609.32870, 2609.33104, 2307.03384, 2308.10621,
//! hep-lat0209026, hep-th9806168, declined 1304.1574, 0811.3142).

use super::{
  super::{
    events::{EventKind, SepKind},
    ledger::{Entry, Outcome},
    resolver::{entry_span_in, read_events},
  },
  dom::norm,
  plan::block_span_tokens,
  *,
};

/// The span of a block's entry the reading keys `key` ([`entry_span_in`] on the block's events at ledger index `index`).
pub(super) fn entry_span(index: usize, span: (u16, u16), key: &str) -> Option<(u16, u16)> {
  with_ledger(|ledger| {
    let Some(Entry::Author(block)) = ledger.entries().get(index) else {
      return None;
    };
    let Outcome::Events { events, .. } = &block.outcome else {
      return None;
    };
    entry_span_in(events, span, key)
  })
}

/// The tokens from `span`'s start of the block at ledger index `index` that print `key`, where `key` ends inside an ink
/// ([`entry_span`] reads whole inks): an item the reading writes without the addresses its line prints last
/// (`Resolver::block`'s row address: `Dresden, Saxony 01062, Germany (martin.waltz@tu-dresden.de)` the affiliation `Dresden,
/// Saxony 01062, Germany`, 2203.10777). The cut ends at the first address's own first token — past every token printing
/// `key`'s letters, the writer's own check of a span's letters ([`block_span_tokens`]) — less the spaces and separators
/// before it, the commands opening it and the bracket it opens, as the reading's cut leaves them (`address_cuts`); a bracket it leaves open closed by
/// its own closing token after the address: `Dresden [Germany] (bob.baker@a.org)` keeps `Dresden [Germany]`, `(Tel. 351
/// 463, bob.baker@a.org)` `(Tel. 351 463)` (review revLO2 2: the shortest letter match dropped the ink after the last letter).
/// A mark among the inks before it is its ink (`Dresden$^{\dagger}$, Germany`, review revLO2 base). `None` where the inks
/// before it don't open `key`, or no run prints it.
pub(super) fn prefix_tokens(
  index: usize,
  span: (u16, u16),
  key: &str,
) -> Option<((u16, u16), Tokens)> {
  let (ends_in, tokens, base) = with_ledger(|ledger| {
    let Some(Entry::Author(block)) = ledger.entries().get(index) else {
      return None;
    };
    let Outcome::Events { events, .. } = &block.outcome else {
      return None;
    };
    let first = events
      .iter()
      .position(|e| e.kind != EventKind::Layout && e.span.is_some_and(|s| s.0 == span.0))?;
    let mut joined = String::new();
    for event in &events[first..] {
      let text = match &event.kind {
        EventKind::Name | EventKind::Contact => norm(&event.text),
        EventKind::Sep(SepKind::Word(word)) => norm(word),
        EventKind::Sep(_) | EventKind::Break | EventKind::Layout | EventKind::Mark => continue,
        _ => return None,
      };
      joined.push_str(&text);
      if joined.len() > key.len() && joined.starts_with(key) {
        return Some((event.span?, block.read_tokens().clone(), block.range.0));
      }
      if !key.starts_with(&joined) {
        return None;
      }
    }
    None
  })?;
  let prints_key = |end: u16| block_span_tokens(index, (span.0, end), Some(key), false);
  // (the first address's own first token: the last end still printing `key` alone — or the address's start, where its
  // first token is a call reading its arguments, `\textit{\{aable, bgh, ccole\}@uber.com, …}`: the end past the call
  // alone is no run (`content::reads_past`, the call reading what follows the run), 2603.07345)
  let mut address = (ends_in.0 + 1..ends_in.1)
    .find(|&end| prints_key(end).is_some())
    .or_else(|| prints_key(ends_in.0).map(|_| ends_in.0))?;
  while address + 1 < ends_in.1 && prints_key(address + 1).is_some() {
    address += 1;
  }
  let content = tokens.unlist_ref();
  let at = |p: u16| {
    usize::from(p)
      .checked_sub(base)
      .and_then(|i| content.get(i))
  };
  let char_of = |p: u16| {
    at(p)
      .filter(|t| t.get_catcode() == Catcode::OTHER)
      .and_then(|t| t.with_str(|s| s.chars().next()))
  };
  let space = |p: u16| at(p).is_some_and(|t| t.get_catcode() == Catcode::SPACE);
  // (and the commands and groups opening the address: `(\texttt{bob@a.org})`)
  let opening = |p: u16| {
    at(p).is_some_and(|t| {
      matches!(
        t.get_catcode(),
        Catcode::SPACE | Catcode::CS | Catcode::BEGIN
      )
    })
  };
  const SEPARATORS: &[char] = &[',', ';', ':', '-', '\u{2013}', '\u{2014}', '/'];
  let mut end = address;
  while end > span.0 && opening(end - 1) {
    end -= 1;
  }
  if end > span.0 && char_of(end - 1).is_some_and(|c| "([<".contains(c)) {
    end -= 1;
  }
  while end > span.0
    && (space(end - 1) || char_of(end - 1).is_some_and(|c| SEPARATORS.contains(&c)))
  {
    end -= 1;
  }
  let cut = (span.0, end);
  let mut kept = prints_key(end)?.unlist();
  // (a bracket the cut leaves open, closed by its own token after the address)
  let mut open: Vec<char> = Vec::new();
  for c in (span.0..end).filter_map(char_of) {
    match c {
      '(' | '[' => open.push(c),
      ')' | ']' => {
        open.pop();
      },
      _ => {},
    }
  }
  let mut from = address;
  for opener in open.into_iter().rev() {
    let closer = if opener == '(' { ')' } else { ']' };
    let Some(p) = (from..ends_in.1).find(|&p| char_of(p) == Some(closer)) else {
      break;
    };
    kept.extend(at(p).copied());
    from = p + 1;
  }
  Some((cut, Tokens::new(kept)))
}

/// The words of a text: its runs of letters and digits, lowercased.
fn words(text: &str) -> Vec<String> {
  text
    .split(|c: char| !c.is_alphanumeric())
    .filter(|w| !w.is_empty())
    .map(str::to_lowercase)
    .collect()
}

/// Whether the reading joined a creator's name of Name events with nothing but layout between them
/// (`name:layout-joined`, the resolver's `joins_previous`) where an alignment's cell boundary is among that layout: the
/// name's words run on from one cell into the next.
pub(super) fn joined_across_cells(c: &RCreator) -> bool {
  if !c.rule.contains("name:layout-joined") {
    return false;
  }
  let name = words(&c.name);
  with_ledger(|ledger| {
    let Some(Entry::Author(block)) = ledger.entries().get(c.source) else {
      return false;
    };
    let Outcome::Events { events, .. } = &block.outcome else {
      return false;
    };
    let events = read_events(events);
    let Some(at) = events.iter().position(|e| {
      let text = words(&e.text);
      e.kind == EventKind::Name && !text.is_empty() && name.starts_with(&text)
    }) else {
      return false;
    };
    let mut read = words(&events[at].text).len();
    let mut cell = false;
    for e in &events[at + 1..] {
      if read >= name.len() {
        break;
      }
      match e.kind {
        EventKind::Layout => cell |= e.role.as_deref() == Some("cell"),
        EventKind::Name => {
          let text = words(&e.text);
          if text.is_empty() || !name[read..].starts_with(&text) {
            return false;
          }
          if cell {
            return true;
          }
          read += text.len();
        },
        _ => return false,
      }
    }
    false
  })
}

/// Whether the ink the span `span` of the author block at ledger index `index` covers runs on from one column of an
/// alignment into another ([`events_cross_columns`]): its tokens hold the alignment's `&`, or join a column's row with
/// another column's — an item no cut of the block can make, which the emitter declines with its block (Perl's
/// whole-block creator, Base_Utility.pool.ltxml:691-693): `$^\star$Univ X & $^\dagger$Univ Y\\ Dept A & Dept B`
/// (1304.3138, 1711.10325, 1704.06967, 2511.14288, 2605.30319, 1802.06894).
pub(super) fn crosses_columns(index: usize, span: (u16, u16)) -> bool {
  with_ledger(|ledger| {
    let Some(Entry::Author(block)) = ledger.entries().get(index) else {
      return false;
    };
    let Outcome::Events { events, .. } = &block.outcome else {
      return false;
    };
    events_cross_columns(events, span)
  })
}

/// Whether, between the first and the last ink of `events` inside `start..end`, an alignment enclosing that ink (at a
/// lower nest than any of it) starts a cell of a later column (its digested `Layout{role="cell"}`, labelled
/// `row,column`) or ends a row of more than one cell (`Break{role="row"}` after a cell of a later column). The rows of
/// one column are lines an item reads on over (`Univ X\\ City`, nucl-th0306055); an alignment the ink holds whole is
/// its own.
fn events_cross_columns(events: &[Event], (start, end): (u16, u16)) -> bool {
  // (a separator is no ink of its own: a gap's is made at the block's level, `Canada~~G1V 0A6` in a cell, 1304.3138)
  let ink = |e: &Event| {
    !matches!(
      e.kind,
      EventKind::Layout | EventKind::Break | EventKind::Sep(_)
    ) && e.span.is_some_and(|(s, t)| s >= start && t <= end)
  };
  let (Some(first), Some(last)) = (events.iter().position(ink), events.iter().rposition(ink))
  else {
    return false;
  };
  let Some(nest) = events[first..=last]
    .iter()
    .filter(|e| ink(e))
    .map(|e| e.nest)
    .min()
  else {
    return false;
  };
  let column = |e: &Event| {
    (e.kind == EventKind::Layout && e.role.as_deref() == Some("cell"))
      .then(|| e.label.as_deref()?.split_once(',')?.1.parse::<usize>().ok())
      .flatten()
  };
  (first..last).any(|at| {
    let e = &events[at];
    if e.nest >= nest {
      return false;
    }
    match e.kind {
      EventKind::Layout => column(e).is_some_and(|c| c > 0),
      // (the row's last cell: the event before the row end at its alignment's nest)
      EventKind::Break if e.role.as_deref() == Some("row") => events[..at]
        .iter()
        .rev()
        .find(|cell| cell.nest <= e.nest)
        .and_then(column)
        .is_some_and(|c| c > 0),
      _ => false,
    }
  })
}
