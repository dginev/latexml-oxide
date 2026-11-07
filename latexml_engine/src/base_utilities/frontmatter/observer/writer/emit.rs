//! The one added step at the flush (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, Route D; OXIDIZED_DESIGN_DIVERGENCES
//! #979), behind `LATEXML_FM_EMIT`: for a paper with an author block read, the queued creators and their annotations are
//! replaced by Perl's own calls built from the reading (`Resolved`) — `\lx@add@frontmatter@now{ltx:creator}`,
//! `\lx@annotate@frontmatter@now{ltx:creator}{ltx:contact}`, `\lx@add@thanks` — the owners by Perl's `label=` and
//! `annotations=`. What Perl's `\lx@add@authors` (Base_Utility.pool.ltxml:687) cuts with `SplitTokens` at the call, this
//! cuts at the digested list's positions, as digested material doesn't exist at the call. A token slice is the API's
//! argument; where it's cut comes from the events' spans.

use std::{
  cell::Cell,
  collections::{BTreeMap, BTreeSet, btree_map::Entry as Entry2},
  sync::OnceLock,
};

use latexml_core::document::tag::RawFrontmatter;

use super::{
  dom::{Kind, norm},
  plan::{self, Key, Owner, api_call},
  repart::entry_span,
  *,
};
use crate::base_utilities::frontmatter::{
  observer::{
    events::{EventKind, Formula, lex_with_formulas},
    field_notes::EnvironmentStep,
    ledger::{ApiCall, AuthorBlock, annotated_parent, field_stamped},
    policy::TitleMarks,
    rehearsal::{Rehearsal, rehearse},
    resolver::{creator_call, read_events, split_pieces},
  },
  relocate::{BLOCK_ZONE, FOOTER_ZONE},
};

thread_local! {
  static FM_EMIT_OVERRIDE: Cell<Option<u8>> = const { Cell::new(None) };
}

/// `LATEXML_FM_EMIT` (read once per process), or this thread's test override: 0 off, 1 the reader still runs at the call,
/// 2 each call queues one whole block.
pub fn fm_emit_mode() -> u8 {
  static MODE: OnceLock<u8> = OnceLock::new();
  FM_EMIT_OVERRIDE.with(Cell::get).unwrap_or_else(|| {
    *MODE.get_or_init(|| {
      std::env::var("LATEXML_FM_EMIT")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0)
    })
  })
}

/// Set (`Some`) or clear (`None`) this thread's override of the switch (tests: the environment is read-only to them).
pub fn set_fm_emit_override(mode: Option<u8>) { FM_EMIT_OVERRIDE.with(|o| o.set(mode)); }

/// The State value saying a flush emitted: the writer has nothing left to do (`write_observed_frontmatter`).
pub(in crate::base_utilities) const EMITTED: &str = "lx_fm_emitted";
/// The State value true inside `\\lx@frontmatter@itemonly`: what an item's content marks labels nothing, the reading's
/// labels stand (`with_pending_entry_attr`).
pub(in crate::base_utilities) const EMITTING: &str = "lx_fm_emitting";
/// The State mappings of what earlier flushes emitted, as `num_<tag>` counts (Perl's way of carrying a flush's counts):
/// a creator's `source#letters#place` to its `role:N`, and an item's `owner|kind|text`.
const EMITTED_CREATORS: &str = "lx_fm_emitted_creators";
const EMITTED_ITEMS: &str = "lx_fm_emitted_items";

/// Tokens of an internal string.
fn internal(text: &str) -> Vec<Token> {
  mouth::tokenize_internal(TeXString::assembled(text.to_string())).unlist()
}

/// `f` on the rehearsed block at ledger index `index`, else `default`.
fn with_block<R>(index: usize, default: R, f: impl FnOnce(&AuthorBlock, &[Event]) -> R) -> R {
  with_ledger(|ledger| match ledger.entries.get(index) {
    Some(Entry::Author(block)) => match &block.outcome {
      Outcome::Events { events, .. } => f(block, events),
      _ => default,
    },
    _ => default,
  })
}

/// `\cs{arg}…`
fn cs_group(cs: &str, args: &[&[Token]]) -> Vec<Token> {
  let mut out = vec![T_CS!(cs)];
  for arg in args {
    out.push(T_BEGIN!());
    out.extend_from_slice(arg);
    out.push(T_END!());
  }
  out
}

/// `[keys]{content}` after `head`.
fn with_keys(mut head: Vec<Token>, keys: &str, content: &[Token]) -> Tokens {
  head.push(T_OTHER!("["));
  head.extend(internal(keys));
  head.push(T_OTHER!("]"));
  head.push(T_BEGIN!());
  head.extend_from_slice(content);
  head.push(T_END!());
  Tokens::new(head)
}

/// `\lx@annotate@frontmatter@now{ltx:creator}{ltx:contact}[keys]{content}`.
fn annotate_now(tag: &str, keys: &str, content: &[Token]) -> Tokens {
  let mut head = vec![T_CS!("\\lx@annotate@frontmatter@now")];
  head.extend(cs_group_args(&["ltx:creator", tag]));
  with_keys(head, keys, content)
}

fn cs_group_args(args: &[&str]) -> Vec<Token> {
  let mut out = Vec::new();
  for arg in args {
    out.push(T_BEGIN!());
    out.extend(internal(arg));
    out.push(T_END!());
  }
  out
}

/// The reading's text as tokens that print it: its specials as their commands (an OTHER `_` in OT1 prints `˙`: 2605.20715).
fn text_tokens(text: &str) -> Vec<Token> {
  let mut out = Vec::new();
  for c in text.chars() {
    match c {
      '_' => out.push(T_CS!("\\_")),
      '#' => out.push(T_CS!("\\#")),
      '%' => out.push(T_CS!("\\%")),
      '&' => out.push(T_CS!("\\&")),
      '$' => out.push(T_CS!("\\$")),
      '{' => out.push(T_CS!("\\{")),
      '}' => out.push(T_CS!("\\}")),
      '~' => out.push(T_CS!("\\textasciitilde")),
      '^' => out.push(T_CS!("\\textasciicircum")),
      '\\' => out.push(T_CS!("\\textbackslash")),
      ' ' => out.push(T_SPACE!()),
      c => out.push(T_OTHER!(&c.to_string())),
    }
  }
  out
}

/// A queued call's tokens with its routing keys gone and `add` first: its other keys (`role`, `name`, …) stand. This reads
/// the API call's own key list, which `queue_front_matter`'s caller built, at its top-level commas: no TeX spelling.
fn rekeyed(
  command: &Tokens,
  add: &str,
  keep_annotations: bool,
  only: bool,
  zone: bool,
) -> Option<Tokens> {
  let v = command.unlist_ref();
  let mut depth = 0;
  let mut open = None;
  let mut close = None;
  for (i, t) in v.iter().enumerate() {
    depth += t.group_step();
    if depth != 0 {
      continue;
    }
    if open.is_none() && t.get_catcode() == Catcode::OTHER && t.with_str(|s| s == "[") {
      open = Some(i);
    } else if open.is_some() && t.get_catcode() == Catcode::OTHER && t.with_str(|s| s == "]") {
      close = Some(i);
      break;
    }
  }
  let mut out: Vec<Token> = Vec::with_capacity(v.len() + 8);
  if let (Some(open), Some(close)) = (open, close) {
    out.extend_from_slice(&v[..=open]);
    out.extend(internal(add));
    let (mut depth, mut from) = (0, open + 1);
    for i in open + 1..=close {
      let t = &v[i];
      depth += t.group_step();
      if i == close || (depth == 0 && t.get_catcode() == Catcode::OTHER && t.with_str(|s| s == ","))
      {
        let pair = &v[from..i];
        let key: String = pair
          .iter()
          .take_while(|t| !t.with_str(|s| s == "="))
          .map(|t| t.with_str(ToString::to_string))
          .collect::<String>()
          .trim()
          .to_string();
        // (the keys that place it go, the reading places it; `_zone`, where it prints, and `_kinded`, read already, stay)
        let routes = (key.starts_with('_')
          && !(matches!(key.as_str(), "_kinded" | "_markshown") || (zone && key == "_zone")))
          || matches!(key.as_str(), "label" | "labelseq" | "annotate" | "owner")
          || (key == "annotations" && !keep_annotations);
        if !key.is_empty() && !routes {
          out.push(T_OTHER!(","));
          out.extend_from_slice(pair);
        }
        from = i + 1;
      }
    }
    out.extend_from_slice(&v[close..]);
    return Some(call_tokens(out, only));
  }
  // (no keys: before the last group)
  let last = v.iter().rposition(|t| t.get_catcode() == Catcode::END)?;
  let mut depth = 0;
  let start = (0..=last).rev().find(|&i| {
    depth += v[i].group_step();
    depth == 0
  })?;
  out.extend_from_slice(&v[..start]);
  out.push(T_OTHER!("["));
  out.extend(internal(add));
  out.push(T_OTHER!("]"));
  out.extend_from_slice(&v[start..]);
  Some(call_tokens(out, only))
}

