//! The observer's shadow switch and dump (OXIDIZED_DESIGN_DIVERGENCES #837). Three states ([`FmShadow`]):
//! `LATEXML_FM_SHADOW=1` (read once per process, at runtime) — the census: the ledger records the document's calls,
//! the flush (`digest_front_matter`) rehearses and lexes each block not rehearsed yet and each API call's content, the
//! CLI writes `<dest>.fmshadow.json` ([`write_fm_shadow`]) with the resolver's tuples (`resolved`, S2a.2), and a
//! written family's paper is written; `=0` — off, nothing of the observer runs; unset — the default (FM S2b): the
//! ledger records every paper, and only a paper whose author blocks are all a `written` family's (revtex4_support) is
//! rehearsed, read and written, with no dump. A family not written keeps today's output in every state. The census's
//! dump also says who decided the paper (`readback`, `json_read_back`), what the old engine's removal is measured by.

use std::{cell::Cell, fmt::Write as _, sync::OnceLock};

use latexml_core::definition::origin::watch_frontmatter_setter_calls;

use super::{
  events::{
    EmailMeaning, Event, EventKind, HeldError, SepKind, creator_roles, lex_events, lex_with_inks,
    mark_held, with_block_content,
  },
  ledger::{Entry, Outcome, SkipReason, Stamp, reset_ledger, stamped_block_tokens, with_ledger},
  policy::{TitleMarks, policy_for},
  rehearsal::{Rehearsal, rehearse},
  resolver::{Assoc, Resolved, ledger_inputs, resolve},
  writer::{fm_emit_mode, writer_stays_out},
  *,
};

/// The shadow switch's states (module docs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FmShadow {
  /// `LATEXML_FM_SHADOW=1`: every family read, the dump written (the census).
  On,
  /// `LATEXML_FM_SHADOW=0`: nothing of the observer runs (today's frontmatter).
  Off,
  /// Unset: every paper recorded, a written family's read and written (FM S2b's default; the fleet).
  Default,
}

static FM_SHADOW_ENV: OnceLock<FmShadow> = OnceLock::new();

thread_local! {
  static FM_SHADOW_OVERRIDE: Cell<Option<bool>> = const { Cell::new(None) };
}

/// The shadow switch: this thread's test override ([`set_fm_shadow_override`]: on, off), else `LATEXML_FM_SHADOW` (a
/// runtime read, once per process). Unset on the fleet: `cortex_worker` runs the default and writes no dump.
pub fn fm_shadow_mode() -> FmShadow {
  match FM_SHADOW_OVERRIDE.with(Cell::get) {
    Some(true) => FmShadow::On,
    Some(false) => FmShadow::Off,
    None => *FM_SHADOW_ENV.get_or_init(|| match std::env::var_os("LATEXML_FM_SHADOW") {
      Some(v) if v == "1" => FmShadow::On,
      Some(v) if v == "0" => FmShadow::Off,
      _ => FmShadow::Default,
    }),
  }
}

/// Whether the frontmatter observer records (the ledger, the binding loads, a footnote's text): unless switched off.
pub fn fm_shadow_enabled() -> bool { fm_shadow_mode() != FmShadow::Off }

/// Whether the flush rehearses and the writer reads: under the census every paper; by default a paper whose recorded
/// author blocks are all a written family's (as `writer::written_resolution` requires), and under the emitter's `=2` every
/// paper as the census reads it — the content check on the placed output (`writer::check_placed_content`) reads the
/// rehearsed creator calls of a paper with no author block too (llncs's `\author` and `\institute`, 1401.5951: blind
/// without the census); off, none.
fn reads_now() -> bool {
  match fm_shadow_mode() {
    FmShadow::On => true,
    FmShadow::Off => false,
    FmShadow::Default if fm_emit_mode() == 2 => true,
    FmShadow::Default => with_ledger(|ledger| {
      let mut blocks = ledger.entries().iter().filter_map(|entry| match entry {
        Entry::Author(block) => Some(block),
        _ => None,
      });
      let mut any = false;
      blocks.all(|block| {
        any = true;
        policy_for(&block.binding, block.hook).written
      }) && any
    }),
  }
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
  // (no frontmatter setter watched until a written family's block finds an unwritten one's defined:
  // `ledger::watch_unwritten_setters`)
  watch_frontmatter_setter_calls(None);
}

