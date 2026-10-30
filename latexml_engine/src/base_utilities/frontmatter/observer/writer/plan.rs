//! The writer's plan: today's frontmatter ([`dom`]) against the resolver's reading, item by item, as `ab_audit
//! fmshadow` compares them — per creator (paired by name, in order) and for the author block's own, each item by its
//! kind and its text's letters and digits — and each difference explained by one of the writer's operations, or the
//! paper left as today's (FM S2b, OXIDIZED_DESIGN_DIVERGENCES #837 "S2b").
//!
//! The operations, each only where the reading's rule holds an adopted ruling's change tag (`resolver::CHANGE_TAGS`):
//! - **re-assign**: an item today's gives other authors than the reading — copied to the authors it lacks, taken from
//!   the ones it should not be on (a leading-mark entry to the names requesting its mark, `api:leading-mark`,
//!   `ruling1:request`; an address left to the block, `owner:block`);
//! - **cut by marks**: one call's content that the reading cuts into entries at their marks (`api:split-by-marks`),
//!   today's one contact — each entry made from its tokens (the stamped rehearsal's spans), given to its authors;
//! - **legend**: an entry the reading takes for a legend note (`api:legend`), today's an affiliation — made a note of
//!   the names showing its mark;
//! - **role**: a url printing one address, an e-mail (`role:url-as-email`);
//! - **write out**: one e-mail call printing several addresses (a braced group `{ann,bob}@x.org`) that the reading
//!   writes out (`ruling3:written-out`), today's one contact (on every carrier of its entry) — each address a contact
//!   of its own, made from today's, given to its owner.

use std::collections::BTreeMap;

use super::{
  super::{
    events::{EventKind, lex_events},
    ledger::{ApiCall, Outcome, field_stamped},
    policy::TitleMarks,
    rehearsal::{Rehearsal, rehearse},
    resolver::{addresses_in, split_pieces},
  },
  apply::text_leaf,
  dom::{Front, Kind, norm, orcid_id},
  *,
};

/// Whose an item is: a resolved creator's (its index), or the author block's own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Owner {
  Creator(usize),
  Block,
}

/// What a made item is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Made {
  /// an affiliation contact, its attributes the template's (today's contact it was cut from)
  Affiliation,
  /// a note of the names showing its mark (the kernel's author note markup, `\lx@add@thanks`)
  Legend,
}

/// One operation on today's frontmatter.
#[derive(Debug, Clone)]
pub(super) enum Op {
  /// a copy of today's item `from` for `to`
  Copy { from: Node, to: Owner },
  /// today's item taken away
  Remove { node: Node },
  /// an item made from the queued `tokens` for `owner` — before `before`, the item of today's it replaces there
  Make {
    owner:    Owner,
    made:     Made,
    tokens:   Tokens,
    template: Node,
    before:   Option<Node>,
  },
  /// today's contact given the reading's role
  Role { node: Node, role: String },
  /// a copy of today's contact `template` printing only `address`, for `owner` (a written-out address)
  WriteOut {
    owner:    Owner,
    template: Node,
    address:  String,
  },
}

/// The plan: its operations, the document's creator of each resolved creator, and where each of the reading's items
/// prints ([`Place`]), which places the items the operations add among an owner's (rev92c S1).
#[derive(Debug, Clone)]
pub(super) struct Plan {
  pub ops:      Vec<Op>,
  pub creators: BTreeMap<usize, Node>,
  pub order:    BTreeMap<(Owner, Key), Place>,
}

pub(super) type Key = (Kind, String);

/// Where one of the reading's items prints: its block or API call's ledger index, then its span's start, one with no
/// span after its block's spanned ones — the order the resolver keeps a creator's contacts and notes in
/// (`resolver::in_source_order`), the print order of the entries an owner's marks request (llncs's and aa's institute
/// list: `\inst{2,1}` prints under institutes 1 and 2, as today's places them).
pub(super) type Place = (usize, (u8, u16));

fn place_of(assoc: &Assoc) -> Option<Place> {
  assoc
    .source
    .map(|(index, span)| (index, span.map_or((1, 0), |(start, _)| (0, start))))
}