/// `\\lx@frontmatter@itemonly{content}`: past the spaces a content opens with, which an item's digestion skips.
fn item_only(content: &[Token]) -> Vec<Token> {
  let spaces = content
    .iter()
    .take_while(|t| t.get_catcode() == Catcode::SPACE)
    .count();
  cs_group("\\lx@frontmatter@itemonly", &[&content[spaces..]])
}

/// A call's tokens with its content, its last group, an emitted item's: its marks label nothing, and with `only` the API
/// calls in it are none.
fn call_tokens(mut call: Vec<Token>, only: bool) -> Tokens {
  call.pop();
  let mut depth = 0;
  let open = call.iter().rposition(|t| {
    depth += t.group_step();
    depth == 1
  });
  if let Some(open) = open {
    let inner = call.split_off(open + 1);
    if only {
      call.extend(item_only(&inner));
    } else {
      call.extend(cs_group("\\lx@frontmatter@item", &[&inner]));
    }
  }
  call.push(T_END!());
  Tokens::new(call)
}

/// The content of a note's span: the argument of the note command it covers (`\thanks{…}`, `\footnote[1]{…}`), by the
/// span's brace structure alone.
fn note_argument(tokens: &Tokens) -> Option<Vec<Token>> {
  let v = tokens.unlist_ref();
  let mut at = v.iter().position(|t| t.get_catcode() != Catcode::SPACE)?;
  (v[at].get_catcode() == Catcode::CS).then_some(())?;
  at += 1;
  if v.get(at)?.with_str(|s| s == "[") {
    while !v.get(at)?.with_str(|s| s == "]") {
      at += 1;
    }
    at += 1;
  }
  (v.get(at)?.get_catcode() == Catcode::BEGIN).then_some(())?;
  let mut depth = 0;
  for (i, t) in v.iter().enumerate().skip(at) {
    depth += t.group_step();
    if depth == 0 {
      return v[i + 1..]
        .iter()
        .all(|t| t.get_catcode() == Catcode::SPACE)
        .then(|| v[at + 1..i].to_vec());
    }
  }
  None
}

/// `start`, or past the environment a span opens and doesn't close: its first box takes the cover of what opened it
/// (1507.02068's `\\begin{center}M. Ablikim`), a column spec with it. As the digestion's frames say it.
fn past_open_environment(
  environments: &[EnvironmentStep],
  base: usize,
  start: usize,
  end: usize,
) -> usize {
  let at = |p: u16| usize::from(p).saturating_sub(base);
  let mut from = start;
  for (n, step) in environments.iter().enumerate() {
    if step.opens && at(step.start) == from && at(step.end) < end {
      let closed = environments[n + 1..]
        .iter()
        .any(|later| !later.opens && at(later.start) < end);
      if !closed {
        from = at(step.end);
      }
    }
  }
  from
}

/// `end`, or past the letter or the group an accent sets, which the cover of its box stops short of, as nothing prints there
/// but through it (`Živkovi\\'c`, `Petri\\v{c}`: 2406.17108, 1202.0103): a control sequence last, no event until past that.
fn past_what_it_set(field: &Field, events: &[Event], end: usize) -> usize {
  let (tokens, pos) = (field.tokens, field.base + end);
  // (or the bytes a multi-byte character's first reads, under `inputenc`: `（李银杰）`, 2201.01905)
  let byte = |t: &Token| t.with_str(|s| s.chars().next().is_some_and(|c| !c.is_ascii()));
  if end > 0 && tokens.get(end - 1).is_some_and(byte) {
    let bytes = tokens[end..].iter().take_while(|t| byte(t)).count();
    let printed = events.iter().any(|e| {
      e.kind != EventKind::Layout
        && e
          .span
          .is_some_and(|(s, _)| usize::from(s) >= pos && usize::from(s) < pos + bytes)
    });
    return if printed { end } else { end + bytes };
  }
  if end == 0
    || tokens
      .get(end - 1)
      .is_none_or(|t| t.get_catcode() != Catcode::CS)
  {
    return end;
  }
  let at = (end..tokens.len())
    .find(|&i| tokens[i].get_catcode() != Catcode::SPACE)
    .unwrap_or(tokens.len());
  let past = match tokens.get(at).map(Token::get_catcode) {
    Some(Catcode::LETTER | Catcode::OTHER) => at + 1,
    Some(Catcode::BEGIN) => {
      let mut depth = 0;
      (at..tokens.len())
        .find(|&i| {
          depth += tokens[i].group_step();
          depth == 0
        })
        .map_or(end, |close| close + 1)
    },
    _ => end,
  };
  let printed = events
    .iter()
    .filter(|e| e.kind != EventKind::Layout)
    .any(|e| {
      e.span.is_some_and(|(start, _)| {
        usize::from(start) >= pos && usize::from(start) < field.base + past
      })
    });
  if printed { end } else { past }
}

/// `at`, or past the `[…]` there.
fn past_option(tokens: &[Token], at: usize) -> usize {
  if tokens.get(at).is_none_or(|t| !t.with_str(|s| s == "[")) {
    return at;
  }
  tokens[at..]
    .iter()
    .position(|t| t.with_str(|s| s == "]"))
    .map_or(at, |close| at + close + 1)
}

/// `start`, or the opening brace of the group `start..end` closes where it starts right after that brace, spaces aside: `{\\large
/// Olga Kichakova}$^{1}$`, its first box's cover from the switch (1208.4825).
fn in_a_group(tokens: &[Token], start: usize, end: usize) -> usize {
  let mut depth = 0;
  let mut lowest = 0;
  for t in tokens.get(start..end).into_iter().flatten() {
    depth += t.group_step();
    lowest = lowest.min(depth);
  }
  let mut at = start;
  for _ in lowest..0 {
    let Some(before) = tokens[..at]
      .iter()
      .rposition(|t| t.get_catcode() != Catcode::SPACE)
    else {
      return at;
    };
    if tokens[before].get_catcode() != Catcode::BEGIN {
      return at;
    }
    at = before;
  }
  at
}

/// The tokens from the event `first` through `last` of a block: their boxes' covers' run, or their own tokens in the font they
/// print in where a cover is a wrapper's holding what else prints (`\\textbf{Ann, Bob}`, 2305.12182) or past an environment's
/// opening, grown to what `grown` adds. `None` for a run in a formula holding more (`$\\text{Ann}^{a},\\text{Bob}^{b}$`,
/// 2602.01228), or no balanced one.
fn event_run(
  field: &Field,
  events: &[Event],
  (first, last): (&Event, &Event),
  grown: Option<(usize, usize)>,
) -> Option<Vec<Token>> {
  let at = |p: u16| usize::from(p).checked_sub(field.base);
  let (start, end) = (at(first.span?.0)?, at(last.span?.1)?);
  let start = past_open_environment(field.environments, field.base, start, end);
  let own_at = |e: &Event| {
    e.marks
      .filter(|_| e.kind == EventKind::Name)
      .and_then(|(s, t)| Some((at(s)?, at(t)?)))
  };
  let inside = |e: &Event| first.span <= e.span && e.span <= last.span;
  let first_own = own_at(first).map_or(start, |(own, _)| own);
  // (the cover its own tokens fill holds nothing else: most names)
  let filled = first_own <= start && own_at(last).is_some_and(|(_, own)| own >= end);
  let wider = !filled
    && events.iter().any(|e| {
      let mark_at = e.formula.and_then(|f| f.span).map(|s| s.0).or(e.own_at);
      (e.kind == EventKind::Name
      && !inside(e)
      && own_at(e).is_some_and(|(own, _)| start <= own && own < end))
      // (a mark its cover holds before it, `\\textbf{$^2$Tencent Hunyuan}`, 2605.25195)
      || (e.kind == EventKind::Mark
        && mark_at.and_then(at).is_some_and(|mark| start <= mark && mark < first_own))
    });
  // (the names a formula holds have no run of their own)
  let formula = (field.formula_start(start), field.formula_end(end));
  let in_formula = (formula.0 < start || formula.1 > end)
    && events.iter().any(|e| {
      e.kind == EventKind::Name
        && !inside(e)
        && e
          .span
          .and_then(|s| at(s.0))
          .is_some_and(|s| formula.0 <= s && s < formula.1)
    });
  if in_formula || (wider && own_at(first).is_some_and(|(own, _)| field.formula_start(own) != own))
  {
    return None;
  }
  let (mut run_start, mut run_end) = match (own_at(first), own_at(last)) {
    (Some((own, _)), Some((_, last))) if wider => (own, past_what_it_set(field, events, last)),
    (_, Some((_, last))) => (start, past_what_it_set(field, events, end.max(last))),
    _ => (start, past_what_it_set(field, events, end)),
  };
  if let Some((s, t)) = grown {
    run_start = run_start.min(s);
    run_end = run_end.max(t);
  }
  let from = in_a_group(field.tokens, run_start, run_end);
  let (tokens, _) = field.content_of((from, run_end), 0, run_start)?;
  Some(if wider || first_own <= start {
    in_its_font(first.font.clone(), tokens.unlist()).unlist()
  } else {
    tokens.unlist()
  })
}

