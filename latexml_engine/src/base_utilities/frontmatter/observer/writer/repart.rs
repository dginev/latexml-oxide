//! The writer's re-part (FM stage 7, "name parting last"; OXIDIZED_DESIGN_DIVERGENCES #928): where the resolver's
//! reading parts the names and today's (the old reader's) parted them otherwise, so that no creator pairs by name and
//! no item-wise operation ([`super::plan`]) applies, today's creators are written again from the reading — each named
//! reading creator one `ltx:creator` whose `ltx:personname` is its name as printed (the reading's digested text, its
//! marks out), then its affiliations, contacts and notes in print order, and the author block's own on a nameless
//! creator after them. An item is today's own, copied, where today's holds one of its kind and text anywhere among the
//! creators; else, an affiliation or a line the reading takes for a legend (never a `\thanks` or `\footnote` of a
//! name) made from the tokens of its entry.
//!
//! Only where the reading refines today's names (the plan declines otherwise): every reading name is printed inside one
//! of today's names — none is made of an item of today's, a url column, an institution row —; every name of today's
//! that reads as names keeps one; an item the reading gives more authors than today's held it is a ruling's change.
//! Fail-safe, on the digested text: the new creators are built beside today's, and kept only when they read as the
//! reading ([`super::plan::difference`] finds nothing) and hold every word today's creators held, none duplicated —
//! names and distinct items word for word, the only words left out or added the separators the blocks printed between
//! names ("and", `SepKind::Word`) —; anything else removes them and leaves today's (1909.05311, 2609.32870,
//! 2609.33104, 2307.03384, 1502.03246, 2308.10621; declined: 2405.08158, 1602.05576, 1304.1574).

use std::collections::BTreeMap;

use super::{
  super::{
    events::{EventKind, SepKind},
    ledger::{Entry, Outcome},
    resolver::{mark_labels, reads_as_names},
  },
  apply::{elements, make, place, reclaim_ids, subtree_ids},
  dom::{Creator, Front, Kind, norm, read_creator, words_without_marks},
  plan::{
    Key, Made, Owner, Place, api_call, block_items, block_span_tokens, creator_items, difference,
    pair_names, reading_order,
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
  },
}

/// One re-parted creator: its name's tokens, the attributes of today's creator its name was printed in, its items.
#[derive(Debug, Clone)]
struct NewCreator {
  /// its index among the resolved creators (its owner in the reading's print order)
  index: usize,
  name:  String,
  /// the marks it requests that no entry answers, shown in its name as today's shows them (`show_unanswered_marks`)
  marks: Vec<String>,
  attrs: TagAttrs,
  items: Vec<Item>,
}