/// The resolved creators paired with today's, by name in order: `None` when they differ.
pub(super) fn pair_names(resolved: &Resolved, front: &Front) -> Option<BTreeMap<usize, usize>> {
  let named: Vec<usize> = (0..resolved.creators.len())
    .filter(|&i| !resolved.creators[i].name.trim().is_empty())
    .collect();
  if named.len() != front.creators.len() {
    return None;
  }
  named
    .iter()
    .zip(&front.creators)
    .enumerate()
    .map(|(j, (&i, creator))| {
      (norm(&resolved.creators[i].name) == norm(&creator.name)).then_some((i, j))
    })
    .collect()
}

/// The reading's items: (owner, kind, text) with the association, as `ab_audit fmshadow`'s `shadow_tuples` reads them
/// (a creator with no name holds none; an item of no letter or digit compares as none, a creator's as the block's — as
/// [`held`] reads today's: a note printing only a superscript or a box, `\thanks{\textsuperscript{}}`, the observer's
/// note of ink in doubt, held the kernel's paper back, OD #925).
fn wanted(resolved: &Resolved) -> BTreeMap<Key, Vec<(Owner, &Assoc)>> {
  let mut out: BTreeMap<Key, Vec<(Owner, &Assoc)>> = BTreeMap::new();
  for (i, c) in resolved.creators.iter().enumerate() {
    if c.name.trim().is_empty() {
      continue;
    }
    // (one item per creator and text: an address the reading gives one creator twice — from its institute line and from
    // `\offprints`, 0906.2451; a link's target and its text before the events read only the text, review rev92c B1, S2
    // — is one contact, the one a ruling decided kept)
    for (key, a) in creator_items(c) {
      want_once(&mut out, key, Owner::Creator(i), a);
    }
  }
  for (key, a) in block_items(resolved) {
    want_once(&mut out, key, Owner::Block, a);
  }
  out
}

/// The owners of `items`, each once: an item is one creator's or the block's, however many times today's or the
/// reading holds it (rev92c S2: one contact per creator and address).
fn owners(items: impl Iterator<Item = Owner>) -> Vec<Owner> {
  let mut out: Vec<Owner> = items.collect();
  out.sort();
  out.dedup();
  out
}

/// The reading's item `a` of `owner` under `key`, once per owner: the one a ruling decided kept.
fn want_once<'a>(
  out: &mut BTreeMap<Key, Vec<(Owner, &'a Assoc)>>,
  key: Key,
  owner: Owner,
  a: &'a Assoc,
) {
  let wants = out.entry(key).or_default();
  match wants.iter_mut().find(|(o, _)| *o == owner) {
    Some(held) if !writes(held.1) && writes(a) => held.1 = a,
    Some(_) => {},
    None => wants.push((owner, a)),
  }
}

/// A resolved creator's items as the comparison keys them — its affiliations, contacts and notes, an item of no letter
/// or digit none ([`wanted`]).
pub(super) fn creator_items(c: &RCreator) -> Vec<(Key, &Assoc)> {
  let mut items: Vec<(Key, &Assoc)> = Vec::new();
  for a in &c.affiliations {
    items.push(((Kind::Affiliation, norm(&a.text)), a));
  }
  for a in &c.contacts {
    let key = match a.role.as_str() {
      "note" | "thanks" => (Kind::Note, norm(&a.text)),
      "orcid" => (Kind::Contact("orcid".into()), norm(orcid_id(&a.text))),
      role => (Kind::Contact(role.to_string()), norm(&a.text)),
    };
    items.push((key, a));
  }
  for a in &c.notes {
    items.push(((Kind::Note, norm(&a.text)), a));
  }
  items.retain(|(key, _)| !key.1.is_empty());
  items
}

/// The author block's own items, as the comparison keys them ([`wanted`]).
pub(super) fn block_items(resolved: &Resolved) -> Vec<(Key, &Assoc)> {
  resolved
    .block
    .iter()
    .map(|a| ((Kind::of_block(&a.role), norm(&a.text)), a))
    .filter(|(key, _)| !key.1.is_empty())
    .collect()
}

/// Today's items: (owner, kind, text) with the node and whether it may move (`dom::Item::movable`).
fn held(front: &Front, pairs: &BTreeMap<usize, usize>) -> BTreeMap<Key, Vec<(Owner, Node, bool)>> {
  let mut out: BTreeMap<Key, Vec<(Owner, Node, bool)>> = BTreeMap::new();
  // (an item of no text is no content to place, a named creator's as a nameless one's: today's empty contact of an
  // `\email{}`, an endnote mark's empty note — the comparison reads none, and the paper was held back for it)
  for (&i, &j) in pairs {
    for item in &front.creators[j].items {
      if !item.key.is_empty() {
        out
          .entry((item.kind.clone(), item.key.clone()))
          .or_default()
          .push((Owner::Creator(i), item.node.clone(), item.movable));
      }
    }
  }
  for creator in &front.nameless {
    for item in &creator.items {
      if !item.key.is_empty() {
        out
          .entry((item.kind.clone(), item.key.clone()))
          .or_default()
          .push((Owner::Block, item.node.clone(), item.movable));
      }
    }
  }
  out
}