/// A block's names' tokens: their marks with them where `marks` (a per-author class's block, whose names keep them printed as
/// Perl does), a `\\footnotemark` (what `relocate_footnote` moves its text to, 2203.12964) and an ORCID written on it, its logo in
/// the name as today's (2208.05251), in `on_names`. `None` for a name its events don't spell.
fn block_names(
  index: usize,
  names: &[String],
  marks: bool,
  on_names: &mut BTreeSet<(usize, (u16, u16))>,
) -> Vec<Option<Vec<Token>>> {
  with_ledger(|ledger| {
    let Some(Entry::Author(block)) = ledger.entries.get(index) else {
      return vec![None; names.len()];
    };
    let Outcome::Events {
      events, formulas, environments, ..
    } = &block.outcome
    else {
      return vec![None; names.len()];
    };
    let read = read_events(events);
    let found: Vec<(usize, &Event)> = read
      .iter()
      .enumerate()
      .filter(|(_, e)| e.kind == EventKind::Name && !norm(&e.text).is_empty())
      .collect();
    let field = Field {
      tokens: block.tokens.unlist_ref(),
      base: block.range.0,
      formulas,
      environments,
    };
    let at = |p: u16| usize::from(p).checked_sub(block.range.0);
    let mut printed_at: BTreeMap<Option<(u16, u16)>, usize> = BTreeMap::new();
    for e in events
      .iter()
      .filter(|e| matches!(e.kind, EventKind::Name | EventKind::Contact))
    {
      *printed_at.entry(e.span).or_default() += 1;
    }
    let mut name_tokens = |from: usize, to: usize| -> Option<Vec<Token>> {
      // (several events of one span: a macro printed them, `\\author{\\authorBlock}`, 2604.09547)
      if found[from..=to]
        .iter()
        .any(|(_, e)| printed_at.get(&e.span).is_some_and(|n| *n > 1))
      {
        return None;
      }
      let is_orcid = |e: &Event| e.kind == EventKind::Contact && e.role.as_deref() == Some("orcid");
      let mark_kept = |e: &Event| {
        e.kind == EventKind::Mark && (marks || e.role.as_deref() == Some("footnotemark"))
      };
      let beside =
        |e: &&Event| matches!(e.kind, EventKind::Mark | EventKind::Layout) || is_orcid(e);
      let (at_name, after_name) = (found[from].0, found[to].0 + 1);
      let mut grown = (usize::MAX, 0);
      if marks {
        for e in read[..at_name]
          .iter()
          .rev()
          .take_while(beside)
          .filter(|e| mark_kept(e) && e.line_first)
        {
          grown.0 = grown.0.min(at(e.span?.0)?);
        }
      }
      // (past marks that stay out of it, each its own run)
      let mut apart: Vec<Token> = Vec::new();
      for e in read[after_name..]
        .iter()
        .take_while(beside)
        .filter(|e| mark_kept(e) || is_orcid(e))
      {
        let span = e.span?;
        let (s, t) = (at(span.0)?, at(span.1)?);
        // (a `\\footnotemark`'s number, past its span: 2209.15209)
        let t = past_option(field.tokens, s + 1).max(t);
        if marks {
          grown.1 = grown.1.max(t);
        } else if let Some((tokens, _)) = field.content_of((s, t), 0, s) {
          apart.extend(tokens.unlist());
        }
        if is_orcid(e) {
          on_names.insert((index, span));
        }
      }
      let tokens = event_run(&field, events, (found[from].1, found[to].1), Some(grown))?;
      // (a footnote symbol in a kept mark is shown apart from its numbers, `rewrite_symbol_superscripts`'s: 1610.06117)
      let mut out = if marks {
        rewrite_symbol_superscripts(Tokens::new(tokens)).unlist()
      } else {
        tokens
      };
      out.extend(apart);
      Some(out)
    };
    // (each name the Name events from the last one on spell, joined)
    let spelled_from = |from: usize, key: &str| {
      let mut joined = String::new();
      found[from..]
        .iter()
        .position(|event| {
          joined.push_str(&norm(&event.1.text));
          joined == key || !key.starts_with(&joined)
        })
        .filter(|_| joined == key)
    };
    let mut cursor = 0;
    let mut tokens_of = |name: &String| {
      let key = norm(name);
      let (from, to) =
        (cursor..found.len()).find_map(|from| Some((from, from + spelled_from(from, &key)?)))?;
      cursor = to + 1;
      name_tokens(from, to)
    };
    names.iter().map(&mut tokens_of).collect()
  })
}

/// Why the emitter stays out of the paper the ledger holds: no block, or a block no rehearsal read.
fn stays_out() -> Option<&'static str> {
  with_ledger(|ledger| {
    let mut blocks = 0;
    for entry in &ledger.entries {
      if let Entry::Author(block) = entry {
        blocks += 1;
        match &block.outcome {
          Outcome::Events { .. } | Outcome::Skipped(SkipReason::Empty) => {},
          Outcome::Skipped(reason) | Outcome::Deferred(reason) => return Some(reason.name()),
          Outcome::Pending => return Some(SkipReason::NoFlush.name()),
        }
      }
    }
    (blocks == 0).then_some(NO_BLOCK)
  })
}

/// An item written from the reading's text: named in the `frontmatter` debug feature, each with its reason.
fn by_text(why: &str, assoc: &Assoc) -> Vec<Token> {
  DebugFeature!(
    "frontmatter",
    "FRONT emit text: {why} [{}] {}",
    assoc.rule,
    assoc.text
  );
  text_tokens(&assoc.text)
}

/// A note's call under `label`, as `\lx@add@thanks` and `\lx@add@note` make it (their `Let`s in an author's content,
/// `digest_frontmatter_item`), on `@now`: what's inside it is the reading's already. A legend's with the `mark` it prints.
fn note_call(footnote: bool, mark: Option<&str>, label: &str, content: Vec<Token>) -> Tokens {
  let kind = classify_thanks(Tokens::new(content.clone()).untex_string().as_ref());
  let body = cs_group("\\lx@frontmatter@itemonly", &[note_body(Tokens::new(
    content,
  ))
  .unlist_ref()]);
  let marked = mark.map_or_else(String::new, |mark| s!("mark={{{mark}}},"));
  let call = if footnote {
    annotate_now("ltx:contact", &s!("role=note,label={{{label}}}"), &body)
  } else {
    let keys =
      s!("role=thanks,class=ltx_note_frontmatter ltx_thanks_{kind},{marked}label={{{label}}}");
    annotate_now("ltx:note", &keys, &body)
  };
  let mut out = vec![T_CS!("\\lx@title@footnote@step")];
  out.extend(call.unlist());
  Tokens::new(out)
}

