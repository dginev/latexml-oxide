//! The observer's flush and its dump (OXIDIZED_DESIGN_DIVERGENCES #837): at every real flush (`digest_front_matter`)
//! the ledger's author blocks not rehearsed yet are rehearsed and lexed, and each API call's content
//! ([`read_at_flush`]); [`fm_reading_json`] gives the ledger as JSON — its entries with their events and the resolver's
//! tuples (`resolved`) — which the guards read the reading by, in process. The census that wrote it beside the output
//! (`LATEXML_FM_SHADOW=1`, `<dest>.fmshadow.json`, `ab_audit fmshadow`) and its off state (`=0`) were deleted with the
//! S2b writer (DEL4): the observer records and reads every paper.

use std::fmt::Write as _;

use super::{
  events::{
    EmailMeaning, Event, EventKind, HeldError, SepKind, creator_roles, lex_events, lex_with_inks,
    mark_held, with_block_content,
  },
  ledger::{
    Entry, Outcome, SkipReason, Stamp, content_text, reset_ledger, stamped_block_tokens,
    with_ledger,
  },
  policy::{TitleMarks, policy_for},
  rehearsal::{Rehearsal, for_the_check, rehearse},
  resolver::{Assoc, Resolved, ledger_inputs, resolve},
  *,
};

/// Forget the observer's ledger: a conversion's start (the converter's session preparation, with the field
/// provenance's reset).
pub fn reset_fm_reading() {
  reset_ledger();
  ledger::reset_author_owner();
  rehearsal::reset_rehearsing();
}

/// The flush's part (`digest_front_matter`, before the queue is digested): rehearse and lex each author block not
/// rehearsed yet. A block without positions (out of positions) is skipped and says why; one no rehearsal could run for
/// (a journal already open: the flush is itself inside a speculative run) waits for the next flush, deferred: left so
/// at the end, the dump reports that reason (`JournalUnavailable`), not "no flush after the call".
pub(in crate::base_utilities) fn read_at_flush() -> Result<HeldUndefined> {
  // (a rehearsal's flush is undone with it; every real flush empties the queue, read or not — the epoch a clear
  // compares, `ledger::record_clear`: an early `\maketitle` before any author block, in default mode, reads nothing
  // but flushes the title it digests, which a later `\title` must not clear; review rev92a_r1 S2)
  if rehearsal::rehearsing() {
    return Ok(HeldUndefined::default());
  }
  let held = read_pending()?;
  // (the flush empties the frontmatter queue: the calls after it are queued from position 0 again)
  with_ledger(|ledger| ledger.flushes += 1);
  Ok(held)
}

/// The undefined commands this flush's rehearsals held an Error for (`Error:undefined:\cs`, State.pm:547-551's
/// `generateErrorStub`), each once, with how many times the conversion had raised it as the flush began (the tally of
/// undefined names, `LogState::undefined`). A rehearsal holds each of its Errors and discards it with its stub, trusting
/// the emitter's digestion of each piece to raise it again; a command whose token falls in no piece the emitter
/// digests (it prints nothing, so no event covers it: `\author{… Lee\textsuperscript{1}\internship, …}`, 2607.29185),
/// or that sits in a piece written as text, is raised by no one — Perl digests the block once and raises it in place
/// (Base_Utility.pool.ltxml:691-693). [`HeldUndefined::raise_unraised`] is the flush's net for those: the Error the
/// rest of the flush did not raise, once, as Perl's first encounter. Not an IJCAI sectioned block's: its Errors are the
/// section markers its class defines only inside its own `\author` (ijcai19.sty:127, `\emails`, `\affiliations`;
/// the resolver's `block_markers`, 2501.02356). OXIDIZED_DESIGN_DIVERGENCES #983; repro
/// sectioning-frontmatter/emitter_undefined_command_in_author_errs_once; witnesses 1101.4098, 1402.2906, 1808.09392,
/// 2205.12847, 2309.09045, 2405.06973, 2405.13308, 2506.10896, 2508.08170, 2508.21091, 2601.17318, 2602.18351,
/// 2604.26688, 2607.29185, 2609.00161, hep-th9306034.
#[derive(Debug, Default)]
pub(in crate::base_utilities) struct HeldUndefined(Vec<(Token, usize)>);

