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
//! unclassified may still be written in its other items.
//!
//! What is written comes from the tokens the document queued: a cut entry's pieces from its call's content, mapped by
//! the spans of a stamped rehearsal of it ([`plan`]), digested as today's digests an entry ([`apply`]). By default (the
//! fleet's) and under the census switch (`LATEXML_FM_SHADOW=1`), for a paper whose author blocks are all a written
//! family's; switched off (`=0`), nothing of it runs (`shadow::FmShadow`).

use super::{
  events::Event,
  ledger::{Entry, Outcome, SkipReason, with_ledger},
  policy::policy_for,
  resolver::{
    Assoc, RCreator, Resolved, in_source_order, is_change_rule, ledger_inputs, mark_labels, resolve,
  },
  *,
};

mod apply;
mod dom;
mod plan;
mod repart;

use plan::{Op, Owner};
// (the writer's cut of a block's entry, which the reading checks an in-block contact's entries by: `events::labelled_entries`)
pub(super) use plan::{entry_forms, labelled_start, span_tokens};

/// The resolution of the document's author blocks when the writer may write them: every block flushed so far
/// rehearsed without an error held, the API calls after them rehearsed, and every block's family `written`.
fn written_resolution() -> Option<Resolved> {
  with_ledger(|ledger| {
    let mut blocks = 0usize;
    for entry in &ledger.entries {
      match entry {
        Entry::Author(block) => {
          blocks += 1;
          match &block.outcome {
            // (an empty call holds nothing to read: llncs's `\and\\` between two names, an empty piece of its split —
            // 2409.09478)
            Outcome::Events { errors: 0, .. } | Outcome::Skipped(SkipReason::Empty) => {},
            _ => return None,
          }
          if !policy_for(&block.binding, block.hook).written {
            return None;
          }
        },
        // (the title is recorded for its marks alone: it never holds the writing back)
        Entry::Api(call) if !call.inside && !call.rehearsed && call.tag != "ltx:title" => {
          return None;
        },
        _ => {},
      }
    }
    (blocks > 0).then(|| resolve(&ledger_inputs(ledger)))
  })
}