/// A block's or a piece's entry: `\\lx@frontmatter@itemkinded`, whose Kind step reads its lines of addresses.
fn kinded_affiliation(keys: &str, content: &[Token]) -> Tokens {
  let mut head = vec![T_CS!("\\lx@frontmatter@itemkinded")];
  head.extend(cs_group_args(&["ltx:creator", "ltx:contact"]));
  with_keys(
    head,
    keys,
    &cs_group("\\lx@affiliation@withinst", &[content]),
  )
}

/// The tokens of the event's `span` of the block at `index`, balanced.
fn span_tokens_of(index: usize, span: (u16, u16)) -> Option<Tokens> {
  with_ledger(|ledger| {
    let Some(Entry::Author(block)) = ledger.entries.get(index) else {
      return None;
    };
    let Outcome::Events { formulas, environments, .. } = &block.outcome else {
      return None;
    };
    let field = Field {
      tokens: block.tokens.unlist_ref(),
      base: block.range.0,
      formulas,
      environments,
    };
    let at = |p: u16| usize::from(p).checked_sub(block.range.0);
    let start = at(span.0)?;
    Some(field.content_of((start, at(span.1)?), 0, start)?.0)
  })
}

/// `call`, its content's printed list written down to the one `address` of it (the reading's `ruling3:written-out`): the
/// list is the innermost, last group holding an `@`, or the content (`\\lx@address@urlfont{a@x, b@y}`,
/// `\\textcolor{blue}{a@x, b@y}`, 2211.14973), what's around it stands.
fn one_address(call: &Tokens, address: &str) -> Tokens {
  let mut tokens = call.unlist_ref().to_vec();
  tokens.pop();
  let mut depth = 0;
  let Some(open) = tokens.iter().rposition(|t| {
    depth += t.group_step();
    depth == 1
  }) else {
    return call.clone();
  };
  let content = tokens.split_off(open + 1);
  let (mut list, mut stack) = ((0, content.len()), Vec::new());
  let mut at_sign = false;
  for (i, t) in content.iter().enumerate() {
    match t.group_step() {
      1 => {
        stack.push(i + 1);
        at_sign = false;
      },
      -1 => {
        let from = stack.pop().unwrap_or(0);
        if at_sign {
          list = (from, i);
          at_sign = false;
        }
      },
      _ => at_sign |= t.with_str(|s| s == "@"),
    }
  }
  tokens.extend_from_slice(&content[..list.0]);
  tokens.extend(text_tokens(address));
  tokens.extend_from_slice(&content[list.1..]);
  tokens.push(T_END!());
  Tokens::new(tokens)
}

/// The labels of the marks of `role` the block at ledger index `source` prints.
fn marks_of(source: usize, role: &str) -> Vec<String> {
  with_block(source, Vec::new(), |_, events| {
    events
      .iter()
      .filter(|e| e.kind == EventKind::Mark && e.role.as_deref() == Some(role))
      .filter_map(|e| e.label.as_deref())
      .flat_map(mark_labels)
      .collect()
  })
}

/// Whether the item is one a label places in Perl's `relocateAnnotations`, or what the reading gives the block with it.
fn by_label(assoc: &Assoc) -> bool { assoc.label.is_some() || assoc.rule.contains("api:owner:") }

/// The event of the block at ledger index `index` printed at `span`, read by `f`.
fn event_at<R>(
  index: usize,
  span: (u16, u16),
  kind: fn(&Event) -> bool,
  f: impl FnOnce(&Event) -> R,
) -> Option<R> {
  with_block(index, None, |_, events| {
    let inner = events.iter().flat_map(|e| &e.inner);
    events
      .iter()
      .chain(inner)
      .find(|e| e.span == Some(span) && kind(e))
      .map(f)
  })
}

/// `tokens`, with the space written just before `span` of the block at `index` first: the entry's, as Perl's `SplitTokens`
/// leaves it after a line break or a mark.
fn spaced(index: usize, span: (u16, u16), tokens: Vec<Token>) -> Vec<Token> {
  let (lead, breaks) = with_block(index, (None, Vec::new()), |block, events| {
    let written = block.tokens.unlist_ref();
    let after_mark = events
      .iter()
      .rfind(|e| e.kind != EventKind::Layout && e.span.is_some_and(|s| s.1 < span.0))
      .is_some_and(|e| e.kind == EventKind::Mark);
    let lead = usize::from(span.0)
      .checked_sub(block.range.0 + 1)
      .and_then(|at| written.get(at).copied())
      // (a plain space after a break the split trims, one after a mark or a line's end it leaves)
      .filter(|t| t.get_catcode() == Catcode::SPACE && (after_mark || *t != T_SPACE!()));
    // (and no space before a line break of it, as Perl's split leaves none: 0902.0240)
    let breaks: Vec<Token> = events
      .iter()
      .filter(|e| e.kind == EventKind::Break)
      .filter_map(|e| {
        written
          .get(usize::from(e.span?.0).checked_sub(block.range.0)?)
          .copied()
      })
      .collect();
    (lead, breaks)
  });
  let mut out: Vec<Token> = lead.into_iter().collect();
  for t in tokens {
    if breaks.contains(&t) {
      while out
        .last()
        .is_some_and(|last| last.get_catcode() == Catcode::SPACE)
      {
        out.pop();
      }
    }
    out.push(t);
  }
  out
}

/// The tokens of an entry whose lines no balanced run holds (`{\\small $^1$Univ A}\\\\ {\\small Town}`, 2203.00365,
/// cond-mat0404185): each line's, at the Break events of `span`, with its break.
fn lines_run(field: &Field, events: &[Event], span: (u16, u16)) -> Option<Vec<Token>> {
  let mut lines: Vec<Vec<&Event>> = vec![Vec::new()];
  for e in events
    .iter()
    .filter(|e| e.span.is_some_and(|s| s.0 >= span.0 && s.1 <= span.1))
  {
    match e.kind {
      EventKind::Break => lines.push(Vec::new()),
      EventKind::Layout => {},
      _ => lines.last_mut()?.push(e),
    }
  }
  if lines.len() < 2 {
    return None;
  }
  let mut out = Vec::new();
  for (n, line) in lines.iter().enumerate() {
    if n > 0 {
      out.push(T_CS!("\\\\"));
    }
    if let (Some(first), Some(last)) = (line.first(), line.last()) {
      out.extend(event_run(field, events, (first, last), None)?);
    }
  }
  Some(out)
}

/// `tokens` digested in the font their first ink printed in, against the flush's: what a group or a switch before them set.
fn in_its_font(font: Option<Rc<Font>>, tokens: Vec<Token>) -> Tokens {
  let changes = font
    .zip(lookup_font())
    .map(|(font, root)| Rc::new(events::font_changes(&root, &font)));
  match changes {
    Some(changes) => plan::entry_font_tokens(&changes, Tokens::new(tokens)),
    None => Tokens::new(tokens),
  }
}

