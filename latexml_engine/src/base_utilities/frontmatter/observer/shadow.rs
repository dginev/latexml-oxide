//! The observer's shadow switch and dump (OXIDIZED_DESIGN_DIVERGENCES #837). Under `LATEXML_FM_SHADOW=1` (read once
//! per process, at runtime; unset on the fleet) the ledger records the document's calls, the flush
//! (`digest_front_matter`) rehearses and lexes each block not rehearsed yet and each API call's content, and the CLI
//! writes `<dest>.fmshadow.json` ([`write_fm_shadow`]) with the resolver's tuples (`resolved`, S2a.2). Without it
//! nothing of the observer runs. With it the conversion's output is the same: nothing the observer finds is used.

use std::{cell::Cell, fmt::Write as _, sync::OnceLock};

use super::{
  events::{Event, EventKind, SepKind, lex_events},
  ledger::{Entry, Outcome, SkipReason, Stamp, reset_ledger, stamped_block_tokens, with_ledger},
  policy::{TitleMarks, policy_for},
  rehearsal::{Rehearsal, rehearse},
  resolver::{Assoc, Resolved, ledger_inputs, resolve},
  *,
};

static FM_SHADOW_ENV: OnceLock<bool> = OnceLock::new();

thread_local! {
  static FM_SHADOW_OVERRIDE: Cell<Option<bool>> = const { Cell::new(None) };
}

/// Whether the frontmatter observer runs: `LATEXML_FM_SHADOW=1` in the environment (a runtime read, once per process),
/// unless a test set this thread's override ([`set_fm_shadow_override`]). The variable stays **unset on the fleet**
/// (`cortex_worker` writes no dump; the shadow census runs through the local CLI: notes decision 5,
/// OXIDIZED_DESIGN_DIVERGENCES #837).
pub fn fm_shadow_enabled() -> bool {
  FM_SHADOW_OVERRIDE.with(Cell::get).unwrap_or_else(|| {
    *FM_SHADOW_ENV.get_or_init(|| std::env::var_os("LATEXML_FM_SHADOW").is_some_and(|v| v == "1"))
  })
}

/// Set (`Some`) or clear (`None`) this thread's override of the shadow switch (tests: the environment is read-only
/// to them).
pub fn set_fm_shadow_override(on: Option<bool>) { FM_SHADOW_OVERRIDE.with(|o| o.set(on)); }

/// Forget the observer's ledger: a conversion's start (the converter's session preparation, with the field
/// provenance's reset).
pub fn reset_fm_shadow() {
  reset_ledger();
  ledger::reset_author_owner();
  rehearsal::reset_rehearsing();
}

