//! The observer's writer (FM S2b; OXIDIZED_DESIGN_DIVERGENCES #837 "S2b"): for a family whose policy entry is
//! `written` (the pilot: revtex4_support's `revtex`), once `insert_frontmatter` has placed today's frontmatter, the
//! paper's creators are compared with the resolver's reading as `ab_audit fmshadow` compares them ([`dom`], the
//! comparator's tuples), and the paper is written only when every difference is one of the writer's operations
//! ([`plan`]) — an entry given to the authors the reading gives it, one call's marked entries cut apart at their marks,
//! a legend made a note of the names showing its mark, a url printing an address made an e-mail — each decided by a
//! rule holding an adopted ruling's change tag (`resolver::CHANGE_TAGS`). Any other difference leaves the paper as
//! today's: a difference no operation explains is one the comparison would report unclassified.
//!
//! What is written comes from the tokens the document queued: a cut entry's pieces from its call's content, mapped by
//! the spans of a stamped rehearsal of it ([`plan`]), digested as today's digests an entry ([`apply`]). Under the
//! shadow switch only (`LATEXML_FM_SHADOW=1`); unset, nothing of it runs.

use super::{
  events::Event,
  ledger::{Entry, Outcome, with_ledger},
  policy::policy_for,
  resolver::{Assoc, Resolved, is_change_rule, ledger_inputs, resolve},
  *,
};

mod apply;
mod dom;
mod plan;

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
            Outcome::Events { errors: 0, .. } => {},
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
  let front = dom::read(document);
  let Some(plan) = plan::plan(&resolved, &front)? else {
    return Ok(());
  };
  DebugFeature!("frontmatter", "FRONT writer: {} operations", plan.ops.len());
  apply::apply(document, &front, &plan)?;
  // (what was written must read as the observer reads it: anything else is the writer's own defect, reported)
  let after = dom::read(document);
  if let Some(difference) = plan::difference(&resolved, &after) {
    Warn!(
      "unexpected",
      "frontmatter",
      s!("The frontmatter written from the observer's reading differs from it: {difference}")
    );
  }
  Ok(())
}

/// The ink of a rehearsed piece, kept for the writer's checks.
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

/// Whether a rule is one the writer writes: a change tag's.
pub(super) fn writes(assoc: &Assoc) -> bool { is_change_rule(&assoc.rule) }