/// `a` less `b`, as multisets.
fn minus(a: &[Owner], b: &[Owner]) -> Vec<Owner> {
  let mut rest = b.to_vec();
  a.iter()
    .filter(|o| match rest.iter().position(|r| r == *o) {
      Some(at) => {
        rest.remove(at);
        false
      },
      None => true,
    })
    .copied()
    .collect()
}

/// Where today's frontmatter and the reading differ, if they do (the writer's check of what it wrote).
pub(super) fn difference(resolved: &Resolved, front: &Front) -> Option<String> {
  let Some(pairs) = pair_names(resolved, front) else {
    return Some("the creators' names".into());
  };
  let want = wanted(resolved);
  let have = held(front, &pairs);
  for key in want.keys().chain(have.keys()) {
    let w = owners(want.get(key).into_iter().flatten().map(|(o, _)| *o));
    let h = owners(have.get(key).into_iter().flatten().map(|(o, ..)| *o));
    if w != h {
      return Some(s!("{:?} {}: reading {w:?}, written {h:?}", key.0, key.1));
    }
  }
  None
}

/// The API call at ledger index `index`.
pub(super) fn api_call(index: usize) -> Option<ApiCall> {
  with_ledger(|ledger| match ledger.entries.get(index) {
    Some(Entry::Api(call)) => Some(call.clone()),
    _ => None,
  })
}

/// The tokens of `content` a piece's span `start..end` covers, past `floor` (where the piece before ended) — with the
/// switches that open the group it sits in, when it sits inside one (`{\bf $^1$Univ A, $^2$Univ B}`: each piece keeps
/// the `\bf`) —, or `None` when no balanced run of tokens holds it. A mark's formula covers its script, not the `$`
/// and braces around it (`${}^*$Blackett …` opens at the `^`): the math shifts, braces and script characters just
/// before the span are the piece's, and the closing braces and shifts just after it that balance them — LaTeX's `\(`
/// and `\)` as the shifts they spell, with the spaces inside them (`\( ^{1} \)Univ A`: hep-ph0110378, 2307.12984).
fn piece_tokens(
  content: &[Token],
  (start, end): (usize, usize),
  floor: usize,
  first: usize,
) -> Option<(Tokens, usize)> {
  if start > end || end > content.len() || start < floor {
    return None;
  }
  let (mut start, mut end) = (start, end);
  loop {
    if start > floor
      && matches!(
        content[start - 1].get_catcode(),
        Catcode::MATH | Catcode::BEGIN | Catcode::SUPER | Catcode::SUB
      )
    {
      start -= 1;
    } else if start > floor + 1
      && content[start - 1].get_catcode() == Catcode::END
      && content[start - 2].get_catcode() == Catcode::BEGIN
    {
      // (an empty group a floating script hangs on, `{}^*`)
      start -= 2;
    } else if let Some(open) = (floor..start)
      .rev()
      .find(|&i| content[i].get_catcode() != Catcode::SPACE)
      && is_cs(&content[open], "\\(")
    {
      start = open;
      if start > floor && is_cs(&content[start - 1], "\\protect") {
        start -= 1;
      }
    } else {
      break;
    }
  }
  let balance = |s: usize, e: usize| -> Option<(i32, usize, i32)> {
    let (mut depth, mut parens) = (0i32, 0i32);
    for t in &content[s..e] {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth -= 1;
          if depth < 0 {
            return None;
          }
        },
        _ if is_cs(t, "\\(") => parens += 1,
        _ if is_cs(t, "\\)") => {
          parens -= 1;
          if parens < 0 {
            return None;
          }
        },
        _ => {},
      }
    }
    let shifts = content[s..e]
      .iter()
      .filter(|t| t.get_catcode() == Catcode::MATH)
      .count();
    Some((depth, shifts, parens))
  };
  loop {
    let (depth, shifts, parens) = balance(start, end)?;
    if depth == 0 && shifts % 2 == 0 && parens == 0 {
      break;
    }
    match content.get(end) {
      Some(t) if t.get_catcode() == Catcode::END && depth > 0 => end += 1,
      Some(t) if t.get_catcode() == Catcode::MATH && shifts % 2 == 1 => end += 1,
      Some(t) if parens > 0 && (t.get_catcode() == Catcode::SPACE || is_cs(t, "\\)")) => end += 1,
      _ => return None,
    }
  }
  // (the group the pieces sit in: its opening brace, and the switches before the first piece in it)
  let mut open: Vec<usize> = Vec::new();
  for (i, t) in content[..start].iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => open.push(i),
      Catcode::END => {
        open.pop();
      },
      _ => {},
    }
  }
  let mut tokens = Vec::new();
  if let Some(&group) = open.last()
    && group < first
  {
    let switches = &content[group + 1..first.min(start)];
    if switches
      .iter()
      .all(|t| matches!(t.get_catcode(), Catcode::CS | Catcode::SPACE))
    {
      tokens.extend_from_slice(switches);
    }
  }
  tokens.extend_from_slice(&content[start..end]);
  Some((Tokens::new(tokens), end))
}