/// The flush's part (`digest_front_matter`, before the queue is digested): rehearse and lex each author block not
/// rehearsed yet. A block without positions (out of positions) is skipped and says why; one no rehearsal could run for
/// (a journal already open: the flush is itself inside a speculative run) waits for the next flush, deferred: left so
/// at the end, the dump reports that reason (`JournalUnavailable`), not "no flush after the call".
pub(in crate::base_utilities) fn shadow_flush() -> Result<()> {
  if !fm_shadow_enabled() || rehearsal::rehearsing() {
    return Ok(());
  }
  let pending: Vec<usize> = with_ledger(|ledger| {
    ledger
      .entries
      .iter()
      .enumerate()
      .filter_map(|(i, entry)| match entry {
        Entry::Author(block)
          if matches!(block.outcome, Outcome::Pending | Outcome::Deferred(_)) =>
        {
          Some(i)
        },
        _ => None,
      })
      .collect()
  });
  for i in pending {
    let Some(block) = with_ledger(|ledger| match ledger.entries.get(i) {
      Some(Entry::Author(block)) => Some(block.clone()),
      _ => None,
    }) else {
      continue;
    };
    let outcome = if block.stamp == Stamp::Overflow {
      Outcome::Skipped(SkipReason::Overflow)
    } else if block.tokens.is_empty() {
      Outcome::Skipped(SkipReason::Empty)
    } else {
      let title_marks = policy_for(&block.binding, block.hook).title_marks;
      match rehearse(stamped_block_tokens(&block), title_marks, lex_events)? {
        Rehearsal::Done { value, errors, warnings } => Outcome::Events {
          events: value,
          errors,
          warnings,
        },
        // (waits for the next flush; if none comes, the dump says why it was never rehearsed)
        Rehearsal::Skipped(SkipReason::JournalUnavailable) => {
          Outcome::Deferred(SkipReason::JournalUnavailable)
        },
        Rehearsal::Skipped(reason) => Outcome::Skipped(reason),
      }
    };
    with_ledger(|ledger| {
      if let Some(Entry::Author(block)) = ledger.entries.get_mut(i) {
        block.outcome = outcome;
      }
    });
  }
  // the API calls made outside the blocks' expansions: their contents rehearsed and lexed as a block's are (the
  // resolver reads an affiliation's leading marks and text, an e-mail's address, a creator's name from them)
  let calls: Vec<(usize, Tokens)> = with_ledger(|ledger| {
    ledger
      .entries
      .iter()
      .enumerate()
      .filter_map(|(i, entry)| match entry {
        Entry::Api(call) if !call.inside && !call.rehearsed => Some((i, call.content.clone())),
        _ => None,
      })
      .collect()
  });
  for (i, content) in calls {
    let events = if content.is_empty() {
      Some(Vec::new())
    } else {
      match rehearse(content, TitleMarks::Fnsymbol, lex_events)? {
        Rehearsal::Done { value, .. } => Some(value),
        // (a flush inside a speculative run: the next flush rehearses it)
        Rehearsal::Skipped(SkipReason::JournalUnavailable) => None,
        Rehearsal::Skipped(_) => Some(Vec::new()),
      }
    };
    if let Some(events) = events {
      with_ledger(|ledger| {
        if let Some(Entry::Api(call)) = ledger.entries.get_mut(i) {
          call.events = events;
          call.rehearsed = true;
        }
      });
    }
  }
  // (the flush empties the frontmatter queue: the calls after it are queued from position 0 again)
  with_ledger(|ledger| ledger.flushes += 1);
  Ok(())
}

/// A JSON string literal.
fn json_str(out: &mut String, s: &str) {
  out.push('"');
  for c in s.chars() {
    match c {
      '"' => out.push_str("\\\""),
      '\\' => out.push_str("\\\\"),
      '\n' => out.push_str("\\n"),
      '\r' => out.push_str("\\r"),
      '\t' => out.push_str("\\t"),
      c if (c as u32) < 0x20 => {
        let _ = write!(out, "\\u{:04x}", c as u32);
      },
      c => out.push(c),
    }
  }
  out.push('"');
}

fn json_opt_str(out: &mut String, s: Option<&str>) {
  match s {
    Some(s) => json_str(out, s),
    None => out.push_str("null"),
  }
}

fn json_span(out: &mut String, span: Option<(usize, usize)>) {
  match span {
    Some((start, end)) => {
      let _ = write!(out, "[{start},{end}]");
    },
    None => out.push_str("null"),
  }
}