/// The flush's part (`digest_front_matter`, before the queue is digested): rehearse and lex each author block not
/// rehearsed yet. A block without positions (out of positions) is skipped and says why; one no rehearsal could run for
/// (a journal already open: the flush is itself inside a speculative run) waits for the next flush, deferred: left so
/// at the end, the dump reports that reason (`JournalUnavailable`), not "no flush after the call".
pub(in crate::base_utilities) fn shadow_flush() -> Result<()> {
  // (a rehearsal's flush is undone with it; every real flush empties the queue, read or not — the epoch a clear
  // compares, `ledger::record_clear`: an early `\maketitle` before any author block, in default mode, reads nothing
  // but flushes the title it digests, which a later `\title` must not clear; review rev92a_r1 S2)
  if rehearsal::rehearsing() {
    return Ok(());
  }
  if reads_now() {
    read_at_flush()?;
  }
  // (the flush empties the frontmatter queue: the calls after it are queued from position 0 again)
  with_ledger(|ledger| ledger.flushes += 1);
  Ok(())
}

/// [`shadow_flush`]'s reading, when the observer reads.
fn read_at_flush() -> Result<()> {
  // (the labels the calls' keyvals defer, read as this flush's digestion expands them: `\lx@label@ref`, cas's
  // `\author[a]` under entries given after it — 2303.00352, 2603.22658, 2504.19452, 2509.10496)
  with_ledger(|ledger| {
    let resolved: Vec<(usize, String)> = ledger
      .entries()
      .iter()
      .enumerate()
      .filter_map(|(i, entry)| {
        let text = match entry {
          Entry::Author(block) => block.keyvals.as_deref()?,
          Entry::Api(call) => call.attrs.as_str(),
          _ => return None,
        };
        match resolve_label_refs(text) {
          Cow::Owned(resolved) => Some((i, resolved)),
          Cow::Borrowed(_) => None,
        }
      })
      .collect();
    for (i, text) in resolved {
      match &mut ledger.entries_mut()[i] {
        Entry::Author(block) => block.keyvals = Some(text),
        Entry::Api(call) => call.attrs = text,
        _ => {},
      }
    }
  });
  let pending: Vec<usize> = with_ledger(|ledger| {
    ledger
      .entries()
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
  // (the blocks rehearsed in sequence share the conditionals, as TeX reads the author list in one pass — a block's run
  // opens one the next block's closes, 2606.17718 — and the document gets them back as they were after the last:
  // `StomachMark`)
  let blocks_mark = StomachMark::take();
  for i in pending {
    let Some(block) = with_ledger(|ledger| match ledger.entries().get(i) {
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
      let policy = policy_for(&block.binding, block.hook);
      let title_marks = policy.title_marks;
      let block_breaks = policy.block_breaks;
      // (the block's tokens beside it: its in-block contacts' entries are read as the writer can cut them,
      // `events::labelled_entries`)
      let rehearsal = with_block_content(block.range.0, block.tokens.clone(), || {
        rehearse(
          stamped_block_tokens(&block),
          title_marks,
          block_breaks,
          lex_with_inks,
        )
      })?;
      match rehearsal {
        Rehearsal::Done {
          value: (mut value, formulas, inks),
          errors,
          warnings,
          declarations,
          breaks,
          held,
          environments,
          creators,
        } => {
          mark_held(&mut value, &held);
          creator_roles(&mut value, &creators);
          Outcome::Events {
            events: value,
            formulas,
            inks: Box::new(inks),
            errors,
            warnings,
            declarations,
            breaks,
            held,
            environments,
            creators,
          }
        },
        // (waits for the next flush; if none comes, the dump says why it was never rehearsed)
        Rehearsal::Skipped(SkipReason::JournalUnavailable) => {
          Outcome::Deferred(SkipReason::JournalUnavailable)
        },
        Rehearsal::Skipped(reason) => Outcome::Skipped(reason),
      }
    };
    with_ledger(|ledger| {
      if let Some(Entry::Author(block)) = ledger.entries_mut().get_mut(i) {
        block.outcome = outcome;
      }
    });
  }
  blocks_mark.restore();
  // the API calls made outside the blocks' expansions: their contents rehearsed and lexed as a block's are (the
  // resolver reads an affiliation's leading marks and text, an e-mail's address, a creator's name from them)
  let calls: Vec<(usize, Tokens)> = with_ledger(|ledger| {
    ledger
      .entries()
      .iter()
      .enumerate()
      .filter_map(|(i, entry)| match entry {
        Entry::Api(call) if !call.inside && !call.rehearsed => Some((i, call.content.clone())),
        _ => None,
      })
      .collect()
  });
  let calls_mark = StomachMark::take();
  for (i, content) in calls {
    let read = if content.is_empty() {
      Some((Vec::new(), 0))
    } else {
      match rehearse(content, TitleMarks::Fnsymbol, &[], lex_events)? {
        Rehearsal::Done { value, errors, .. } => Some((value, errors)),
        // (a flush inside a speculative run: the next flush rehearses it)
        Rehearsal::Skipped(SkipReason::JournalUnavailable) => None,
        Rehearsal::Skipped(_) => Some((Vec::new(), 0)),
      }
    };
    if let Some((events, errors)) = read {
      with_ledger(|ledger| {
        if let Some(Entry::Api(call)) = ledger.entries_mut().get_mut(i) {
          call.events = events;
          call.rehearsed = true;
          call.errors = errors;
        }
      });
    }
  }
  calls_mark.restore();
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
  if let Some(formula) = &event.formula {
    out.push_str(",\"formula_span\":");
    json_span(
      out,
      formula.span.map(|(s, e)| (usize::from(s), usize::from(e))),
    );
    if let Some(base) = formula.base {
      out.push_str(",\"base\":");
      json_str(out, base.name());
    }
    let _ = write!(out, ",\"ends_formula\":{}", formula.ends);
  }
  if let Some(value) = &event.unprinted {
    out.push_str(",\"unprinted\":");
    json_str(out, value);
  }
  if let Some(kind) = event.break_kind {
    out.push_str(",\"break\":");
    json_str(out, kind.name());
  }
  if event.braces > 0 {
    let _ = write!(out, ",\"braces\":{}", event.braces);
  }
  if event.ordinal {
    out.push_str(",\"ordinal\":true");
  }
  if !event.printed_labels.is_empty() {
    out.push_str(",\"printed_labels\":[");
    for (i, label) in event.printed_labels.iter().enumerate() {
      if i > 0 {
        out.push(',');
      }
      out.push_str("{\"label\":");
      json_str(out, &label.label);
      let _ = write!(
        out,
        ",\"word\":{},\"line_first\":{},\"span\":",
        label.word, label.line_first
      );
      json_span(
        out,
        label.span.map(|(s, e)| (usize::from(s), usize::from(e))),
      );
      out.push('}');
    }
    out.push(']');
  }
  if event.held {
    out.push_str(",\"held\":true");
  }
  if event.meaning != EmailMeaning::Binding {
    out.push_str(",\"meaning\":");
    json_str(out, event.meaning.name());
  }
  if event.inkless {
    out.push_str(",\"inkless\":true");
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

/// A list of events, under `key`.
fn json_events(out: &mut String, key: &str, events: &[Event]) {
  let _ = write!(out, ",\"{key}\":[");
  for (i, event) in events.iter().enumerate() {
    if i > 0 {
      out.push(',');
    }
    json_event(out, event);
  }
  out.push(']');
}

/// The errors a block's rehearsal held, by position.
fn json_held(out: &mut String, held: &[HeldError]) {
  out.push_str(",\"held\":[");
  for (i, error) in held.iter().enumerate() {
    if i > 0 {
      out.push(',');
    }
    out.push_str("{\"span\":");
    json_span(
      out,
      error.span.map(|(s, e)| (usize::from(s), usize::from(e))),
    );
    out.push_str(",\"message\":");
    json_str(out, &error.message);
    out.push('}');
  }
  out.push(']');
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
    for (n, entry) in ledger.entries().iter().enumerate() {
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
        Entry::NoteText(text) => {
          let _ = write!(
            out,
            "{{\"index\":{},\"kind\":\"notetext\",\"epoch\":{},\"mark\":",
            text.index, text.epoch
          );
          json_str(&mut out, &text.mark);
          out.push_str(",\"value\":");
          json_opt_str(&mut out, text.value.as_deref());
          out.push_str(",\"events\":[");
          for (i, event) in text.events.iter().enumerate() {
            if i > 0 {
              out.push(',');
            }
            json_event(&mut out, event);
          }
          out.push_str("]}");
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
          if let Some(binding) = &call.binding {
            out.push_str(",\"binding\":");
            json_str(&mut out, binding);
          }
          out.push_str(",\"attrs\":");
          json_str(&mut out, &call.attrs);
          if call.errors > 0 {
            let _ = write!(out, ",\"errors\":{}", call.errors);
          }
          if !call.parent.is_empty() {
            out.push_str(",\"parent\":");
            json_str(&mut out, &call.parent);
          }
          if call.cleared {
            out.push_str(",\"cleared\":true");
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
          if block.cleared {
            out.push_str(",\"cleared\":true");
          }
          let _ = write!(
            out,
            ",\"replace\":{},\"sectioned\":{},\"late\":{},\"cautious\":{},\"keyvals\":",
            block.replace, block.sectioned, block.late, block.cautious
          );
          json_opt_str(&mut out, block.keyvals.as_deref());
          let _ = write!(
            out,
            ",\"epoch\":{},\"queue_position\":{},\"authors_queued\":{},\"stamp\":",
            block.epoch,
            block.queue_position,
            block
              .authors_queued
              .map_or_else(|| "null".to_string(), |n| n.to_string())
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
              declarations,
              breaks,
              held: held_errors,
              ..
            } => {
              json_str(&mut out, "events");
              let _ = write!(out, ",\"errors\":{errors},\"warnings\":{warnings}");
              with_events += 1;
              if errors + warnings > 0 {
                held += 1;
              }
              events.clone_from(lexed);
              if !declarations.is_empty() {
                json_events(&mut out, "declarations", declarations);
              }
              if !breaks.is_empty() {
                json_events(&mut out, "breaks", breaks);
              }
              if !held_errors.is_empty() {
                json_held(&mut out, held_errors);
              }
            },
          }
          // the setters called outside the field after the block, up to the next block
          for later in &ledger.entries()[n + 1..] {
            match later {
              Entry::Author(_) => break,
              Entry::Api(call) if !call.inside && call.after_block == Some(block.index) => {
                let mut event = Event::new(EventKind::ApiCall);
                event.label = Some(s!("queue:{}:{}", call.epoch, call.queue_position));
                event.role = Some(s!("{} {}", call.tag, call.role));
                events.push(event);
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
    out.push(']');
    if !ledger.unprinted.is_empty() {
      out.push_str(",\"unprinted\":[");
      for (i, call) in ledger.unprinted.iter().enumerate() {
        if i > 0 {
          out.push(',');
        }
        let _ = write!(
          out,
          "{{\"at\":{},\"after_block\":{},\"inside\":{},\"attrs\":",
          call.at,
          call
            .after_block
            .map_or("null".to_string(), |b| b.to_string()),
          call.inside
        );
        json_str(&mut out, &call.attrs);
        out.push_str(",\"event\":");
        json_event(&mut out, &call.event);
        out.push('}');
      }
      out.push(']');
    }
    out.push_str(",\"resolved\":");
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
    // (the writer's outcome for a written family's paper: `same`, `written`, `declined: …`, `written-different: …`)
    out.push_str("}},\"writer\":");
    json_opt_str(&mut out, ledger.writer.as_deref());
    json_read_back(&mut out, ledger, blocks);
    out.push_str("}\n");
    out
  })
}

/// The paper's outcome (E0, `ab_audit fmshadow`'s per-family table): `same`, `written` and `written-different` the
/// writer's own check (its reason the difference), `declined` with its reason (`Ledger::refusal`); where the writer did
/// not run, `no-reading` (the resolver read nothing), `no-block` (no author block, `writer_stays_out`) or
/// `held-or-skipped` with its reason. And the read-back at the end of `insert_frontmatter`, none where it never got
/// there: `difference`, the first place the final document differs from the reading; `given_back`, from the reading as
/// resolved where the writer gave an item back.
fn json_read_back(out: &mut String, ledger: &ledger::Ledger, blocks: usize) {
  let read_nothing = || {
    let resolved = resolve(&ledger_inputs(ledger));
    resolved.creators.is_empty() && resolved.block.is_empty()
  };
  let (outcome, reason) = match ledger.writer.as_deref() {
    Some(writer) => match writer.split_once(": ") {
      Some(("written-different", difference)) => ("written-different", Some(difference)),
      Some(("declined", _)) => ("declined", ledger.refusal.as_deref()),
      None if writer == "same" || writer == "written" => (writer, None),
      // (fail toward flagging: a shape no reader knows, which the table refuses)
      _ => (writer, None),
    },
    None if read_nothing() => ("no-reading", None),
    None if blocks == 0 => ("no-block", None),
    None => (
      "held-or-skipped",
      Some(writer_stays_out(ledger).unwrap_or("the writer did not run")),
    ),
  };
  out.push_str(",\"readback\":{\"outcome\":");
  json_str(out, outcome);
  out.push_str(",\"reason\":");
  json_opt_str(out, reason);
  let _ = write!(out, ",\"reached\":{}", ledger.read_back.is_some());
  out.push_str(",\"difference\":");
  json_opt_str(
    out,
    ledger
      .read_back
      .as_ref()
      .and_then(|read_back| read_back.difference.as_deref()),
  );
  out.push_str(",\"given_back\":");
  json_opt_str(
    out,
    ledger
      .read_back
      .as_ref()
      .and_then(|read_back| read_back.given_back.as_deref()),
  );
  out.push('}');
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
  // (a link's target, where it points: FM S2a.2 r9)
  if let Some(target) = &assoc.target {
    out.push_str(",\"target\":");
    json_str(out, target);
  }
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
    out.push_str(",\"joins\":[");
    for (j, join) in creator.joins.iter().enumerate() {
      if j > 0 {
        out.push(',');
      }
      json_str(out, join.tag());
    }
    out.push(']');
    let _ = write!(out, ",\"source\":{}", creator.source);
    // (its creator call's role, where a call in its block printed it: `RCreator::role`)
    if let Some(role) = &creator.role {
      out.push_str(",\"role\":");
      json_str(out, role);
    }
    json_assocs(out, "affiliations", &creator.affiliations);
    json_assocs(out, "contacts", &creator.contacts);
    json_assocs(out, "notes", &creator.notes);
    out.push('}');
  }
  out.push(']');
  json_assocs(out, "block", &resolved.block);
  json_assocs(out, "dropped", &resolved.dropped);
  out.push_str(",\"continued\":[");
  for (i, lines) in resolved.continued.iter().flatten().enumerate() {
    if i > 0 {
      out.push(',');
    }
    out.push('[');
    for (j, line) in lines.iter().enumerate() {
      if j > 0 {
        out.push(',');
      }
      json_str(out, line);
    }
    out.push(']');
  }
  out.push(']');
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

/// Write the ledger ([`fm_shadow_json`]) to `<destination>.fmshadow.json`, under the census switch only (the CLI, after
/// the conversion's output is written). Its directory is the destination's own.
pub fn write_fm_shadow(destination: &str) -> std::io::Result<()> {
  if fm_shadow_mode() != FmShadow::On {
    return Ok(());
  }
  std::fs::write(s!("{destination}.fmshadow.json"), fm_shadow_json())
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Every real flush counts, whether the observer reads at it or not: the epoch a clear compares
  /// (`ledger::record_clear`). Switched off, no block recorded yet — what default mode reads at an early
  /// `\maketitle` before any author block — the flush still empties the queue (review rev92a_r1 S2).
  #[test]
  fn a_flush_that_reads_nothing_still_counts() {
    set_fm_shadow_override(Some(false));
    reset_fm_shadow();
    assert!(!reads_now());
    shadow_flush().unwrap();
    shadow_flush().unwrap();
    assert_eq!(with_ledger(|ledger| ledger.flushes), 2);
    set_fm_shadow_override(None);
    reset_fm_shadow();
  }

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
        formulas:     Vec::new(),
        inks:         Box::default(),
        events:       Vec::new(),
        errors:       0,
        warnings:     0,
        declarations: Vec::new(),
        breaks:       Vec::new(),
        held:         Vec::new(),
        environments: Vec::new(),
        creators:     Vec::new(),
      }),
      None
    );
  }
}