/// The call for one item of the reading, under `label`.
fn item_call(
  key: &Key,
  assoc: &Assoc,
  label: &str,
  separate: &BTreeSet<usize>,
  pieces: &mut BTreeMap<usize, Vec<(String, Vec<Token>)>>,
) -> Result<Tokens> {
  let role = assoc.role.as_str();
  let keys = s!("role={role},label={{{label}}}");
  let contact_role = matches!(key.0, Kind::Contact(_));
  let note = key.0 == Kind::Note;
  let written_out = assoc.rule.contains("ruling3:written-out");
  // (a legend an API call holds: its carriers' note, its mark printed, as the Kind step makes one: `kind_legend_contacts`,
  // 2607.02294, 1611.06354)
  if role == "legend"
    && let Some((index, None)) = assoc.source
    && let Some(call) = api_call(index)
    && let Some(mark) = call
      .events
      .iter()
      .find(|e| e.kind == EventKind::Mark)
      .map(|e| e.text.as_str())
      // (or the label its binding prints: `\\affil[*]{Corresponding author.}`, 2107.01943, 1812.03816)
      .or(assoc.label.as_deref())
  {
    return Ok(note_call(
      false,
      Some(mark),
      label,
      call.content.unlist_ref().to_vec(),
    ));
  }
  // (the call, where it is this item: `\\email{…}` as the document gave it; an entry's Kind steps read it whole: a legend,
  // `kind_legend_contacts`, its addresses, `link_entry_addresses`. A contact inside it is its own item where the reading
  // doesn't leave it to the entry, [`separate`])
  if let Some((index, None)) = assoc.source
    && let Some(call) = api_call(index)
    && (call.role == role || (note && call.parent == "ltx:creator"))
    && let Some(command) = call.command.as_ref()
    && split_pieces(&call.events, &call.role).len() < 2
    // (an entry whose inner contact prints through no API call stays in its content: its piece, or its text, 2401.05290)
    && (contact_role
      || !call.events.iter().any(|e| {
        e.kind == EventKind::Contact
          && e.role.as_deref() == Some("email")
          && e.meaning == events::EmailMeaning::Own
      }))
  {
    if written_out {
      if let Some(tokens) = rekeyed(command, &s!("label={{{label}}}"), true, false, true) {
        return Ok(one_address(&tokens, &assoc.text));
      }
    } else if let Some(tokens) =
      // (an entry the block prints moves back to its place by its zone, what a label places alone: `at_queue_position`)
      rekeyed(
        command,
        &s!("label={{{label}}}"),
        true,
        separate.contains(&index),
        by_label(assoc) || !matches!(role, "affiliation" | "address"),
      )
    {
      return Ok(tokens);
    }
  }
  let wrap = |content: &[Token]| -> Tokens {
    if note {
      note_call(false, None, label, content.to_vec())
    } else if key.0 == Kind::Affiliation {
      kinded_affiliation(&keys, content)
    } else {
      annotate_now("ltx:contact", &keys, &item_only(content))
    }
  };
  match assoc.source {
    // (a block's: the tokens its span covers)
    Some((index, Some(span))) => {
      // (a note's: its argument, under the note's own kind, but an entry a mark leads, as `\\lx@add@thanks` reads it: 2102.08072)
      let argument = event_at(index, span, |e| e.kind == EventKind::Note, |_| ())
        .and_then(|()| span_tokens_of(index, span))
        .as_ref()
        .and_then(note_argument);
      let an_entry = key.0 == Kind::Affiliation
        || (role == "legend"
          && argument
            .as_ref()
            .is_some_and(|argument| starts_with_affiliation_mark(&Tokens::new(argument.clone()))));
      if note
        && !an_entry
        && let Some(footnote) = event_at(index, span, |e| e.kind == EventKind::Note, |e| e.footnote)
      {
        let content = argument;
        return Ok(note_call(
          footnote,
          None,
          label,
          content.unwrap_or_else(|| by_text("note no argument", assoc)),
        ));
      }
      // (a contact's command: as written, its meaning the class's, a call inside the contact taking its label as Perl's
      // `\lx@annotate@frontmatter@now` has it, an empty contact around it dropped)
      // (one a mark's formula holds is its text, the formula its run would grow to printing the mark: an ORCID icon,
      // `$^{1\,\orcidicon{…}}$`, 2210.03748, 2206.03602)
      let in_a_mark = with_block(index, false, |_, events| {
        events.iter().any(|e| {
          let formula = e.formula.and_then(|f| f.span);
          e.kind == EventKind::Mark && formula.is_some_and(|s| s.0 <= span.0 && span.1 <= s.1)
        })
      });
      if contact_role && in_a_mark {
        return Ok(wrap(&by_text("contact in a mark's formula", assoc)));
      }
      if contact_role
        && event_at(
          index,
          span,
          |e| e.kind == EventKind::Contact,
          |e| role != "email" || norm(&e.text) == key.1,
        ) == Some(true)
        && let Some(tokens) = span_tokens_of(index, span)
      {
        return Ok(annotate_now("ltx:contact", &keys, tokens.unlist_ref()));
      }
      // (an entry a note holds, a mark leading it: its argument under the mark's reader, as `\\lx@add@thanks` makes it,
      // each mark's segment apart: 2102.09361, 2503.08005)
      if an_entry && let Some(argument) = argument {
        let segments = split_wrapped_affiliation_marks(Tokens::new(argument));
        let its_mark = assoc
          .label
          .as_deref()
          .and_then(|label| label.rsplit(':').next());
        let segment = segments
          .iter()
          .find(|seg| {
            entry_mark(seg.unlist_ref())
              .and_then(|mark| mark_label(&mark))
              .is_some_and(|mark| Some(mark.rsplit(':').next().unwrap_or(&mark)) == its_mark)
          })
          .or(segments.first());
        if let Some(segment) = segment {
          return Ok(kinded_affiliation(
            &s!("role=affiliation,label={{{label}}}"),
            &cs_group("\\lx@affiliation@withsup", &[segment.unlist_ref()]),
          ));
        }
      }
      // (one address of what prints several, or in a note: its text in the font it prints in there)
      if role == "email" {
        let font = with_block(index, None, |_, events| {
          let holds =
            |e: &&Event| e.span.is_some_and(|s| s.0 <= span.0 && span.1 <= s.1) && e.font.is_some();
          events.iter().find(holds).and_then(|e| e.font.clone())
        });
        let content = in_its_font(font, text_tokens(&assoc.text));
        // (named by the label word printed before it, as the Kind steps name it: `E-mail: \\texttt{a@x}`, 0802.3384)
        let printed = with_block(index, String::new(), |_, events| {
          let name = events.iter().find(|e| {
            e.kind == EventKind::Name && e.span.is_some_and(|s| s.0 <= span.0 && span.1 <= s.1)
          });
          name.map_or_else(String::new, |e| e.text.clone())
        });
        let label_word = printed
          .split(&assoc.text)
          .next()
          .filter(|before| before.len() < printed.len())
          .map(str::trim)
          .filter(|before| before.ends_with(':') && before.split_whitespace().count() <= 2);
        let keys = match label_word {
          Some(word) => s!("{keys},name={{{word}~}}"),
          None => keys,
        };
        return Ok(annotate_now("ltx:contact", &keys, content.unlist_ref()));
      }
      // (several events of one span: a macro printed them, 2604.09547)
      let printed_there =
        |e: &&Event| matches!(e.kind, EventKind::Name | EventKind::Contact) && e.span == Some(span);
      if with_block(index, 0, |_, events| {
        events.iter().filter(printed_there).count()
      }) > 1
      {
        return Ok(wrap(&by_text("a macro's span", assoc)));
      }
      let found = entry_span(index, span, &key.1);
      if found.is_none()
        && event_at(
          index,
          span,
          |e| e.kind != EventKind::Layout,
          |e| norm(&e.text) == key.1,
        ) != Some(true)
      {
        return Ok(wrap(&by_text("span letters differ", assoc)));
      }
      let span = found.unwrap_or(span);
      // (its tokens, from its mark for a note's legend; in the font it prints in, the group or the switch before its mark too)
      let run = with_ledger(|ledger| {
        let Some(Entry::Author(block)) = ledger.entries.get(index) else {
          return None;
        };
        let Outcome::Events {
          events, formulas, environments, ..
        } = &block.outcome
        else {
          return None;
        };
        let field = Field {
          tokens: block.tokens.unlist_ref(),
          base: block.range.0,
          formulas,
          environments,
        };
        let within = |e: &&Event| {
          e.kind != EventKind::Layout && e.span.is_some_and(|s| s.0 >= span.0 && s.1 <= span.1)
        };
        let (first, last) = (events.iter().find(within)?, events.iter().rfind(within)?);
        let mark = events
          .iter()
          .rfind(|e| e.kind != EventKind::Layout && e.span.is_some_and(|s| s.1 <= span.0))
          .filter(|e| e.kind == EventKind::Mark && note);
        let grown = mark.and_then(|e| Some((usize::from(e.span?.0).checked_sub(field.base)?, 0)));
        let tokens = event_run(&field, events, (first, last), grown)
          .or_else(|| lines_run(&field, events, span))?;
        Some((tokens, first.font.clone(), mark.map(|e| e.text.clone())))
      });
      let Some((tokens, font, mark)) = run else {
        return Ok(wrap(&by_text("span no balanced run", assoc)));
      };
      if note {
        let legend = mark.as_deref().filter(|_| role == "legend");
        let tokens = in_its_font(font, tokens).unlist();
        return Ok(note_call(false, legend, label, tokens));
      }
      if key.0 != Kind::Affiliation {
        return Ok(wrap(&tokens));
      }
      Ok(wrap(
        in_its_font(font, spaced(index, span, tokens)).unlist_ref(),
      ))
    },
    // (an API call's: its piece where the reading cut it)
    Some((index, None)) => {
      let Some(call) = api_call(index) else {
        return Ok(wrap(&by_text("note text", assoc)));
      };
      // (a contact inside it: its command as written, as a block's)
      if contact_role && let Some(tokens) = call_contact(&call, &key.1, written_out)? {
        let call = annotate_now("ltx:contact", &keys, tokens.unlist_ref());
        return Ok(if written_out {
          one_address(&call, &assoc.text)
        } else {
          call
        });
      }
      let found = match pieces.entry(index) {
        Entry2::Occupied(held) => held.into_mut(),
        Entry2::Vacant(place) => place.insert(call_pieces(&call)?),
      };
      let piece = found.iter().find(|(text, _)| *text == key.1);
      if let Some((_, tokens)) = piece.or(found.first().filter(|_| found.len() == 1)) {
        return Ok(wrap(tokens));
      }
      Ok(wrap(&by_text("call: no piece holds it", assoc)))
    },
    None => Ok(wrap(&by_text("no source", assoc))),
  }
}

