//! A reading whose names lines enumerate their teams as authors, kept only where it is rc6's reading plus those authors
//! (OXIDIZED_DESIGN_DIVERGENCES #459): the resolver reads with the enumeration on and, where a decision departed from
//! rc6's, reads again with it off and returns rc6's reading whole unless [`promotion_only`] holds; the emitter's word
//! check may still keep rc6's ([`Readings::keep`]).

use std::{cell::RefCell, collections::BTreeSet};

use super::*;

type Log = Box<dyn FnOnce()>;

thread_local! {
  /// the log calls of the reading being made, held until [`checked`] chooses it: none of the other reading's is emitted
  static HELD: RefCell<Option<Vec<Log>>> = const { RefCell::new(None) };
}

/// Emit a log call of the reading (`Info!`, a counted `Warn!`), or hold it while [`checked`] reads.
pub(in crate::base_utilities::frontmatter::observer) fn reading_log(log: impl FnOnce() + 'static) {
  let mut log = Some(log);
  HELD.with_borrow_mut(|held| {
    if let Some(held) = held
      && let Some(log) = log.take()
    {
      held.push(Box::new(log));
    }
  });
  if let Some(log) = log {
    log();
  }
}

/// Restores the log calls held before a reading, a panicking reading's too.
struct Holding(Option<Vec<Log>>);

impl Drop for Holding {
  fn drop(&mut self) { HELD.set(self.0.take()); }
}

/// `read`'s value and the log calls it made.
fn held<T>(read: impl FnOnce() -> T) -> (T, Vec<Log>) {
  let _outer = Holding(HELD.replace(Some(Vec::new())));
  let value = read();
  let logs = HELD.replace(None).unwrap_or_default();
  (value, logs)
}

/// The two readings of a document whose names lines enumerated a team: the enumeration's, and rc6's where a decision
/// departed from it and the enumeration's is not [`promotion_only`] over it. Each holds its log calls until [`Self::keep`].
pub struct Readings {
  enumerated: (Resolved, Vec<Log>),
  rc6:        Option<(Resolved, Vec<Log>)>,
}

impl Readings {
  /// The enumeration's reading, or rc6's whole where it is not [`promotion_only`] over rc6's.
  pub fn reading(&self) -> &Resolved { &self.enumerated.0 }

  /// rc6's reading, where the enumeration's is rc6's plus its teams.
  pub fn rc6(&self) -> Option<&Resolved> { self.rc6.as_ref().map(|(rc6, _)| rc6) }

  /// The reading kept, `rc6`'s if asked and there is one, its log calls emitted (none of the other's).
  pub fn keep(self, rc6: bool) -> Resolved {
    let (kept, logs) = match self.rc6 {
      Some(rc6_reading) if rc6 => rc6_reading,
      _ => self.enumerated,
    };
    for log in logs {
      log();
    }
    kept
  }
}

/// The reading `read(true)` if no decision departed from rc6's; else `read(false)`'s, rc6's, unless `read(true)`'s holds
/// [`promotion_only`] over it, which the emitter's word check may still decline ([`Readings::keep`]).
pub(super) fn checked(inputs: &[Input], read: impl Fn(bool) -> (Resolved, bool)) -> Readings {
  let ((enumerated, fired), logs) = held(|| read(true));
  if !fired {
    return Readings {
      enumerated: (enumerated, logs),
      rc6:        None,
    };
  }
  let ((rc6, _), rc6_logs) = held(|| read(false));
  if promotion_only(inputs, &rc6, &enumerated) {
    Readings {
      enumerated: (enumerated, logs),
      rc6:        Some((rc6, rc6_logs)),
    }
  } else {
    Readings {
      enumerated: (rc6, rc6_logs),
      rc6:        None,
    }
  }
}

/// Two items alike but for the rule that placed them.
fn same(a: &Assoc, b: &Assoc) -> bool {
  a.role == b.role
    && a.text == b.text
    && a.label == b.label
    && a.source == b.source
    && a.target == b.target
}