/// Whether `t` is the control sequence `name`.
fn is_cs(t: &Token, name: &str) -> bool {
  t.get_catcode() == Catcode::CS && t.with_cs_name(|n| n == name)
}

/// One call's content cut into the reading's entries: each piece's text (normalised), its tokens.
fn cut(call: &ApiCall) -> Result<Option<Vec<(String, Tokens)>>> {
  let content = call.content.unlist_ref();
  if content.len() > usize::from(FIELD_PROV_POS_MASK) {
    DebugFeature!("frontmatter", "FRONT writer: the entry is out of positions");
    return Ok(None);
  }
  let events = match rehearse(
    field_stamped(2, 0, content),
    TitleMarks::Fnsymbol,
    lex_events,
  )? {
    Rehearsal::Done { value, .. } => value,
    Rehearsal::Skipped(_) => return Ok(None),
  };
  let pieces = split_pieces(&events, &call.role);
  let spans: Vec<Option<(usize, usize)>> = pieces.iter().map(|(_, p)| piece_span(p)).collect();
  let Some(first) = spans.iter().flatten().map(|(s, _)| *s).min() else {
    return Ok(None);
  };
  let mut out = Vec::new();
  let mut floor = 0;
  for ((text, _), span) in pieces.iter().zip(&spans) {
    let Some(span) = *span else {
      return Ok(None);
    };
    let Some((tokens, end)) = piece_tokens(content, span, floor, first) else {
      DebugFeature!(
        "frontmatter",
        "FRONT writer: piece {span:?} is no balanced run"
      );
      return Ok(None);
    };
    floor = end;
    out.push((norm(text), tokens));
  }
  Ok(Some(out))
}

/// The tokens of an author block's own line, the reading's entry at `span` of the block at ledger index `index`: from
/// the mark it opens with (the event printed just before it, a Mark with only spaces between: `$^4$ ModelBest.Inc`,
/// `\textsuperscript{*}Equal contribution`) to its end, as [`piece_tokens`] balances a piece — `None` when the index is
/// no rehearsed block, no balanced run holds it, or the span's own letters and digits are not the entry's text `key`
/// (an entry printing through a command, an accent, a line read on to: what the writer would make is not the entry).
fn block_line_tokens(index: usize, span: (u16, u16), key: &str) -> Option<Tokens> {
  block_span_tokens(index, span, Some(key))
}