fn json_event(out: &mut String, event: &Event) {
  out.push_str("{\"kind\":");
  json_str(out, event.kind.name());
  if let EventKind::Sep(sep) = &event.kind {
    out.push_str(",\"sep\":");
    match sep {
      SepKind::And(command) => {
        out.push_str("\"and\",\"command\":");
        json_str(out, command);
      },
      SepKind::Column(character) => {
        out.push_str("\"column\",\"command\":");
        json_str(out, character);
      },
      SepKind::Comma => json_str(out, "comma"),
      SepKind::Word(word) => {
        out.push_str("\"word\",\"word\":");
        json_str(out, word);
      },
      SepKind::Gap { items, width, fill } => {
        let _ = write!(
          out,
          "\"gap\",\"items\":{items},\"width\":{width},\"fill\":{fill}"
        );
      },
    }
  }
  out.push_str(",\"span\":");
  json_span(
    out,
    event.span.map(|(s, e)| (usize::from(s), usize::from(e))),
  );
  if !event.text.is_empty() {
    out.push_str(",\"text\":");
    json_str(out, &event.text);
  }
  if let Some(label) = &event.label {
    out.push_str(",\"label\":");
    json_str(out, label);
  }
  if event.kind == EventKind::Mark {
    let _ = write!(
      out,
      ",\"symbol\":{},\"line_first\":{}",
      event.symbol, event.line_first
    );
  }
  if let Some(role) = &event.role {
    out.push_str(",\"role\":");
    json_str(out, role);
  }
  if let Some(target) = &event.target {
    out.push_str(",\"target\":");
    json_str(out, target);
  }
  if event.nest > 0 {
    let _ = write!(out, ",\"nest\":{}", event.nest);
  }
  if !event.inner.is_empty() {
    out.push_str(",\"inner\":[");
    for (i, inner) in event.inner.iter().enumerate() {
      if i > 0 {
        out.push(',');
      }
      json_event(out, inner);
    }
    out.push(']');
  }
  out.push('}');
}