/// The items of `from` not in `kept`, each matched once; `None` if `kept` holds one `from` does not.
fn lost<'a>(from: &'a [Assoc], kept: &[Assoc]) -> Option<Vec<&'a Assoc>> {
  let mut left: Vec<&Assoc> = kept.iter().collect();
  let mut lost = Vec::new();
  for item in from {
    match left.iter().position(|k| same(k, item)) {
      Some(at) => {
        left.remove(at);
      },
      None => lost.push(item),
    }
  }
  left.is_empty().then_some(lost)
}

fn items(c: &RCreator) -> impl Iterator<Item = &Assoc> {
  c.affiliations.iter().chain(&c.contacts).chain(&c.notes)
}

/// An e-mail, or an item printing one: never moved to a team.
fn mail(a: &Assoc) -> bool {
  a.role == "email" || a.text.contains('@') || a.target.as_deref().is_some_and(|t| t.contains('@'))
}

/// A team's name as its piece prints it, its "and" and "the" aside.
fn key(text: &str) -> String {
  let mut rest = text.trim();
  for lead in ["and ", "& ", "the ", "The "] {
    rest = rest.strip_prefix(lead).unwrap_or(rest).trim_start();
  }
  rest
    .chars()
    .filter(|c| c.is_alphanumeric())
    .flat_map(char::to_lowercase)
    .collect()
}

/// A creator the enumeration made: a team's name, no label before a colon (`R Core Team: KM3NeT Consortium`, which the
/// safety net counts as two names).
fn team(c: &RCreator) -> bool {
  c.rule.contains(LISTED_GROUP)
    && !c.name.trim().is_empty()
    && net::label_before_colon(&c.name).is_none()
}

/// Whether `rc6`'s item is the piece a team of the reading was in rc6's reading: its names' collaboration the names line
/// read (`Resolver::collaboration`), or the affiliation its column of its own was every name's.
fn piece_of(item: &Assoc, team: &RCreator) -> bool {
  key(&item.text) == key(&team.name)
    && if team.rule.contains(COLUMN_GROUP) {
      item.role == "affiliation"
    } else {
      item.role == "collaboration" && item.rule.starts_with("line:names collaboration")
    }
}

/// The events of the block or call at ledger index `index`.
fn events_of(inputs: &[Input], index: usize) -> &[Event] {
  let events = inputs.iter().find_map(|input| match input {
    Input::Block { index: i, events, .. } | Input::Api { index: i, events, .. } if *i == index => {
      Some(events.as_slice())
    },
    _ => None,
  });
  events.unwrap_or_default()
}

/// Whether `span` is printed on a promoted piece of its block, only marks and notes between: a team's own mark or note
/// (`the GAPS Team$^{2}$\thanks{…}`), never one a later piece or line prints (harness: `A \& the Speech Team \\ CERN,
/// Cat Cole\thanks{…}`, Ann Able's as rc6's).
fn on_a_piece(
  inputs: &[Input],
  span: Option<(usize, Option<(u16, u16)>)>,
  pieces: &[&Assoc],
) -> bool {
  let Some((index, Some((start, _)))) = span else {
    return false;
  };
  let events = events_of(inputs, index);
  let decoration = |e: &Event| match &e.kind {
    EventKind::Mark
    | EventKind::Note
    | EventKind::Layout
    | EventKind::Declaration
    | EventKind::Sep(SepKind::Gap { .. }) => true,
    EventKind::Name => !e.text.chars().any(char::is_alphanumeric),
    _ => false,
  };
  pieces.iter().any(|p| match p.source {
    Some((i, Some((_, end)))) if i == index && end <= start => events
      .iter()
      .all(|e| !e.span.is_some_and(|(s, _)| end <= s && s < start) || decoration(e)),
    _ => false,
  })
}

/// A text's words, lowercased.
fn words(text: &str) -> impl Iterator<Item = String> + '_ {
  text
    .split(|c: char| !c.is_alphanumeric())
    .filter(|w| !w.is_empty())
    .map(str::to_lowercase)
}

