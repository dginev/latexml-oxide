//! The observer's writer (FM S2b; OXIDIZED_DESIGN_DIVERGENCES #837 "S2b"): for a family whose binding declared its
//! conventions `written` (the pilot: revtex4_support's `revtex`), once `insert_frontmatter` has placed today's
//! frontmatter, the paper's creators are compared with the resolver's reading as `ab_audit fmshadow` compares them
//! ([`dom`], the comparator's tuples), and what differs is written by the writer's operations ([`plan`]) — an entry
//! given to the authors the reading gives it, one call's marked entries cut apart at their marks, a legend made a note
//! of the names showing its mark, a url printing an address made an e-mail — each decided by a rule holding an adopted
//! ruling's change tag (`resolver::CHANGE_TAGS`). The fail-safe declines per item, each declined item left as today's
//! output holds it: an entry whose address take-out today's does not print so is given back whole
//! (`give_back_addresses`: 2102.08148's Hongyun Yu keeps today's entry while the other authors' are cut), the names
//! re-parted only where every word today's held is kept (`repart`), and a difference of the plan no operation explains
//! (the ledger's `declined: …`) leaves every item the plan would write as today's. So a paper the comparison reports
//! unclassified may still be written in its other items. An item an unwritten family's call made — the call's setter
//! a binding not `written` (`ledger::ApiCall::binding`: svjour3's `\institute` beside authblk's `\author`, 2609.05466) —
//! is never written: it stays as today's holds it ([`keep_as_today`]), the paper's other items written, or the paper
//! declined whole where a written item depends on its reading (a re-part, a creator it made).
//!
//! What is written comes from the tokens the document queued: a cut entry's pieces from its call's content, mapped by
//! the spans of a stamped rehearsal of it ([`plan`]), digested as today's digests an entry ([`apply`]). By default (the
//! fleet's) and under the census switch (`LATEXML_FM_SHADOW=1`), for a paper whose author blocks are all a written
//! family's; switched off (`=0`), nothing of it runs (`shadow::FmShadow`).

use std::collections::{BTreeMap, BTreeSet};

use super::{
  events::{Event, Formula},
  ledger::{Entry, Ledger, Outcome, ReadBack, SkipReason, with_ledger},
  policy::{family_written, policy_for},
  resolver::{
    Assoc, NameJoin, Placed, PlacedItem, PlacedKind, RCreator, Resolved, addresses_in,
    content_check, in_source_order, is_change_rule, ledger_inputs, mark_labels, resolve,
  },
  shadow::{FmShadow, fm_shadow_mode},
  *,
};

mod address_cut;
mod apply;
// (the emitter's vocabulary, moved unchanged from the old reader by the plan's row "Knowledge in, driver out, one pass"; re-exported by `observer/mod.rs`)
pub(in crate::base_utilities) mod calls;
mod content;
mod dom;
mod emit;
mod plan;
mod repart;

// (the writer's cut of a block's entry, which the reading checks an in-block contact's entries by: `events::labelled_entries`)
pub(super) use content::Field;
pub(crate) use emit::fm_emit_mode;
pub use emit::set_fm_emit_override;
pub(in crate::base_utilities) use emit::{Analysis, EMITTING, emit_commands, emitted};
use plan::{Op, Owner};
pub(super) use plan::{entry_forms, labelled_start, span_tokens};

/// Why the writer stays out of the paper the ledger holds, `None` where it may write: every block flushed so far
/// rehearsed without an error held, the API calls after them rehearsed, and every block's family `written`.
pub(super) fn writer_stays_out(ledger: &Ledger) -> Option<&'static str> {
  let mut blocks = 0usize;
  for entry in ledger.entries() {
    match entry {
      Entry::Author(block) => {
        blocks += 1;
        match &block.outcome {
          // (an empty call holds nothing to read: llncs's `\and\\` between two names, an empty piece of its split —
          // 2409.09478)
          Outcome::Events { errors: 0, .. } | Outcome::Skipped(SkipReason::Empty) => {},
          Outcome::Events { .. } => return Some(HELD_DIAGNOSTICS),
          Outcome::Skipped(reason) | Outcome::Deferred(reason) => return Some(reason.name()),
          Outcome::Pending => return Some(SkipReason::NoFlush.name()),
        }
        if !policy_for(&block.binding, block.hook).written {
          return Some(FAMILY_NOT_WRITTEN);
        }
      },
      // (the title is recorded for its marks alone: it never holds the writing back)
      Entry::Api(call) if !call.inside && !call.rehearsed && call.tag != "ltx:title" => {
        return Some(CALL_NOT_REHEARSED);
      },
      _ => {},
    }
  }
  (blocks == 0).then_some(NO_BLOCK)
}

/// The writer's reasons of its own to stay out ([`writer_stays_out`]), beside the blocks' `SkipReason`s.
const HELD_DIAGNOSTICS: &str = "held diagnostics";
const FAMILY_NOT_WRITTEN: &str = "a family not written";
const CALL_NOT_REHEARSED: &str = "an API call not rehearsed";
const NO_BLOCK: &str = "no author block";

/// The resolution of the document's author blocks when the writer may write them ([`writer_stays_out`]).
fn written_resolution() -> Option<Resolved> {
  with_ledger(|ledger| {
    writer_stays_out(ledger)
      .is_none()
      .then(|| resolve(&ledger_inputs(ledger)))
  })
}

/// Under the census (`ab_audit fmshadow`'s closed list of decline reasons), why the plan or the re-part refused: the
/// first refusal of the paper, what its `declined: …` counts under. Nothing runs by default.
pub(super) fn note_refusal(reason: &str) {
  if fm_shadow_mode() == FmShadow::On {
    with_ledger(|ledger| {
      ledger.refusal.get_or_insert_with(|| reason.to_string());
    });
  }
}