/// The ledger as JSON (`<dest>.fmshadow.json`): its entries in call order — author blocks with their events, title
/// stamps, API calls — and a summary (blocks with events, skipped by reason, out of positions, rehearsals that held
/// diagnostics). An author block's ApiCall events are the API calls after it its own expansion did not make.
pub fn fm_shadow_json() -> String {
  with_ledger(|ledger| {
    let mut out = String::from("{\"version\":1,\"entries\":[");
    let mut with_events = 0usize;
    let mut held = 0usize;
    let mut skipped: Vec<(&str, usize)> = Vec::new();
    let mut blocks = 0usize;
    let mut overflow = 0usize;
    for (n, entry) in ledger.entries.iter().enumerate() {
      if n > 0 {
        out.push(',');
      }
      match entry {
        Entry::Title(title) => {
          let _ = write!(
            out,
            "{{\"index\":{},\"kind\":\"title\",\"length\":{},\"stamp\":{},\"range\":",
            title.index,
            title.length,
            if title.range.is_some() {
              "\"kernel\""
            } else {
              "\"overflow\""
            }
          );
          json_span(&mut out, title.range);
          out.push('}');
          if title.range.is_none() {
            overflow += 1;
          }
        },
        Entry::Api(call) => {
          let _ = write!(out, "{{\"index\":{},\"kind\":\"api\",\"tag\":", call.index);
          json_str(&mut out, &call.tag);
          out.push_str(",\"role\":");
          json_str(&mut out, &call.role);
          let _ = write!(
            out,
            ",\"epoch\":{},\"queue_position\":{},\"after_block\":{},\"inside\":{}",
            call.epoch,
            call.queue_position,
            call
              .after_block
              .map_or("null".to_string(), |b| b.to_string()),
            call.inside
          );
          out.push_str(",\"attrs\":");
          json_str(&mut out, &call.attrs);
          if !call.parent.is_empty() {
            out.push_str(",\"parent\":");
            json_str(&mut out, &call.parent);
          }
          if call.rehearsed {
            out.push_str(",\"events\":[");
            for (i, event) in call.events.iter().enumerate() {
              if i > 0 {
                out.push(',');
              }
              json_event(&mut out, event);
            }
            out.push(']');
          }
          out.push('}');
        },
        Entry::Author(block) => {
          blocks += 1;
          let _ = write!(
            out,
            "{{\"index\":{},\"kind\":\"author\",\"hook\":",
            block.index
          );
          json_str(&mut out, block.hook);
          out.push_str(",\"policy\":");
          json_str(&mut out, &block.policy);
          out.push_str(",\"binding\":");
          json_str(&mut out, &block.binding);
          out.push_str(",\"owner\":");
          json_str(&mut out, &block.owner);
          if let Some(counter) = block.counter {
            let _ = write!(out, ",\"counter\":{counter}");
          }
          let _ = write!(
            out,
            ",\"replace\":{},\"sectioned\":{},\"cautious\":{},\"keyvals\":",
            block.replace, block.sectioned, block.cautious
          );
          json_opt_str(&mut out, block.keyvals.as_deref());
          let _ = write!(
            out,
            ",\"epoch\":{},\"queue_position\":{},\"authors_queued\":{},\"stamp\":",
            block.epoch, block.queue_position, block.authors_queued
          );
          json_str(&mut out, block.stamp.name());
          out.push_str(",\"range\":");
          json_span(
            &mut out,
            (block.stamp != Stamp::Overflow).then_some(block.range),
          );
          if block.stamp == Stamp::Overflow {
            overflow += 1;
          }
          out.push_str(",\"source\":");
          json_str(&mut out, &block.source);
          out.push_str(",\"status\":");
          let mut events: Vec<Event> = Vec::new();
          match &block.outcome {
            Outcome::Pending | Outcome::Skipped(_) | Outcome::Deferred(_) => {
              let reason = skip_reason(&block.outcome).unwrap_or(SkipReason::NoFlush);
              json_str(&mut out, "skipped");
              out.push_str(",\"reason\":");
              json_str(&mut out, reason.name());
              bump(&mut skipped, reason.name());
            },
            Outcome::Events {
              events: lexed,
              errors,
              warnings,
            } => {
              json_str(&mut out, "events");
              let _ = write!(out, ",\"errors\":{errors},\"warnings\":{warnings}");
              with_events += 1;
              if errors + warnings > 0 {
                held += 1;
              }
              events.clone_from(lexed);
            },
          }
          // the setters called outside the field after the block, up to the next block
          for later in &ledger.entries[n + 1..] {
            match later {
              Entry::Author(_) => break,
              Entry::Api(call) if !call.inside && call.after_block == Some(block.index) => {
                events.push(Event {
                  kind:       EventKind::ApiCall,
                  span:       None,
                  text:       String::new(),
                  label:      Some(s!("queue:{}:{}", call.epoch, call.queue_position)),
                  symbol:     false,
                  line_first: false,
                  role:       Some(s!("{} {}", call.tag, call.role)),
                  target:     None,
                  inner:      Vec::new(),
                  nest:       0,
                });
              },
              _ => {},
            }
          }
          if matches!(block.outcome, Outcome::Events { .. }) {
            out.push_str(",\"events\":[");
            for (i, event) in events.iter().enumerate() {
              if i > 0 {
                out.push(',');
              }
              json_event(&mut out, event);
            }
            out.push(']');
          }
          out.push('}');
        },
      }
    }
    out.push_str("],\"resolved\":");
    json_resolved(&mut out, &resolve(&ledger_inputs(ledger)));
    let _ = write!(
      out,
      ",\"summary\":{{\"blocks\":{blocks},\"with_events\":{with_events},\"held\":{held},\"overflow\":{overflow},\"skipped\":{{"
    );
    for (i, (reason, count)) in skipped.iter().enumerate() {
      if i > 0 {
        out.push(',');
      }
      json_str(&mut out, reason);
      let _ = write!(out, ":{count}");
    }
    out.push_str("}}}\n");
    out
  })
}

fn json_assoc(out: &mut String, assoc: &Assoc) {
  out.push_str("{\"role\":");
  json_str(out, &assoc.role);
  out.push_str(",\"text\":");
  json_str(out, &assoc.text);
  if let Some(label) = &assoc.label {
    out.push_str(",\"label\":");
    json_str(out, label);
  }
  out.push_str(",\"rule\":");
  json_str(out, &assoc.rule);
  if let Some((index, span)) = assoc.source {
    // (keyed by the block and the span: spans are unique only within a block)
    let _ = write!(out, ",\"source\":[{index},");
    json_span(out, span.map(|(s, e)| (usize::from(s), usize::from(e))));
    out.push(']');
  }
  out.push('}');
}