/// The tokens of the block at ledger index `index` an entry's `span` covers, from the mark it opens with, as
/// [`block_line_tokens`] reads them — the span's own letters and digits checked against `key` when one is given (the
/// re-part checks what it made on the digested text instead, `repart`).
pub(super) fn block_span_tokens(
  index: usize,
  (start, end): (u16, u16),
  key: Option<&str>,
) -> Option<Tokens> {
  with_ledger(|ledger| {
    let Some(Entry::Author(block)) = ledger.entries.get(index) else {
      return None;
    };
    let Outcome::Events { events, .. } = &block.outcome else {
      return None;
    };
    let base = block.range.0;
    let content = block.tokens.unlist_ref();
    let at = |p: u16| usize::from(p).checked_sub(base);
    let (mut from, to) = (at(start)?, at(end)?);
    let letters: String = content
      .get(from..to)?
      .iter()
      .filter(|t| matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER))
      .map(|t| t.with_str(ToString::to_string))
      .collect();
    if key.is_some_and(|key| norm(&letters) != key) {
      return None;
    }
    // (the event printed last before the line, a layout change aside)
    if let Some(mark) = events
      .iter()
      .rev()
      .find(|e| e.kind != EventKind::Layout && e.span.is_some_and(|(_, e)| e <= start))
      && mark.kind == EventKind::Mark
      && let Some((mark_start, mark_end)) = mark.span
      && let (Some(mark_start), Some(mark_end)) = (at(mark_start), at(mark_end))
      && content.get(mark_end..from).is_some_and(|between| {
        // (spaces, and the math shift closing the mark's formula: `$^{1}$Univ X`, whose Mark spans its script)
        between
          .iter()
          .all(|t| matches!(t.get_catcode(), Catcode::SPACE | Catcode::MATH))
      })
    {
      from = mark_start;
    }
    // (a group closing past the spaces after the entry: `{\it Department …, Japan }`, 2307.03384)
    let mut past = to;
    while content
      .get(past)
      .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
    {
      past += 1;
    }
    piece_tokens(content, (from, to), 0, from)
      .or_else(|| {
        (past > to)
          .then(|| piece_tokens(content, (from, past), 0, from))
          .flatten()
      })
      .map(|(tokens, _)| tokens)
  })
}

/// The writer's cut of an author block's own lines (FM stage 7, OD #925): today's one affiliation joining several of
/// the reading's entries, each one line of the same block (`$^4$ ModelBest.Inc $^5$ Renmin University of China`, one
/// contact today, four entries each requested by its mark: 2404.02078, 2509.24142), or one legend line the reading
/// takes for a note (`\textsuperscript{*}Equal contribution`, ruling 2: 2407.07614) — each entry made from its line's
/// tokens for the authors the reading gives it, today's item taken. Only entries the reading decided by a ruling's
/// change, none of which today's holds anywhere, in one run of consecutive lines whose texts joined are today's item's;
/// anything else is left to the final check, which declines.
fn cut_block_lines(
  want: &BTreeMap<Key, Vec<(Owner, &Assoc)>>,
  have: &BTreeMap<Key, Vec<(Owner, Node, bool)>>,
  wanted_only: &mut Vec<Key>,
  held_only: &mut Vec<Key>,
  ops: &mut Vec<Op>,
) {
  // (the reading's entries no item of today's holds, each one span of one block, by a ruling's change, in print order)
  let mut lines: Vec<(usize, (u16, u16), Key)> = wanted_only
    .iter()
    .filter(|key| !matches!(key.0, Kind::Contact(_)))
    .filter_map(|key| {
      let wants = &want[key];
      let (index, Some(span)) = wants.first()?.1.source? else {
        return None;
      };
      // (one line's entry, opened by its mark: not one a line under it continued, nor one whose mark trails it)
      let one_line = |a: &Assoc| {
        !["line:continued", "-joined", "mark:trailing"]
          .iter()
          .any(|tag| a.rule.contains(tag))
      };
      wants
        .iter()
        .all(|(_, a)| writes(a) && one_line(a) && a.source == Some((index, Some(span))))
        .then(|| (index, span, key.clone()))
    })
    .collect();
  lines.sort();
  for whole in held_only.clone() {
    if whole.0 != Kind::Affiliation {
      continue;
    }
    let holds = &have[&whole];
    if holds.iter().any(|(_, _, movable)| !movable) {
      continue;
    }
    let Some(run) = (0..lines.len()).find_map(|from| {
      let mut joined = String::new();
      for to in from..lines.len() {
        if lines[to].0 != lines[from].0 {
          return None;
        }
        joined.push_str(&lines[to].2.1);
        if joined == whole.1 {
          return Some(from..=to);
        }
        if !whole.1.starts_with(&joined) {
          return None;
        }
      }
      None
    }) else {
      continue;
    };
    let run: Vec<(usize, (u16, u16), Key)> = lines[run].to_vec();
    // (one line alone is today's item under another kind only: a legend)
    if run.len() == 1 && run[0].2.0 != Kind::Note {
      continue;
    }
    let Some(pieces) = run
      .iter()
      .map(|(index, span, key)| block_line_tokens(*index, *span, &key.1))
      .collect::<Option<Vec<Tokens>>>()
    else {
      DebugFeature!("frontmatter", "FRONT writer: a block line does not cut");
      continue;
    };
    for ((_, _, key), tokens) in run.iter().zip(pieces) {
      let made = if key.0 == Kind::Note {
        Made::Legend
      } else {
        Made::Affiliation
      };
      for (owner, _) in &want[key] {
        ops.push(Op::Make {
          owner: *owner,
          made,
          tokens: tokens.clone(),
          template: holds[0].1.clone(),
          before: holds
            .iter()
            .find(|(o, ..)| o == owner)
            .map(|(_, n, _)| n.clone()),
        });
      }
    }
    for (_, node, _) in holds {
      ops.push(Op::Remove { node: node.clone() });
    }
    let keys: Vec<&Key> = run.iter().map(|(_, _, key)| key).collect();
    wanted_only.retain(|k| !keys.contains(&k));
    lines.retain(|(_, _, k)| !keys.contains(&k));
    held_only.retain(|k| *k != whole);
  }
}