/// Whether `enumerated` is `rc6`'s reading plus the teams its names lines enumerated: each of rc6's creators kept in
/// order with its items but the team's piece and the team's own marks and notes, printed on it (no e-mail); a nameless
/// creator only where rc6 had it, or healed onto a team; each new creator a team whose piece rc6 read as the names'
/// collaboration or a column's own affiliation, holding only what moved, what it healed, or a copy of what a person
/// keeps that no other name or item prints the words of.
pub(super) fn promotion_only(inputs: &[Input], rc6: &Resolved, enumerated: &Resolved) -> bool {
  if (&rc6.policies, &rc6.continued, &rc6.drops, &rc6.mark_ledgers)
    != (
      &enumerated.policies,
      &enumerated.continued,
      &enumerated.drops,
      &enumerated.mark_ledgers,
    )
    || !lost(&rc6.dropped, &enumerated.dropped).is_some_and(|l| l.is_empty())
  {
    return false;
  }
  // (rc6's creators in order, each its reading's; the teams; rc6's nameless ones healed)
  let (mut pairs, mut teams, mut healed) = (Vec::new(), Vec::new(), Vec::new());
  let mut j = 0;
  for (c, creator) in enumerated.creators.iter().enumerate() {
    loop {
      match rc6.creators.get(j) {
        Some(was) if was.name == creator.name => {
          pairs.push((j, c));
          j += 1;
        },
        _ if team(creator) => teams.push(c),
        Some(was) if was.name.trim().is_empty() => {
          healed.push(j);
          j += 1;
          continue;
        },
        _ => return false,
      }
      break;
    }
  }
  if rc6.creators[j..]
    .iter()
    .any(|was| !was.name.trim().is_empty())
  {
    return false;
  }
  healed.extend(j..rc6.creators.len());
  let teams: Vec<&RCreator> = teams.iter().map(|&t| &enumerated.creators[t]).collect();
  // (each team's piece as rc6 read it)
  let mut pieces: Vec<&Assoc> = Vec::new();
  for team in &teams {
    let of_it: Vec<&Assoc> = pairs
      .iter()
      .flat_map(|&(was, _)| items(&rc6.creators[was]))
      .filter(|item| piece_of(item, team))
      .collect();
    if of_it.is_empty() {
      return false;
    }
    pieces.extend(of_it);
  }
  // (a run promotes all its teams or none: a names line's collaboration a person keeps in a block where a team was promoted
  // is a half-flip — `Ann Able -- the XYZ Collaboration and the ABC Collaboration`, its `;` form, advisor TEAMAUTH6 B)
  let half = pairs.iter().any(|&(_, c)| {
    items(&enumerated.creators[c]).any(|item| {
      item.role == "collaboration"
        && item.rule.starts_with("line:names collaboration")
        && teams
          .iter()
          .any(|t| item.source.is_some_and(|(at, _)| at == t.source))
    })
  });
  if half {
    return false;
  }
  let on_a_team = |item: &Assoc| teams.iter().any(|t| items(t).any(|i| same(i, item)));
  let mut moved: Vec<&Assoc> = Vec::new();
  let mut moved_marks: Vec<&NameMark> = Vec::new();
  for &(was, c) in &pairs {
    let (was, now) = (&rc6.creators[was], &enumerated.creators[c]);
    if (
      &was.joins,
      was.source,
      &was.role,
      was.call,
      was.corresponding,
    ) != (
      &now.joins,
      now.source,
      &now.role,
      now.call,
      now.corresponding,
    ) {
      return false;
    }
    let mut left: Vec<&NameMark> = now.marks.iter().collect();
    for mark in &was.marks {
      match left.iter().position(|m| *m == mark) {
        Some(at) => {
          left.remove(at);
        },
        None
          if teams.iter().any(|t| t.marks.contains(mark))
            && on_a_piece(inputs, mark.span.map(|s| (now.source, Some(s))), &pieces) =>
        {
          moved_marks.push(mark)
        },
        None => return false,
      }
    }
    if !left.is_empty() {
      return false;
    }
    for (from, kept, notes) in [
      (&was.affiliations, &now.affiliations, false),
      (&was.contacts, &now.contacts, false),
      (&was.notes, &now.notes, true),
    ] {
      let Some(gone) = lost(from, kept) else {
        return false;
      };
      for item in gone {
        let piece = pieces.iter().any(|p| same(p, item));
        // (a team's own: a note printed on its piece, or what its own mark requests — never a later remark's contact,
        // `A \& KM3NeT Consortium \& representing the LIGO …`, harness)
        let own = !mail(item)
          && on_a_team(item)
          && ((notes && on_a_piece(inputs, item.source, &pieces))
            || item
              .label
              .as_ref()
              .is_some_and(|l| moved_marks.iter().any(|m| &m.label == l)));
        if own {
          moved.push(item);
        } else if !piece {
          return false;
        }
      }
    }
  }
  // (a team holds what moved to it, what rc6's nameless creators or its block (the XML's trailing nameless creator) held,
  // or a copy of what a person keeps)
  let Some(block_heals) = lost(&rc6.block, &enumerated.block) else {
    return false;
  };
  let heals: Vec<&Assoc> = healed
    .iter()
    .flat_map(|&h| items(&rc6.creators[h]))
    .chain(block_heals)
    .collect();
  if !heals.iter().all(|item| on_a_team(item)) {
    return false;
  }
  // (a copy only of a line no other name or item prints the words of, a separator's "and" aside: the placed check counts
  // a name's print as the line's too, so `A \and B and the X teams \\ A` would place "A" more often than printed —
  // harness L10212, and its `\\ B and the X teams`)
  let printed_once = |item: &Assoc| {
    let others: BTreeSet<String> = rc6
      .creators
      .iter()
      .flat_map(|c| {
        let texts = items(c).filter(|i| !same(i, item)).map(|i| i.text.as_str());
        std::iter::once(c.name.as_str()).chain(texts)
      })
      .flat_map(words)
      .collect();
    !words(&item.text).all(|w| w == "and" || others.contains(&w))
  };
  let kept = |item: &Assoc| {
    !mail(item)
      && pairs
        .iter()
        .any(|&(_, c)| items(&enumerated.creators[c]).any(|i| same(i, item)))
      && printed_once(item)
  };
  let healed_marks: Vec<&NameMark> = healed
    .iter()
    .flat_map(|&h| &rc6.creators[h].marks)
    .collect();
  let teams_hold = teams.iter().all(|t| {
    items(t).all(|item| moved.iter().chain(&heals).any(|m| same(m, item)) || kept(item))
      && t.marks.iter().all(|m| {
        moved_marks.contains(&m)
          || healed_marks.contains(&m)
          || (!rc6.creators.iter().any(|c| c.marks.contains(m))
            && on_a_piece(inputs, m.span.map(|s| (t.source, Some(s))), &pieces))
      })
  });
  teams_hold && taken_alike(rc6, enumerated, &pairs)
}

/// Whether the entries the two readings took addresses out of are alike, each holder a creator kept in both.
fn taken_alike(rc6: &Resolved, enumerated: &Resolved, pairs: &[(usize, usize)]) -> bool {
  rc6.taken.len() == enumerated.taken.len()
    && rc6.taken.iter().zip(&enumerated.taken).all(|(was, now)| {
      (
        &was.source,
        &was.original,
        &was.stripped,
        &was.lined,
        &was.cuts,
        &was.runs,
      ) == (
        &now.source,
        &now.original,
        &now.stripped,
        &now.lined,
        &now.cuts,
        &now.runs,
      ) && was.holders.len() == now.holders.len()
        && was.holders.iter().zip(&now.holders).all(|(a, b)| {
          let holder = match (a.0, b.0) {
            (None, None) => true,
            (Some(a), Some(b)) => pairs.contains(&(a, b)),
            _ => false,
          };
          holder && a.1 == b.1 && same(&a.2, &b.2)
        })
    })
}
