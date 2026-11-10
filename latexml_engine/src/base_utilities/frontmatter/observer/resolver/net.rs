//! The reading's coordination and separator checks (brief N1; OXIDIZED_DESIGN_DIVERGENCES #980's safety net), beside the
//! word check (`words.rs`, their model): a pure pass over an author block's digested events and the reading
//! ([`Resolved`]), what TeX printed against what the reading makes of it, in the block's own terms. Perl has none: its
//! `\lx@add@authors` (Base_Utility.pool.ltxml:687-694) cuts by the spelling and checks only for a `{tabular}`.
//!
//! - **Marks** ([`uncoordinated_marks`]): a mark printed on a name whose label leads one of the block's lines reaches an item
//!   of that name's creator carrying it; and an item a line's leading mark labels, which a name prints too, is some
//!   creator's.
//! - **Separators** ([`merged_names`]): two Names a separator parts (`Sep`: `\and`, a comma, the word "and"; or a `Break`)
//!   are no one creator's name.
//!
//! Each is linear in the block (a label set's lookups, a word index of the names), as the reading is.

use std::collections::{BTreeMap, BTreeSet};

use super::{
  super::ledger::{Entry, Ledger, Outcome},
  labels::unprefixed,
  *,
};

/// A block's events, by ledger index, the blocks the reading reads (not cleared).
fn block_events(ledger: &Ledger, index: usize) -> Option<&[Event]> {
  match ledger.entries().get(index) {
    Some(Entry::Author(block)) if !block.cleared => match &block.outcome {
      Outcome::Events { events, .. } => Some(events),
      _ => None,
    },
    _ => None,
  }
}

/// The reading's creators by the block or call they come from, and the block's own items.
fn by_source(resolved: &Resolved) -> BTreeMap<usize, (Vec<&RCreator>, Vec<&Assoc>)> {
  let mut out: BTreeMap<usize, (Vec<&RCreator>, Vec<&Assoc>)> = BTreeMap::new();
  for c in &resolved.creators {
    out.entry(c.source).or_default().0.push(c);
  }
  for a in &resolved.block {
    if let Some((at, _)) = a.source
      && let Some((_, own)) = out.get_mut(&at)
    {
      own.push(a);
    }
  }
  out
}

/// A set of labels, matched as `labels::label_matches` matches a request and a label (equal, or equal past one prefix or
/// both), by lookups.
#[derive(Default)]
struct Labels<'a> {
  exact: BTreeSet<&'a str>,
  bare:  BTreeSet<&'a str>,
}

impl<'a> Labels<'a> {
  fn insert(&mut self, label: &'a str) {
    self.exact.insert(label);
    if let Some(bare) = unprefixed(label) {
      self.bare.insert(bare);
    }
  }

  fn matches(&self, label: &str) -> bool {
    let bare = unprefixed(label);
    self.exact.contains(label)
      || self.bare.contains(label)
      || bare.is_some_and(|bare| self.exact.contains(bare) || self.bare.contains(bare))
  }
}

/// The labels the marks of `events` print, those leading their line (an entry's: its first ink's run of marks, `$^1$ $^2$
/// Univ A`, 1703.06483) and the others (a name's or an entry's own after its text), as the resolver reads a mark
/// ([`mark_requests`]).
fn printed_labels(events: &[Event]) -> (BTreeSet<String>, BTreeSet<String>) {
  let (mut leading, mut other) = (BTreeSet::new(), BTreeSet::new());
  let mut leads = false;
  for e in events {
    match &e.kind {
      EventKind::Mark => {
        leads |= e.line_first;
        let into = if leads { &mut leading } else { &mut other };
        into.extend(mark_requests(e));
      },
      EventKind::Name | EventKind::Contact | EventKind::Note | EventKind::Break => leads = false,
      _ => {},
    }
  }
  (leading, other)
}