impl HeldUndefined {
  /// Raise, through `generate_error_stub`, each held command the flush did not raise again — its tally of undefined
  /// names unmoved since the flush began — that no code of the flush gave a meaning, even locally (`defined`, the
  /// flush's meaning recorder: a class's title code defining its name parts only inside its own `\author` or
  /// `\maketitle`, where the emitter's pieces are digested — vmsta's `\fnm`/`\snm`/`\inits`, the IOS classes'
  /// `\fnms`, JAIR's row `\name`/`\email`/`\addr` —, as pdflatex and Perl digest them there, no error; the recorder
  /// opens as the rehearsals' stubs are gone, so a command it saw no assignment of has no meaning): its Error once, its
  /// stub standing for the rest of the document, as Perl's first encounter leaves it. Decided by the tally and the recorder, not by spelled names: a piece that raised it in place counted it there,
  /// so nothing is raised twice.
  pub(in crate::base_utilities) fn raise_unraised(
    self,
    defined: &[(SymStr, Option<Stored>)],
  ) -> Result<()> {
    for (token, before) in self.0 {
      let (raised, assigned) = token.with_cs_name(|name| {
        (
          undefined_tally(name) > before,
          defined.iter().any(|(key, _)| with(*key, |k| k == name)),
        )
      });
      if !raised && !assigned {
        generate_error_stub(&token)?;
      }
    }
    Ok(())
  }
}

/// How many times the conversion raised `name` undefined (the summary's tally, `LogState::undefined`).
fn undefined_tally(name: &str) -> usize {
  latexml_core::report!()
    .undefined
    .get(name)
    .copied()
    .unwrap_or(0)
}

/// The command a held `Error:undefined:\cs` record names (`generate_error_stub`'s message, its first line, which names a
/// control sequence's token `T_CS[…]`): `None` for any other record.
fn held_undefined(message: &str) -> Option<Token> {
  let rest = message.strip_prefix("Error:undefined:")?;
  let (name, tail) = rest.split_once(' ')?;
  tail
    .starts_with(&s!("The token T_CS[{name}] is not defined"))
    .then_some(T_CS!(name))
}