/// At the end of `insert_frontmatter`: write the observer's reading of a written family's paper where it differs from
/// today's by the writer's operations alone; else leave the document as it is.
pub(in crate::base_utilities) fn write_observed_frontmatter(document: &mut Document) -> Result<()> {
  if !fm_shadow_enabled() || rehearsal::rehearsing() {
    return Ok(());
  }
  let Some(resolved) = written_resolution() else {
    return Ok(());
  };
  let mut front = dom::read(document);
  // (a whole creator of today's the reading made an item from its words alone is a creator again, as today's, with the
  // ink it printed: no word reading licenses a writer operation — review rev94a r3, r4 B1, OD #928; its ink in doubt,
  // nothing is written)
  // (the re-part reads the names and the words of each item as today's holds them: every entry the reading took
  // addresses out of whole again there, and every e-mail call the reading wrote out address by address one contact
  // again on the authors its call's placement gives it — the take-out and the write-out are the plan's,
  // `take_out_again`: 2308.10621; 2109.07363, brief 95e)
  let taken_out = resolved.clone();
  let resolved = give_back_addresses(&resolved, &dom::Front::default(), true);
  let Some(resolved) = repart::concede(&resolved, &front) else {
    with_ledger(|ledger| {
      if ledger.writer.is_none() {
        ledger.writer = Some("declined: a conceded creator's ink".to_string());
      }
    });
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
  if let Some(repart) = repart
    && repart::apply(document, &resolved, &front, &repart)?
  {
    DebugFeature!("frontmatter", "FRONT writer: the names re-parted");
    with_ledger(|ledger| ledger.writer = Some("written".to_string()));
    front = dom::read(document);
  }
  // (an entry the reading took addresses out of that today's holds no item of — cut by the same reader — is given back
  // as it was: the reading's take-out written only where today's prints it so, brief 95c's fail-safe; read against
  // today's as the re-part left it, 2308.10621)
  let resolved = Resolved {
    taken: Vec::new(),
    listed: Vec::new(),
    ..resolved
  };
  let resolved = take_out_again(&resolved, &taken_out, &front);
  let mut before = plan::difference(&resolved, &front);
  let mut planned = plan::plan(&resolved, &front)?;
  // (the fail-safe of the write-out: where the plan cannot write a call's e-mails out address by address, each call
  // stays one contact on its authors, as today's holds it — the reading the re-part wrote — and the plan writes the
  // rest; an e-mail split never declines a paper, brief 95e — and where that leaves today's as it was, the ledger still
  // says the write-out was declined, not `same`: the improvement the reading wanted is not hidden, rev95e S2, 1412.1769)
  let mut kept_whole = None;
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
  let Some(plan) = planned else {
    // (a later pass that finds the document reading as the reading keeps the earlier pass's outcome: `--streaming`
    // places the frontmatter again at the document's end, after this writer wrote it)
    with_ledger(|ledger| match before {
      None if ledger.writer.is_some() => {},
      None => {
        ledger.writer = Some(match &kept_whole {
          Some(d) => s!("declined: {d}"),
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

/// The reading `resolved` with each entry it took printed addresses out of (`Resolved::taken`, brief 95c) given back as
/// it was — its text whole, the e-mails made of it (`owner:entry-address`, from its source) gone — where today's holds
/// no affiliation that reads, or cut by the reading's address reader reads (`dom::address_cut`), as what the entry
/// keeps (an entry of addresses alone: one today's holds whole cuts to nothing, or today's holds it nowhere): the cut
/// is the writer's to write, never a difference that would decline the paper.
/// With `all`, every e-mail call the reading wrote out address by address is given back too ([`give_back_listed`]);
/// else they stay written out, the plan's to write (its fail-safe in [`write_observed_frontmatter`]).
fn give_back_addresses(resolved: &Resolved, front: &dom::Front, all: bool) -> Resolved {
  let out = give_back_entries(resolved, front, all);
  if all { give_back_listed(&out) } else { out }
}

/// The entries of [`give_back_addresses`].
fn give_back_entries(resolved: &Resolved, front: &dom::Front, all: bool) -> Resolved {
  if resolved.taken.is_empty() {
    return resolved.clone();
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
  let mut out = resolved.clone();
  for taken in &resolved.taken {
    let (original, stripped) = (dom::norm(&taken.original), dom::norm(&taken.stripped));
    let cut_to =
      |item: &dom::Item| dom::address_cut(&item.node, &spells, &mailed).map(|cut| cut.key);
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
    if holds && !all {
      continue;
    }
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
  out
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
fn take_out_again(resolved: &Resolved, taken_out: &Resolved, front: &dom::Front) -> Resolved {
  let whole = Resolved {
    taken: Vec::new(),
    listed: Vec::new(),
    ..give_back_addresses(taken_out, &dom::Front::default(), true)
  };
  if (taken_out.taken.is_empty() && taken_out.listed.is_empty()) || *resolved != whole {
    return resolved.clone();
  }
  give_back_addresses(taken_out, front, false)
}

/// The writer's invariant: it never makes a creator without a name (review rev94b r3 B2). An item made from the
/// queued tokens for the author block's own (`Owner::Block`) where today's holds no nameless creator would make one
/// (`apply::block_creator`, the re-part's block creator): an entry the reading could not give a name — a cut entry no
/// mark labels. Every written family's paper is then declined whole — today's output — and a Warning says so
/// ([`decline_nameless`]). A copy of today's item a ruling gives the block — an address no author spells
/// (`owner:inner-email`, `ruling3:written-out`: 1811.05243, 1812.02463, 2403.14668, 2501.11467; 2105.11343, 2107.09414,
/// 2508.05637, which the invariant over every item had left with today's shared e-mail) — is the block's by that
/// ruling, not by doubt ([`plan::Op::made_for`], `repart::Repart::makes_block_items`).
fn writes_a_nameless_creator(owners: impl IntoIterator<Item = Owner>, front: &dom::Front) -> bool {
  front.nameless.is_empty() && owners.into_iter().any(|owner| owner == Owner::Block)
}

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
  with_ledger(|ledger| ledger.writer = Some("declined: a creator with no name".to_string()));
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
  let unify = |m: &str| m.replace('\u{2217}', "*");
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
/// numbered line under names printing no mark, 77n's `parbox_names_split_and_drop_the_box`, 1205.5573).
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
    };
    assert!(!writes_a_nameless_creator([Owner::Block], &nameless));
    // (only an item made from the queued tokens counts: a copy of today's, a written-out address, a removal and a role
    // set make none — `Op::made_for`)
    let make = |owner| Op::Make {
      owner,
      made: plan::Made::Affiliation,
      tokens: Tokens::new(Vec::new()),
      template: node.clone(),
      before: None,
      label: None,
    };
    assert_eq!(make(Owner::Block).made_for(), Some(Owner::Block));
    assert_eq!(make(Owner::Creator(1)).made_for(), Some(Owner::Creator(1)));
    for op in [
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