/// The writer's own decline: its outcome, unless an earlier pass wrote one (`keep`), and under the census its reason.
fn decline(reason: &str, keep: bool) {
  with_ledger(|ledger| {
    if !keep || ledger.writer.is_none() {
      ledger.writer = Some(s!("declined: {reason}"));
      if fm_shadow_mode() == FmShadow::On {
        ledger.refusal = Some(reason.to_string());
      }
    }
  });
}

/// The read-back, the last step of `insert_frontmatter` and under the census only (E0; FRONTMATTER_SINGLE_ENGINE_PLAN.md,
/// definition of done item 5): the final document's owners against the reading, by the writer's own comparator
/// ([`plan::difference`]) — the reading the writer last compared with ([`Ledger::compared`]) where it got that far, else
/// the resolver's. A paper the writer found the same or wrote that no longer reads so was changed by a pass after the
/// writer: what Z8 and Z9 pay down. It reads; it changes nothing.
///
/// That reading has its give-backs applied, so the final document is read against the resolver's too: an item whose text
/// the writer gave back or conceded reads the same under the first, and shows here (`given_back`): the e-mails of
/// 2310.19837, read as contacts, written as an affiliation's text, under `written` and a null difference.
///
/// [`Ledger::compared`]: super::ledger::Ledger::compared
pub(in crate::base_utilities) fn read_back_frontmatter(document: &mut Document) {
  if fm_shadow_mode() != FmShadow::On || rehearsal::rehearsing() {
    return;
  }
  let (reading, resolved) = with_ledger(|ledger| {
    let resolved = resolve(&ledger_inputs(ledger));
    (ledger.compared.clone(), resolved)
  });
  let front = dom::read(document);
  let difference = plan::difference(reading.as_ref().unwrap_or(&resolved), &front);
  let given_back = plan::difference(&resolved, &front).filter(|_| difference.is_none());
  with_ledger(|ledger| ledger.read_back = Some(ReadBack { difference, given_back }));
}

/// The content invariant's check on the placed output (`resolver::placed`; the safety net's bar (2)), the last step of
/// `insert_frontmatter` under `LATEXML_FM_EMIT=2` only: the words the ledger's author blocks and creator calls printed
/// against the placed creators' (their names, contacts and notes, as [`dom::read`] reads them; a publication note's, a title's
/// note's and a contact's label holding a word too), each finding counted in one Warning per paper naming its words — a printed word or
/// address the output lacks (1605.03019's `IDSIA`), one placed more often than printed (2607.00684's `mtablo@postech.ac.kr`
/// twice, 2506.02663's footnote junk). It counts; it changes nothing.
pub(in crate::base_utilities) fn check_placed_content(document: &mut Document) {
  if fm_emit_mode() != 2 || rehearsal::rehearsing() {
    return;
  }
  let front = dom::read(document);
  let mut placed = Placed::default();
  // (a link's target, which an ink may print: an ORCID's id behind its logo, 2607.00684; each target once, a shared
  // entry's link under every holder one print)
  let mut targets = BTreeSet::new();
  for (holder, creator) in front.creators.iter().chain(&front.nameless).enumerate() {
    let mut links = Vec::new();
    dom::descendants(&creator.node, "ltx:ref", &mut links);
    targets.extend(links.iter().filter_map(|link| link.get_attribute("href")));
    if !creator.name.trim().is_empty() {
      // (its words parted at every layout part, as the block's lines print them: a block written whole, 1908.02923; read again
      // only where a layout part holds them)
      let names = creator.node.get_child_nodes().into_iter();
      let names: Vec<Node> = names
        .filter(|child| dom::is(child, "ltx:personname"))
        .collect();
      let text = if names.iter().any(dom::holds_a_layout_part) {
        let words: Vec<String> = names.iter().map(dom::words_without_marks).collect();
        words.join(" ")
      } else {
        creator.name.clone()
      };
      placed.items.push(PlacedItem {
        holder,
        kind: PlacedKind::Name,
        text,
        label: None,
      });
    }
    // (what a personname holds besides its notes, an ORCID link, its name's text holds)
    for item in creator
      .items
      .iter()
      .filter(|item| item.movable || item.kind == dom::Kind::Note)
    {
      // (one shared item is one kind's under its text: a note on one creator and an affiliation on another printing the same
      // words are two, each printed)
      let kind = match &item.kind {
        dom::Kind::Contact(role) if role == "email" => PlacedKind::Email(item.key.clone()),
        kind => PlacedKind::Item(s!("{kind:?}|{}", item.key)),
      };
      placed.items.push(PlacedItem {
        holder,
        kind,
        text: item.text.clone(),
        label: item.node.get_attribute("name"),
      });
    }
  }
  placed.beside.extend(targets);
  // (a creator call's note relocated to the title, elsarticle's `\tnotetext[t1]{…}` labelled `tnote:t1`, 1611.08809)
  for note in document.findnodes("//ltx:pubnote | //ltx:title//ltx:note", None) {
    placed.beside.push(dom::text_without_marks(&note));
  }
  let check = with_ledger(|ledger| content_check(ledger, &placed, &emit::kept_whole));
  for (words, what) in [
    (
      &check.lost,
      "words the author blocks print are missing from the placed frontmatter",
    ),
    (
      &check.excess,
      "words or addresses appear in the placed frontmatter more often than the author blocks print them",
    ),
  ] {
    if !words.is_empty() {
      Warn!(
        "unexpected",
        "frontmatter",
        s!(
          "Frontmatter content check: {} {what}: {}.",
          words.len(),
          words.join(" ")
        )
      );
    }
  }
}