fn json_assocs(out: &mut String, key: &str, assocs: &[Assoc]) {
  let _ = write!(out, ",\"{key}\":[");
  for (i, assoc) in assocs.iter().enumerate() {
    if i > 0 {
      out.push(',');
    }
    json_assoc(out, assoc);
  }
  out.push(']');
}

/// The resolver's tuples: the creators (name without marks, requests, the rule that split it out, its block) with
/// their affiliations, contacts and notes, and the block's own.
fn json_resolved(out: &mut String, resolved: &Resolved) {
  out.push_str("{\"creators\":[");
  for (i, creator) in resolved.creators.iter().enumerate() {
    if i > 0 {
      out.push(',');
    }
    out.push_str("{\"name\":");
    json_str(out, &creator.name);
    out.push_str(",\"requests\":[");
    for (j, request) in creator.requests.iter().enumerate() {
      if j > 0 {
        out.push(',');
      }
      json_str(out, request);
    }
    out.push_str("],\"rule\":");
    json_str(out, &creator.rule);
    let _ = write!(out, ",\"source\":{}", creator.source);
    json_assocs(out, "affiliations", &creator.affiliations);
    json_assocs(out, "contacts", &creator.contacts);
    json_assocs(out, "notes", &creator.notes);
    out.push('}');
  }
  out.push(']');
  json_assocs(out, "block", &resolved.block);
  json_assocs(out, "dropped", &resolved.dropped);
  out.push_str(",\"policies\":[");
  for (i, (index, name)) in resolved.policies.iter().enumerate() {
    if i > 0 {
      out.push(',');
    }
    let _ = write!(out, "[{index},");
    json_str(out, name);
    out.push(']');
  }
  out.push_str("]}");
}

/// Why a block that has no events was not rehearsed: its skip's reason; one deferred by a flush no rehearsal could run
/// at (`JournalUnavailable`) and never rehearsed later, that reason; one no flush came after, `NoFlush`.
fn skip_reason(outcome: &Outcome) -> Option<SkipReason> {
  match outcome {
    Outcome::Pending => Some(SkipReason::NoFlush),
    Outcome::Skipped(reason) | Outcome::Deferred(reason) => Some(*reason),
    Outcome::Events { .. } => None,
  }
}

fn bump<'a>(counts: &mut Vec<(&'a str, usize)>, reason: &'a str) {
  match counts.iter_mut().find(|(r, _)| *r == reason) {
    Some((_, count)) => *count += 1,
    None => counts.push((reason, 1)),
  }
}

/// Write the ledger ([`fm_shadow_json`]) to `<destination>.fmshadow.json`, under the shadow switch only (the CLI, after
/// the conversion's output is written). Its directory is the destination's own.
pub fn write_fm_shadow(destination: &str) -> std::io::Result<()> {
  if !fm_shadow_enabled() {
    return Ok(());
  }
  std::fs::write(s!("{destination}.fmshadow.json"), fm_shadow_json())
}

#[cfg(test)]
mod tests {
  use super::*;

  /// A block a flush deferred because no journal could open there (the flush was inside a speculative run) and no
  /// later flush rehearsed says so, not "no flush after the call" (review revfms2a1_2 finding 6).
  #[test]
  fn deferred_block_reports_the_journal_not_the_flush() {
    assert_eq!(
      skip_reason(&Outcome::Deferred(SkipReason::JournalUnavailable)).map(SkipReason::name),
      Some("journal unavailable")
    );
    assert_eq!(
      skip_reason(&Outcome::Pending).map(SkipReason::name),
      Some("no flush after the call")
    );
    assert_eq!(
      skip_reason(&Outcome::Skipped(SkipReason::Empty)).map(SkipReason::name),
      Some("empty")
    );
    assert_eq!(
      skip_reason(&Outcome::Events {
        events:   Vec::new(),
        errors:   0,
        warnings: 0,
      }),
      None
    );
  }
}