/// [`read_at_flush`]'s reading, with the undefined commands its rehearsals held ([`HeldUndefined`]).
fn read_pending() -> Result<HeldUndefined> {
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
  let mut undefined = HeldUndefined::default();
  for i in pending {
    let Some(block) = with_ledger(|ledger| match ledger.entries().get(i) {
      Some(Entry::Author(block)) => Some(block.clone()),
      _ => None,
    }) else {
      continue;
    };
    let mut queued_tags = Vec::new();
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
          left_open,
          unsaid,
          queued,
        } => {
          queued_tags = queued;
          if !block.sectioned {
            for token in held
              .iter()
              .filter_map(|error| held_undefined(&error.message))
            {
              if !undefined.0.iter().any(|(t, _)| *t == token) {
                let before = token.with_cs_name(undefined_tally);
                undefined.0.push((token, before));
              }
            }
          }
          mark_held(&mut value, &held, &inks);
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
            creators: creators.into_boxed_slice(),
            unread: left_open.then(|| content_text(&block.tokens).into()),
            unsaid,
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
        block.queued = queued_tags;
      }
    });
  }
  blocks_mark.restore();
  // the API calls made outside the blocks' expansions: their contents rehearsed and lexed as a block's are (the
  // resolver reads an affiliation's leading marks and text, an e-mail's address, a creator's name from them); and a
  // call a block's own expansion made that annotates a creator, for the content check alone (`ApiCall::inside_events`:
  // its words count unless its block's events print them), the block's own creator entry being the block
  let calls: Vec<(usize, Tokens, bool)> = with_ledger(|ledger| {
    ledger
      .entries()
      .iter()
      .enumerate()
      .filter_map(|(i, entry)| match entry {
        Entry::Api(call) if !call.rehearsed && (!call.inside || call.parent == "ltx:creator") => {
          Some((i, call.content.clone(), call.inside))
        },
        _ => None,
      })
      .collect()
  });
  let calls_mark = StomachMark::take();
  for (i, content, inside) in calls {
    let read = if content.is_empty() {
      Some((Vec::new(), 0, None, Vec::new()))
    } else {
      // (an inside call's, for the check alone, from the check's own allowance: `rehearsal::for_the_check`)
      let rehearsal = || rehearse(content.clone(), TitleMarks::Fnsymbol, &[], lex_events);
      let rehearsed = if inside {
        for_the_check(rehearsal)
      } else {
        rehearsal()
      };
      match rehearsed? {
        Rehearsal::Done { value, errors, queued, .. } => Some((value, errors, None, queued)),
        // (a flush inside a speculative run: the next flush rehearses it)
        Rehearsal::Skipped(SkipReason::JournalUnavailable) => None,
        // (given up on: what it printed is its content's text, `ApiCall::unread`)
        Rehearsal::Skipped(_) => Some((
          Vec::new(),
          0,
          Some(content_text(&content).into()),
          Vec::new(),
        )),
      }
    };
    if let Some((events, errors, unread, queued)) = read {
      with_ledger(|ledger| {
        if let Some(Entry::Api(call)) = ledger.entries_mut().get_mut(i) {
          if call.inside {
            call.inside_events = events.into();
          } else {
            call.events = events;
            call.errors = errors;
            call.queued = queued;
          }
          call.rehearsed = true;
          call.unread = unread;
        }
      });
    }
  }
  calls_mark.restore();
  Ok(undefined)
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

/// The ledger as JSON (read by the guards, in process): its entries in call order — author blocks with their events, title
/// stamps, API calls — and a summary (blocks with events, skipped by reason, out of positions, rehearsals that held
/// diagnostics). An author block's ApiCall events are the API calls after it its own expansion did not make.
pub fn fm_reading_json() -> String {
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
          if let Some(text) = &call.unread {
            out.push_str(",\"unread\":");
            json_str(&mut out, text);
          }
          if call.rehearsed {
            // (an inside call's, rehearsed for the content check alone: `ApiCall::inside_events`)
            let events: &[Event] = if call.inside {
              &call.inside_events
            } else {
              &call.events
            };
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
            ",\"epoch\":{},\"queue_position\":{},\"stamp\":",
            block.epoch, block.queue_position
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
              unread,
              ..
            } => {
              json_str(&mut out, "events");
              let _ = write!(out, ",\"errors\":{errors},\"warnings\":{warnings}");
              if let Some(text) = unread {
                out.push_str(",\"unread\":");
                json_str(&mut out, text);
              }
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
    // (whether a flush's emitter wrote the paper's frontmatter, `Ledger::emitted`)
    let _ = writeln!(out, "}}}},\"emitted\":{}}}", ledger.emitted);
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

#[cfg(test)]
mod tests {
  use super::*;

  /// Every real flush counts, whether it reads anything or not: the epoch a clear compares (`ledger::record_clear`). No
  /// block recorded yet — an early `\maketitle` before any author block — the flush still empties the queue (review
  /// rev92a_r1 S2).
  #[test]
  fn a_flush_that_reads_nothing_still_counts() {
    reset_fm_reading();
    read_at_flush().unwrap();
    read_at_flush().unwrap();
    assert_eq!(with_ledger(|ledger| ledger.flushes), 2);
    reset_fm_reading();
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
        creators:     Box::default(),
        unread:       None,
        unsaid:       None,
      }),
      None
    );
  }
}
