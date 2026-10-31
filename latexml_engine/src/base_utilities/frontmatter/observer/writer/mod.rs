//! The observer's writer (FM S2b; OXIDIZED_DESIGN_DIVERGENCES #837 "S2b"): for a family whose binding declared its
//! conventions `written` (the pilot: revtex4_support's `revtex`), once `insert_frontmatter` has placed today's
//! frontmatter, the paper's creators are compared with the resolver's reading as `ab_audit fmshadow` compares them
//! ([`dom`], the comparator's tuples), and the paper is written only when every difference is one of the writer's
//! operations ([`plan`]) — an entry given to the authors the reading gives it, one call's marked entries cut apart at
//! their marks, a legend made a note of the names showing its mark, a url printing an address made an e-mail — each
//! decided by a rule holding an adopted ruling's change tag (`resolver::CHANGE_TAGS`). Any other difference leaves the
//! paper as today's: a difference no operation explains is one the comparison would report unclassified.
//!
//! What is written comes from the tokens the document queued: a cut entry's pieces from its call's content, mapped by
//! the spans of a stamped rehearsal of it ([`plan`]), digested as today's digests an entry ([`apply`]). Under the
//! shadow switch only (`LATEXML_FM_SHADOW=1`); unset, nothing of it runs.

use super::{
  events::Event,
  ledger::{Entry, Outcome, SkipReason, with_ledger},
  policy::policy_for,
  resolver::{Assoc, RCreator, Resolved, is_change_rule, ledger_inputs, mark_labels, resolve},
  *,
};

mod apply;
mod dom;
mod plan;
mod repart;

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
  if let Some(repart) = repart::plan(&resolved, &front)
    && repart::apply(document, &resolved, &front, &repart)?
  {
    DebugFeature!("frontmatter", "FRONT writer: the names re-parted");
    with_ledger(|ledger| ledger.writer = Some("written".to_string()));
    front = dom::read(document);
  }
  let before = plan::difference(&resolved, &front);
  let Some(plan) = plan::plan(&resolved, &front)? else {
    // (a later pass that finds the document reading as the reading keeps the earlier pass's outcome: `--streaming`
    // places the frontmatter again at the document's end, after this writer wrote it)
    with_ledger(|ledger| match before {
      None if ledger.writer.is_some() => {},
      None => ledger.writer = Some("same".to_string()),
      Some(d) => ledger.writer = Some(s!("declined: {d}")),
    });
    return Ok(());
  };
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
    for mut sup in document.findnodes(
      "ltx:personname/ltx:sup[@_unanswered]",
      Some(&front.creators[j].node),
    ) {
      let shown = sup.get_content();
      let kept: Vec<&str> = shown
        .split(',')
        .filter(|m| !answered.contains(&unify(m)))
        .collect();
      if kept.is_empty() {
        document.remove_node(sup);
      } else if kept.len() < shown.split(',').count() {
        let _ = sup.set_content(&kept.join(","));
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