/// Per block (its ledger index) the reading of which fails the marks check, the labels that fail it: a creator's request
/// (a mark shown on its name, ruling 1) that a line of the block leads with, which no item the creator holds carries by its
/// own label; an item's label a line leads with and a name's mark prints, which no creator holds (the block's own); a mark a
/// creator requests printed inside an item, not leading it.
pub(in crate::base_utilities::frontmatter::observer) fn uncoordinated_marks(
  ledger: &Ledger,
  resolved: &Resolved,
) -> BTreeMap<usize, Vec<String>> {
  let mut out: BTreeMap<usize, Vec<String>> = BTreeMap::new();
  for (index, (creators, own)) in by_source(resolved) {
    let Some(events) = block_events(ledger, index) else {
      continue;
    };
    let (leading, other) = printed_labels(events);
    if leading.is_empty() {
      continue;
    }
    let (mut leads, mut others) = (Labels::default(), Labels::default());
    leading.iter().for_each(|l| leads.insert(l));
    other.iter().for_each(|l| others.insert(l));
    let (mut held, mut requested) = (Labels::default(), Labels::default());
    let mut failed: BTreeSet<String> = BTreeSet::new();
    for c in &creators {
      // (an item's own label, what the emitter answers a request by: an entry led by `$^1$ $^2$` labelled `1` alone leaves
      // the name marked 2 its mark printed, 1703.06483)
      let mut labels = Labels::default();
      let items = c.affiliations.iter().chain(&c.contacts).chain(&c.notes);
      for label in items.filter_map(|a| a.label.as_deref()) {
        labels.insert(label);
        held.insert(label);
      }
      for request in &c.requests {
        requested.insert(request);
        if leads.matches(request) && !labels.matches(request) {
          failed.insert(request.clone());
        }
      }
    }
    // (a mark a creator requests printed inside an item, not leading it: an entry the reading didn't part at its mark, an
    // e-mail line's `$^1$ann@x.org, $^2$bob@y.org` read into an affiliation, 1703.06483; the items' spans by their start,
    // the farthest end before each)
    let mut spans: Vec<(u16, u16)> = creators
      .iter()
      .flat_map(|c| c.affiliations.iter().chain(&c.contacts))
      .chain(own.iter().copied())
      .filter_map(|a| match a.source {
        Some((at, Some(span))) if at == index => Some(span),
        _ => None,
      })
      .collect();
    spans.sort_unstable();
    let reach: Vec<u16> = spans
      .iter()
      .scan(0u16, |far, &(_, to)| {
        *far = (*far).max(to);
        Some(*far)
      })
      .collect();
    for mark in events.iter().filter(|e| e.kind == EventKind::Mark) {
      let Some((from, to)) = mark.span else {
        continue;
      };
      let before = spans.partition_point(|&(start, _)| start < from);
      if before > 0 && reach[before - 1] >= to {
        failed.extend(
          mark_requests(mark)
            .into_iter()
            .filter(|label| requested.matches(label)),
        );
      }
    }
    for label in own.iter().filter_map(|a| a.label.as_deref()) {
      if leads.matches(label) && others.matches(label) && !held.matches(label) {
        failed.insert(label.to_string());
      }
    }
    if !failed.is_empty() {
      out.insert(index, failed.into_iter().collect());
    }
  }
  out
}

