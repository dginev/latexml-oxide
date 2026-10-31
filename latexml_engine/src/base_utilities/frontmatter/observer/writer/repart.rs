//! The writer's re-part (FM stage 7, "name parting last"; OXIDIZED_DESIGN_DIVERGENCES #928): where the resolver's
//! reading parts the names and today's (the old reader's) parted them otherwise, so that no creator pairs by name and
//! no item-wise operation ([`super::plan`]) applies, today's creators are written again from the reading — each named
//! reading creator one `ltx:creator` whose `ltx:personname` is the run of today's digested personname its words are
//! ([`slice_name`]: its fonts and notes kept, its marks out; a name holding a link is not written — the reading takes an
//! `\orcidlink` for the creator's ORCID and an `\href` for a url contact, which the cut name keeps inside it, so the
//! fail-safe takes it back or the plan declines, review rev94a r2 S2), then its affiliations, contacts and notes in print
//! order, and the author block's own on a nameless creator after them. An item is today's own, copied, where today's
//! holds one of its kind and text anywhere among the creators; else, an affiliation or a line the reading takes for a
//! legend (never a `\thanks` or `\footnote` of a name) made from the tokens of its entry, in today's name's font, an
//! affiliation without its entry's mark.
//!
//! Only where the reading refines today's names, across a boundary the digested structure shows (the plan declines
//! otherwise): every reading name is printed inside one of today's names — none is made of an item of today's, a url
//! column, an institution row —; every name of today's that reads as names, or does once the remark after it is set
//! aside, keeps one; an item the reading gives more authors than today's held it is a ruling's change; a creator of
//! today's becomes an item only on structural evidence — the item answers a mark, is a labelled line or row, or came
//! from an API call ([`structural`]) —, and no item read from its words alone is made ([`by_words`]: the classify
//! helpers' institution, neither carved from one name, `John Smith, University of Toronto`, nor made of creators
//! printed apart, `Bob Baker for the Pierre Auger Observatory` — review rev94a r3), nor one that may name a person or a
//! team from its place alone (a `tabular` row under a name, `Ann Able\\ Bob Baker`) or continued over a line that may
//! (`Resolved::continued`), nor one across an alignment's cells ([`super::plan::block_span_tokens`]' cell check — review
//! rev94a r4); no author is cut from a row of today's name under no names row ([`opens_a_row_under_no_names`] — r5);
//! no name or made item leaves a bracket open. Fail-safe, on the digested text: the new creators are built beside today's, and
//! kept only when they read as the reading ([`super::plan::difference`] finds nothing), every written name is one name
//! — it prints a word, and no separator outside its brackets ([`not_one_name`]: no author of invisible ink, no two
//! joined) —, they hold every word today's creators held, none duplicated — names and distinct items word for word, the
//! only words left out or added the separators the blocks printed between names ("and", `SepKind::Word`) —, no made
//! affiliation prints a mark ([`holds_mark`]) and no author keeping today's name loses the lead of an item the reading
//! cut ([`dispossessed`]); anything else removes them and leaves today's (1909.05311, 2609.32870, 2609.33104,
//! 2307.03384, 2308.10621, hep-lat0209026, hep-th9806168; declined: 1304.1574, 0811.3142's bracket in 94a; review
//! rev94a, its r2 B-1 and B-2). On any doubt today's stays.

use std::collections::BTreeMap;

use super::{
  super::{
    events::{EventKind, SepKind, is_separator_word},
    ledger::{Entry, Outcome},
    resolver::{
      label_matches, mark_labels, mark_requests, may_name_a_person, name_before_remark,
      read_events, reads_as_names,
    },
  },
  apply::{elements, make, place, reclaim_ids, subtree_ids},
  dom::{
    Creator, Front, Kind, LAYOUT_PARTS, Seg, is, norm, read_creator, segments, text_without_marks,
    words_without_marks,
  },
  plan::{
    Key, Made, Owner, Place, api_call, block_items, block_span_tokens, creator_items, difference,
    pair_names, place_of, reading_order,
  },
  *,
};

/// What one item of a re-parted creator is written from.
#[derive(Debug, Clone)]
enum Item {
  /// a copy of today's item of the same kind and text
  Copy(Node),
  /// made from the tokens of its span: an affiliation of `role`, or a legend note
  Make {
    made:   Made,
    role:   String,
    tokens: Tokens,
    /// a legend's label, the mark its note shows
    label:  Option<String>,
  },
}

/// One re-parted creator: its name's tokens, the attributes of today's creator its name was printed in, its items.
#[derive(Debug, Clone)]
struct NewCreator {
  /// its index among the resolved creators (its owner in the reading's print order)
  index:     usize,
  name:      String,
  /// today's creator its name was printed in, and the run of that name's words it is (`personname_words`)
  from:      (usize, (usize, usize)),
  /// the marks it requests that no entry answers, shown in its name as today's shows them (`show_unanswered_marks`)
  marks:     Vec<String>,
  /// the reading joined a qualifier into the name (`Robert W. Heath, Jr.`, `Eve Ern | PhD`) or wrote it surname first
  /// (`Della Valle, M.`): the comma or the separator glyph before it is the name's
  qualified: bool,
  attrs:     TagAttrs,
  items:     Vec<Item>,
}

/// The re-part of a paper's creators: the reading's creators and the author block's own items.
#[derive(Debug, Clone)]
pub(super) struct Repart {
  creators:   Vec<NewCreator>,
  block:      Vec<Item>,
  /// the words the blocks printed as separators between names (`SepKind::Word`: "and"), which no creator keeps
  separators: Vec<String>,
}

impl Repart {
  /// Whether it makes an item of the author block's own from the queued tokens (a creator of no name would hold it).
  pub(super) fn makes_block_items(&self) -> bool {
    self
      .block
      .iter()
      .any(|item| matches!(item, Item::Make { .. }))
  }
}

/// No re-part, and why (the `frontmatter` debug feature).
fn declined(reason: &str) -> Option<Repart> {
  DebugFeature!("frontmatter", "FRONT writer: no re-part, {reason}");
  None
}

/// The span of a block's entry the reading keys `key`, its source `span` its first event's (an entry of several pieces,
/// `$^1$Department of Physics, Univ X, Japan`, is sourced at its first Name: 2307.03384): through the events printed
/// after it, until their ink joined is the entry's text — over the lines an unmarked line continued — `None` when it
/// never is (a note, a mark, or other ink first).
fn entry_span(index: usize, span: (u16, u16), key: &str) -> Option<(u16, u16)> {
  with_ledger(|ledger| {
    let Some(Entry::Author(block)) = ledger.entries.get(index) else {
      return None;
    };
    let Outcome::Events { events, .. } = &block.outcome else {
      return None;
    };
    let first = events
      .iter()
      .position(|e| e.kind != EventKind::Layout && e.span.is_some_and(|s| s.0 == span.0))?;
    let (mut joined, mut end) = (String::new(), span.1);
    for event in &events[first..] {
      match &event.kind {
        EventKind::Name | EventKind::Contact => joined.push_str(&norm(&event.text)),
        // (the word "and" inside an entry is the entry's: `Computer Science and Engineering`, 1304.1574)
        EventKind::Sep(SepKind::Word(word)) => joined.push_str(&norm(word)),
        EventKind::Sep(_) | EventKind::Break | EventKind::Layout => continue,
        _ => return None,
      }
      if let Some((_, e)) = event.span {
        end = end.max(e);
      }
      if joined == key {
        return Some((span.0, end));
      }
      if !key.starts_with(&joined) {
        return None;
      }
    }
    None
  })
}

/// The words a block printed as separators between names (`SepKind::Word`, the word "and"), each once per print.
fn separator_words(index: usize) -> Vec<String> {
  fn walk(events: &[Event], out: &mut Vec<String>) {
    for event in events {
      if let EventKind::Sep(SepKind::Word(word)) = &event.kind {
        out.extend(words(word));
      }
      walk(&event.inner, out);
    }
  }
  with_ledger(|ledger| {
    let mut out = Vec::new();
    if let Some(Entry::Author(block)) = ledger.entries.get(index)
      && let Outcome::Events { events, .. } = &block.outcome
    {
      walk(events, &mut out);
    }
    out
  })
}

/// The words of a text: its runs of letters and digits, lowercased.
fn words(text: &str) -> Vec<String> {
  text
    .split(|c: char| !c.is_alphanumeric())
    .filter(|w| !w.is_empty())
    .map(str::to_lowercase)
    .collect()
}