/// The API calls the writer leaves as today's output holds them: each made by a frontmatter setter whose binding's
/// family is not written (`ledger::ApiCall::binding`, [`family_written`]), whatever the author block's family —
/// svjour3's `\institute` (sv_support.sty's) under authblk's written `\author`, whose reading numbered the names before
/// svjour3's `\at` as entry 1 and gave them to every author (2609.05466; 95b r3).
fn unwritten_calls() -> BTreeSet<usize> {
  with_ledger(|ledger| {
    ledger
      .entries()
      .iter()
      .filter_map(|entry| match entry {
        Entry::Api(call) if call.binding.as_deref().is_some_and(|b| !family_written(b)) => {
          Some(call.index)
        },
        _ => None,
      })
      .collect()
  })
}

/// The reading `resolved` with every item the calls `unwritten` made ([`unwritten_calls`]) as today's `front` holds it:
/// their take-outs and write-outs given back (the entry whole, the e-mail call one contact: [`give_back_addresses`]),
/// then each item of such a text — any call's: one item per text — owned as today's owns it, none where today's holds
/// none, so no operation writes it; the paper's other items are still written — with whether the reading wanted any of
/// them otherwise (its owners, a take-out, a write-out, a dropped note: the writer's outcome says it declined them).
/// `None` — the paper declined whole, today's kept — where the reading's names do not pair with today's (the re-part
/// writes every item from the reading) or an unwritten call made a creator: a written item would depend on the
/// unwritten reading.
fn keep_as_today(
  resolved: &Resolved,
  front: &dom::Front,
  unwritten: &BTreeSet<usize>,
) -> Result<Option<(Resolved, bool)>> {
  let from = |a: &Assoc| {
    a.source
      .is_some_and(|(index, _)| unwritten.contains(&index))
  };
  if resolved
    .creators
    .iter()
    .any(|c| unwritten.contains(&c.source))
  {
    return Ok(None);
  }
  let (taken, kept_taken): (Vec<_>, Vec<_>) = resolved.taken.iter().cloned().partition(|t| {
    t.source
      .is_some_and(|(index, _)| unwritten.contains(&index))
  });
  let (listed, kept_listed): (Vec<_>, Vec<_>) = resolved
    .listed
    .iter()
    .cloned()
    .partition(|l| from(&l.whole));
  let mut otherwise = !taken.is_empty() || !listed.is_empty();
  let mut out = give_back_addresses(
    &Resolved {
      taken,
      listed,
      ..resolved.clone()
    },
    &dom::Front::default(),
    true,
  )?;
  out.taken = kept_taken;
  out.listed = kept_listed;
  let mut keys: BTreeMap<plan::Key, Assoc> = BTreeMap::new();
  for c in out.creators.iter().filter(|c| !c.name.trim().is_empty()) {
    for (key, a) in plan::creator_items(c) {
      if from(a) {
        keys.entry(key).or_insert_with(|| a.clone());
      }
    }
  }
  for (key, a) in plan::block_items(&out) {
    if from(a) {
      keys.entry(key).or_insert_with(|| a.clone());
    }
  }
  // (a note the reading dropped, ruling 4: today's note kept where today's holds it)
  for a in out.dropped.iter().filter(|a| from(a)) {
    let key = (dom::Kind::Note, dom::norm(&a.text));
    if !key.1.is_empty() {
      otherwise = true;
      keys.entry(key).or_insert_with(|| a.clone());
    }
  }
  if keys.is_empty() {
    return Ok(Some((out, otherwise)));
  }
  let Some(pairs) = plan::pair_names(&out, front) else {
    return Ok(None);
  };
  let have = plan::held(front, &pairs);
  let todays = |key: &plan::Key| -> Vec<Owner> {
    let mut owners: Vec<Owner> = have
      .get(key)
      .into_iter()
      .flatten()
      .map(|(owner, ..)| *owner)
      .collect();
    owners.sort();
    owners.dedup();
    owners
  };
  // (the reading's owners of each such text, as the comparison reads them: whether it wanted them otherwise)
  let mut wanted: BTreeMap<&plan::Key, Vec<Owner>> = BTreeMap::new();
  for (i, c) in out
    .creators
    .iter()
    .enumerate()
    .filter(|(_, c)| !c.name.trim().is_empty())
  {
    for (key, _) in plan::creator_items(c) {
      if let Some((key, _)) = keys.get_key_value(&key) {
        wanted.entry(key).or_default().push(Owner::Creator(i));
      }
    }
  }
  for (key, _) in plan::block_items(&out) {
    if let Some((key, _)) = keys.get_key_value(&key) {
      wanted.entry(key).or_default().push(Owner::Block);
    }
  }
  otherwise |= keys.keys().any(|key| {
    let mut owners = wanted.get(key).cloned().unwrap_or_default();
    owners.sort();
    owners.dedup();
    owners != todays(key)
  });
  let keyed = |key: plan::Key| keys.contains_key(&key);
  for c in &mut out.creators {
    c.affiliations
      .retain(|a| !keyed((dom::Kind::Affiliation, dom::norm(&a.text))));
    c.contacts.retain(|a| !keyed(plan::contact_key(a)));
    c.notes
      .retain(|a| !keyed((dom::Kind::Note, dom::norm(&a.text))));
  }
  out
    .block
    .retain(|a| !keyed((dom::Kind::of_block(&a.role), dom::norm(&a.text))));
  out.dropped.retain(|a| !from(a));
  for (key, assoc) in keys {
    // (today's placement owns it, not the reading's mark: no answered mark is hidden for it, `hide_answered_marks`)
    let assoc = Assoc { label: None, ..assoc };
    for owner in todays(&key) {
      let slots = match (owner, &key.0) {
        (Owner::Block, _) => &mut out.block,
        (Owner::Creator(i), dom::Kind::Affiliation) => &mut out.creators[i].affiliations,
        (Owner::Creator(i), dom::Kind::Note) => &mut out.creators[i].notes,
        (Owner::Creator(i), dom::Kind::Contact(_)) => &mut out.creators[i].contacts,
      };
      insert_in_source_order(slots, assoc.clone());
    }
  }
  Ok(Some((out, otherwise)))
}