/// A call's content as a stamped rehearsal reads it ([`rehearsed`]).
type CallRead = (Vec<Event>, Vec<Formula>, Vec<EnvironmentStep>);

/// `call`'s content's events, formulas and environments, from a stamped rehearsal of it, as `plan::cut` reads a piece.
fn rehearsed(call: &ApiCall) -> Result<Option<CallRead>> {
  let content = call.content.unlist_ref();
  if content.len() > usize::from(FIELD_PROV_POS_MASK) {
    return Ok(None);
  }
  let stamped = field_stamped(2, 0, content);
  Ok(
    match rehearse(stamped, TitleMarks::Fnsymbol, &[], lex_with_formulas)? {
      Rehearsal::Done {
        value: (events, formulas),
        environments,
        ..
      } => Some((events, formulas, environments)),
      Rehearsal::Skipped(_) => None,
    },
  )
}

/// The tokens of the contact printing `key` in `call`'s content, or with `within` of its first (one printing a list).
fn call_contact(call: &ApiCall, key: &str, within: bool) -> Result<Option<Tokens>> {
  let Some((events, formulas, environments)) = rehearsed(call)? else {
    return Ok(None);
  };
  let field = Field {
    tokens:       call.content.unlist_ref(),
    base:         0,
    formulas:     &formulas,
    environments: &environments,
  };
  Ok(events.iter().find_map(|e| {
    let (start, end) = e.span?;
    let (start, end) = (usize::from(start), usize::from(end));
    (e.kind == EventKind::Contact && (within || norm(&e.text) == key))
      .then(|| field.content_of((start, end), 0, start))
      .flatten()
      .map(|(tokens, _)| tokens)
  }))
}

/// `call`'s content cut into the reading's entries (`split_pieces`), each its text and what it prints but its marks and its
/// contacts, cut at their spans, in the font it prints in: an entry whose contact prints through no API call (0911.3853's `\\email`
/// in an `\\institute`, 1001.0337).
fn call_pieces(call: &ApiCall) -> Result<Vec<(String, Vec<Token>)>> {
  let Some((events, formulas, environments)) = rehearsed(call)? else {
    return Ok(Vec::new());
  };
  let field = Field {
    tokens:       call.content.unlist_ref(),
    base:         0,
    formulas:     &formulas,
    environments: &environments,
  };
  let pieces = split_pieces(&events, &call.role);
  // (a list cut into entries shows the mark each opens with, as the Cut's `_markshown`: 1305.6194, hep-lat0501016)
  let shown = pieces.len() > 1;
  let mut out = Vec::new();
  for (text, piece) in pieces {
    let inks: Vec<&Event> = piece
      .iter()
      .filter(|e| e.kind == EventKind::Name || (shown && e.kind == EventKind::Mark))
      .collect();
    let (Some(first), Some(last)) = (inks.first(), inks.last()) else {
      DebugFeature!("frontmatter", "FRONT emit: a piece prints no name: {text}");
      continue;
    };
    let start = first.formula.and_then(|f| f.span).or(first.span);
    let (Some((start, _)), Some((_, end))) = (start, last.span) else {
      continue;
    };
    let cuts = piece
      .iter()
      .filter(|e| {
        !(matches!(e.kind, EventKind::Name | EventKind::Sep(_))
          || (shown && e.kind == EventKind::Mark))
      })
      .filter_map(|e| e.span)
      .filter(|&(s, t)| s >= start && t <= end)
      .chain(std::iter::once((end, end)));
    let (mut tokens, mut from) = (Some(Vec::new()), usize::from(start));
    for (s, t) in cuts {
      let s = usize::from(s);
      if s > from {
        let run = field
          .content_of((from, s), 0, from)
          .map(|(run, _)| run.unlist());
        tokens = tokens.zip(run).map(|(mut held, run)| {
          held.extend(run);
          held
        });
      }
      from = usize::from(t).max(from);
    }
    match tokens {
      Some(tokens) => out.push((
        norm(&text),
        in_its_font(first.font.clone(), tokens).unlist(),
      )),
      None => DebugFeature!(
        "frontmatter",
        "FRONT emit: a piece holds no balanced run: {text}"
      ),
    }
  }
  Ok(out)
}

/// The declarations of the blocks not yet emitted: an entry of their tokens, digested in the flush's group.
fn declarations<'a>(sources: impl Iterator<Item = &'a usize>) -> Vec<Token> {
  with_ledger(|ledger| {
    let mut out = Vec::new();
    for &index in sources {
      if let Some(Entry::Author(block)) = ledger.entries.get(index)
        && let Outcome::Events { declarations, .. } = &block.outcome
      {
        for (start, end) in declarations.iter().filter_map(|event| event.span) {
          let at = |p: u16| usize::from(p).checked_sub(block.range.0);
          if let (Some(start), Some(end)) = (at(start), at(end)) {
            out.extend_from_slice(
              block
                .tokens
                .unlist_ref()
                .get(start..end)
                .unwrap_or_default(),
            );
          }
        }
      }
    }
    out
  })
}

/// Where an association came from: `Assoc::source`.
type Source = (usize, Option<(u16, u16)>);

/// One item of the reading the emitter makes a call of: its key, the owners that don't hold it yet, the association, and its
/// place among the calls in source order.
struct Item<'a> {
  key:    Key,
  owners: Vec<Owner>,
  assoc:  &'a Assoc,
  place:  i64,
}

