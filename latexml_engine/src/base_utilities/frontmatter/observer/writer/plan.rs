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
//! - **role**: a url printing one address, an e-mail (`role:url-as-email`).

use std::collections::BTreeMap;

use super::{
  super::{
    events::lex_events,
    ledger::{ApiCall, field_stamped},
    policy::TitleMarks,
    rehearsal::{Rehearsal, rehearse},
    resolver::split_pieces,
  },
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
}

/// The plan: its operations, and the document's creator of each resolved creator.
#[derive(Debug, Clone)]
pub(super) struct Plan {
  pub ops:      Vec<Op>,
  pub creators: BTreeMap<usize, Node>,
}

type Key = (Kind, String);

/// The resolved creators paired with today's, by name in order: `None` when they differ.
fn pair_names(resolved: &Resolved, front: &Front) -> Option<BTreeMap<usize, usize>> {
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
/// (a creator with no name holds none; the block's items of no letter or digit compare as none).
fn wanted(resolved: &Resolved) -> BTreeMap<Key, Vec<(Owner, &Assoc)>> {
  let mut out: BTreeMap<Key, Vec<(Owner, &Assoc)>> = BTreeMap::new();
  for (i, c) in resolved.creators.iter().enumerate() {
    if c.name.trim().is_empty() {
      continue;
    }
    let owner = Owner::Creator(i);
    for a in &c.affiliations {
      out
        .entry((Kind::Affiliation, norm(&a.text)))
        .or_default()
        .push((owner, a));
    }
    for a in &c.contacts {
      let key = match a.role.as_str() {
        "note" | "thanks" => (Kind::Note, norm(&a.text)),
        "orcid" => (Kind::Contact("orcid".into()), norm(orcid_id(&a.text))),
        role => (Kind::Contact(role.to_string()), norm(&a.text)),
      };
      out.entry(key).or_default().push((owner, a));
    }
    for a in &c.notes {
      out
        .entry((Kind::Note, norm(&a.text)))
        .or_default()
        .push((owner, a));
    }
  }
  for a in &resolved.block {
    let key = norm(&a.text);
    if !key.is_empty() {
      out
        .entry((Kind::of_block(&a.role), key))
        .or_default()
        .push((Owner::Block, a));
    }
  }
  out
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
    let mut w: Vec<Owner> = want
      .get(key)
      .map_or_else(Vec::new, |v| v.iter().map(|(o, _)| *o).collect());
    let mut h: Vec<Owner> = have
      .get(key)
      .map_or_else(Vec::new, |v| v.iter().map(|(o, ..)| *o).collect());
    w.sort();
    h.sort();
    if w != h {
      return Some(s!("{:?} {}: reading {w:?}, written {h:?}", key.0, key.1));
    }
  }
  None
}

/// The API call at ledger index `index`.
fn api_call(index: usize) -> Option<ApiCall> {
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
    let w: Vec<Owner> = wants.iter().map(|(o, _)| *o).collect();
    let h: Vec<Owner> = holds.iter().map(|(o, ..)| *o).collect();
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
    let mut taken: Vec<usize> = Vec::new();
    for owner in removed {
      let Some(at) = holds
        .iter()
        .enumerate()
        .position(|(at, (o, _, movable))| *o == owner && *movable && !taken.contains(&at))
      else {
        return Ok(declined("an item to remove that cannot move"));
      };
      taken.push(at);
      ops.push(Op::Remove { node: holds[at].1.clone() });
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
  if !wanted_only.is_empty() || !held_only.is_empty() || ops.is_empty() {
    return Ok(declined("a difference no operation explains"));
  }
  let creators = pairs
    .iter()
    .map(|(&i, &j)| (i, front.creators[j].node.clone()))
    .collect();
  Ok(Some(Plan { ops, creators }))
}