/// Insert `assoc` among `items` before the first whose source is past its own (`resolver::in_source_order`'s order),
/// the others left in their order.
fn insert_in_source_order(items: &mut Vec<Assoc>, assoc: Assoc) {
  let place = |a: &Assoc| {
    a.source
      .map(|(index, span)| (index, span.map_or((1, 0), |(start, _)| (0, start))))
  };
  let at = place(&assoc).and_then(|mine| {
    items
      .iter()
      .position(|a| place(a).is_some_and(|theirs| theirs > mine))
  });
  items.insert(at.unwrap_or(items.len()), assoc);
}

/// At the end of `insert_frontmatter`: write the observer's reading of a written family's paper where it differs from
/// today's by the writer's operations alone; else leave the document as it is.
pub(in crate::base_utilities) fn write_observed_frontmatter(document: &mut Document) -> Result<()> {
  if !fm_shadow_enabled() || rehearsal::rehearsing() || emitted() {
    return Ok(());
  }
  let Some(resolved) = written_resolution() else {
    return Ok(());
  };
  let mut front = dom::read(document);
  // (an item an unwritten family's call made is not written: it stays as today's holds it, the paper's other items
  // written — or, where a written item would depend on its reading, the paper declined whole; 95b r3)
  let unwritten = unwritten_calls();
  let mut unwritten_kept = false;
  let resolved = if unwritten.is_empty() {
    resolved
  } else {
    let Some(kept) = keep_as_today(&resolved, &front, &unwritten)? else {
      decline("an unwritten family's call beside a re-part", true);
      return Ok(());
    };
    unwritten_kept = kept.1;
    kept.0
  };
  // (a whole creator of today's the reading made an item from its words alone is a creator again, as today's, with the
  // ink it printed: no word reading licenses a writer operation — review rev94a r3, r4 B1, OD #928; its ink in doubt,
  // nothing is written)
  // (the re-part reads the names and the words of each item as today's holds them: every entry the reading took
  // addresses out of whole again there, and every e-mail call the reading wrote out address by address one contact
  // again on the authors its call's placement gives it — the take-out and the write-out are the plan's,
  // `take_out_again`: 2308.10621; 2109.07363, brief 95e)
  // (a membership grade the reading left out of a name is the name's again where today's holds no contact of it:
  // `give_back_grades`)
  let resolved = give_back_grades(&resolved, &front);
  let taken_out = resolved.clone();
  let resolved = give_back_addresses(&resolved, &dom::Front::default(), true)?;
  let Some(resolved) = repart::concede(&resolved, &front) else {
    decline("a conceded creator's ink", true);
    return Ok(());
  };
  // (today's names parted otherwise than the reading's: the creators written again from the reading, when what is
  // written holds every word today's held — the re-part, OD #928)
  let repart = repart::plan(&resolved, &front);
  if let Some(repart) = &repart
    && writes_a_nameless_creator(repart.makes_block_items().then_some(Owner::Block), &front)
  {
    return decline_nameless(&resolved);
  }
  if let Some(repart) = repart {
    if repart::apply(document, &resolved, &front, &repart)? {
      DebugFeature!("frontmatter", "FRONT writer: the names re-parted");
      with_ledger(|ledger| ledger.writer = Some("written".to_string()));
      front = dom::read(document);
    } else {
      note_refusal("the re-part did not read as the reading");
    }
  }
  // (an entry the reading took addresses out of that today's holds no item of — cut by the same reader — is given back
  // as it was: the reading's take-out written only where today's prints it so, brief 95c's fail-safe; read against
  // today's as the re-part left it, 2308.10621)
  let resolved = Resolved {
    taken: Vec::new(),
    listed: Vec::new(),
    ..resolved
  };
  let mut kept = take_out_again(&resolved, &taken_out, &front, true)?;
  let mut before = plan::difference(&kept, &front);
  let mut planned = plan::plan(&kept, &front)?;
  // (the fail-safe of an entry kept taken out because today's prints its addresses in an item the address reader cuts:
  // where the plan cannot make it cut, it is given back whole, as today's prints it, and the plan writes the rest —
  // astro-ph0007423's own `\address{ e--mail: bulgac@… and magiersk@… }`, whose `e–mail:` the cut reads as no label, had
  // been declined whole, every numbered address on both authors; an address today's holds as an author's contact stays
  // taken out, the paper declined, as 2310.19837's; where that leaves today's as it was, the ledger says the take-out was
  // declined, not `same`, as the write-out's fail-safe below: 0705.3751, 1510.04162, 2506.05001)
  let mut kept_whole = None;
  if planned.is_none() && before.is_some() {
    let whole = take_out_again(&resolved, &taken_out, &front, false)?;
    if whole != kept {
      let whole_before = plan::difference(&whole, &front);
      let whole_plan = plan::plan(&whole, &front)?;
      if whole_plan.is_some() || whole_before.is_none() {
        DebugFeature!(
          "frontmatter",
          "FRONT writer: the entries today's prints the addresses of given back: {before:?}"
        );
        if whole_before.is_none() {
          kept_whole = before.as_ref().map(|d| d.to_string());
        }
        (kept, before, planned) = (whole, whole_before, whole_plan);
      }
    }
  }
  let resolved = kept;
  // (the fail-safe of the write-out: where the plan cannot write a call's e-mails out address by address, each call
  // stays one contact on its authors, as today's holds it — the reading the re-part wrote — and the plan writes the
  // rest; an e-mail split never declines a paper, brief 95e — and where that leaves today's as it was, the ledger still
  // says the write-out was declined, not `same`: the improvement the reading wanted is not hidden, rev95e S2, 1412.1769)
  let resolved = if planned.is_none() && before.is_some() && !resolved.listed.is_empty() {
    let whole = give_back_listed(&resolved);
    let whole_before = plan::difference(&whole, &front);
    let whole_plan = plan::plan(&whole, &front)?;
    if whole_plan.is_some() || whole_before.is_none() {
      DebugFeature!(
        "frontmatter",
        "FRONT writer: the e-mail calls kept whole, their write-out declined: {before:?}"
      );
      kept_whole = before.as_ref().map(|d| d.to_string());
      (before, planned) = (whole_before, whole_plan);
      whole
    } else {
      resolved
    }
  } else {
    resolved
  };
  if fm_shadow_mode() == FmShadow::On {
    with_ledger(|ledger| ledger.compared = Some(resolved.clone()));
  }
  let Some(plan) = planned else {
    // (a later pass that finds the document reading as the reading keeps the earlier pass's outcome: `--streaming`
    // places the frontmatter again at the document's end, after this writer wrote it)
    with_ledger(|ledger| match before {
      None if ledger.writer.is_some() => {},
      None => {
        // (an unwritten family's items kept as today's, nothing else to write: declined, not `same` — the reading's
        // reading of them is not hidden)
        ledger.writer = Some(match &kept_whole {
          Some(d) => s!("declined: {d}"),
          None if unwritten_kept => {
            if fm_shadow_mode() == FmShadow::On {
              ledger.refusal = Some(KEPT_AS_TODAY.to_string());
            }
            s!("declined: {KEPT_AS_TODAY}")
          },
          None => "same".to_string(),
        })
      },
      Some(d) => ledger.writer = Some(s!("declined: {d}")),
    });
    return Ok(());
  };
  if writes_a_nameless_creator(plan.ops.iter().filter_map(Op::made_for), &front) {
    return decline_nameless(&resolved);
  }
  DebugFeature!("frontmatter", "FRONT writer: {} operations", plan.ops.len());
  apply::apply(document, &front, &plan)?;
  // (what was written must read as the observer reads it: anything else is the writer's own defect, reported)
  let after = dom::read(document);
  let difference = plan::difference(&resolved, &after);
  if let Some(difference) = &difference {
    Warn!(
      "unexpected",
      "frontmatter",
      s!("The frontmatter written from the observer's reading differs from it: {difference}")
    );
  }
  if difference.is_none() {
    hide_answered_marks(document, &resolved);
  }
  let outcome = difference.map_or_else(|| "written".to_string(), |d| s!("written-different: {d}"));
  with_ledger(|ledger| ledger.writer = Some(outcome));
  Ok(())
}