/// Whether an e-mail contact's `text` prints its addresses alone: with its braced groups (`{ann,bob}@a.org`,
/// `{ann,bob@a.org}`) taken out, every word between the separators (`,`, `;`, spaces) an address — no label, no remark
/// beside them (rev92c B2).
fn addresses_alone(text: &str) -> bool {
  let mut flat = String::new();
  let mut rest = text;
  while let Some(open) = rest.find('{') {
    let Some(close) = rest[open..].find('}').map(|c| open + c) else {
      return false;
    };
    flat.push_str(&rest[..open]);
    flat.push(' ');
    let after = &rest[close + 1..];
    if let Some(domain) = after.strip_prefix('@') {
      let end = domain
        .find(|c: char| c.is_whitespace() || ",;".contains(c))
        .unwrap_or(domain.len());
      rest = &domain[end..];
    } else if rest[open + 1..close].contains('@') {
      rest = after;
    } else {
      return false;
    }
  }
  flat.push_str(rest);
  flat
    .split(|c: char| c.is_whitespace() || ",;".contains(c))
    .filter(|w| !w.is_empty())
    .all(|w| {
      w.split_once('@').is_some_and(|(local, domain)| {
        !local.is_empty() && domain.contains('.') && !domain.contains('@')
      })
    })
}

/// No plan, and why (the `frontmatter` debug feature).
fn declined(reason: &str) -> Option<Plan> {
  DebugFeature!("frontmatter", "FRONT writer: declined, {reason}");
  None
}

fn rule_has(assoc: &Assoc, tag: &str) -> bool { assoc.rule.split_whitespace().any(|t| t == tag) }