/// The items of `resolved` still to emit, one per source and text (a line printed for each of its authors, a call for each of
/// its creators: 0704.3133), in the reading's order, with what stays out of them: a footnote's text, what `relocate_footnote`
/// moves to the mark its name keeps; a title's note, kept as its call (`kept`); a line's note the old reader queued as the
/// document's; an ORCID left on its name; an entry's contact its entry takes along, the Link step placing it among the entry's
/// holders (one the reading gives anyone else `separate`: its own item, the entry's copy silent). Each once per owner, over flushes.
fn items_to_emit<'a>(
  resolved: &'a Resolved,
  owners: &[String],
  on_names: &BTreeSet<(usize, (u16, u16))>,
  kept: &mut Vec<RawFrontmatter>,
) -> (Vec<Item<'a>>, BTreeSet<usize>) {
  let pubnotes: BTreeSet<String> = with_ledger(|ledger| {
    let notes = ledger.entries.iter().filter_map(|entry| match entry {
      Entry::Api(call) if call.tag == "ltx:pubnote" => {
        Some(norm(&events::printed_text(&call.events)))
      },
      _ => None,
    });
    notes.collect()
  });
  let named = resolved
    .creators
    .iter()
    .enumerate()
    .filter(|(_, c)| !c.name.trim().is_empty());
  let held: Vec<(Owner, Key, &Assoc)> = named
    .flat_map(|(i, c)| {
      let items = plan::creator_items(c);
      items
        .into_iter()
        .map(move |(key, a)| (Owner::Creator(i), key, a))
    })
    .chain(
      plan::block_items(resolved)
        .into_iter()
        .map(|(key, a)| (Owner::Block, key, a)),
    )
    .collect();
  let inner_contact =
    |a: &Assoc| a.rule.contains("api:inner-contact") && !a.rule.contains("ruling3:written-out");
  let prints_itself = |index: usize| {
    api_call(index).is_some_and(|call| {
      let own = |e: &Event| e.kind == EventKind::Contact && e.meaning == events::EmailMeaning::Own;
      call.events.iter().any(own)
    })
  };
  let separate: BTreeSet<usize> = held
    .iter()
    .filter(|(owner, _, a)| {
      let entry_of = |(o, key, other): &(Owner, Key, &Assoc)| {
        o == owner && key.0 == Kind::Affiliation && other.source == a.source
      };
      inner_contact(a)
        && (!held.iter().any(entry_of) || a.source.is_some_and(|(index, _)| prints_itself(index)))
    })
    .filter_map(|(_, _, a)| a.source.map(|(index, _)| index))
    .collect();
  let mut items: Vec<Item> = Vec::new();
  let mut at: BTreeMap<(Key, Option<Source>), usize> = BTreeMap::new();
  for (owner, key, assoc) in held {
    let source = assoc.source;
    let of_footnote = source.is_some_and(|(index, _)| {
      with_ledger(|ledger| matches!(ledger.entries.get(index), Some(Entry::NoteText(_))))
    });
    if of_footnote {
      continue;
    }
    if assoc.rule.contains("api:title-note") {
      let once = s!("title-note|{}", key.1);
      if let Some(command) = source
        .and_then(|(index, _)| api_call(index))
        .and_then(|c| c.command)
        && lookup_mapping(EMITTED_ITEMS, &once).is_none()
      {
        assign_mapping(EMITTED_ITEMS, &once, Some(Stored::Bool(true)));
        kept.push(("ltx:note".to_string(), TagAttrs::default(), command));
      }
      continue;
    }
    if owner == Owner::Block
      && (assoc.rule.contains("note:unrequested") || assoc.rule.contains("note:address-line"))
      && pubnotes.contains(&key.1)
    {
      continue;
    }
    if inner_contact(assoc) && source.is_some_and(|(index, _)| !separate.contains(&index)) {
      continue;
    }
    let id = match owner {
      Owner::Creator(i) => owners[i].as_str(),
      Owner::Block => "block",
    };
    let id = s!("{id}|{:?}|{}", key.0, key.1);
    if lookup_mapping(EMITTED_ITEMS, &id).is_some() {
      continue;
    }
    assign_mapping(EMITTED_ITEMS, &id, Some(Stored::Bool(true)));
    if matches!(source, Some((index, Some(span))) if on_names.contains(&(index, span))) {
      continue;
    }
    match at.get(&(key.clone(), source)) {
      Some(&n) => items[n].owners.push(owner),
      None => {
        at.insert((key.clone(), source), items.len());
        items.push(Item {
          key,
          owners: vec![owner],
          assoc,
          place: 0,
        });
      },
    }
  }
  (items, separate)
}

/// `items` in Perl's order of a creator's contacts: what no label places attaches as it digests, then `relocateAnnotations`
/// appends what a label places, the queue digested in source order, each call with that place (`\\lx@frontmatter@queued`),
/// which an entry the block prints returns to (`relocate::at_queue_position`). What attached as it digested moves nothing back, past
/// a contact of the footer's zone before it too.
fn in_perl_order(items: &mut [Item]) {
  let in_source = |assoc: &Assoc| {
    assoc
      .source
      .map(|(index, span)| (index, span.map_or((1, 0), |(s, _)| (0, s))))
  };
  items.sort_by_key(|item| (in_source(item.assoc).is_none(), in_source(item.assoc)));
  for (n, item) in items.iter_mut().enumerate() {
    item.place = i64::try_from(n).unwrap_or(i64::MAX) + 1;
  }
  items.sort_by_key(|item| (by_label(item.assoc), item.place));
  let mut past_footer: BTreeSet<Owner> = BTreeSet::new();
  for item in items {
    let footer = item.assoc.source.is_some_and(|(index, _)| {
      with_ledger(|ledger| {
        matches!(ledger.entries.get(index), Some(Entry::Api(call)) if call.attrs.contains("=footer"))
      })
    });
    if !footer && !by_label(item.assoc) && item.owners.iter().any(|o| past_footer.contains(o)) {
      item.place += FOOTER_ZONE - BLOCK_ZONE;
    }
    if footer {
      past_footer.extend(item.owners.iter().copied());
    }
  }
}