/// The reading `resolved` with each membership grade it left out of a name (`name:membership`, resolver/lines.rs) given
/// back to the name — its text after the name's comma, a qualifier the name joins (`name:qualifier`), as the reading
/// read it before OXIDIZED_DESIGN_DIVERGENCES #967 — where today's holds no contact of its role and text: the reading
/// leaves a grade out of the name only where `\lx@add@author` took it out (`membership_read_out`), which it never does
/// out of a macro's text (`\IEEEauthorblockN{\AuthorOne,~\AuthorTwo,~…}`, each `\AuthorN` "Name, {\em{Grade, IEEE}}",
/// 1510.03162) nor out of a membership the author filled with the next name (`\IEEEmembership{Member,~IEEE, Hua~Wei,
/// Member,~IEEE}`, 2507.19818). A grade the re-part has nothing to write from, or two authors' alike that today's holds
/// once, would decline it and leave the names line one creator.
fn give_back_grades(resolved: &Resolved, front: &dom::Front) -> Resolved {
  let held: BTreeSet<plan::Key> = front
    .creators
    .iter()
    .chain(&front.nameless)
    .flat_map(|c| &c.items)
    .filter(|item| !item.key.is_empty())
    .map(|item| (item.kind.clone(), item.key.clone()))
    .collect();
  let mut out = resolved.clone();
  for c in &mut out.creators {
    let (given, kept): (Vec<Assoc>, Vec<Assoc>) =
      std::mem::take(&mut c.contacts).into_iter().partition(|a| {
        a.rule.split_whitespace().any(|t| t == "name:membership")
          && !held.contains(&plan::contact_key(a))
      });
    c.contacts = kept;
    for grade in given {
      c.name = s!("{}, {}", c.name, grade.text);
      c.join(NameJoin::Qualifier);
    }
  }
  out
}

/// The reading `resolved` with each entry it took printed addresses out of (`Resolved::taken`, brief 95c) given back as
/// it was — its text whole, the e-mails made of it (`owner:entry-address`, from its source) gone — where today's holds
/// no affiliation that reads, or cut by the reading's address reader reads (`dom::address_cut`), as what the entry
/// keeps (an entry of addresses alone: one today's holds whole cuts to nothing, or today's holds it nowhere): the cut
/// is the writer's to write, never a difference that would decline the paper.
/// With `all`, every e-mail call the reading wrote out address by address is given back too ([`give_back_listed`]);
/// else they stay written out, the plan's to write (its fail-safe in [`write_observed_frontmatter`]).
fn give_back_addresses(resolved: &Resolved, front: &dom::Front, all: bool) -> Result<Resolved> {
  give_back_addresses_but(resolved, front, all, true)
}

/// [`give_back_addresses`], an entry whose addresses today's prints in an item the address reader cuts kept taken out
/// only with `printed` ([`give_back_entries`]).
fn give_back_addresses_but(
  resolved: &Resolved,
  front: &dom::Front,
  all: bool,
  printed: bool,
) -> Result<Resolved> {
  let out = give_back_entries(resolved, front, all, printed)?;
  Ok(if all { give_back_listed(&out) } else { out })
}