/// The re-part of a paper's creators: the reading's creators and the author block's own items.
#[derive(Debug, Clone)]
pub(super) struct Repart {
  creators:   Vec<NewCreator>,
  block:      Vec<Item>,
  /// the words the blocks printed as separators between names (`SepKind::Word`: "and"), which no creator keeps
  separators: Vec<String>,
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
      block_span_tokens(index, span, None)?
    },
    (index, None) => api_call(index)?.content,
  };
  Some(Item::Make {
    made,
    role: assoc.role.clone(),
    tokens,
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

/// An assoc's place in print order: its block or call, and its span's start.
fn print_place(assoc: &Assoc) -> (usize, u16) {
  match assoc.source {
    Some((index, span)) => (index, span.map_or(0, |s| s.0)),
    None => (usize::MAX, u16::MAX),
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
  if let Some((key, _)) = owners.iter().find(|(key, assocs)| {
    assocs.len() > holders.get(*key).copied().unwrap_or(0).max(1)
      && !assocs.iter().all(|(_, a)| writes(a))
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
  for c in &resolved.creators {
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
    parted.push((at, c));
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
  if let Some(lost) = front
    .creators
    .iter()
    .zip(&kept)
    .find(|(t, kept)| !**kept && reads_as_names(&t.name))
  {
    return declined(&s!(
      "a name of today's the reading makes none: {}",
      lost.0.name
    ));
  }
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
    assocs.sort_by_key(|(_, a)| print_place(a));
    let mut items = Vec::new();
    for (key, assoc) in assocs {
      let Some(item) = item_source(&key, assoc, &todays) else {
        return declined(&s!("an item with nothing to write it from: {key:?}"));
      };
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
    creators.push(NewCreator {
      index,
      name,
      marks,
      attrs,
      items,
    });
  }
  if creators.is_empty() {
    return declined("no named creator");
  }
  let mut assocs = block_items(resolved);
  assocs.sort_by_key(|(_, a)| print_place(a));
  let mut block = Vec::new();
  for (key, assoc) in assocs {
    let Some(item) = item_source(&key, assoc, &todays) else {
      return declined(&s!("a block item with nothing to write it from: {key:?}"));
    };
    block.push(item);
  }
  let separators = blocks.into_iter().flat_map(separator_words).collect();
  Some(Repart { creators, block, separators })
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

/// Insert a new `ltx:creator` of `attrs` before `anchor`, holding an `ltx:personname` of `name` when given: the
/// reading's name, its printed text (Unicode, its marks out) — not its tokens digested again, whose span may open a
/// layout it does not close (`\begin{tabular}{c}Ann Able`, the first ink carrying the environment's cover).
fn new_creator(
  document: &mut Document,
  anchor: &Node,
  attrs: &TagAttrs,
  name: Option<&str>,
) -> Result<Option<Node>> {
  let Some(parent) = anchor.get_parent() else {
    return Ok(None);
  };
  let entry = TagData {
    tag:     "ltx:creator".to_string(),
    attr:    attrs.clone(),
    content: name
      .map(|name| {
        TagContent::Entry(TagData {
          tag:     "ltx:personname".to_string(),
          attr:    TagAttrs::default(),
          content: vec![TagContent::Box(Digested::from(name.to_string()))],
        })
      })
      .into_iter()
      .collect(),
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
  Ok(Some(creator))
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

/// Write `items` into `owner`.
fn write_items(document: &mut Document, owner: &Node, items: &[Item]) -> Result<()> {
  for item in items {
    match item {
      Item::Copy(node) => {
        let mut owner = owner.clone();
        document.append_clone(&mut owner, vec![node.clone()])?;
      },
      Item::Make { made, role, tokens } => {
        DebugFeature!(
          "frontmatter",
          "FRONT writer: re-part makes {made:?} {role} from {}",
          tokens.clone().untex_string()
        );
        make(document, owner, *made, tokens, None, role, None)?;
      },
    }
  }
  Ok(())
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
  let mut built: Vec<Node> = Vec::new();
  let mut named: Vec<Creator> = Vec::new();
  let mut nameless: Vec<Creator> = Vec::new();
  let mut complete = true;
  for creator in &repart.creators {
    let Some(node) = new_creator(document, &anchor, &creator.attrs, Some(&creator.name))? else {
      complete = false;
      break;
    };
    built.push(node.clone());
    if !creator.marks.is_empty()
      && let Some(mut personname) = elements(&node).into_iter().next()
    {
      let mut sup = Node::new("sup", None, document.get_document())?;
      if let Some(namespace) = personname.get_namespace() {
        let _ = sup.set_namespace(&namespace);
      }
      sup.append_text(&creator.marks.join(","))?;
      personname.add_child(&mut sup)?;
    }
    write_items(document, &node, &creator.items)?;
    in_print_order(&node, Owner::Creator(creator.index), &order);
    named.push(read_creator(&node));
  }
  if complete && !repart.block.is_empty() {
    let mut attrs = TagAttrs::default();
    attrs.insert("role".to_string(), "author".to_string());
    match new_creator(document, &anchor, &attrs, None)? {
      Some(node) => {
        built.push(node.clone());
        write_items(document, &node, &repart.block)?;
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
  if !complete || !lost.is_empty() || !added.is_empty() || reads.is_some() {
    DebugFeature!(
      "frontmatter",
      "FRONT writer: re-part taken back: lost {lost:?}, added {added:?}, {reads:?}"
    );
    for node in built {
      document.remove_node(node);
    }
    return Ok(false);
  }
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
  for node in old {
    document.remove_node(node);
  }
  for ids in &taken {
    reclaim_ids(document, ids)?;
  }
  Ok(true)
}

#[cfg(test)]
mod tests {
  use super::*;

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