/// What an item of the reading is written from: today's item of its key, else its span's tokens — `None` when neither.
fn item_source(key: &Key, assoc: &Assoc, todays: &BTreeMap<Key, Node>) -> Option<Item> {
  if let Some(node) = todays.get(key) {
    return Some(Item::Copy(node.clone()));
  }
  let made = match key.0 {
    Kind::Affiliation => Made::Affiliation,
    // (a line of the block read as a legend; a note of a name, `\thanks`, `\footnote`, is today's own, never remade)
    Kind::Note
      if !assoc
        .rule
        .split_whitespace()
        .any(|t| t.starts_with("note:")) =>
    {
      Made::Legend
    },
    _ => return None,
  };
  let tokens = match assoc.source? {
    (index, Some(span)) => {
      let span = entry_span(index, span, &key.1).unwrap_or(span);
      // (a made affiliation carries no entry mark, as today's copies hold none; a legend keeps its mark's ink, the
      // label-text ruling — review rev94a S3)
      block_span_tokens(index, span, None, made == Made::Legend)?
    },
    (index, None) => api_call(index)?.content,
  };
  Some(Item::Make {
    made,
    role: assoc.role.clone(),
    tokens,
    label: assoc.label.clone(),
  })
}

/// A resolved creator's items, one per kind and text (the one a ruling decided kept), as the writer compares them
/// (`plan::wanted`; rev92c S2).
fn items_once(c: &RCreator) -> Vec<(Key, &Assoc)> {
  let mut out: Vec<(Key, &Assoc)> = Vec::new();
  for (key, assoc) in creator_items(c) {
    match out.iter_mut().find(|(k, _)| *k == key) {
      Some(held) if !writes(held.1) && writes(assoc) => held.1 = assoc,
      Some(_) => {},
      None => out.push((key, assoc)),
    }
  }
  out
}

/// An assoc's place in print order: a note's after the block's (`plan::place_of`), then its block or call, and its
/// span's start.
fn print_place(kind: &Kind, assoc: &Assoc) -> (bool, usize, u16) {
  match place_of(assoc, kind) {
    Some((note, index, (_, start))) => (note, index, start),
    None => (true, usize::MAX, u16::MAX),
  }
}

/// The re-part of today's creators by the reading, when today's names do not pair with the reading's and every
/// named reading creator's name and item has something to be written from; `None` otherwise.
pub(super) fn plan(resolved: &Resolved, front: &Front) -> Option<Repart> {
  if pair_names(resolved, front).is_some() {
    return None;
  }
  // (today's items, by the comparison's key: the first of each)
  let mut todays: BTreeMap<Key, Node> = BTreeMap::new();
  let mut holders: BTreeMap<Key, usize> = BTreeMap::new();
  for creator in front.creators.iter().chain(&front.nameless) {
    for item in &creator.items {
      if !item.key.is_empty() {
        let key = (item.kind.clone(), item.key.clone());
        *holders.entry(key.clone()).or_default() += 1;
        todays.entry(key).or_insert_with(|| item.node.clone());
      }
    }
  }
  // (an item the reading gives more authors than today's held it — one a name held, given to its row: none — only by
  // a ruling's change, as the writer moves any item: a role or note line under a row of names given to every name,
  // 77t's rank 15, stays as today's)
  // (the owners counted once each, an item being one per owner and text: rev92c S2)
  let mut owners: BTreeMap<Key, Vec<(Owner, &Assoc)>> = BTreeMap::new();
  for (i, c) in resolved.creators.iter().enumerate() {
    if c.name.trim().is_empty() {
      continue;
    }
    for (key, assoc) in items_once(c) {
      owners
        .entry(key)
        .or_default()
        .push((Owner::Creator(i), assoc));
    }
  }
  for (key, assoc) in block_items(resolved) {
    let wants = owners.entry(key).or_default();
    if !wants.iter().any(|(o, _)| *o == Owner::Block) {
      wants.push((Owner::Block, assoc));
    }
  }
  // (nor by today's API: an unlabelled annotation of the run of new authors is every name of the run's, as today's API
  // attaches it — today's old reader merged names across two runs, so fewer of today's creators held it: 1805.12376)
  if let Some((key, _)) = owners.iter().find(|(key, assocs)| {
    assocs.len() > holders.get(*key).copied().unwrap_or(0).max(1)
      && !assocs.iter().all(|(_, a)| writes(a))
      && !assocs
        .iter()
        .all(|(_, a)| rule_has(a, "owner:annotate:run"))
  }) {
    return declined(&s!("an item given more authors by no ruling: {key:?}"));
  }
  // (the names refine today's: each reading name printed inside today's names, never an item of today's made a name —
  // a url column, an institution row; and each of today's names that reads as names keeps one, never made an item)
  let mut todays_names: Vec<(Vec<String>, usize)> = front
    .creators
    .iter()
    .map(|t| (personname_words(&t.node), 0))
    .collect();
  let mut kept = vec![false; front.creators.len()];
  let mut parted: Vec<(usize, &RCreator)> = Vec::new();
  let mut runs: BTreeMap<usize, (usize, (usize, usize))> = BTreeMap::new();
  for (index, c) in resolved.creators.iter().enumerate() {
    if c.name.trim().is_empty() {
      continue;
    }
    let name = words(&c.name);
    let Some(at) =
      todays_names
        .iter_mut()
        .position(|(today, from)| match find_run(today, *from, &name) {
          Some(end) => {
            *from = end;
            true
          },
          None => false,
        })
    else {
      return declined(&s!("a name today's names do not print: {}", c.name));
    };
    kept[at] = true;
    let end = todays_names[at].1;
    runs.insert(index, (at, (end - name.len(), end)));
    parted.push((at, c));
    // (a name of the reading's own whose brackets do not close is cut inside one: `(on behalf of the H1`, 0811.3142 —
    // review rev94a B2)
    if !balanced(&c.name)
      && !front
        .creators
        .iter()
        .any(|t| norm(&t.name) == norm(&c.name))
    {
      return declined(&s!("a name whose brackets do not close: {}", c.name));
    }
  }
  // (a one-word name parted from today's name at a comma is the given name of a name written surname first, `Smith,
  // Jr., John`: uncertain, so today's stays)
  if let Some((_, c)) = parted.iter().find(|(at, c)| {
    words(&c.name).len() == 1
      && c.rule.split_whitespace().any(|t| t == "sep:comma")
      && parted.iter().filter(|(other, _)| other == at).count() > 1
  }) {
    return declined(&s!("a one-word name parted at a comma: {}", c.name));
  }
  // (a name the reading cuts from one of today's names at a line or row of it — no separator before it on its line —
  // is an author only on structure: a names row above it ending with the name before (`Ann Able, Bob Baker,\\ Cid
  // Cole`), a mark or a note on it (`Ann Able$^1$\\ Bob Baker$^2$`); a line's font, its words read as a name, are no
  // boundary the page shows, so today's one creator stays — `$^1$Univ X,\\ \bf City Z` in a `tabular` made "City Z" an
  // author, review rev94a r5 t06: the mirror of the person no item is made of, below)
  if let Some((_, c)) = parted.iter().enumerate().find_map(|(k, (at, c))| {
    let before = parted[..k].iter().rev().find(|(other, _)| other == at)?;
    (!c.rule.contains("sep:")
      && c.requests.is_empty()
      && c.notes.is_empty()
      && opens_a_row_under_no_names(c, before.1))
    .then_some(parted[k])
  }) {
    return declined(&s!("a name cut from a row on no structure: {}", c.name));
  }
  // (a name with a remark after it is a name too: `Bob Baker for the ATLAS Collaboration`, `Bob Baker (MIT)` — review
  // rev94a r2 B-1)
  if let Some(lost) =
    front.creators.iter().zip(&kept).find(|(t, kept)| {
      !**kept && (reads_as_names(&t.name) || name_before_remark(&t.name).is_some())
    })
  {
    return declined(&s!(
      "a name of today's the reading makes none: {}",
      lost.0.name
    ));
  }
  // (a creator of today's the reading makes no name of becomes an item only on structural evidence — an entry a mark
  // labels, a labelled line or row, an API call ([`structural`]): a made item without it — an unmarked line, a legend
  // no mark labels — may hold no word of one, nor be held by one; the classify helpers' items, the word readings, were
  // given back or are declined below ([`concede`], [`by_words`]) — review rev94a r3)
  let unkept: Vec<&Vec<String>> = todays_names
    .iter()
    .zip(&kept)
    .filter(|((name, _), kept)| !**kept && !name.is_empty())
    .map(|((name, _), _)| name)
    .collect();
  let takes_a_creator = |assoc: &Assoc| {
    let text = words(&assoc.text);
    !structural(assoc)
      && !text.is_empty()
      && unkept
        .iter()
        .any(|name| find_run(name, 0, &text).is_some() || find_run(&text, 0, name).is_some())
  };
  // (nor is a row or line of today's name that may name a person or a team made an item on no structural evidence: a
  // row of a `tabular` in `\author` printed under a name — `Ann Able\\ Bob Baker`, `Bob Baker for the Pierre Auger
  // Observatory`, `the PE11 team of Kepler` — is no affiliation by its place alone; today's one creator stays, as a
  // line of today's creators does. A word reading, read only to decline — review rev94a r4)
  // (each line of an entry continued over lines — `resolved.continued` —, a line under the first one as a row of its
  // own: `Ann Able\\ {\it Univ X}\\ Bob Baker\\ {\it Univ Y}` in a `tabular`, whose third row the reading continues
  // into the second's entry now no cell's font sets it apart)
  let lines_of = |assoc: &Assoc| -> Vec<String> {
    resolved
      .continued
      .iter()
      .flatten()
      .find(|lines| norm(&lines.join(" ")) == norm(&assoc.text))
      .map_or_else(Vec::new, |lines| lines[1..].to_vec())
  };
  let reads_as_a_person = |assoc: &Assoc| {
    let person = !structural(assoc)
      && (may_name_a_person(&assoc.text)
        || [" for ", " on behalf of "].iter().any(|said| {
          assoc
            .text
            .split_once(said)
            .is_some_and(|(name, _)| reads_as_names(name))
        })
        || words(&assoc.text).iter().any(|w| {
          [
            "team",
            "collaboration",
            "collaborations",
            "consortium",
            "group",
          ]
          .contains(&w.as_str())
        }));
    // (a line an entry is continued over, labelled or not: the label is its first line's)
    person || lines_of(assoc).iter().any(|line| may_name_a_person(line))
  };
  // (the labels the reading's entries answer)
  let answered: Vec<String> = resolved
    .creators
    .iter()
    .flat_map(|c| c.affiliations.iter().chain(&c.contacts).chain(&c.notes))
    .chain(&resolved.block)
    .filter_map(|a| a.label.as_deref())
    .flat_map(mark_labels)
    .collect();
  let mut creators = Vec::new();
  let mut blocks: Vec<usize> = Vec::new();
  for (index, c) in resolved.creators.iter().enumerate() {
    if c.name.trim().is_empty() {
      continue;
    }
    let name = c.name.clone();
    if !blocks.contains(&c.source) {
      blocks.push(c.source);
    }
    let mut assocs = items_once(c);
    assocs.sort_by_key(|(key, a)| print_place(&key.0, a));
    let mut items = Vec::new();
    for (key, assoc) in assocs {
      let Some(item) = item_source(&key, assoc, &todays) else {
        return declined(&s!("an item with nothing to write it from: {key:?}"));
      };
      if let Item::Make { .. } = item {
        if !balanced(&assoc.text) {
          return declined(&s!("an item whose brackets do not close: {}", assoc.text));
        }
        // (an item read from its words alone — the classify helpers' institution — is no boundary the page shows, so
        // none is made: amsart's `\author{John Smith, University of Toronto}` is one creator as written, as Perl's —
        // review rev94a B1 —, and today's creators the classify helpers read as an institution stay creators — r3)
        if by_words(assoc) {
          return declined(&s!("an item read from its words alone: {}", assoc.text));
        }
        if takes_a_creator(assoc) {
          return declined(&s!(
            "a creator of today's made an item on no structural evidence: {}",
            assoc.text
          ));
        }
        if reads_as_a_person(assoc) {
          return declined(&s!(
            "a person made an item on no structural evidence: {}",
            assoc.text
          ));
        }
      }
      items.push(item);
    }
    // (the attributes of today's creator whose name printed it: its role, its separator)
    let key = norm(&c.name);
    let attrs = front
      .creators
      .iter()
      .find(|t| norm(&t.name).contains(&key))
      .or(front.creators.first())
      .map(|t| creator_attributes(&t.node))
      .unwrap_or_default();
    // (a request no entry of the reading answers stays printed on the name: `Hal Hunt\ \ $^{1}$` with no `1` line)
    let marks: Vec<String> = c
      .requests
      .iter()
      .filter(|r| !answered.contains(r))
      .filter_map(|r| unanswered_mark_text(r))
      .fold(Vec::new(), |mut marks, m| {
        if !marks.contains(&m) {
          marks.push(m);
        }
        marks
      });
    let Some(&from) = runs.get(&index) else {
      return declined(&s!("a name with no run: {name}"));
    };
    let qualified = c
      .rule
      .split_whitespace()
      .any(|t| t == "name:qualifier" || t == "name:reversed");
    creators.push(NewCreator {
      index,
      name,
      from,
      marks,
      qualified,
      attrs,
      items,
    });
  }
  if creators.is_empty() {
    return declined("no named creator");
  }
  let mut assocs = block_items(resolved);
  assocs.sort_by_key(|(key, a)| print_place(&key.0, a));
  let mut block = Vec::new();
  for (key, assoc) in assocs {
    let Some(item) = item_source(&key, assoc, &todays) else {
      return declined(&s!("a block item with nothing to write it from: {key:?}"));
    };
    if matches!(item, Item::Make { .. })
      && (by_words(assoc) || takes_a_creator(assoc) || reads_as_a_person(assoc))
    {
      return declined(&s!(
        "a creator of today's made a block item on no structural evidence: {}",
        assoc.text
      ));
    }
    block.push(item);
  }
  let separators = blocks.into_iter().flat_map(separator_words).collect();
  Some(Repart { creators, block, separators })
}