/// The entries of [`give_back_addresses`]. An entry the take-out empties whose own printed label no e-mail contact made
/// of it would carry — every address of it already an e-mail contact of its owner, another print's — is given back too:
/// aa's `\mail{x}` holder "Correspondence to " (aa.cls v7.0 `\footnotetext[0]{\mailname\ \@mail}`) beside the same
/// address in an `\institute` (astro-ph0107243's `email: faraggiana@ts.astro.it`, 1005.3784's `\email`) prints that
/// label nowhere else (brief 97b: a take-out removes what it read, and nothing else the class prints).
/// An address today's holds as a contact is not given back: giving it back took the contact from its author and left
/// the address in the affiliation's text (`\IEEEauthorblockA{$^\dagger$Univ A\\ $^\ddagger$Univ B\\ Email: ann@a.org,
/// bob@b.org}`, 2310.19837): the entry is made whole and cut where the plan makes it (`Op::Make`'s `cut`), or the paper
/// declined, today's contacts kept. One whose addresses today's prints in an item the address reader cuts is kept so only
/// with `printed`: without, it is given back (the fail-safe in [`write_observed_frontmatter`], astro-ph0007423).
fn give_back_entries(
  resolved: &Resolved,
  front: &dom::Front,
  all: bool,
  printed: bool,
) -> Result<Resolved> {
  if resolved.taken.is_empty() {
    return Ok(resolved.clone());
  }
  let names: Vec<&str> = resolved.creators.iter().map(|c| c.name.as_str()).collect();
  let spells = plan::spells_one_of(&names);
  let items: Vec<&dom::Item> = front
    .creators
    .iter()
    .chain(&front.nameless)
    .flat_map(|c| &c.items)
    .filter(|item| item.kind == dom::Kind::Affiliation)
    .collect();
  // (the addresses the reading holds as e-mails: the runs a cut of today's takes)
  let mailed: Vec<String> = resolved
    .creators
    .iter()
    .flat_map(|c| &c.contacts)
    .chain(&resolved.block)
    .filter(|a| a.role == "email")
    .map(|a| dom::norm(&a.text))
    .collect();
  let mailed = |a: &str| mailed.contains(&dom::norm(a));
  // (the addresses today's authors' contacts print — a nameless creator's are no author's: the block's, which a pass
  // after the writer may move, 2303.18104)
  let contacted: BTreeSet<String> = front
    .creators
    .iter()
    .flat_map(|c| &c.items)
    .filter(|item| item.kind == dom::Kind::Contact("email".into()))
    .flat_map(|item| addresses_in(&item.node.get_content()))
    .map(|a| dom::norm(&a))
    .collect();
  let mut out = resolved.clone();
  // (an entry given back is taken no more: the plan reads `taken` as the entries it makes cut, `plan::cut_block_lines`
  // and an API call's pieces — a given-back piece read there as its cut text joined no item of today's, and the fail-safe
  // below declined 1712.07118, hep-ph0108215 still)
  out.taken.clear();
  for taken in &resolved.taken {
    let (original, stripped) = (dom::norm(&taken.original), dom::norm(&taken.stripped));
    let cut_to =
      |item: &dom::Item| dom::address_cut(&item.node, &spells, &mailed).map(|cut| cut.key);
    // (its e-mails, each a contact of today's or printed in an affiliation of today's the address reader cuts — this one
    // not where the reading gave it the entry's only carrier while the entry prints an address another author's own:
    // the entry's text then lists the block's addresses, no evidence the carrier's is among them, 2403.12766's
    // `20200142@hznu.edu.cn`, Qian Wang's, read as Zheng Zhang's under `$^6$NYU Shanghai`, whose line lists
    // `{wangcunxiang, zhangyue}@westlake.edu.cn; ruoxining@outlook.com` too)
    let carriers: Vec<usize> = taken.holders.iter().filter_map(|(c, ..)| *c).collect();
    let printed_here: BTreeSet<String> = addresses_in(&taken.original)
      .iter()
      .map(|a| dom::norm(a))
      .collect();
    let lists_others = resolved
      .creators
      .iter()
      .enumerate()
      .filter(|(i, _)| !carriers.contains(i))
      .flat_map(|(_, c)| &c.contacts)
      .any(|a| a.rule.contains("owner:spelled") && printed_here.contains(&dom::norm(&a.text)));
    let entry_mails: Vec<(String, bool)> = resolved
      .creators
      .iter()
      .flat_map(|c| &c.contacts)
      .chain(&resolved.block)
      .filter(|a| a.source == taken.source && a.rule.contains("owner:entry-address"))
      .map(|a| {
        let weak = lists_others && a.rule.contains("owner:only-carrier");
        (dom::norm(&a.text), weak)
      })
      .collect();
    let printed_cut = |mail: &String| {
      items.iter().any(|item| {
        dom::address_cut(&item.node, &spells, &mailed).is_some_and(|cut| {
          cut
            .runs
            .iter()
            .flat_map(|run| &run.addresses)
            .any(|(a, _)| dom::norm(a) == *mail)
        })
      })
    };
    let held_as_contact = !entry_mails.is_empty()
      && entry_mails
        .iter()
        .all(|(mail, weak)| contacted.contains(mail) || (printed && !*weak && printed_cut(mail)));
    let holds = if stripped.is_empty() {
      !items.iter().any(|item| item.key == original)
        || items
          .iter()
          .any(|item| item.key == original && cut_to(item).is_some_and(|k| k.is_empty()))
    } else {
      items.iter().any(|item| {
        item.key == stripped
          || (item.key == original && cut_to(item).as_deref() == Some(stripped.as_str()))
      })
    };
    let label_lost = || -> Result<bool> {
      if !stripped.is_empty()
        || resolved
          .creators
          .iter()
          .flat_map(|c| &c.contacts)
          .any(|a| a.source == taken.source && a.rule.contains("owner:entry-address"))
      {
        return Ok(false);
      }
      for item in items.iter().filter(|item| item.key == original) {
        if plan::holder_label(&item.node)?.is_some() {
          return Ok(true);
        }
      }
      Ok(false)
    };
    if (holds || held_as_contact) && !all && !label_lost()? {
      out.taken.push(taken.clone());
      continue;
    }
    give_back_entry(&mut out, taken);
  }
  Ok(out)
}