/// Per block (its ledger index) the reading of which fails the separators check, the pairs of names it joins: two Names
/// of its events a separator parts (a `Sep` other than a gap, or a `Break`, between them, only marks, notes and layout
/// beside it), whose words one creator's name holds both of, the second no qualifier or remark and the name joined by no
/// rule of the reading's (`RCreator::joins`) — or, each carrying its own mark, joined by any rule (review revZ18k B1); and a
/// name holding a label before a colon and a space.
pub(in crate::base_utilities::frontmatter::observer) fn merged_names(
  ledger: &Ledger,
  resolved: &Resolved,
) -> BTreeMap<usize, Vec<String>> {
  let words = |text: &str| -> Vec<String> { printed_words(text).map(str::to_lowercase).collect() };
  let mut out: BTreeMap<usize, Vec<String>> = BTreeMap::new();
  for (index, (creators, _)) in by_source(resolved) {
    let Some(events) = block_events(ledger, index) else {
      continue;
    };
    // (a name the reading joined across a separator by a rule of its own is explained ([`NameJoin`]): written surname first,
    // `Eckart, A.`, 1501.02171; a qualifier, a remark, a macro's, a layout's, a gap's or initials' join)
    let names: Vec<Vec<String>> = creators
      .iter()
      .map(|c| {
        if !c.joins.is_empty() {
          Vec::new()
        } else {
          words(&c.name)
        }
      })
      .collect();
    // (each word's names: a pair's candidates are the names holding its first word)
    let mut holding: BTreeMap<&str, BTreeSet<usize>> = BTreeMap::new();
    for (n, name) in names.iter().enumerate() {
      for word in name {
        holding.entry(word).or_default().insert(n);
      }
    }
    let holds = |name: &[String], part: &[String]| {
      let mut left = name.to_vec();
      part.iter().all(|w| match left.iter().position(|l| l == w) {
        Some(at) => {
          left.remove(at);
          true
        },
        None => false,
      })
    };
    let mut failed = Vec::new();
    let mut last: Option<(&Event, bool)> = None;
    for e in events {
      match &e.kind {
        EventKind::Name => {
          // (a qualifier or a remark after a comma is the name's, the reading's rule: `Robert W. Heath, Jr.`, 1004.2377)
          if let Some((before, true)) = last
            && !is_name_qualifier(&e.text)
            && !is_name_remark(&e.text)
          {
            let (a, b) = (words(&before.text), words(&e.text));
            let joined = [a.clone(), b.clone()].concat();
            let candidates = a.first().and_then(|w| holding.get(w.as_str()));
            if !b.is_empty()
              && candidates
                .is_some_and(|names_of| names_of.iter().any(|&n| holds(&names[n], &joined)))
            {
              failed.push(s!("{} | {}", before.text, e.text));
            }
          }
          last = Some((e, false));
        },
        // (a gap parts no names: `S. A. \,Levshakov`, 0911.3853; `Micha\l  \,\,Boczek`, 1902.08839)
        EventKind::Sep(SepKind::Gap { .. }) => {},
        EventKind::Sep(_) | EventKind::Break => {
          if let Some((before, _)) = last {
            last = Some((before, true));
          }
        },
        EventKind::Mark | EventKind::Note | EventKind::Layout | EventKind::Declaration => {},
        EventKind::Contact | EventKind::ApiCall => last = None,
      }
    }
    // (two names a separator parts, each carrying its own mark, are two people whatever rule of the reading's joined them:
    // a qualifier or a remark carries no mark of its own — `Y. Dong\affil{6}, \add[ver3]{S. Curry}\affil{1}`, 2308.02720,
    // review revZ18k B1, which the names line now reads as two names; any such join left is counted here)
    let all: Vec<Vec<String>> = creators.iter().map(|c| words(&c.name)).collect();
    // (each Name with whether a separator parts it from the Name before and whether a mark follows it before the next
    // separator; a contact or a call ends the run)
    let mut pieces: Vec<Option<(&Event, bool, bool)>> = Vec::new();
    let mut parted = false;
    for e in events {
      match &e.kind {
        EventKind::Name => {
          pieces.push(Some((e, parted, false)));
          parted = false;
        },
        EventKind::Mark => {
          if let Some(Some((_, _, marked))) = pieces.last_mut()
            && !parted
          {
            *marked = true;
          }
        },
        EventKind::Sep(SepKind::Gap { .. }) => {},
        EventKind::Sep(_) | EventKind::Break => parted = true,
        EventKind::Note | EventKind::Layout | EventKind::Declaration => {},
        EventKind::Contact | EventKind::ApiCall => {
          pieces.push(None);
          parted = false;
        },
      }
    }
    for pair in pieces.windows(2) {
      if let [Some((before, _, true)), Some((after, true, true))] = pair {
        // (each a word of its own: a bracket's `(` before a line's `$^1$` names no one, `($^1$) Univ A`)
        let (a, b) = (words(&before.text), words(&after.text));
        let joined = [a.clone(), b.clone()].concat();
        let pair = s!("{} | {}", before.text, after.text);
        if !a.is_empty()
          && !b.is_empty()
          && all.iter().any(|name| holds(name, &joined))
          && !failed.contains(&pair)
        {
          failed.push(pair);
        }
      }
    }
    // (a label printed before a colon inside a name: a role's heading read into it, `Editor: Andrei V. Gritsan`, 2205.07715;
    // `{\bf Authors:} F.~Arleo`, hep-ph0311131)
    for c in &creators {
      // (a colon a space follows: not a link's, `http://www.ligo.org`, gr-qc0505029)
      if let Some((head, tail)) = c.name.split_once(':')
        && printed_words(head).next().is_some()
        && tail.starts_with(char::is_whitespace)
        && printed_words(tail).next().is_some()
      {
        failed.push(s!("{head}: | {}", tail.trim()));
      }
    }
    if !failed.is_empty() {
      out.insert(index, failed);
    }
  }
  out
}