/// The plan for a written family's paper: `None` when the reading and today's agree, or when a difference is no
/// operation's.
pub(super) fn plan(resolved: &Resolved, front: &Front) -> Result<Option<Plan>> {
  let Some(pairs) = pair_names(resolved, front) else {
    return Ok(declined("names differ"));
  };
  let want = wanted(resolved);
  let have = held(front, &pairs);
  let mut ops = Vec::new();
  let mut wanted_only: Vec<Key> = Vec::new();
  let mut held_only: Vec<Key> = Vec::new();
  let keys: Vec<Key> = want
    .keys()
    .chain(have.keys())
    .cloned()
    .collect::<std::collections::BTreeSet<_>>()
    .into_iter()
    .collect();
  for key in keys {
    let wants: &[(Owner, &Assoc)] = want.get(&key).map_or(&[], Vec::as_slice);
    let holds: &[(Owner, Node, bool)] = have.get(&key).map_or(&[], Vec::as_slice);
    let w = owners(wants.iter().map(|(o, _)| *o));
    let h = owners(holds.iter().map(|(o, ..)| *o));
    let (added, removed) = (minus(&w, &h), minus(&h, &w));
    if added.is_empty() && removed.is_empty() {
      continue;
    }
    if wants.is_empty() {
      held_only.push(key);
      continue;
    }
    if holds.is_empty() {
      wanted_only.push(key);
      continue;
    }
    // re-assign: the reading's owners by a ruling's change, today's item copied there and taken from the rest
    if !wants.iter().any(|(_, a)| writes(a)) {
      let rules: Vec<&str> = wants.iter().map(|(_, a)| a.rule.as_str()).collect();
      return Ok(declined(&s!(
        "an owner change no ruling decided: {key:?}, reading {w:?} ({rules:?}), today's {h:?}"
      )));
    }
    let Some(from) = holds
      .iter()
      .find(|(_, _, movable)| *movable)
      .map(|(_, n, _)| n.clone())
    else {
      return Ok(declined("no item to copy"));
    };
    for owner in added {
      if !wants.iter().any(|(o, a)| *o == owner && writes(a)) {
        return Ok(declined("an owner added by no ruling"));
      }
      ops.push(Op::Copy {
        from: from.clone(),
        to:   owner,
      });
    }
    // (every copy today's gives an owner the reading does not: one item per owner, S2)
    for owner in removed {
      let mine: Vec<&(Owner, Node, bool)> = holds.iter().filter(|(o, ..)| *o == owner).collect();
      if mine.iter().any(|(_, _, movable)| !movable) {
        return Ok(declined("an item to remove that cannot move"));
      }
      for (_, node, _) in mine {
        ops.push(Op::Remove { node: node.clone() });
      }
    }
  }
  // the reading's entries cut from one call by its marks, today's one item
  let mut groups: BTreeMap<usize, Vec<Key>> = BTreeMap::new();
  for key in &wanted_only {
    let wants = &want[key];
    if wants
      .iter()
      .all(|(_, a)| rule_has(a, "api:split-by-marks") && writes(a))
      && let Some((index, _)) = wants[0].1.source
      && wants
        .iter()
        .all(|(_, a)| a.source.map(|(i, _)| i) == Some(index))
    {
      groups.entry(index).or_default().push(key.clone());
    }
  }
  for (index, keys) in groups {
    let Some(call) = api_call(index) else {
      return Ok(declined("no API call"));
    };
    if !matches!(call.role.as_str(), "affiliation" | "address") {
      return Ok(declined("a cut entry of another role"));
    }
    let Some(pieces) = cut(&call)? else {
      return Ok(declined("the entry does not cut"));
    };
    // (every piece is one of the reading's entries, and today's item is them all, joined)
    let joined: String = pieces.iter().map(|(text, _)| text.as_str()).collect();
    let whole = (Kind::Affiliation, joined);
    if !held_only.contains(&whole) || pieces.len() != keys.len() {
      return Ok(declined("today's item is not the pieces joined"));
    }
    let holds = &have[&whole];
    if holds.iter().any(|(_, _, movable)| !movable) {
      return Ok(declined("an item to remove that cannot move"));
    }
    for (text, tokens) in &pieces {
      let Some(key) = keys.iter().find(|(_, k)| k == text) else {
        return Ok(declined("a piece the reading lacks"));
      };
      let made = match key.0 {
        Kind::Affiliation => Made::Affiliation,
        Kind::Note => Made::Legend,
        Kind::Contact(_) => return Ok(declined("a contact piece")),
      };
      for (owner, _) in &want[key] {
        ops.push(Op::Make {
          owner: *owner,
          made,
          tokens: tokens.clone(),
          template: holds[0].1.clone(),
          before: holds
            .iter()
            .find(|(o, ..)| o == owner)
            .map(|(_, n, _)| n.clone()),
        });
      }
      wanted_only.retain(|k| k != key);
    }
    for (_, node, _) in holds {
      ops.push(Op::Remove { node: node.clone() });
    }
    held_only.retain(|k| *k != whole);
  }
  // the reading's entries of an author block's own lines, today's one item joining them (or a legend line, today's an
  // affiliation)
  cut_block_lines(&want, &have, &mut wanted_only, &mut held_only, &mut ops);
  // a legend the reading takes for a note, today's an affiliation of the same text
  for key in wanted_only.clone() {
    let wants = &want[&key];
    let whole = (Kind::Affiliation, key.1.clone());
    if key.0 != Kind::Note
      || !held_only.contains(&whole)
      || !wants
        .iter()
        .all(|(_, a)| rule_has(a, "api:legend") && writes(a))
    {
      continue;
    }
    let Some((index, None)) = wants[0].1.source else {
      return Ok(declined("a legend of no API call"));
    };
    if wants.iter().any(|(_, a)| a.source != Some((index, None))) {
      return Ok(declined("a legend of no API call"));
    }
    let Some(call) = api_call(index) else {
      return Ok(declined("no API call"));
    };
    let holds = &have[&whole];
    if holds.iter().any(|(_, _, movable)| !movable) {
      return Ok(declined("an item to remove that cannot move"));
    }
    for (owner, _) in wants {
      ops.push(Op::Make {
        owner:    *owner,
        made:     Made::Legend,
        tokens:   call.content.clone(),
        template: holds[0].1.clone(),
        before:   None,
      });
    }
    for (_, node, _) in holds {
      ops.push(Op::Remove { node: node.clone() });
    }
    wanted_only.retain(|k| *k != key);
    held_only.retain(|k| *k != whole);
  }
  // a url printing one address, the reading's e-mail
  for key in wanted_only.clone() {
    let url = (Kind::Contact("url".into()), key.1.clone());
    if key.0 != Kind::Contact("email".into()) || !held_only.contains(&url) {
      continue;
    }
    let wants = &want[&key];
    if !wants
      .iter()
      .all(|(_, a)| rule_has(a, "role:url-as-email") && writes(a))
    {
      continue;
    }
    let holds = &have[&url];
    let mut w: Vec<Owner> = wants.iter().map(|(o, _)| *o).collect();
    let mut h: Vec<Owner> = holds.iter().map(|(o, ..)| *o).collect();
    w.sort();
    h.sort();
    if w != h || holds.iter().any(|(_, _, movable)| !movable) {
      return Ok(declined("a url's owners differ"));
    }
    for (_, node, _) in holds {
      ops.push(Op::Role {
        node: node.clone(),
        role: "email".into(),
      });
    }
    wanted_only.retain(|k| *k != key);
    held_only.retain(|k| *k != url);
  }
  // a group of addresses one e-mail call printed, written out: today's one contact (on each carrier of its entry, or
  // the block's), the reading's addresses each its own — every address of the group one of the reading's written-out
  // items, none of them today's but one its owners hold already from another print (S2: written once), and the contact
  // printing the addresses alone (rev92c B2: `\{ann,bob\}@a.org (corresponding)`, `Emails: \{ann,bob\}@a.org` —
  // written out, the words beside the group were lost; such a contact stays today's, the paper declined)
  let email = Kind::Contact("email".into());
  for key in held_only.clone() {
    if key.0 != email {
      continue;
    }
    let holds = &have[&key];
    if holds.iter().any(|(_, _, movable)| !movable) {
      continue;
    }
    let Some(leaf) = holds
      .iter()
      .map(|(_, n, _)| text_leaf(n))
      .collect::<Option<Vec<Node>>>()
    else {
      continue;
    };
    let text = leaf[0].get_content();
    let addresses = addresses_in(&text);
    let keys: Vec<Key> = addresses.iter().map(|a| (email.clone(), norm(a))).collect();
    let held_elsewhere = |k: &Key| {
      want.get(k).is_some_and(|wants| {
        have.get(k).is_some_and(|holds| {
          owners(wants.iter().map(|(o, _)| *o)) == owners(holds.iter().map(|(o, ..)| *o))
        })
      })
    };
    if addresses.len() < 2
      || !addresses_alone(&text)
      || !keys.iter().any(|k| wanted_only.contains(k))
      || !keys.iter().all(|k| {
        held_elsewhere(k)
          || (wanted_only.contains(k)
            && want[k]
              .iter()
              .all(|(_, a)| rule_has(a, "ruling3:written-out") && writes(a)))
      })
    {
      continue;
    }
    let mut written: Vec<&Key> = Vec::new();
    for (address, k) in addresses.iter().zip(&keys) {
      // (each address once, however often the group prints it)
      if !wanted_only.contains(k) || written.contains(&k) {
        continue;
      }
      written.push(k);
      for (owner, _) in &want[k] {
        ops.push(Op::WriteOut {
          owner:    *owner,
          template: holds[0].1.clone(),
          address:  address.clone(),
        });
      }
    }
    for (_, node, _) in holds {
      ops.push(Op::Remove { node: node.clone() });
    }
    wanted_only.retain(|k| !keys.contains(k));
    held_only.retain(|k| *k != key);
  }
  if !wanted_only.is_empty() || !held_only.is_empty() || ops.is_empty() {
    return Ok(declined("a difference no operation explains"));
  }
  let creators = pairs
    .iter()
    .map(|(&i, &j)| (i, front.creators[j].node.clone()))
    .collect();
  let order = reading_order(resolved);
  Ok(Some(Plan { ops, creators, order }))
}

/// Where each of the reading's items prints, by its owner and key ([`Place`]): what the print-order pass places an
/// owner's items by (`apply::place`).
pub(super) fn reading_order(resolved: &Resolved) -> BTreeMap<(Owner, Key), Place> {
  wanted(resolved)
    .iter()
    .flat_map(|(key, wants)| {
      wants
        .iter()
        .filter_map(|(owner, a)| place_of(a).map(|place| ((*owner, key.clone()), place)))
    })
    .collect()
}