/// The entry `taken` given back in the reading `out`: its addresses, taken out, no contacts of its holders.
pub(super) fn give_back_entry(out: &mut Resolved, taken: &resolver::Taken) {
  for creator in &mut out.creators {
    creator
      .contacts
      .retain(|a| !(a.source == taken.source && a.rule.contains("owner:entry-address")));
  }
  for (holder, at, assoc) in &taken.holders {
    let slots = match holder {
      Some(c) => &mut out.creators[*c].affiliations,
      None => &mut out.block,
    };
    match slots.iter().position(|a| {
      a.source == taken.source
        && a.text == taken.stripped
        && a.rule.ends_with(" ruling3:address-out")
    }) {
      Some(i) => slots[i] = assoc.clone(),
      None => slots.insert((*at).min(slots.len()), assoc.clone()),
    }
  }
}

/// The reading `resolved` with each e-mail call printing several addresses alone that it wrote out address by address
/// (`Resolved::listed`, brief 95c G3) given back as today's holds it — its written-out addresses (`ruling3:written-out`
/// after the call's own rule, from its source) gone, the one contact the call printed on each of the authors its
/// call's placement gives it (`api:annotate:last`: amsart's `\email` after a `\author` naming several people,
/// 2109.07363), in source order: what the name re-part reads (OD #928 writes only an item today's holds, brief 95e).
fn give_back_listed(resolved: &Resolved) -> Resolved {
  let mut out = resolved.clone();
  for listed in &resolved.listed {
    let rule = s!("{} ruling3:written-out ", listed.whole.rule);
    let written = |a: &Assoc| {
      a.source == listed.whole.source && a.role == listed.whole.role && a.rule.starts_with(&rule)
    };
    for creator in &mut out.creators {
      creator.contacts.retain(|a| !written(a));
    }
    for &c in &listed.owners {
      let Some(creator) = out.creators.get_mut(c) else {
        continue;
      };
      if !creator.contacts.contains(&listed.whole) {
        creator.contacts.push(listed.whole.clone());
        in_source_order(&mut creator.contacts);
      }
    }
  }
  out
}

/// The reading the plan writes: the take-outs of `taken_out` (the reading as resolved) that today's `front` can cut
/// ([`give_back_addresses`]) and its e-mail calls written out address by address — when the re-part's reading
/// `resolved` is `taken_out` with every take-out and write-out given back (no creator conceded); else `resolved`,
/// nothing taken out, each call one contact (the fail-safe, as today's holds it).
fn take_out_again(
  resolved: &Resolved,
  taken_out: &Resolved,
  front: &dom::Front,
  printed: bool,
) -> Result<Resolved> {
  let whole = Resolved {
    taken: Vec::new(),
    listed: Vec::new(),
    ..give_back_addresses(taken_out, &dom::Front::default(), true)?
  };
  if (taken_out.taken.is_empty() && taken_out.listed.is_empty()) || *resolved != whole {
    return Ok(resolved.clone());
  }
  give_back_addresses_but(taken_out, front, false, printed)
}

/// The writer's invariant: it never makes a creator without a name (review rev94b r3 B2). An item made from the
/// queued tokens for the author block's own (`Owner::Block`) where today's holds no nameless creator would make one
/// (`apply::block_creator`, the re-part's block creator): an entry the reading could not give a name — a cut entry no
/// mark labels. Every written family's paper is then declined whole — today's output — and a Warning says so
/// ([`decline_nameless`]). A copy of today's item a ruling gives the block — an address no author spells
/// (`owner:inner-email`, `ruling3:written-out`: 1811.05243, 1812.02463, 2403.14668, 2501.11467; 2105.11343, 2107.09414,
/// 2508.05637, which the invariant over every item had left with today's shared e-mail) — is the block's by that
/// ruling, not by doubt ([`plan::Op::made_for`], `repart::Repart::makes_block_items`); so is a cut block line no name
/// requests (`plan::Op::Make`'s `unanswered`: a labelled item nobody answers goes to the block, 2007.02542).
fn writes_a_nameless_creator(owners: impl IntoIterator<Item = Owner>, front: &dom::Front) -> bool {
  front.nameless.is_empty() && owners.into_iter().any(|owner| owner == Owner::Block)
}

/// The writer's decline where an unwritten family's items kept as today's are all it could write ([`keep_as_today`]).
const KEPT_AS_TODAY: &str = "an unwritten family's items kept as today's";

/// Decline the paper's writing for [`writes_a_nameless_creator`]: a Warning naming the block the reading's own items
/// come from, today's frontmatter kept.
fn decline_nameless(resolved: &Resolved) -> Result<()> {
  let block = resolved
    .block
    .iter()
    .find_map(|a| a.source.map(|(index, _)| index))
    .map_or_else(|| "?".to_string(), |index| index.to_string());
  Warn!(
    "unexpected",
    "frontmatter",
    s!(
      "The frontmatter observer's reading would write a creator with no name for the author block's own items (ledger entry {block}); the frontmatter is kept as the API wrote it"
    )
  );
  decline("a creator with no name", false);
  Ok(())
}