/// The flush's added step: `commands`' creators and their annotations replaced by calls built from the reading. Whether
/// it did.
pub(in crate::base_utilities) fn emit_commands(commands: &mut Vec<RawFrontmatter>) -> Result<bool> {
  if fm_emit_mode() == 0 || rehearsal::rehearsing() {
    return Ok(false);
  }
  if let Some(why) = stays_out() {
    DebugFeature!("frontmatter", "FRONT emit stays out: {why}");
    return Ok(false);
  }
  let reading = with_ledger(|ledger| resolve(&ledger_inputs(ledger)));
  // (an entry's addresses, taken out, are the Kind steps' to cut, in place, from the entry whole: they run after the flush as
  // they do for a paper with no block, `insert_frontmatter`)
  let mut resolved = reading.clone();
  reading
    .taken
    .iter()
    .for_each(|taken| give_back_entry(&mut resolved, taken));
  if resolved.creators.is_empty() && resolved.block.is_empty() {
    DebugFeature!("frontmatter", "FRONT emit stays out: no reading");
    return Ok(false);
  }
  // (a call printing no word of its own stays as it is, the note it holds the document's: `\\IEEEauthorblockA{\\thanks{…}}`,
  // 1502.03038, 2005.06428, 2110.05178)
  let words_of = |call: &ApiCall| {
    let words: Vec<&Event> = call
      .events
      .iter()
      .filter(|e| e.kind != EventKind::Note)
      .collect();
    norm(&events::events_text(&words))
  };
  let wordless_calls: Vec<Tokens> = with_ledger(|ledger| {
    let calls = ledger.entries.iter().filter_map(|entry| match entry {
      Entry::Api(call)
        if call.events.iter().any(|e| e.kind == EventKind::Note) && words_of(call).is_empty() =>
      {
        call.command.clone()
      },
      _ => None,
    });
    calls.collect()
  });
  // (the queue: what the emitter takes, and what it leaves, in place)
  let (mut before, mut after, mut unprinted): (Vec<_>, Vec<_>, Vec<RawFrontmatter>) =
    Default::default();
  let mut seen = false;
  for entry in std::mem::take(commands) {
    let parent = annotated_parent(&entry.2);
    if entry.0 == "ltx:rdf" && parent == "ltx:creator" {
      unprinted.push(entry);
    } else if creator_call(&entry.0, &parent) && !wordless_calls.contains(&entry.2) {
      seen = true;
    } else if seen {
      after.push(entry);
    } else {
      before.push(entry);
    }
  }
  // (the creators: each its Perl label, `author:N`, new or emitted at an earlier flush: as `num_<tag>` carries a flush's counts)
  let mut owners: Vec<String> = Vec::with_capacity(resolved.creators.len());
  let mut fresh: Vec<usize> = Vec::new();
  let mut role_counts: BTreeMap<String, i64> = BTreeMap::new();
  for (i, c) in resolved.creators.iter().enumerate() {
    let id = s!("{}#{}#{i}", c.source, norm(&c.name));
    owners.push(match lookup_mapping(EMITTED_CREATORS, &id) {
      Some(owner) => owner.to_string(),
      None => {
        let role = api_call(c.source)
          .filter(|call| !call.role.is_empty())
          .map(|call| call.role);
        let role = role.unwrap_or_else(|| "author".to_string());
        let n = role_counts
          .entry(role.clone())
          .or_insert_with(|| lookup_mapping_int("num_ltx:creator", &role));
        *n += 1;
        let owner = s!("{role}:{n}");
        assign_mapping(EMITTED_CREATORS, &id, Some(Stored::String(pin(&owner))));
        fresh.push(i);
        owner
      },
    });
  }
  // (the blocks' names, block by block)
  let mut by_block: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
  for &i in &fresh {
    by_block
      .entry(resolved.creators[i].source)
      .or_default()
      .push(i);
  }
  by_block.retain(|source, _| {
    with_ledger(|ledger| matches!(ledger.entries.get(*source), Some(Entry::Author(_))))
  });
  let keeps_marks: BTreeMap<usize, bool> = by_block
    .keys()
    .map(|&source| (source, per_author_marks(source)))
    .collect();
  let keeps = |source: usize| keeps_marks.get(&source).copied().unwrap_or(false);
  let mut names: BTreeMap<usize, Option<Vec<Token>>> = BTreeMap::new();
  let mut on_names: BTreeSet<(usize, (u16, u16))> = BTreeSet::new();
  for (&source, members) in &by_block {
    let spelled: Vec<String> = members
      .iter()
      .map(|&i| resolved.creators[i].name.clone())
      .collect();
    let tokens = block_names(source, &spelled, keeps(source), &mut on_names);
    names.extend(members.iter().copied().zip(tokens));
  }
  let mut kept: Vec<RawFrontmatter> = Vec::new();
  let (mut items, separate) = items_to_emit(&resolved, &owners, &on_names, &mut kept);
  in_perl_order(&mut items);
  // (the labels: an item's own on every new owner; an owner made at an earlier flush holds its own copy under its Perl
  // `author:N`, its annotations long digested)
  let is_fresh: BTreeSet<usize> = fresh.iter().copied().collect();
  let mut annotations: Vec<Vec<String>> = vec![Vec::new(); resolved.creators.len()];
  let mut calls: Vec<(String, &Item)> = Vec::new();
  for (n, item) in items.iter().enumerate() {
    let new: Vec<usize> = item
      .owners
      .iter()
      .filter_map(|owner| match owner {
        Owner::Creator(i) if is_fresh.contains(i) => Some(*i),
        _ => None,
      })
      .collect();
    if item.owners.contains(&Owner::Block) {
      calls.push(("addresses:block".to_string(), item));
    }
    for owner in &item.owners {
      if let Owner::Creator(i) = owner
        && !is_fresh.contains(i)
      {
        calls.push((owners[*i].clone(), item));
      }
    }
    if !new.is_empty() {
      let label = s!("item:{}:{n}", lookup_int(FRONTMATTER_FLUSHES));
      for &i in &new {
        annotations[i].push(label.clone());
      }
      calls.push((label, item));
    }
  }
  // (a request no item answers stays a request, which shows it: `show_unanswered_marks`)
  let answered: BTreeSet<String> = resolved
    .creators
    .iter()
    .flat_map(|c| c.affiliations.iter().chain(&c.contacts).chain(&c.notes))
    .chain(&resolved.block)
    .filter_map(|a| a.label.as_deref())
    .flat_map(|label| mark_labels(label.rsplit(':').next().unwrap_or(label)))
    // (a `\\footnotemark` stays printed on its name: no request)
    .chain(
      by_block
        .keys()
        .flat_map(|&source| marks_of(source, "footnotemark")),
    )
    .collect();
  // (a mark kept on a name prints itself; a `\\inst` among them only requests: 1602.07956)
  let inst_marks: BTreeMap<usize, Vec<String>> = keeps_marks
    .iter()
    .filter(|(_, keep)| **keep)
    .map(|(&source, _)| (source, marks_of(source, "inst")))
    .collect();
  for &i in &fresh {
    let c = &resolved.creators[i];
    let keeps = keeps(c.source);
    let inst_marks = inst_marks.get(&c.source).map_or(&[][..], Vec::as_slice);
    for request in c
      .requests
      .iter()
      .filter(|request| !keeps || inst_marks.contains(request))
    {
      // (a key's value is tokenized: a request spelled with a command, `\\author[a,b\\thanks{…}]`'s, 2207.11842, is no label)
      if !answered.contains(request)
        && !request.contains(['\\', '_', '^', '%', '#', '$', '&', '{', '}', '~'])
      {
        annotations[i].extend(clean_frontmatter_labels(request, "affiliation"));
      }
    }
  }
  // (the emission)
  let mut per_source: BTreeMap<usize, usize> = BTreeMap::new();
  for c in &resolved.creators {
    *per_source.entry(c.source).or_default() += 1;
  }
  let mut out = before;
  let entry =
    |tag: &str, tokens: Vec<Token>| (tag.to_string(), TagAttrs::default(), Tokens::new(tokens));
  let decls = declarations(by_block.keys());
  if !decls.is_empty() {
    out.push(entry("ltx:emit", decls));
  }
  for &i in &fresh {
    let c = &resolved.creators[i];
    let keys = s!("annotations={{{}}}", annotations[i].join(","));
    // (the call, where it is this creator alone)
    let sole = per_source.get(&c.source) == Some(&1);
    let call = (!by_block.contains_key(&c.source) && sole)
      .then(|| api_call(c.source).and_then(|call| call.command))
      .flatten()
      .and_then(|command| rekeyed(&command, &s!("{keys},"), false, true, true));
    let tokens = call.unwrap_or_else(|| {
      let name = names.remove(&i).flatten().unwrap_or_else(|| {
        DebugFeature!(
          "frontmatter",
          "FRONT emit text name: {} [{}]",
          c.name,
          c.rule
        );
        text_tokens(&c.name)
      });
      let content = cs_group("\\lx@frontmatter@item", &[&name]);
      let content = cs_group("\\lx@personname", &[&cs_group("\\lx@author@withinst", &[
        &content,
      ])]);
      let mut head = vec![T_CS!("\\lx@add@frontmatter@now")];
      head.extend(cs_group_args(&["ltx:creator"]));
      with_keys(head, &s!("role=author,{keys}"), &content)
    });
    out.push(entry("ltx:creator", tokens.unlist()));
  }
  // (an unprinted contact, where its call stands among the items: its author's, the last creator before it. Its call's place
  // in the ledger: `UnprintedCall::at`, of this flush's last ones)
  let unprinted_at: Vec<usize> = with_ledger(|ledger| {
    let at: Vec<usize> = ledger.unprinted.iter().map(|call| call.at).collect();
    at[at.len().saturating_sub(unprinted.len())..].to_vec()
  });
  let mut unprinted = unprinted_at.into_iter().zip(unprinted).peekable();
  let rdf_entry = |at: usize, rdf: RawFrontmatter| {
    let owner = resolved
      .creators
      .iter()
      .rposition(|c| c.source < at)
      .map(|last| &owners[last]);
    let keyed =
      owner.and_then(|owner| rekeyed(&rdf.2, &s!("label={{{owner}}}"), true, false, true));
    (rdf.0, rdf.1, keyed.unwrap_or(rdf.2))
  };
  let mut pieces = BTreeMap::new();
  let base = lookup_int(QUEUE_ORDINAL_COUNTER);
  for (label, item) in calls {
    while let Some((at, rdf)) =
      unprinted.next_if(|(at, _)| item.assoc.source.is_some_and(|(index, _)| *at <= index))
    {
      out.push(rdf_entry(at, rdf));
    }
    let tokens = item_call(&item.key, item.assoc, &label, &separate, &mut pieces)?;
    let tag = if item.key.0 == Kind::Note {
      "ltx:note"
    } else {
      "ltx:contact"
    };
    let mut placed = cs_group("\\lx@frontmatter@queued", &[&internal(
      &(base + item.place).to_string(),
    )]);
    placed.extend(tokens.unlist());
    out.push(entry(tag, placed));
  }
  out.extend(unprinted.map(|(at, rdf)| rdf_entry(at, rdf)));
  out.extend(kept);
  out.extend(after);
  // (a document's note's own `\\email` is the reading's item already: `api:note-spelled`, 0910.1640)
  for entry in out.iter_mut().filter(|entry| entry.0 == "ltx:pubnote") {
    entry.2 = Tokens::new(cs_group("\\lx@frontmatter@itemonly", &[entry
      .2
      .unlist_ref()]));
  }
  *commands = out;
  assign_value(EMITTED, Stored::Bool(true), Some(Scope::Global));
  with_ledger(|ledger| {
    ledger.writer = Some("written".to_string());
    ledger.compared = Some(reading);
  });
  Ok(true)
}

/// Whether the block at ledger index `source` keeps its marks printed on its names, as Perl's `\\lx@add@author` leaves them (a
/// name's marks are its content where no `\\lx@author@withsup` reads them): a name list a binding split, or a per-author class's
/// call whose lines answer none of them (`\\author[*]{… \\\\ $^1$DeepSeek-AI …}`, 2401.14196, is one that does).
fn per_author_marks(source: usize) -> bool {
  with_block(source, false, |block, events| {
    let marks = |first: bool| {
      events
        .iter()
        .filter(move |e| e.kind == EventKind::Mark && e.line_first == first)
    };
    let answers = marks(true).filter_map(|e| e.label.as_deref()).any(|label| {
      marks(false)
        .filter_map(|e| e.label.as_deref())
        .any(|names| mark_labels(names).iter().any(|name| name == label))
    });
    block.hook == "\\lx@add@author@split" || (block.cautious && !answers)
  })
}