/// Whether a reading creator's name opens a line or row of its block that no names row above it leads to: the Name
/// event of its block printing its first words follows a line break, and the line above does not end with the name
/// before it, `before` — past the marks, notes, separators and spacing after that name (`Ann Able, Bob Baker,\\ Cid
/// Cole`; `\textbf{Xiaochen Li} \quad \\ \textbf{Michael L.~Littman}`, 2407.03321, whose `\quad` the break clears).
/// A line above that ends with other ink (`$^1$Univ X,\\ City Z`: an institution, its comma its own) is no names row.
/// A name on its line after other ink (a mark, a note) is no row's.
fn opens_a_row_under_no_names(c: &RCreator, before: &RCreator) -> bool {
  let (name, before) = (words(&c.name), words(&before.name));
  with_ledger(|ledger| {
    let Some(Entry::Author(block)) = ledger.entries.get(c.source) else {
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
    // (back from it, layout and spacing aside: the line break, then the ink the line above ends with)
    let mut back = events[..at].iter().rev().filter(|e| {
      !matches!(
        e.kind,
        EventKind::Layout | EventKind::Sep(SepKind::Gap { .. })
      )
    });
    if !back.next().is_some_and(|e| e.kind == EventKind::Break) {
      return false;
    }
    let ink = back.find(|e| {
      !matches!(
        e.kind,
        EventKind::Mark | EventKind::Note | EventKind::Sep(_)
      )
    });
    !ink.is_some_and(|e| {
      let text = words(&e.text);
      e.kind == EventKind::Name && !text.is_empty() && before.ends_with(&text)
    })
  })
}

/// Whether a written name is no one name: it prints nothing, or, outside its brackets, a separator the blocks part
/// names at — the word "and", an `&`, and, unless the reading joined a qualifier or a reversed name into it
/// (`qualified`: `Robert W. Heath, Jr.`, `Eve Ern — PhD`), a comma or a separator glyph standing alone
/// ([`is_separator_word`]). The re-part invents no author of invisible ink and joins no two (review rev94a r2 B-2:
/// 0909.3056's and 1507.02405's `\phantom{agga}` an author, hep-th9806168's `\phantom{a}and Klaus Kirsten` one).
fn not_one_name(text: &str, qualified: bool) -> bool {
  let mut depth = 0i32;
  let mut outside = String::new();
  for c in text.chars() {
    match c {
      '(' | '[' => depth += 1,
      ')' | ']' => depth -= 1,
      _ if depth <= 0 => outside.push(c),
      _ => {},
    }
  }
  words(text).is_empty()
    || outside.contains('&')
    || (!qualified && outside.contains(','))
    || outside.split_whitespace().any(|w| {
      let w = w.trim_matches(|c: char| ",.;".contains(c));
      w.eq_ignore_ascii_case("and") || (!qualified && is_separator_word(w))
    })
}

/// Whether a text's brackets close: as many `(` as `)` and `[` as `]`, none closed before it opened.
fn balanced(text: &str) -> bool {
  let (mut round, mut square) = (0i32, 0i32);
  for c in text.chars() {
    match c {
      '(' => round += 1,
      ')' => round -= 1,
      '[' => square += 1,
      ']' => square -= 1,
      _ => {},
    }
    if round < 0 || square < 0 {
      return false;
    }
  }
  round == 0 && square == 0
}

/// The reading the writer writes from: the shadow reading, but where it took one whole creator of today's — no name of
/// the reading printed in it — for another creator's affiliation from its words alone ([`by_words`]), that creator
/// given back, in print order after the name holding it, as today's printed it apart (`conceded`), with the ink it
/// printed ([`give_back_ink`]). A word reading never demotes a creator of today's, a person or a team (`Bob Baker for
/// the Pierre Auger Observatory`, 1109.6488's `the PE11 team of {\it Kepler} WG\#1`): the reading's other items stand,
/// so 1109.6488's `*` legend stays García's note, the name showing its mark (review rev94a r3). An item made of several
/// of today's creators or carved from one stays the reading's, and the re-part declines it ([`by_words`]). `None` —
/// the writer writes nothing, today's stays — when a conceded creator's ink is in doubt (review rev94a r4 B1).
pub(super) fn concede(resolved: &Resolved, front: &Front) -> Option<Resolved> {
  let names: Vec<Vec<String>> = front
    .creators
    .iter()
    .map(|t| personname_words(&t.node))
    .collect();
  let named: Vec<Vec<String>> = resolved
    .creators
    .iter()
    .map(|c| words(&c.name))
    .filter(|w| !w.is_empty())
    .collect();
  let mut free: Vec<bool> = names
    .iter()
    .map(|name| {
      !name.is_empty()
        && !named
          .iter()
          .any(|reading| find_run(name, 0, reading).is_some())
    })
    .collect();
  let mut out = resolved.clone();
  let mut at = 0;
  while at < out.creators.len() {
    let mut given = Vec::new();
    if !out.creators[at].name.trim().is_empty() {
      let source = out.creators[at].source;
      let mut k = 0;
      while k < out.creators[at].affiliations.len() {
        let a = &out.creators[at].affiliations[k];
        let text = words(&a.text);
        let whole = if by_words(a) && a.label.is_none() {
          (0..names.len()).find(|&j| free[j] && names[j] == text)
        } else {
          None
        };
        let Some(j) = whole else {
          k += 1;
          continue;
        };
        free[j] = false;
        let a = out.creators[at].affiliations.remove(k);
        let mut creator = RCreator {
          name: a.text.clone(),
          rule: "line:names conceded".into(),
          source: a.source.map_or(source, |(index, _)| index),
          ..RCreator::default()
        };
        if give_back_ink(&mut out.creators[at], &mut creator, &a).is_none() {
          DebugFeature!(
            "frontmatter",
            "FRONT writer: the ink of a conceded creator in doubt: {}",
            a.text
          );
          return None;
        }
        given.push(creator);
      }
    }
    let count = given.len();
    for (k, creator) in given.into_iter().enumerate() {
      out.creators.insert(at + 1 + k, creator);
    }
    at += 1 + count;
  }
  Some(out)
}

/// What a piece of a names line printed after it, before the next separator: the labels its marks request, and the
/// spans of its notes and contacts — the ink the shadow reading gave the name before it when it took the piece for that
/// name's institution (`names_line_events`: a mark, a note, a contact to the last name made). `None` when the reading's
/// attribution is in doubt: no Name event of the piece's span and text, ink between the separator before it and it (a
/// prefix mark, which the reading holds for the next name), or ink after the separator after it before the next name
/// (one the source distance gave either side).
fn piece_ink(index: usize, span: (u16, u16), text: &str) -> Option<PieceInk> {
  let ink = |e: &Event| {
    matches!(
      e.kind,
      EventKind::Mark | EventKind::Note | EventKind::Contact
    )
  };
  with_ledger(|ledger| {
    let Some(Entry::Author(block)) = ledger.entries.get(index) else {
      return None;
    };
    let Outcome::Events { events, .. } = &block.outcome else {
      return None;
    };
    let events = read_events(events);
    let at = events
      .iter()
      .position(|e| e.kind == EventKind::Name && e.span == Some(span))?;
    if norm(&events[at].text) != norm(text) {
      return None;
    }
    if events[..at]
      .iter()
      .rev()
      .take_while(|e| ink(e) || e.kind == EventKind::Layout)
      .any(ink)
    {
      return None;
    }
    let (mut marks, mut items) = (Vec::new(), Vec::new());
    let mut next = at + 1;
    while let Some(e) = events.get(next) {
      match e.kind {
        EventKind::Layout => {},
        EventKind::Mark => marks.extend(mark_requests(e)),
        EventKind::Note if e.inkless => {},
        EventKind::Note | EventKind::Contact => items.push(e.span?),
        _ => break,
      }
      next += 1;
    }
    if events
      .get(next)
      .is_some_and(|e| matches!(e.kind, EventKind::Sep(_)))
      && events[next + 1..]
        .iter()
        .take_while(|e| ink(e) || matches!(e.kind, EventKind::Layout | EventKind::Sep(_)))
        .any(ink)
    {
      return None;
    }
    Some(PieceInk { marks, spans: items })
  })
}

/// The ink a piece of a names line printed after it ([`piece_ink`]): the labels its marks request, and the spans of
/// its notes and contacts.
struct PieceInk {
  marks: Vec<String>,
  spans: Vec<(u16, u16)>,
}

/// A creator's item lists: its affiliations, contacts and notes.
fn lists(c: &mut RCreator) -> [&mut Vec<Assoc>; 3] {
  [&mut c.affiliations, &mut c.contacts, &mut c.notes]
}

/// Give a conceded creator `piece` the ink it printed, which the reading gave `holder`, the name before it: its marks
/// (from the holder's requests), its notes and contacts, and the items its marks answer — a copy where the holder's
/// own marks answer one too, else moved — so that the holder keeps only its own (`Ann Able$^{1}$, Kitt Peak
/// Observatory$^{2}$` under `$^1$Univ X\\ $^2$Univ Y`: Univ Y the observatory's, as today's — review rev94a r4 B1).
/// `None` when it cannot be told apart ([`piece_ink`]; a mark no request of the holder's is; a mark of the piece that
/// none of the items answers while the holder keeps one its own marks do not answer).
fn give_back_ink(holder: &mut RCreator, piece: &mut RCreator, assoc: &Assoc) -> Option<()> {
  let (index, Some(span)) = assoc.source? else {
    return None;
  };
  let PieceInk { marks, spans } = piece_ink(index, span, &assoc.text)?;
  for mark in &marks {
    let k = holder
      .requests
      .iter()
      .position(|r| label_matches(r, mark))?;
    piece.requests.push(holder.requests.remove(k));
  }
  for span in spans {
    let source = Some((index, Some(span)));
    let mut found = false;
    for (from, into) in lists(holder).into_iter().zip(lists(piece)) {
      let (theirs, ours): (Vec<Assoc>, Vec<Assoc>) = std::mem::take(from)
        .into_iter()
        .partition(|a| a.source == source);
      found |= !theirs.is_empty();
      into.extend(theirs);
      *from = ours;
    }
    if !found {
      return None;
    }
  }
  if piece.requests.is_empty() {
    return Some(());
  }
  // (an item's labels are its legend's, as printed: the reading keys a legend answering several marks by the one its
  // owner requested — `$^{1,2}$Univ X` under `Ann Able$^{1}$` is labelled `1` —, so the piece's `2` read it as none
  // of its own; review rev94a r5 g13)
  let answers = |requests: &[String], item: &Assoc| {
    item_labels(item)
      .iter()
      .any(|l| requests.iter().any(|r| label_matches(r, l)))
  };
  let (own, shown) = (holder.requests.clone(), piece.requests.clone());
  for (from, into) in lists(holder).into_iter().zip(lists(piece)) {
    let mut kept = Vec::new();
    for item in std::mem::take(from) {
      if answers(&shown, &item) {
        into.push(item.clone());
        if !answers(&own, &item) {
          continue;
        }
      }
      kept.push(item);
    }
    *from = kept;
  }
  let answered = |c: &RCreator, mark: &String| {
    [&c.affiliations, &c.contacts, &c.notes]
      .into_iter()
      .flatten()
      .any(|a| answers(std::slice::from_ref(mark), a))
  };
  let foreign = [&holder.affiliations, &holder.contacts, &holder.notes]
    .into_iter()
    .flatten()
    .any(|a| a.label.is_some() && !answers(&own, a));
  if foreign && shown.iter().any(|m| !answered(piece, m)) {
    return None;
  }
  Some(())
}

/// The labels an item of the reading answers: those its legend's Mark prints — the Mark event its line opens with, just
/// before its span in its block (Layout events aside), every label of it, a number range spelled out (`$^{1,2}$`,
/// `$^{1-3}$`, `$^{1,*}$`: [`mark_requests`]) — else its own label's ([`mark_labels`]); none for an item of no label.
fn item_labels(item: &Assoc) -> Vec<String> {
  let Some(label) = item.label.as_deref() else {
    return Vec::new();
  };
  let printed = item.source.and_then(|(index, span)| {
    let (start, _) = span?;
    with_ledger(|ledger| {
      let Some(Entry::Author(block)) = ledger.entries.get(index) else {
        return None;
      };
      let Outcome::Events { events, .. } = &block.outcome else {
        return None;
      };
      read_events(events)
        .iter()
        .rev()
        .find(|e| e.kind != EventKind::Layout && e.span.is_some_and(|(_, end)| end <= start))
        .filter(|e| e.kind == EventKind::Mark)
        .map(mark_requests)
    })
  });
  let own = mark_labels(label);
  match printed {
    // (the legend's Mark, when it prints the item's own label: the event is the item's)
    Some(printed)
      if own
        .iter()
        .all(|l| printed.iter().any(|p| label_matches(p, l))) =>
    {
      printed
    },
    _ => own,
  }
}

/// Whether a reading's rule holds `tag`.
fn rule_has(assoc: &Assoc, tag: &str) -> bool { assoc.rule.split_whitespace().any(|t| t == tag) }

/// Whether a reading's item was read from its words alone: a rule of the classify helpers (`classify:institution`, a
/// piece of a row of names `reads_as_affiliation` takes for an institution).
fn by_words(assoc: &Assoc) -> bool {
  assoc
    .rule
    .split_whitespace()
    .any(|t| t.starts_with("classify:"))
}

/// Whether a reading's item rests on structural evidence, so that a creator of today's may become it: it answers a mark
/// or is a labelled line or row (its label), or it came from an API call of an item (`\affil`, `\address`,
/// `\institute`, `\email`, …: [`item_call`]; a creator's call, `\lx@add@creator`, read into a name and its institution
/// is a word reading) — and no word-based rule read it ([`by_words`]). A creator printed apart and read as an institution by
/// its words is no item (`John Smith` and `University of Toronto`; `Bob Baker for the Pierre Auger Observatory`):
/// person or place, the digested structure does not say, so today's stays (review rev94a r3).
fn structural(assoc: &Assoc) -> bool {
  !by_words(assoc)
    && (assoc
      .label
      .as_deref()
      .is_some_and(|l| l.chars().any(|c| !c.is_whitespace()))
      || assoc.source.is_some_and(|(index, _)| item_call(index)))
}

/// Whether the ledger entry at `index` is an API call of an item: a contact or a note, an annotation of the creators
/// (`\lx@annotate@frontmatter{ltx:creator}{ltx:contact}…`) — not a creator's own call (`\lx@add@creator`, tag
/// `ltx:creator`) nor its name's.
fn item_call(index: usize) -> bool {
  api_call(index).is_some_and(|call| call.tag != "ltx:creator" && call.tag != "ltx:personname")
}

/// The words of a creator's personname, its marks out, a layout's cells parting words.
fn personname_words(node: &Node) -> Vec<String> {
  elements(node)
    .iter()
    .filter(|child| document::get_node_qname(child) == pin("ltx:personname"))
    .flat_map(|child| words(&words_without_marks(child)))
    .collect()
}

/// Where the run `name` ends in `words` at or after `from`, if it is there.
fn find_run(words: &[String], from: usize, name: &[String]) -> Option<usize> {
  if name.is_empty() {
    return None;
  }
  words
    .get(from..)?
    .windows(name.len())
    .position(|run| run == name)
    .map(|at| from + at + name.len())
}

/// A creator's attributes, the internal ones and its ids aside.
fn creator_attributes(node: &Node) -> TagAttrs {
  let mut attrs = TagAttrs::default();
  for (key, value) in node.get_attributes() {
    if !key.starts_with('_') && key != "id" && key != "xml:id" {
      attrs.insert(key, value);
    }
  }
  attrs
    .entry("role".to_string())
    .or_insert_with(|| "author".to_string());
  attrs
}

/// The words a set of creators holds: each name's, and each distinct item's (by kind and text) once — what a
/// creator's name printed is counted where it now stands; a layout's cells and rows part words
/// (`dom::words_without_marks`).
fn creator_words(creators: &[Creator]) -> Vec<String> {
  let mut seen: Vec<Key> = Vec::new();
  let mut out = Vec::new();
  for creator in creators {
    out.extend(personname_words(&creator.node));
    for item in &creator.items {
      let key = (item.kind.clone(), item.key.clone());
      if item.key.is_empty() || seen.contains(&key) {
        continue;
      }
      seen.push(key);
      out.extend(words(&words_without_marks(&item.node)));
    }
  }
  out.sort();
  out
}

/// Insert a new, empty `ltx:creator` of `attrs` before `anchor`, in the font of `like` (today's creator it stands for)
/// — never the font current at the document's end, where the writer runs (review rev94a S1).
fn new_creator(
  document: &mut Document,
  anchor: &Node,
  attrs: &TagAttrs,
  like: &Node,
) -> Result<Option<Node>> {
  let Some(parent) = anchor.get_parent() else {
    return Ok(None);
  };
  let entry = TagData {
    tag:     "ltx:creator".to_string(),
    attr:    attrs.clone(),
    content: Vec::new(),
  };
  let saved = document.get_node().clone();
  document.set_node(&parent);
  document.set_box_to_absorb(
    Tbox::new(
      pin!(""),
      lookup_font(),
      None,
      Tokens!(T_SPACE!()),
      SymHashMap::default(),
    )
    .into(),
  );
  let inserted = insert_frontmatter_entry(document, &entry);
  document.expire_box_to_absorb();
  document.set_node(&saved);
  inserted?;
  let Some(mut creator) = elements(&parent)
    .pop()
    .filter(|n| document::get_node_qname(n) == pin("ltx:creator"))
  else {
    return Ok(None);
  };
  let mut anchor = anchor.clone();
  if anchor.add_prev_sibling(&mut creator).is_err() {
    return Ok(None);
  }
  document.copy_node_font(like, &mut creator)?;
  Ok(Some(creator))
}

/// A new `ltx:element` in `like`'s namespace.
fn new_element(document: &Document, like: &Node, name: &str) -> Result<Node> {
  let mut node = Node::new(name, None, document.get_document())?;
  if let Some(namespace) = like.get_namespace() {
    let _ = node.set_namespace(&namespace);
  }
  Ok(node)
}

/// The words of a text with where each starts and ends, in characters: [`words`]' runs.
fn word_spans(text: &[char]) -> Vec<(usize, usize)> {
  let mut out = Vec::new();
  let mut start = None;
  for (i, c) in text.iter().enumerate() {
    match (c.is_alphanumeric(), start) {
      (true, None) => start = Some(i),
      (false, Some(s)) => {
        out.push((s, i));
        start = None;
      },
      _ => {},
    }
  }
  if let Some(s) = start {
    out.push((s, text.len()));
  }
  out
}

/// `into` (a new, empty `ltx:personname`) made the run `run` of the words of today's personname `from`, as its digested
/// content printed it — its fonts and a note set in the name kept (review rev94a S1, S2: a re-parted name set as
/// a string took the end of the document's font, `fontsize="90%"` after a `\small` there, 1607.00911; and lost
/// 1909.05311's `\footnotemark[1]`) — its marks out (as the reading has them), a layout's parts unwrapped (a name in a
/// cell of a tabular, 2609.32870), the name's own leading and trailing punctuation (`Jr.`, a closing bracket) kept.
/// Today's notes it kept, or `None` when the run cuts through a formula or no run is there (`into` emptied).
fn slice_name(
  document: &mut Document,
  from: &Node,
  into: &mut Node,
  run: (usize, usize),
  name: &str,
) -> Result<Option<Vec<Node>>> {
  document.append_clone(into, from.get_child_nodes())?;
  let copied = segments(into, true);
  let source = segments(from, true);
  let texts: Vec<Vec<char>> = copied
    .iter()
    .map(|seg| seg.text().chars().collect())
    .collect();
  let stream: Vec<char> = texts.concat();
  let spans = word_spans(&stream);
  let fail = |document: &mut Document, into: &Node| {
    for child in into.get_child_nodes() {
      document.remove_node(child);
    }
    Ok(None)
  };
  if copied.len() != source.len() || run.0 >= run.1 || run.1 > spans.len() {
    return fail(document, into);
  }
  let (mut start, mut end) = (spans[run.0].0, spans[run.1 - 1].1);
  // (the name's own punctuation before its first word and after its last, where the text prints it there)
  let name: Vec<char> = name.trim().chars().collect();
  let lead = name.iter().take_while(|c| !c.is_alphanumeric()).count();
  let trail = name
    .iter()
    .rev()
    .take_while(|c| !c.is_alphanumeric())
    .count();
  if lead > 0 && start >= lead && stream[start - lead..start] == name[..lead] {
    start -= lead;
  }
  if trail > 0
    && name.len() > trail
    && stream.get(end..end + trail) == Some(&name[name.len() - trail..])
  {
    end += trail;
  }
  // (a note set after the name, before the next one's first word, is the name's: `Daya Guo\footnotemark[1]`)
  let next = spans.get(run.1).map_or(stream.len(), |w| w.0);
  let mut kept = Vec::new();
  let mut at = 0;
  for ((seg, text), original) in copied.iter().zip(&texts).zip(&source) {
    let (s, e) = (at, at + text.len());
    at = e;
    match seg {
      Seg::Text(node) => {
        let (from, to) = (s.max(start), e.min(end));
        let mut node = node.clone();
        if from >= to {
          node.unlink();
        } else if (from, to) != (s, e) {
          let part: String = text[from - s..to - s].iter().collect();
          let _ = node.set_content(&part);
        }
      },
      Seg::Atom(node, _) => {
        if s >= start && e <= end {
          continue;
        }
        if e > start && s < end {
          return fail(document, into);
        }
        document.remove_node(node.clone());
      },
      Seg::Hidden(node) => document.remove_node(node.clone()),
      Seg::Note(node) => {
        if s > start && s <= next {
          if let Seg::Note(original) = original {
            kept.push(original.clone());
          }
        } else {
          document.remove_node(node.clone());
        }
      },
      Seg::Space(Some(node)) => {
        if !(s > start && s < end) {
          document.remove_node(node.clone());
        }
      },
      Seg::Space(None) => {},
    }
  }
  // (what the cut left empty taken, a layout's parts unwrapped)
  loop {
    let empty: Vec<Node> = document
      .findnodes("descendant::*", Some(into))
      .into_iter()
      .filter(|n| {
        !is(n, "ltx:break")
          && n
            .get_child_nodes()
            .iter()
            .all(|c| c.get_type() == Some(NodeType::TextNode) && c.get_content().trim().is_empty())
      })
      .collect();
    if empty.is_empty() {
      break;
    }
    for node in empty {
      document.remove_node(node);
    }
  }
  for part in document
    .findnodes("descendant::*", Some(into))
    .into_iter()
    .rev()
  {
    if LAYOUT_PARTS
      .iter()
      .chain(&["ltx:tbody", "ltx:thead", "ltx:tfoot"])
      .any(|q| is(&part, q))
    {
      let mut part = part;
      for mut child in part.get_child_nodes() {
        child.unlink();
        let _ = part.add_prev_sibling(&mut child);
      }
      document.remove_node(part);
    }
  }
  // (a box's geometry is the layout's, no name's: `\makebox{\begin{tabular}…}`'s `width`, `align` — a text wrapper
  // that was only a box, in its parent's font, unwrapped; one of a font of its own, `\textsc{…}`, kept: 2609.33104)
  for mut node in document
    .findnodes("descendant::*", Some(into))
    .into_iter()
    .rev()
  {
    let mut boxed = false;
    for attribute in [
      "width",
      "height",
      "depth",
      "align",
      "vattach",
      "xoffset",
      "yoffset",
      "pad-width",
      "pad-height",
      "framed",
    ] {
      if node.has_attribute(attribute) {
        let _ = node.remove_attribute(attribute);
        boxed = true;
      }
    }
    let parent_font = node.get_parent().and_then(|p| p.get_attribute("_font"));
    if boxed
      && is(&node, "ltx:text")
      && node.get_attributes().keys().all(|key| key.starts_with('_'))
      && (node.get_attribute("_font").is_none() || node.get_attribute("_font") == parent_font)
    {
      for mut child in node.get_child_nodes() {
        child.unlink();
        let _ = node.add_prev_sibling(&mut child);
      }
      document.remove_node(node);
    }
  }
  // (the spaces a layout set around its cells, at the name's edges, are none of it)
  let texts = document.findnodes("descendant::text()", Some(into));
  if let Some(first) = texts.first() {
    let _ = first.clone().set_content(first.get_content().trim_start());
  }
  if let Some(last) = texts.last() {
    let _ = last.clone().set_content(last.get_content().trim_end());
  }
  if words(&words_without_marks(into)).is_empty() {
    return fail(document, into);
  }
  Ok(Some(kept))
}

/// A written creator's items in the reading's print order, by the writer's own pass (`apply::place`; rev92c S1): its
/// notes stay where they were written, as the pass leaves them.
fn in_print_order(owner: &Node, who: Owner, order: &BTreeMap<(Owner, Key), Place>) {
  let items: Vec<Node> = read_creator(owner)
    .items
    .into_iter()
    .filter(|item| item.movable && item.kind != Kind::Note)
    .map(|item| item.node)
    .collect();
  for item in items {
    place(owner, who, &item, order);
  }
}

/// Write `items` into `owner`, but a copy of a note its name holds already, an item made digested in `font` (the
/// font today's name was digested in: not the document's end's, where the writer runs — review rev94a S1); whether
/// every affiliation made carries no mark (a mark in a made affiliation is another entry's, joined: review rev94a B3).
fn write_items(
  document: &mut Document,
  owner: &Node,
  items: &[Item],
  in_name: &[Node],
  font: &Font,
) -> Result<bool> {
  let mut clean = true;
  for item in items {
    if let Item::Copy(node) = item
      && in_name.contains(node)
    {
      continue;
    }
    match item {
      Item::Copy(node) => {
        let mut owner = owner.clone();
        document.append_clone(&mut owner, vec![node.clone()])?;
      },
      Item::Make { made, role, tokens, label } => {
        DebugFeature!(
          "frontmatter",
          "FRONT writer: re-part makes {made:?} {role} from {}",
          tokens.clone().untex_string()
        );
        bgroup();
        assign_value("font", font.clone(), Some(Scope::Local));
        let made_now = make(
          document,
          owner,
          *made,
          tokens,
          None,
          role,
          None,
          label.as_deref(),
        );
        egroup()?;
        made_now?;
        if let Some(new) = elements(owner).pop() {
          trim_breaks(document, &new);
          if *made == Made::Affiliation {
            drop_leading_marks(document, &new);
          }
        }
        if *made == Made::Affiliation
          && let Some(new) = elements(owner).pop()
          && holds_mark(&new)
        {
          DebugFeature!(
            "frontmatter",
            "FRONT writer: re-part made a marked affiliation: {}",
            new.get_content()
          );
          clean = false;
        }
      },
    }
  }
  Ok(clean)
}

/// The line breaks and spaces a made item opens or ends with taken, as today's cleans an entry's trailing break
/// (`clean_trailing_break`): the `\\` a class's in-name `\affiliation{…}` prints before its lines (asme2e,
/// 2107.11510 — review rev94a S6).
fn trim_breaks(document: &mut Document, node: &Node) {
  for last in [false, true] {
    let mut at = node.clone();
    loop {
      let children = at.get_child_nodes();
      let edge = if last {
        children
          .into_iter()
          .rev()
          .find(|c| c.get_type() != Some(NodeType::TextNode) || !c.get_content().trim().is_empty())
      } else {
        children
          .into_iter()
          .find(|c| c.get_type() != Some(NodeType::TextNode) || !c.get_content().trim().is_empty())
      };
      match edge {
        Some(child) if is(&child, "ltx:break") => document.remove_node(child),
        Some(child) if is(&child, "ltx:text") => at = child,
        _ => break,
      }
    }
  }
  // (and the space it ends with, inside its last group: `{\it Department …, Japan }`, a group the entry now opens with
  // its switch — 2307.03384)
  if let Some(mut last) = document.findnodes("descendant::text()", Some(node)).pop() {
    let content = last.get_content();
    if content.ends_with(char::is_whitespace) {
      let _ = last.set_content(content.trim_end());
    }
  }
}

/// The marks a made affiliation opens with taken: its entry's label, which today's copies do not print either (an
/// `\affil{$^1$…}` in `\author`, whose span 94b's reading opens at its mark — OD #937 — review rev94a S3).
fn drop_leading_marks(document: &mut Document, node: &Node) {
  for seg in segments(node, false) {
    match seg {
      Seg::Hidden(mark) if !is(&mark, "ltx:tags") => document.remove_node(mark),
      Seg::Text(text) if text.get_content().trim().is_empty() => {},
      Seg::Text(mut text) => {
        let _ = text.set_content(text.get_content().trim_start());
        break;
      },
      _ => break,
    }
  }
}

/// Whether `node` prints a mark: a superscript or a formula its text reads without ([`segments`]).
fn holds_mark(node: &Node) -> bool {
  segments(node, false)
    .iter()
    .any(|seg| matches!(seg, Seg::Hidden(n) if !is(n, "ltx:tags")))
}

/// Carry out the re-part: the new creators built before today's first, kept — today's taken, their copied items' ids
/// given back — only when they read as the reading and hold every word of today's creators; else taken again. Whether
/// they were kept.
pub(super) fn apply(
  document: &mut Document,
  resolved: &Resolved,
  front: &Front,
  repart: &Repart,
) -> Result<bool> {
  let old: Vec<Node> = document.findnodes("//ltx:creator", None);
  let Some(anchor) = old.first().cloned() else {
    return Ok(false);
  };
  let order = reading_order(resolved);
  // (what is built is speculative until kept: the diagnostics its digestion raised are the document's only if it is —
  // a made item taken back leaves none, review rev94a r4 B2; one that raised an Error is taken back)
  let hold = util::logger::DiagnosticsHold::begin();
  let mut built: Vec<Node> = Vec::new();
  let mut named: Vec<Creator> = Vec::new();
  let mut nameless: Vec<Creator> = Vec::new();
  let mut complete = true;
  let mut notes_kept: Vec<Node> = Vec::new();
  for creator in &repart.creators {
    let like = front
      .creators
      .get(creator.from.0)
      .map_or(&anchor, |t| &t.node);
    let Some(mut node) = new_creator(document, &anchor, &creator.attrs, like)? else {
      complete = false;
      break;
    };
    built.push(node.clone());
    let (at, run) = creator.from;
    let Some(today) = front.creators.get(at) else {
      complete = false;
      break;
    };
    let mut personname = new_element(document, &node, "personname")?;
    node.add_child(&mut personname)?;
    let sources: Vec<Node> = elements(&today.node)
      .into_iter()
      .filter(|n| is(n, "ltx:personname"))
      .collect();
    // (the name's own attributes, its font among them, as today's digested it)
    let font = match sources.first() {
      Some(source) => {
        for (key, value) in source.get_attributes() {
          if key != "id" && key != "xml:id" {
            personname.set_attribute(&key, &value)?;
          }
        }
        document.get_node_font(source).clone()
      },
      None => document.get_node_font(&today.node).clone(),
    };
    let in_name = match sources.as_slice() {
      [source] => slice_name(document, source, &mut personname, run, &creator.name)?,
      _ => None,
    };
    let in_name = match in_name {
      Some(notes) => notes,
      None => {
        // (no run of today's digested name to keep: its printed text, in no font of its own)
        DebugFeature!(
          "frontmatter",
          "FRONT writer: re-part sets {} as text",
          creator.name
        );
        personname.append_text(&creator.name)?;
        Vec::new()
      },
    };
    // (one name each, printed: never a name of none, nor one holding a separator the blocks part names at — review
    // rev94a r2 B-2)
    if not_one_name(&text_without_marks(&personname), creator.qualified) {
      DebugFeature!(
        "frontmatter",
        "FRONT writer: re-part wrote no one name: {}",
        text_without_marks(&personname)
      );
      complete = false;
    }
    // (a mark the name's own note prints is shown by it: `Daya Guo\footnotemark[1]`'s `*`, 1909.05311)
    let asterisk = |m: &str| m.replace('\u{2217}', "*");
    let shown: Vec<String> = in_name
      .iter()
      .filter_map(|note| note.get_attribute("mark"))
      .map(|m| asterisk(&m))
      .collect();
    let marks: Vec<&str> = creator
      .marks
      .iter()
      .filter(|m| !shown.contains(&asterisk(m)))
      .map(String::as_str)
      .collect();
    if !marks.is_empty() {
      let mut sup = new_element(document, &personname, "sup")?;
      sup.append_text(&marks.join(","))?;
      personname.add_child(&mut sup)?;
    }
    complete &= write_items(document, &node, &creator.items, &in_name, &font)?;
    notes_kept.extend(in_name);
    in_print_order(&node, Owner::Creator(creator.index), &order);
    named.push(read_creator(&node));
  }
  if complete && !repart.block.is_empty() {
    let mut attrs = TagAttrs::default();
    attrs.insert("role".to_string(), "author".to_string());
    let like = front.nameless.first().map_or(&anchor, |c| &c.node);
    let font = document.get_node_font(like).clone();
    match new_creator(document, &anchor, &attrs, like)? {
      Some(node) => {
        built.push(node.clone());
        complete &= write_items(document, &node, &repart.block, &[], &font)?;
        in_print_order(&node, Owner::Block, &order);
        nameless.push(read_creator(&node));
      },
      None => complete = false,
    }
  }
  // (what was built reads as the reading, and holds today's words, the separators aside, and no other)
  let today: Vec<Creator> = front
    .creators
    .iter()
    .chain(&front.nameless)
    .cloned()
    .collect();
  let mut lost = creator_words(&today);
  let new_words = creator_words(&[named.clone(), nameless.clone()].concat());
  let mut separators = repart.separators.clone();
  let mut added = Vec::new();
  for word in new_words {
    match lost.iter().position(|w| *w == word) {
      Some(at) => {
        lost.remove(at);
      },
      None => added.push(word),
    }
  }
  // (a separator word is the block's print either way: one today's old reader read as a separator and the reading
  // reads inside an entry, `Computer Science and Engineering` — 1304.1574 —, or the reverse; each print once)
  let mut unprinted = |word: &String| match separators.iter().position(|s| s == word) {
    Some(at) => {
      separators.remove(at);
      false
    },
    None => true,
  };
  lost.retain(&mut unprinted);
  added.retain(&mut unprinted);
  let built_front = Front {
    creators: named
      .into_iter()
      .filter(|c| !c.name.trim().is_empty())
      .collect(),
    nameless,
  };
  let reads = difference(resolved, &built_front);
  let dispossessed = dispossessed(&today, &built_front.creators);
  let errors = hold.errors_raised();
  if !complete
    || !lost.is_empty()
    || !added.is_empty()
    || reads.is_some()
    || dispossessed.is_some()
    || errors > 0
  {
    DebugFeature!(
      "frontmatter",
      "FRONT writer: re-part taken back: lost {lost:?}, added {added:?}, {reads:?}, {dispossessed:?}, {errors} errors"
    );
    for node in built {
      document.remove_node(node);
    }
    // (a Fatal stays Fatal: logged without an `Err`, it is the document's, as `defer` commits one — review rev94a r5)
    if hold.fatal_raised() {
      hold.commit();
    } else {
      hold.discard();
    }
    return Ok(false);
  }
  hold.commit();
  // (today's creators taken, with the ids of the items copied from them: each copy takes its source's back)
  let mut taken = Vec::new();
  for item in repart
    .creators
    .iter()
    .flat_map(|c| &c.items)
    .chain(&repart.block)
  {
    if let Item::Copy(node) = item {
      let ids: Vec<(SymStr, String)> = subtree_ids(document, node)
        .into_iter()
        .map(|(holder, id)| (document::get_node_qname(&holder), id))
        .collect();
      if !taken.contains(&ids) {
        taken.push(ids);
      }
    }
  }
  for note in &notes_kept {
    let ids: Vec<(SymStr, String)> = subtree_ids(document, note)
      .into_iter()
      .map(|(holder, id)| (document::get_node_qname(&holder), id))
      .collect();
    if !taken.contains(&ids) {
      taken.push(ids);
    }
  }
  for node in old {
    document.remove_node(node);
  }
  for ids in &taken {
    reclaim_ids(document, ids)?;
  }
  Ok(true)
}

/// A name of today's that the re-part keeps as it was but whose creator holds less than today's gave it: an item of
/// today's creator, cut into the reading's entries (no written creator holds it whole), some word of which before its
/// first mark none of the new one's items holds (an institute line joined to the next and given to one author, the
/// other's taken away: 2308.11662 — review rev94a B3). An item kept whole goes where the reading gives it, by the call
/// or line it prints after: today's merged name put every item after it one author down (`CP-PACS Collaboration :
/// T.~Umeda\rlap,\address{…}`, T. Umeda's address today's S. Aoki's, hep-lat0209026). `None` when every author kept
/// what was theirs.
fn dispossessed(today: &[Creator], built: &[Creator]) -> Option<String> {
  let whole: Vec<(&Kind, &String)> = built
    .iter()
    .flat_map(|c| &c.items)
    .map(|item| (&item.kind, &item.key))
    .collect();
  for old in today.iter().filter(|c| !c.name.trim().is_empty()) {
    let Some(new) = built.iter().find(|c| norm(&c.name) == norm(&old.name)) else {
      continue;
    };
    let held: Vec<String> = new
      .items
      .iter()
      .flat_map(|item| words(&words_without_marks(&item.node)))
      .collect();
    for item in old.items.iter().filter(|item| {
      item.movable && !item.key.is_empty() && !whole.contains(&(&item.kind, &item.key))
    }) {
      // (what an item prints before its first mark is the creator's own; a piece after a mark is that mark's entry,
      // which the reading gives whoever requests it: today's one contact joining `Sun Yat-sen University $^4$Peking
      // University`, the second Jingjing Xu's — 1909.05311)
      let lead: String = segments(&item.node, true)
        .iter()
        .take_while(|seg| !matches!(seg, Seg::Hidden(n) if !is(n, "ltx:tags")))
        .map(Seg::text)
        .collect();
      if let Some(word) = words(&lead).into_iter().find(|w| !held.contains(w)) {
        return Some(s!("{} loses {word:?} of {}", old.name, item.key));
      }
    }
  }
  None
}

#[cfg(test)]
mod tests {
  use super::{super::super::ledger::ApiCall, *};

  /// A creator of today's becomes an item only on structural evidence: a label, an API call; never a classify rule
  /// (review rev94a r3).
  #[test]
  fn an_item_rests_on_structure() {
    let assoc = |label: Option<&str>, rule: &str| Assoc {
      role:   "affiliation".into(),
      text:   "University of Toronto".into(),
      label:  label.map(str::to_string),
      rule:   rule.into(),
      source: None,
      target: None,
    };
    assert!(structural(&assoc(Some("1"), "mark:parted")));
    assert!(!structural(&assoc(Some(" "), "mark:parted")));
    assert!(!structural(&assoc(None, "line:names classify:institution")));
    assert!(!structural(&assoc(
      Some("1"),
      "line:names classify:institution"
    )));
    assert!(!structural(&assoc(None, "line:unmarked R12:row-above")));
    assert!(by_words(&assoc(None, "line:names classify:institution")));
    assert!(!by_words(&assoc(Some("1"), "mark:parted")));
    // (an API call's item: an item's call is evidence, a creator's call is not — its content's parting is a reading;
    // no call at the index, none — review rev94a r4 NIT)
    let call = |tag: &str, parent: &str| {
      Entry::Api(ApiCall {
        index:          0,
        tag:            tag.into(),
        role:           "affiliation".into(),
        epoch:          0,
        queue_position: 0,
        after_block:    None,
        inside:         false,
        attrs:          String::new(),
        parent:         parent.into(),
        content:        Tokens::new(Vec::new()),
        events:         Vec::new(),
        rehearsed:      true,
        cleared:        false,
      })
    };
    with_ledger(|ledger| {
      ledger.entries = vec![
        call("ltx:contact", "ltx:creator"),
        call("ltx:creator", ""),
        call("ltx:personname", "ltx:creator"),
      ];
    });
    let from = |index: usize, rule: &str| Assoc {
      source: Some((index, None)),
      ..assoc(None, rule)
    };
    assert!(structural(&from(0, "api:affiliation")));
    assert!(!structural(&from(1, "api:creator")));
    assert!(!structural(&from(2, "api:creator")));
    assert!(!structural(&from(3, "api:affiliation")));
    assert!(!structural(&from(
      0,
      "api:affiliation classify:institution"
    )));
    with_ledger(|ledger| ledger.entries.clear());
  }

  /// Only an item made from the queued tokens is one of the block's own a creator of no name would hold
  /// (`writer::writes_a_nameless_creator`); a copy of today's is not (review rev94b r4 N1).
  #[test]
  fn a_block_item_is_made_or_copied() {
    let doc = libxml::tree::Document::new().expect("a document");
    let node = Node::new("contact", None, &doc).expect("a node");
    let repart = |block: Vec<Item>| Repart {
      creators: Vec::new(),
      block,
      separators: Vec::new(),
    };
    assert!(!repart(Vec::new()).makes_block_items());
    assert!(!repart(vec![Item::Copy(node.clone())]).makes_block_items());
    assert!(
      repart(vec![Item::Copy(node), Item::Make {
        made:   Made::Affiliation,
        role:   "affiliation".to_string(),
        tokens: Tokens::new(Vec::new()),
        label:  None,
      }])
      .makes_block_items()
    );
  }

  /// A text's brackets close, in order (review rev94a B2).
  #[test]
  fn brackets_close() {
    assert!(balanced(
      "Ann Able (on behalf of the H1 and ZEUS collaborations)"
    ));
    assert!(!balanced("Laurent Schoeffel (on behalf of the H1"));
    assert!(!balanced("ZEUS collaborations)"));
    assert!(!balanced("a) b ("));
    assert!(balanced("[for the C and D groups]"));
  }

  /// A written name is one name: printed, no separator outside its brackets (review rev94a r2 B-2).
  #[test]
  fn a_written_name_is_one_name() {
    assert!(!not_one_name("Klaus Kirsten", false));
    assert!(!not_one_name(
      "Ann Able (on behalf of the A and B teams)",
      false
    ));
    assert!(!not_one_name("Robert W. Heath, Jr.", true));
    assert!(!not_one_name("Eve Ern — PhD", true));
    assert!(not_one_name("Eve Ern — Fay Fox", false));
    assert!(not_one_name("Ann Able and Bob Baker, Jr.", true));
    assert!(not_one_name("", false));
    assert!(not_one_name(" ", false));
    assert!(not_one_name("and Klaus Kirsten", false));
    assert!(not_one_name("Ann Able & Bob Baker", false));
    assert!(not_one_name("Ann Able, Bob Baker", false));
    assert!(not_one_name("Ulderico Dardano - Silvana Rinauro", false));
  }

  /// A name's words found as a run of today's name's words, from where the last name found ended; a name longer than
  /// what is left is none (it panicked, slicing past the end).
  #[test]
  fn a_name_is_found_as_a_run_of_words() {
    let today = words("Ann Able, Bob Baker");
    assert_eq!(find_run(&today, 0, &words("Ann Able")), Some(2));
    assert_eq!(find_run(&today, 2, &words("Bob Baker")), Some(4));
    assert_eq!(find_run(&today, 2, &words("Ann Able")), None);
    assert_eq!(find_run(&today, 0, &words("Ann Able And Bob Baker")), None);
    assert_eq!(find_run(&today[..1], 0, &words("Ann Able")), None);
    assert_eq!(find_run(&today, 5, &words("Bob")), None);
  }
}