/// The ink of a rehearsed piece, kept for the writer's checks.
/// The marks today's name shows as no entry's (`show_unanswered_marks`' `ltx:sup`, marked `_unanswered`) that an item
/// the writer wrote answers, taken from the name — as today's takes those its own entries answer: the writer's cut gave
/// the entry its label (2510.24717's `<sup>5</sup>` on the names the block-line cut gave the Beijing Academy of
/// Artificial Intelligence; `Bob Baker$^2$` over `{\small $~^2$ Univ of Colorado}`, the joined line today's held —
/// review rev94a r2 S3). Each name read paired with the reading's (`plan::pair_names`), the labels of its items.
fn hide_answered_marks(document: &mut Document, resolved: &Resolved) {
  let front = dom::read(document);
  let Some(pairs) = plan::pair_names(resolved, &front) else {
    return;
  };
  // (the printed glyphs keyed as the labels are, `*` and `∗` one: `queue_digest::one_asterisk`)
  let unify = one_asterisk;
  for (i, j) in pairs {
    let c = &resolved.creators[i];
    let answered: Vec<String> = c
      .affiliations
      .iter()
      .chain(&c.contacts)
      .chain(&c.notes)
      .filter_map(|a| a.label.as_deref())
      .flat_map(mark_labels)
      .filter_map(|label| unanswered_mark_text(&label))
      .map(|m| unify(&m))
      .collect();
    if answered.is_empty() {
      continue;
    }
    for sup in document.findnodes(
      "ltx:personname/ltx:sup[@_unanswered]",
      Some(&front.creators[j].node),
    ) {
      let shown = sup.get_content();
      let kept: Vec<&str> = shown
        .split(',')
        .filter(|m| !answered.contains(&unify(m)))
        .collect();
      if kept.is_empty()
        || (kept.len() < shown.split(',').count()
          && !dom::take_sup_marks(&sup, &|m| answered.contains(&unify(m))))
      {
        document.remove_node(sup);
      }
    }
  }
}

pub(super) fn piece_span(events: &[Event]) -> Option<(usize, usize)> {
  fn walk(events: &[Event], span: &mut Option<(usize, usize)>) {
    for event in events {
      if let Some((start, end)) = event.span {
        let (start, end) = (usize::from(start), usize::from(end));
        *span = Some(match *span {
          Some((s, e)) => (s.min(start), e.max(end)),
          None => (start, end),
        });
      }
      walk(&event.inner, span);
    }
  }
  let mut span = None;
  walk(events, &mut span);
  span
}

/// Whether a rule is one the writer writes: a change tag's — but not an entry the reading leaves to the block because
/// no name requests its label (`label:unrequested owner:block`, `ruling2:uncarried owner:block`): its tag says how its
/// label was read, and no ruling decided an owner, so today's placement stays (OD #925: the kernel `\author`'s
/// `\address` inside it, espcrc's after each name — 78s's `espcrc_addressmark_key_not_printed`, hep-lat0409072 —, a
/// numbered line under names printing no mark, 77n's `parbox_names_split_and_drop_the_box`, 1205.5573) — except as a
/// piece of a block-line cut whose other lines a ruling decided (`plan::cut_block_lines`, 2007.02542).
pub(super) fn writes(assoc: &Assoc) -> bool {
  let unowned = |tag: &str| assoc.rule.contains(tag);
  is_change_rule(&assoc.rule)
    && !unowned("label:unrequested owner:block")
    && !unowned("ruling2:uncarried owner:block")
}

#[cfg(test)]
mod tests {
  use super::*;

  /// The invariant: an item for the block's own creator makes a nameless one only where today's holds none.
  #[test]
  fn the_writer_never_writes_a_nameless_creator() {
    let none = dom::Front::default();
    assert!(writes_a_nameless_creator(
      [Owner::Creator(0), Owner::Block],
      &none
    ));
    assert!(!writes_a_nameless_creator(
      [Owner::Creator(0), Owner::Creator(1)],
      &none
    ));
    assert!(!writes_a_nameless_creator(None, &none));
    // (today's holds a nameless creator already: the block's own items are its, no new one — review rev94b r4 N1)
    let doc = libxml::tree::Document::new().expect("a document");
    let node = Node::new("creator", None, &doc).expect("a node");
    let nameless = dom::Front {
      creators: Vec::new(),
      nameless: vec![dom::Creator {
        node:  node.clone(),
        name:  String::new(),
        items: Vec::new(),
      }],
      pubnotes: Vec::new(),
    };
    assert!(!writes_a_nameless_creator([Owner::Block], &nameless));
    // (only an item made from the queued tokens counts: a copy of today's, a cut block line no name requests — the
    // block's by the placement order, 98e —, a written-out address, a removal and a role set make none — `Op::made_for`)
    let make = |owner, unanswered| Op::Make {
      owner,
      made: plan::Made::Affiliation,
      tokens: Tokens::new(Vec::new()),
      template: node.clone(),
      before: None,
      label: None,
      unanswered,
      cut: None,
    };
    assert_eq!(make(Owner::Block, false).made_for(), Some(Owner::Block));
    assert_eq!(
      make(Owner::Creator(1), false).made_for(),
      Some(Owner::Creator(1))
    );
    for op in [
      make(Owner::Block, true),
      Op::Copy {
        from: node.clone(),
        to:   Owner::Block,
      },
      Op::WriteOut {
        owner:    Owner::Block,
        template: node.clone(),
        address:  "a@x.org".to_string(),
      },
      Op::Remove { node: node.clone() },
      Op::Role {
        node: node.clone(),
        role: "email".to_string(),
      },
    ] {
      assert_eq!(op.made_for(), None, "{op:?}");
    }
  }

  /// A declined paper says so: one Warning, the ledger's writer state naming the reason ([`decline_nameless`]). No real
  /// shape reaches it today (an unrequested entry the reading cuts is the one entry its leading marks label,
  /// `events::lex_events`, and the API's own cut makes the nameless creator itself): driven directly.
  #[test]
  fn a_declined_paper_warns_once() {
    use latexml_core::common::error::{LogStatus, get_status};
    let before = get_status(LogStatus::Warning);
    decline_nameless(&Resolved::default()).expect("declined");
    assert_eq!(get_status(LogStatus::Warning), before + 1);
    assert_eq!(
      with_ledger(|ledger| ledger.writer.clone()),
      Some("declined: a creator with no name".to_string())
    );
    with_ledger(|ledger| ledger.writer = None);
  }
}
