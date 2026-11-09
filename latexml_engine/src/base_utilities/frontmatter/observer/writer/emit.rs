//! The one added step at the flush (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, Route D; OXIDIZED_DESIGN_DIVERGENCES
//! #980), on by default (`LATEXML_FM_EMIT=0` the old driver until its deletion, [`fm_emit_mode`]): for a paper with an
//! author block read, the queued creators and their annotations are replaced by Perl's own calls built from the reading
//! (`Resolved`) — `\lx@add@frontmatter@now{ltx:creator}`,
//! `\lx@annotate@frontmatter@now{ltx:creator}{ltx:contact}`, `\lx@add@thanks` — the owners by Perl's `label=` and
//! `annotations=`. What Perl's `\lx@add@authors` (Base_Utility.pool.ltxml:687) cuts with `SplitTokens` at the call, this
//! cuts at the digested list's positions, as digested material doesn't exist at the call. A token slice is the API's
//! argument; where it's cut comes from the events' spans.

use std::{
  cell::Cell,
  collections::{BTreeMap, BTreeSet, btree_map::Entry as Entry2},
  rc::Rc,
  sync::OnceLock,
};

use latexml_core::document::tag::RawFrontmatter;

use super::{
  address_cut::{
    EntryCut, cut_entry_call, cut_mail_call, entry_cuts, span_tokens_cut, span_tokens_within,
  },
  calls::{document_note_call, note_call_end},
  dom::{Kind, norm},
  plan::{self, Key, Owner, api_call},
  repart::{entry_span, prefix_tokens},
  *,
};
use crate::base_utilities::{
  defs::{
    author_macros::KEPT_WHOLE,
    frontmatter_api::{BLOCK_CREATOR, LABEL_PRINTS, PRINTED_LABEL},
  },
  frontmatter::{
    fallback::{tabular_juggles_in_a_whole_block, whole_block_author},
    observer::{
      events::{EventKind, Formula, InkTable, lex_with_inks},
      field_notes::{EnvironmentStep, FieldCreator},
      ledger::{ApiCall, AuthorBlock, annotated_parent, content_text, field_stamped},
      policy::TitleMarks,
      rehearsal::{Rehearsal, rehearse},
      resolver::{
        BlockCheck, LostInk, RCreator, RuleDrop, Written, check_blocks, contact_parts,
        creator_call, is_creator_call, merged_names, printed_email_label, read_events,
        split_pieces, uncoordinated_marks,
      },
    },
    relocate::{BLOCK_ZONE, FOOTER_ZONE},
  },
};

thread_local! {
  static FM_EMIT_OVERRIDE: Cell<Option<u8>> = const { Cell::new(None) };
}

/// The frontmatter emitter's switch, the one place `LATEXML_FM_EMIT` is read (once per process), or this thread's test
/// override: 2, the default (unset or unreadable), each call queues one whole block and the emitter writes it; 0 the old
/// driver (the reader at the call and the writer), reachable until its deletion (Z18-Z19); 1 the reader still runs at the
/// call (OD #980).
pub fn fm_emit_mode() -> u8 {
  static MODE: OnceLock<u8> = OnceLock::new();
  FM_EMIT_OVERRIDE.with(Cell::get).unwrap_or_else(|| {
    *MODE.get_or_init(|| {
      std::env::var("LATEXML_FM_EMIT")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(2)
    })
  })
}

/// Set (`Some`) or clear (`None`) this thread's override of the switch (tests: the environment is read-only to them).
pub fn set_fm_emit_override(mode: Option<u8>) { FM_EMIT_OVERRIDE.with(|o| o.set(mode)); }

/// The State value saying a flush emitted (the writer has nothing left to do, `write_observed_frontmatter`): the ledger's
/// generation whose reading the last emit wrote to its end ([`Ledger::generation`]), which a later flush with nothing queued
/// and nothing new read writes again as nothing. A State value, as a speculative run's flush is undone with it.
const EMITTED: &str = "lx_fm_emitted";

/// Whether a flush of this conversion emitted.
pub(in crate::base_utilities) fn emitted() -> bool { lookup_value(EMITTED).is_some() }
/// The State value true inside `\\lx@frontmatter@itemonly`: what an item's content marks labels nothing, the reading's
/// labels stand (`with_pending_entry_attr`).
pub(in crate::base_utilities) const EMITTING: &str = "lx_fm_emitting";
/// The State mappings of what earlier flushes emitted, as `num_<tag>` counts (Perl's way of carrying a flush's counts):
/// a creator's `source#letters#place` to its `role:N`, and an item's `owner|kind|text` (and a ledger call a flush digested
/// as queued, `taken|<index>`: [`take_digested`]).
const EMITTED_CREATORS: &str = "lx_fm_emitted_creators";
const EMITTED_ITEMS: &str = "lx_fm_emitted_items";
/// The State value saying a flush wrote the author block's own creator ([`block_creator`]).
const BLOCK_CREATOR_WRITTEN: &str = "lx_fm_block_creator";
/// Whether a flush wrote the author block's own creator ([`block_creator`]), which `relocate_annotations` then settles.
pub(in crate::base_utilities) fn block_creator_written() -> bool {
  lookup_value(BLOCK_CREATOR_WRITTEN).is_some()
}

/// `\lx@add@frontmatter@now`, the flush's own creator call.
fn add_now() -> Token { T_CS!("\\lx@add@frontmatter@now") }

/// Tokens of an internal string.
fn internal(text: &str) -> Vec<Token> {
  mouth::tokenize_internal(TeXString::assembled(text.to_string())).unlist()
}

/// `f` on the rehearsed block at ledger index `index`, else `default`.
fn with_block<R>(index: usize, default: R, f: impl FnOnce(&AuthorBlock, &[Event]) -> R) -> R {
  with_ledger(|ledger| match ledger.entries().get(index) {
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
fn with_keys(head: Vec<Token>, keys: &str, content: &[Token]) -> Tokens {
  with_key_tokens(head, internal(keys), content)
}

/// `[keys]{content}` after `head`, the keys tokens.
fn with_key_tokens(mut head: Vec<Token>, keys: Vec<Token>, content: &[Token]) -> Tokens {
  head.push(T_OTHER!("["));
  head.extend(keys);
  head.push(T_OTHER!("]"));
  head.push(T_BEGIN!());
  head.extend_from_slice(content);
  head.push(T_END!());
  Tokens::new(head)
}

/// `\lx@annotate@frontmatter@now{ltx:creator}{ltx:contact}[keys]{content}`.
pub(super) fn annotate_now(tag: &str, keys: &str, content: &[Token]) -> Tokens {
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
pub(super) fn text_tokens(text: &str) -> Vec<Token> {
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
    // (a label its binding prints with the line, `_labelprints`: the call's label goes with it, in a key of its own,
    // `_printedlabel`, which says it prints, the reading's label replacing the call's — the Kind step for a legend reads it,
    // `insert::legend_mark`: authblk's `\\affil[*]{Corresponding author …}`, 2605.31330, 2201.11910; P2 NT, P2r2)
    let (mut label, mut prints) = (None, false);
    let (mut depth, mut from) = (0, open + 1);
    for i in open + 1..=close {
      let t = &v[i];
      depth += t.group_step();
      if i == close || (depth == 0 && t.get_catcode() == Catcode::OTHER && t.with_str(|s| s == ","))
      {
        let pair = &v[from..i];
        let mut eq = 0;
        let key: String = pair
          .iter()
          .take_while(|t| !t.with_str(|s| s == "="))
          .inspect(|_| eq += 1)
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
        if key == "label" {
          label = pair.get(eq..);
        }
        prints |= key == LABEL_PRINTS;
        from = i + 1;
      }
    }
    if prints && let Some(label) = label {
      out.extend(internal(&s!(",{PRINTED_LABEL}")));
      out.extend_from_slice(label);
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

/// How many spaces `content` opens with, which print nothing there: an item's digestion skips them.
fn leading_spaces(content: &[Token]) -> usize {
  content
    .iter()
    .take_while(|t| t.get_catcode() == Catcode::SPACE)
    .count()
}

/// `\\lx@frontmatter@itemonly{content}`: past the spaces a content opens with ([`leading_spaces`]).
pub(super) fn item_only(content: &[Token]) -> Vec<Token> {
  cs_group("\\lx@frontmatter@itemonly", &[
    &content[leading_spaces(content)..]
  ])
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
/// (1507.02068's `\\begin{center}M. Ablikim`), a column spec with it. As the digestion's frames say it. And past the
/// spaces after each such opening, which print nothing (the line's end after `\\begin{small}` and `\\begin{center}`,
/// 1507.02068, 2609.17357).
fn past_open_environment(field: &Field, start: usize, end: usize) -> usize {
  let at = |p: u16| usize::from(p).saturating_sub(field.base);
  let environments = field.environments;
  let mut from = start;
  for (n, step) in environments.iter().enumerate() {
    if step.opens && at(step.start) == from && at(step.end) < end {
      let closed = environments[n + 1..]
        .iter()
        .any(|later| !later.opens && at(later.start) < end);
      if !closed {
        from = at(step.end);
        from += field.tokens.get(from..end).map_or(0, leading_spaces);
      }
    }
  }
  from
}

/// `end`, or past the letter, the control sequence or the group an accent sets, which the cover of its box stops short of, as
/// nothing prints there but through it (`Živkovi\\'c`, `Petri\\v{c}`, ``Chiriv\\`\\i``: 2406.17108, 1202.0103, 1709.03335): a
/// control sequence last, no event until past that. The argument as TeX reads an undelimited one: one token (a control word
/// such as `\\i` included) or a balanced group.
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
    Some(Catcode::LETTER | Catcode::OTHER | Catcode::CS) => at + 1,
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
/// print in where a cover is a wrapper's holding what else prints (`\\textbf{Ann, Bob}`, 2305.12182; a mark or a note after
/// the name, `\\textbf{Yan Yan$^{1}$\\thanks{…}}`, 2309.13682, 2212.02936, which the reading writes apart: P2 B) or past
/// an environment's opening, grown to what `grown` adds. `None` for a run in a formula holding more
/// (`$\\text{Ann}^{a},\\text{Bob}^{b}$`, 2602.01228), or no balanced one.
fn event_run(
  field: &Field,
  events: &[Event],
  ends: (&Event, &Event),
  grown: Option<(usize, usize)>,
  cuts: &[(usize, usize)],
) -> Option<Vec<Token>> {
  event_run_at(field, events, ends, grown, cuts).map(|(tokens, _)| tokens)
}

/// [`event_run`], and the run of the block's tokens it was taken from (from, to).
fn event_run_at(
  field: &Field,
  events: &[Event],
  (first, last): (&Event, &Event),
  grown: Option<(usize, usize)>,
  cuts: &[(usize, usize)],
) -> Option<(Vec<Token>, (usize, usize))> {
  let at = |p: u16| usize::from(p).checked_sub(field.base);
  let (start, end) = (at(first.span?.0)?, at(last.span?.1)?);
  let start = past_open_environment(field, start, end);
  let own_at = |e: &Event| {
    e.marks
      .filter(|_| e.kind == EventKind::Name)
      .and_then(|(s, t)| Some((at(s)?, at(t)?)))
  };
  let inside = |e: &Event| first.span <= e.span && e.span <= last.span;
  let first_own = own_at(first).map_or(start, |(own, _)| own);
  let last_own = own_at(last).map(|(_, own)| own);
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
      // (or a mark or a note after it, which the reading answers or writes apart: `\\textbf{Yan Yan$^{1}$\\thanks{…}}`
      // kept its answered mark on the name and wrote the note twice, 2309.13682, 2212.02936 — P2 B)
      || (matches!(e.kind, EventKind::Mark | EventKind::Note)
        && last_own.is_some_and(|own| {
          mark_at
            .or(e.span.map(|s| s.0))
            .and_then(at)
            .is_some_and(|mark| own <= mark && mark < end)
        }))
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
    // (a name a creator call in the block printed is the call's name, its argument, the call left out: `\speaker{Ann Able}`
    // written again would add its creator twice, one with no name — the call's role is the creator's, `creator_role`)
    (Some((own, _)), Some((_, last))) if first.creator_call.is_some() => (own, last),
    (Some((own, _)), Some((_, last))) if wider => (own, past_what_it_set(field, events, last)),
    (_, Some((_, last))) => (start, past_what_it_set(field, events, end.max(last))),
    _ => (start, past_what_it_set(field, events, end)),
  };
  if let Some((s, t)) = grown {
    run_start = run_start.min(s);
    run_end = run_end.max(t);
  }
  let from = in_a_group(field.tokens, run_start, run_end);
  let (tokens, _) = field.content_cut((from, run_end), 0, run_start, cuts)?;
  let tokens = if wider || first_own <= start {
    in_its_font(first.font.clone(), tokens.unlist()).unlist()
  } else {
    tokens.unlist()
  };
  Some((tokens, (from, run_end)))
}

/// A block's names' tokens: their marks with them where `marks` (a per-author class's block, whose names keep them printed as
/// Perl does), a `\\footnotemark` (what `relocate_footnote` moves its text to, 2203.12964) and an ORCID written on it, its logo in
/// the name as today's (2208.05251), in `on_names`; a `\\cite` with it always, a mark of no text of its own that the old reader
/// keeps in the personname (`name_takes_annotations`). `None` for a name its events don't spell, with the cites the
/// reading moved onto it from the remark its event opens (`Ann Able (for the\\cite{k} ATLAS Collaboration)`, review revG1
/// 3), which the writer writes after its text.
fn block_names(
  index: usize,
  names: &[String],
  marks: bool,
  on_names: &mut BTreeSet<(usize, (u16, u16))>,
) -> Vec<(Option<Vec<Token>>, Vec<Token>)> {
  with_ledger(|ledger| {
    let Some(Entry::Author(block)) = ledger.entries().get(index) else {
      return vec![(None, Vec::new()); names.len()];
    };
    let Outcome::Events {
      events, formulas, environments, ..
    } = &block.outcome
    else {
      return vec![(None, Vec::new()); names.len()];
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
        e.kind == EventKind::Mark
          && (marks || matches!(e.role.as_deref(), Some("footnotemark" | "cite")))
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
      // (an IEEE ordinal the reading joined to the name, `join_ordinals`, stays with it, as today's personname:
      // `1\\textsuperscript{st} Sertac`, 2412.10792, 2202.06596, 1907.06250, 2605.00900)
      let name_span = found[from].1.span;
      if events.iter().any(|e| {
        e.kind == EventKind::Mark
          && e.ordinal
          && e
            .span
            .zip(name_span)
            .is_some_and(|(mark, name)| name.0 <= mark.0 && mark.1 <= name.1)
      }) {
        grown.0 = grown.0.min(at(name_span?.0)?);
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
        // (a mark past the close of a group of names the name's run opens no part of is the last name's, its own run
        // after the name: `{Ann Able$^{\star}$, …, David López-Pérez}$^{\sharp}$`, 2504.02633 — the old reader's names
        // split reads it so, `split_author_line`'s step 1 (`names_in_wrapper`, `author_block/names_split.rs`: a wrapper
        // holding names with only marks after it split into them, the marks the last name's; `math_active_scripts` only
        // makes breqn's `^` a superscript); grown over the group's close, the run was no balanced one and the name lost
        // its markup, FB1's P4; the depth is counted from the name's first token, so only a close of a group opened before
        // the name counts — a group the name's run opens and closes itself is none: revtex's `D. ${\rm Dorn}^{*}$`, whose
        // `}` closes `{\rm Dorn}` and whose mark the run already holds, written once; nor a mark in the formula the
        // name's first token is in, the formula opening before it, which the run takes whole: revtex's `${\rm
        // Ann~Able}^{a}$`, whose `}` closes the `{\rm` the name opens inside, its mark written twice — review revH5
        // finding 2)
        let past_group = || {
          let name_from = at(found[from].1.span?.0)?;
          let formula = field.formula_start(name_from);
          if formula < name_from && field.formula_start(s) == formula {
            return Some(false);
          }
          let closes = field.tokens.get(name_from..s)?.iter().scan(0, |depth, t| {
            *depth += t.group_step();
            Some(*depth)
          });
          Some(
            closes.min().is_some_and(|d| d < 0)
              && in_a_group(field.tokens, name_from, s) == name_from,
          )
        };
        if marks && past_group() != Some(true) {
          grown.1 = grown.1.max(t);
        } else if let Some((tokens, _)) = field.content_of((s, t), 0, s) {
          apart.extend(tokens.unlist());
        }
        if is_orcid(e) {
          on_names.insert((index, span));
        }
      }
      let (tokens, (run_from, run_to)) = event_run_at(
        &field,
        events,
        (found[from].1, found[to].1),
        Some(grown),
        &[],
      )?;
      // (a note the name's tokens print is written with it, not again as an item: `\\newcommand{\\ann}{Ann
      // Able\\thanks{…}}`, whose name is the one token `\\ann` — revLO4; a wrapper's note is cut out of the run,
      // `event_run`: P2 B. So is an ORCID link the run prints, the link printing the name (`link_names`): the name's
      // `ltx:ref` to orcid.org is its ORCID, as off and Perl hold it, not a contact writing the link and the name again —
      // `\\href{https://orcid.org/…}{\\usebox{\\orcid}\\hspace{1mm}Luca Fehlings\\thanks{…}}`, 2410.07791, 2507.22485,
      // 2602.05817; `\\href{https://orcid.org/…}{E. L. Gagnon}$^{*,1,2}$`, 2312.16289)
      let printed_with_name = |e: &&Event| {
        e.kind == EventKind::Note || (e.kind == EventKind::Contact && contact_parts(e).0 == "orcid")
      };
      for e in events.iter().filter(printed_with_name) {
        if let Some(span) = e.span
          && at(span.0).is_some_and(|s| run_from <= s)
          && at(span.1).is_some_and(|t| t <= run_to)
        {
          on_names.insert((index, span));
        }
      }
      // (a footnote symbol in a kept mark is shown apart from its numbers, `rewrite_symbol_superscripts`'s: 1610.06117)
      let mut out = if marks {
        rewrite_symbol_superscripts(Tokens::new(tokens)).unlist()
      } else {
        tokens
      };
      if marks && !apart.is_empty() {
        out.extend(rewrite_symbol_superscripts(Tokens::new(apart)).unlist());
      } else {
        out.extend(apart);
      }
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
    // (the cites after a Name a name's text opens, up to the line's next separator: the remark's, which the reading moved
    // onto the name)
    let cites_of = |from: usize, key: &str| -> Vec<Token> {
      let Some(&(at_name, _)) = found[from..].iter().find(|(_, e)| {
        let text = norm(&e.text);
        text.len() > key.len() && text.starts_with(key)
      }) else {
        return Vec::new();
      };
      let within = |e: &&Event| {
        matches!(
          e.kind,
          EventKind::Name | EventKind::Mark | EventKind::Note | EventKind::Layout
        )
      };
      let mut cites = Vec::new();
      for e in read[at_name + 1..].iter().take_while(within) {
        if e.kind == EventKind::Mark
          && e.role.as_deref() == Some("cite")
          && let Some((s, t)) = e.span.and_then(|(s, t)| Some((at(s)?, at(t)?)))
          && let Some((tokens, _)) = field.content_of((s, t), 0, s)
        {
          cites.extend(tokens.unlist());
        }
      }
      cites
    };
    let mut cursor = 0;
    let mut tokens_of = |name: &String| {
      let key = norm(name);
      let Some((from, to)) =
        (cursor..found.len()).find_map(|from| Some((from, from + spelled_from(from, &key)?)))
      else {
        return (None, cites_of(cursor, &key));
      };
      cursor = to + 1;
      (name_tokens(from, to), Vec::new())
    };
    names.iter().map(&mut tokens_of).collect()
  })
}

/// Why the emitter stays out of the paper the ledger holds: no block, a block no rehearsal read, or a block calling a
/// binding's list setter (`AuthorBlock::list_setter`: the rehearsal reads the call as one affiliation of the block, the real
/// call queues entries numbered for the marks, which no reading points to — cond-mat0605195's `\institute` in an author's
/// group; each block then written whole, its queued entries as today's).
fn stays_out() -> Option<&'static str> {
  with_ledger(|ledger| {
    let mut blocks = 0;
    for entry in ledger.entries() {
      if let Entry::Author(block) = entry {
        blocks += 1;
        if block.list_setter.is_some() {
          return Some(LIST_SETTER_IN_BLOCK);
        }
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

/// The emitter's reason to stay out of a paper a block of which calls a binding's list setter ([`stays_out`]).
const LIST_SETTER_IN_BLOCK: &str = "a binding's list setter called inside the block";

/// An item or a name written from the reading's text, its markup lost (fonts, links, math): a Warning naming it and `why`, the
/// charter's count of what the emitter could not take from the tokens. `None` for an item that is its text (an ORCID's id).
fn by_text(why: Option<&str>, text: &str) -> Vec<Token> {
  if let Some(why) = why {
    Warn!(
      "unexpected",
      "frontmatter",
      s!("The frontmatter emitter writes \"{text}\" as its plain text, its markup lost: {why}")
    );
  }
  text_tokens(text)
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
  span_tokens_cut(index, span, &[])
}

/// A call's tokens around its last argument, and that argument's: through its opening brace, inside, from its closing one.
pub(super) fn last_argument(call: &Tokens) -> Option<(Vec<Token>, Vec<Token>, Vec<Token>)> {
  let mut tokens = call.unlist_ref().to_vec();
  let close = tokens
    .iter()
    .rposition(|t| t.get_catcode() == Catcode::END)?;
  let mut depth = 0;
  let open = (0..=close).rev().find(|&i| {
    depth += tokens[i].group_step();
    depth == 0
  })?;
  let after = tokens.split_off(close);
  let content = tokens.split_off(open + 1);
  Some((tokens, content, after))
}

/// The tokens of the block at ledger index `index` that the span `span` covers, as written: a call's own, whatever formula
/// holds it ([`span_tokens_of`] takes that formula whole).
fn span_call_tokens(index: usize, span: (u16, u16)) -> Option<Tokens> {
  with_ledger(|ledger| {
    let Some(Entry::Author(block)) = ledger.entries().get(index) else {
      return None;
    };
    let at = |p: u16| usize::from(p).checked_sub(block.range.0);
    let tokens = block.tokens.unlist_ref();
    tokens
      .get(at(span.0)?..at(span.1)?)
      .map(|call| Tokens::new(call.to_vec()))
  })
}

/// `call`, its content's printed list written down to the one `address` of it (the reading's `ruling3:written-out`): its last
/// argument rehearsed, the run from its first ink to its last is the list, and the address the tokens of the ink printing it
/// (`\\textcolor{blue}{a@x}, \\textcolor{blue}{b@y}`, review revZ4a; `\\lx@address@urlfont{a@x, b@y}`, `\\textcolor{blue}{a@x,
/// b@y}`, 2211.14973), what's around the list stands; its text in the list's font where no ink prints it alone.
fn one_address(call: &Tokens, address: &str) -> Result<Tokens> {
  let Some((mut tokens, content, after)) = last_argument(call) else {
    return Ok(call.clone());
  };
  let read = rehearsed(&content)?;
  let (events, formulas, environments) = read
    .as_ref()
    .map_or((&[][..], &[][..], &[][..]), |(e, f, n, _)| {
      (e.as_slice(), f.as_slice(), n.as_slice())
    });
  let field = Field {
    tokens: &content,
    base: 0,
    formulas,
    environments,
  };
  let inks: Vec<&Event> = events
    .iter()
    .filter(|e| matches!(e.kind, EventKind::Name | EventKind::Contact) && e.span.is_some())
    .collect();
  let list = inks
    .iter()
    .filter_map(|e| e.span)
    .fold(None, |list: Option<(u16, u16)>, (s, t)| {
      Some(list.map_or((s, t), |(from, to)| (from.min(s), to.max(t))))
    });
  // (the whole argument, where no run of inks reads as the list)
  let (from, to) = list
    .and_then(|(s, t)| field.balanced(usize::from(s), usize::from(t), 0))
    .unwrap_or((0, content.len()));
  // (the ink that is the address, as printed: a brace list's `gilba\}@x` is no ink of `gilba@x`, 2411.10891)
  let spaced = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
  let printed = inks
    .iter()
    .find(|e| spaced(&e.text) == spaced(address))
    .and_then(|e| event_run(&field, events, (e, e), None, &[]));
  tokens.extend_from_slice(&content[..from]);
  // (or its text in the font the list prints in, where no ink is it alone, `a@x; b@y`, 1305.0477, 1510.05412, or no rehearsal
  // reads the argument, past the positions: counted, its link lost)
  let font = inks.first().and_then(|e| e.font.clone());
  let why = if read.is_none() {
    "one address of a list past the positions"
  } else {
    "one address of a list no ink prints alone"
  };
  tokens.extend(printed.unwrap_or_else(|| in_its_font(font, by_text(Some(why), address)).unlist()));
  tokens.extend_from_slice(&content[to..]);
  tokens.extend(after);
  Ok(Tokens::new(tokens))
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

/// The note `span` of the block at ledger index `index` is one piece of, strictly inside it: from one of its body's
/// events' start to one's end (the reading's cut of a marked `\thanks`, `Resolver::keep_note`), its span.
fn note_piece(index: usize, span: (u16, u16)) -> Option<(u16, u16)> {
  with_block(index, None, |_, events| {
    let inner = events.iter().flat_map(|e| &e.inner);
    events.iter().chain(inner).find_map(|e| {
      e.span.filter(|&s| {
        e.kind == EventKind::Note
          && s != span
          && s.0 <= span.0
          && span.1 <= s.1
          && e
            .body
            .iter()
            .any(|b| b.extent().is_some_and(|b| b.0 == span.0))
          && e
            .body
            .iter()
            .any(|b| b.extent().is_some_and(|b| b.1 == span.1))
      })
    })
  })
}

/// A note the reading gives the author block alone, as Perl prints it: digested inside the entry's contact, where `\\thanks`
/// is `\\lx@add@pubnote@thanks` and `\\footnote` `\\lx@add@pubnote` (Base_Utility.pool.ltxml:331-332), one `<pubnote>` of
/// the document — its own tokens so made (`document_note_call`, the token reader's one conversion of a note to the document's).
/// 2504.09745's `$^{\\ddagger}$Ericsson Canada\\thanks{…}` once, where the switch off prints it per relocated copy and the
/// block's creator held it (review revLO2 1). `None` for an item of another kind or owners, or a note of another command
/// (`\\thanks[…]{…}`, `\\thanksref`), which the block's creator holds as before.
fn block_pubnote(item: &Item) -> Option<Vec<Token>> {
  if item.key.0 != Kind::Note || item.owners != [Owner::Block] {
    return None;
  }
  let (index, Some(span)) = item.assoc.source? else {
    return None;
  };
  event_at(index, span, |e| e.kind == EventKind::Note, |_| ())?;
  // (the note's command through its closing brace, out of a group switching its font: `{\\small\\thanks{…}}`)
  let tokens = span_tokens_of(index, span)?;
  let note = tokens.unlist_ref();
  let (start, end) = (0..note.len()).find_map(|i| note_call_end(note, i).map(|end| (i, end)))?;
  document_note_call(&note[start..end])
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
/// cond-mat0404185): each line's, at the Break events of `span`, with its break — without the runs `cuts` (an entry's address
/// runs, `address_cut`), each inside one line's (`\\begin{small} …\\end{small} \\\\ \\begin{small} … a@x \\end{small}`,
/// 1312.5468) or a whole line's, which goes with one break beside it (a label line, `\\ \\small{\\textbf{Correspondence:}
/// \\href{mailto:a@x}{a@x},}`: 2604.06805, review revP1br2 S1), else `None`.
fn lines_run(
  field: &Field,
  events: &[Event],
  span: (u16, u16),
  cuts: &[(usize, usize)],
) -> Option<Vec<Token>> {
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
  let at = |p: u16| usize::from(p).checked_sub(field.base);
  let mut out = Vec::new();
  let mut runs = Vec::new();
  let mut kept = 0;
  for line in &lines {
    let ends = line
      .first()
      .zip(line.last())
      .and_then(|(first, last)| Some((at(first.span?.0)?, at(last.span?.1)?)));
    // (a line a cut takes whole, and the break before it — the one after it, the first line's)
    if let Some((start, end)) = ends
      && let Some(&whole) = cuts.iter().find(|&&(s, t)| s <= start && end <= t)
    {
      runs.push(whole);
      continue;
    }
    if kept > 0 {
      out.push(T_CS!("\\\\"));
    }
    kept += 1;
    if let (Some(first), Some(last)) = (line.first(), line.last()) {
      let (tokens, run) = event_run_at(field, events, (first, last), None, cuts)?;
      out.extend(tokens);
      runs.push(run);
    }
  }
  cuts
    .iter()
    .all(|&(s, t)| runs.iter().any(|&(from, to)| from <= s && t <= to))
    .then_some(out)
}

/// `tokens` digested in the font their first ink printed in, against the flush's: what a group or a switch before them set.
pub(super) fn in_its_font(font: Option<Rc<Font>>, tokens: Vec<Token>) -> Tokens {
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
  pieces: &mut BTreeMap<usize, Vec<Piece>>,
  whole: &mut BTreeSet<usize>,
  cuts: &[EntryCut],
) -> Result<Tokens> {
  let role = assoc.role.as_str();
  let mut keys = s!("role={role},label={{{label}}}");
  // (an entry the reading took addresses out of: written as it printed, its tokens cut at the runs, `address_cut` — written by its
  // text, counted, where the path its whole takes doesn't cut; each address its owner's e-mail as printed, named by its label)
  if let Some(call) = cut_mail_call(cuts, assoc, &keys) {
    return Ok(call);
  }
  if key.0 == Kind::Affiliation
    && let Some(call) = cut_entry_call(
      cuts,
      assoc,
      |entry, cut| {
        let key = (Kind::Affiliation, norm(&entry.text));
        item_call(
          &key,
          entry,
          label,
          separate,
          pieces,
          whole,
          std::slice::from_ref(cut),
        )
      },
      || {
        kinded_affiliation(
          &keys,
          &by_text(Some("an entry's addresses cut"), &assoc.text),
        )
      },
    )
  {
    return call;
  }
  let cut = cuts.iter().find(|cut| cut.is_entry(assoc));
  // (an address the block prints after its label word is named by it, as printed, as the Kind steps name it: `E-mail:
  // \\email{…}`, 1412.5940; `E-mail: \\texttt{a@x}`, 0802.3384; `Emails:` on each of a list's, 2511.22090; `email`, 1209.0869)
  if role == "email"
    && let Some((index, Some(span))) = assoc.source
    && let Some(word) = with_block(index, None, |_, events| {
      printed_email_label(events, span, &assoc.text)
    })
  {
    keys = s!("{keys},name={{{word} }}");
  }
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
    whole.insert(index);
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
      let command = one_address(command, &assoc.text)?;
      if let Some(tokens) = rekeyed(&command, &s!("label={{{label}}}"), true, false, true) {
        return Ok(tokens);
      }
    } else if let Some(tokens) =
      // (an entry the block prints moves back to its place by its zone, what a label places alone: `at_queue_position`)
      rekeyed(
        cut.and_then(|cut| cut.command.as_ref()).unwrap_or(command),
        &s!("label={{{label}}}"),
        true,
        separate.contains(&index),
        by_label(assoc) || !matches!(role, "affiliation" | "address"),
      )
    {
      whole.insert(index);
      cut
        .filter(|cut| cut.command.is_some())
        .inspect(|cut| cut.applied.set(true));
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
      // (a note in a formula is its own call's, the formula around it the name's mark: `Derek F. Wong$^1\thanks{Corresponding
      // Author}$`, 2603.20851, 2407.15815 — its span's content is the whole formula, `content_of`)
      let cut_runs = cut.map_or(&[][..], |cut| &cut.cuts[..]);
      let argument = event_at(index, span, |e| e.kind == EventKind::Note, |_| ()).and_then(|()| {
        (span_tokens_cut(index, span, cut_runs)
          .as_ref()
          .and_then(note_argument))
        .or_else(|| {
          span_call_tokens(index, span)
            .as_ref()
            .and_then(note_argument)
        })
      });
      // (or one piece of a note the reading cut at its marks, sourced at its own span: those tokens, the reading's cut, never
      // cut again — `\\thanks{$^{1}$Univ A; $^{2}$Univ B. $^{*}$Corresponding author.}`, whose piece the writer's own cut of
      // the whole note lost, review revG5 1; 2509.19480, 2206.07648 — read within the note, whose argument is the footnote's,
      // never the formula's around it: `Bob Baker$^{2\thanks{$^{1}$Univ A; $^{2}$Univ B}}$`, whose pieces were that
      // formula, the `\\thanks` digested twice, review rck2; else the span's exact slice)
      let piece = argument
        .is_none()
        .then(|| note_piece(index, span))
        .flatten();
      let argument = argument.or_else(|| {
        piece
          .and_then(|note| {
            span_tokens_within(index, span, Some(note), cut_runs)
              .or_else(|| span_call_tokens(index, span))
          })
          .map(Tokens::unlist)
      });
      let piece = piece.is_some();
      // (never a `\\footnote`'s, which `\\lx@add@note` keeps a note: `\\footnote{\\textsuperscript{1}These authors
      // contributed equally …}`, 2110.12892 — P2 NT)
      let footnote = event_at(index, span, |e| e.kind == EventKind::Note, |e| e.footnote);
      let an_entry = key.0 == Kind::Affiliation
        || (role == "legend"
          && footnote != Some(true)
          && argument.as_ref().is_some_and(|argument| {
            piece || starts_with_affiliation_mark(&Tokens::new(argument.clone()))
          }));
      if note
        && !an_entry
        && let Some(footnote) = footnote
      {
        let content = argument;
        return Ok(note_call(
          footnote,
          None,
          label,
          content.unwrap_or_else(|| by_text(Some("note no argument"), &assoc.text)),
        ));
      }
      // (a contact's command: as written, its meaning the class's, a call inside the contact taking its label as Perl's
      // `\lx@annotate@frontmatter@now` has it, an empty contact around it dropped)
      // (one a mark's formula holds is its text, the formula its run would grow to printing the mark: an ORCID icon,
      // `$^{1\,\orcidicon{…}}$`, 2210.03748, 2206.03602; the id is the item, no markup of it lost, no Warning)
      let in_a_mark = with_block(index, false, |_, events| {
        events.iter().any(|e| {
          let formula = e.formula.and_then(|f| f.span);
          e.kind == EventKind::Mark && formula.is_some_and(|s| s.0 <= span.0 && span.1 <= s.1)
        })
      });
      if contact_role && in_a_mark {
        return Ok(wrap(&by_text(None, &assoc.text)));
      }
      // (a note a contact command of the block makes is that command too, a contact as its binding makes it: jheppub's
      // `\note{…}` = `\lx@add@contact[role=note]{…}` inside an `\author`, Perl's jheppub.sty.ltxml:39 — 1403.1099,
      // 2001.06318, 2308.04217, which lost it inside a `ltx:note` around the call. Only where the span prints this one
      // event: one a macro holding a name and the command spans, `\newcommand{\ann}{Ann Able\note{…}}`, or two commands,
      // `\note{…}\note{…}`, is the whole macro, written for each item — the name twice, a note moved to the next author
      // (review revLO3 1); it is its text, as below)
      let events_there = with_block(index, 0, |_, events| {
        let inner = events.iter().flat_map(|e| &e.inner);
        events
          .iter()
          .chain(inner)
          .filter(|e| {
            matches!(
              e.kind,
              EventKind::Name | EventKind::Contact | EventKind::Note
            ) && e.span == Some(span)
          })
          .count()
      });
      if (contact_role || note)
        && events_there == 1
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
        let segments = if piece {
          vec![Tokens::new(argument)]
        } else {
          split_wrapped_affiliation_marks(Tokens::new(argument))
        };
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
          cut.inspect(|cut| cut.applied.set(true));
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
        return Ok(annotate_now("ltx:contact", &keys, content.unlist_ref()));
      }
      // (several events of one span: a macro printed them, 2604.09547)
      let printed_there =
        |e: &&Event| matches!(e.kind, EventKind::Name | EventKind::Contact) && e.span == Some(span);
      if with_block(index, 0, |_, events| {
        events.iter().filter(printed_there).count()
      }) > 1
      {
        let text = by_text(Some("a macro's span"), &assoc.text);
        // (a note a contact command made there is that contact, by its text: `<contact role="note">`, as the switch off has
        // it — jheppub's `\note{…}` in `\newcommand{\ann}{Ann Able\note{…}}`, review revLO3 1)
        if note && event_at(index, span, |e| e.kind == EventKind::Contact, |_| ()).is_some() {
          return Ok(annotate_now("ltx:contact", &keys, &item_only(&text)));
        }
        return Ok(wrap(&text));
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
        // (an affiliation the reading wrote without the address its line prints last: the tokens printing it, cut inside the
        // ink the address ends — 2203.10777, `prefix_tokens`)
        if key.0 == Kind::Affiliation
          && let Some((cut, tokens)) = prefix_tokens(index, span, &key.1)
        {
          let font = event_at(
            index,
            span,
            |e| e.kind != EventKind::Layout,
            |e| e.font.clone(),
          );
          return Ok(wrap(
            in_its_font(font.flatten(), spaced(index, cut, tokens.unlist())).unlist_ref(),
          ));
        }
        return Ok(wrap(&by_text(Some("span letters differ"), &assoc.text)));
      }
      let span = found.unwrap_or(span);
      // (its tokens, from its mark for a note's legend; in the font it prints in, the group or the switch before its mark too)
      let run = with_ledger(|ledger| {
        let Some(Entry::Author(block)) = ledger.entries().get(index) else {
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
        let tokens = match event_run(&field, events, (first, last), grown, cut_runs) {
          Some(tokens) => {
            cut.inspect(|cut| cut.applied.set(true));
            tokens
          },
          None => {
            let tokens = lines_run(&field, events, span, cut_runs)?;
            cut.inspect(|cut| cut.applied.set(true));
            tokens
          },
        };
        Some((tokens, first.font.clone(), mark.map(|e| e.text.clone())))
      });
      let Some((tokens, font, mark)) = run else {
        return Ok(wrap(&by_text(Some("span no balanced run"), &assoc.text)));
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
        return Ok(wrap(&by_text(Some("note text"), &assoc.text)));
      };
      // (a contact inside it: its command as written, as a block's)
      if contact_role && let Some(tokens) = call_contact(&call, assoc, written_out)? {
        return Ok(annotate_now("ltx:contact", &keys, tokens.unlist_ref()));
      }
      // (a note inside it: its command's argument as written, under the note's own kind, as a block's note — its fonts,
      // links and formulas kept: `\author{Craig Rae\thanks{Note \textit{italic} and $x^2$.}}`, review revG5 3)
      if note && let Some((footnote, argument)) = call_note(&call, assoc)? {
        return Ok(note_call(footnote, None, label, argument));
      }
      let found = match pieces.entry(index) {
        Entry2::Occupied(held) => held.into_mut(),
        Entry2::Vacant(place) => place.insert(call_pieces(&call)?),
      };
      // (the one piece by the reading's count, not the runs cut: one no run holds gives no other its place, review revZ4a)
      let piece = found.iter().find(|(text, _)| *text == key.1);
      match piece.or(found.first().filter(|_| found.len() == 1)) {
        Some((_, Some(tokens))) => Ok(wrap(tokens)),
        Some(_) => Ok(wrap(&by_text(Some("call: no run holds it"), &assoc.text))),
        None => Ok(wrap(&by_text(Some("call: no piece holds it"), &assoc.text))),
      }
    },
    None => Ok(wrap(&by_text(Some("no source"), &assoc.text))),
  }
}

/// A call's content as a stamped rehearsal reads it ([`rehearsed`]): its events, formulas, environments and where its inks
/// printing an `@` print.
type CallRead = (Vec<Event>, Vec<Formula>, Vec<EnvironmentStep>, InkTable);

/// A creator's role, Perl's `role=` key: its creator call's in the block that printed it (PoS's `\speaker`, 0910.5686,
/// 1007.2355, 1111.2428, 1111.6944), an API call's own, else `author`.
fn creator_role(c: &RCreator) -> String {
  c.role
    .clone()
    .or_else(|| {
      api_call(c.source)
        .filter(|call| !call.role.is_empty())
        .map(|call| call.role)
    })
    .unwrap_or_else(|| "author".to_string())
}

/// A creator's keys on Perl's call, `\lx@add@creator[#1]`'s `[role=author,#1]` (Base_Utility.pool.ltxml:586), with the
/// reading's `labels`: the keyvals of the creator call in its block that printed it as that call gave them, tokens (PoS's
/// `\speaker`'s `role=speaker`, ams `\contrib`'s `name=`), the labels joining the call's own annotations; else its role.
fn creator_keys(c: &RCreator, labels: &[String]) -> Vec<Token> {
  let labels = labels.join(",");
  let Some(call) = creator_call_of(c) else {
    return internal(&s!("role={},annotations={{{labels}}}", creator_role(c)));
  };
  let mut keys = internal("role=author,");
  keys.extend(call.keys.unlist());
  keys.extend(internal(",annotations="));
  keys.push(T_BEGIN!());
  if let Some(own) = call.annotations {
    keys.extend(own.unlist());
    if !labels.is_empty() {
      keys.push(T_OTHER!(","));
    }
  }
  keys.extend(internal(&labels));
  keys.push(T_END!());
  keys
}

/// The creator call in its block that printed `c`'s name, its keyvals (`RCreator::call`, `field_notes::FieldCreator`).
fn creator_call_of(c: &RCreator) -> Option<FieldCreator> {
  c.call.and_then(|k| {
    with_ledger(|ledger| match ledger.entries().get(c.source) {
      Some(Entry::Author(block)) => match &block.outcome {
        Outcome::Events { creators, .. } => creators.get(usize::from(k)).cloned(),
        _ => None,
      },
      _ => None,
    })
  })
}

/// A call's content's events, formulas and environments, from a stamped rehearsal of it, as `plan::cut` reads a piece (its
/// positions the call's own, from 0: the author field's 15 bits).
pub(super) fn rehearsed(content: &[Token]) -> Result<Option<CallRead>> {
  if content.len() > usize::from(AUTHOR_PROV_POS_MASK) {
    return Ok(None);
  }
  let stamped = field_stamped(2, 0, content);
  Ok(
    match rehearse(stamped, TitleMarks::Fnsymbol, &[], lex_with_inks)? {
      Rehearsal::Done {
        value: (events, formulas, inks),
        environments,
        ..
      } => Some((events, formulas, environments, inks)),
      Rehearsal::Skipped(_) => None,
    },
  )
}

/// The tokens of the contact printing `assoc` in `call`'s content, or with `within` its first's (one printing a list), written down to
/// its address ([`one_address`]).
fn call_contact(call: &ApiCall, assoc: &Assoc, within: bool) -> Result<Option<Tokens>> {
  let key = plan::contact_key(assoc).1;
  let Some((events, formulas, environments, _)) = rehearsed(call.content.unlist_ref())? else {
    return Ok(None);
  };
  let field = Field {
    tokens:       call.content.unlist_ref(),
    base:         0,
    formulas:     &formulas,
    environments: &environments,
  };
  let contact = events.iter().find_map(|e| {
    let (start, end) = e.span?;
    let (start, end) = (usize::from(start), usize::from(end));
    (e.kind == EventKind::Contact && (within || norm(&e.text) == key))
      .then(|| field.content_of((start, end), 0, start))
      .flatten()
      .map(|(tokens, _)| tokens)
  });
  match contact {
    Some(tokens) if within => one_address(&tokens, &assoc.text).map(Some),
    contact => Ok(contact),
  }
}

/// The argument of the note command printing `assoc` in `call`'s content (the one by its text, else its only note), and
/// whether it is a `\\footnote`'s, as [`note_argument`] reads a block's note.
fn call_note(call: &ApiCall, assoc: &Assoc) -> Result<Option<(bool, Vec<Token>)>> {
  let Some((events, formulas, environments, _)) = rehearsed(call.content.unlist_ref())? else {
    return Ok(None);
  };
  let field = Field {
    tokens:       call.content.unlist_ref(),
    base:         0,
    formulas:     &formulas,
    environments: &environments,
  };
  let notes: Vec<&Event> = events
    .iter()
    .filter(|e| e.kind == EventKind::Note)
    .collect();
  let key = norm(&assoc.text);
  let Some(note) = notes
    .iter()
    .find(|e| norm(&e.text) == key)
    .or(notes.first().filter(|_| notes.len() == 1))
  else {
    return Ok(None);
  };
  let Some((start, end)) = note.span else {
    return Ok(None);
  };
  let (start, end) = (usize::from(start), usize::from(end));
  Ok(
    field
      .content_of((start, end), 0, start)
      .and_then(|(tokens, _)| note_argument(&tokens))
      .map(|argument| (note.footnote, argument)),
  )
}

/// A call's piece ([`call_pieces`]): its text, and its tokens, `None` where no balanced run holds it.
type Piece = (String, Option<Vec<Token>>);

/// `call`'s content cut into the reading's entries (`split_pieces`), one each, its text and what it prints but its marks and its
/// contacts, cut at their spans, in the font it prints in: an entry whose contact prints through no API call (0911.3853's `\\email`
/// in an `\\institute`, 1001.0337). A container (a Layout event) and what it or a formula holds are the piece's, whole: no cut at
/// one, nor inside one (`nest`, `Field::formulas`): `$^{1}$Univ X \\begin{tabular}{c}a\\\\ b\\end{tabular}`, `\\begin{ee}a b\\end{ee}`,
/// `$\\begin{array}{c}a\\end{array}$` (review revZ4a).
fn call_pieces(call: &ApiCall) -> Result<Vec<Piece>> {
  let Some((events, formulas, environments, _)) = rehearsed(call.content.unlist_ref())? else {
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
    let start = inks
      .first()
      .and_then(|first| first.formula.and_then(|f| f.span).or(first.span));
    let (Some((start, _)), Some((_, end))) = (start, inks.last().and_then(|last| last.span)) else {
      out.push((norm(&text), None));
      continue;
    };
    let cuts = piece
      .iter()
      .filter(|e| {
        !(matches!(
          e.kind,
          EventKind::Name | EventKind::Sep(_) | EventKind::Layout
        ) || (shown && e.kind == EventKind::Mark)
          || e.nest > 0)
      })
      .filter_map(|e| e.span)
      .filter(|&(s, t)| {
        let (s, t) = (usize::from(s), usize::from(t));
        field.formula_start(s) == s && field.formula_end(t) == t
      })
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
    let font = inks.first().and_then(|first| first.font.clone());
    out.push((
      norm(&text),
      tokens.map(|tokens| in_its_font(font, tokens).unlist()),
    ));
  }
  Ok(out)
}

/// The declarations of the blocks not yet emitted: an entry of their tokens, digested in the flush's group.
fn declarations<'a>(sources: impl Iterator<Item = &'a usize>) -> Vec<Token> {
  with_ledger(|ledger| {
    let mut out = Vec::new();
    for &index in sources {
      if let Some(Entry::Author(block)) = ledger.entries().get(index)
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
    let notes = ledger.entries().iter().filter_map(|entry| match entry {
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
  // (a note's link its owner took (`ruling4:inner-contact`, `Resolver::note`) while a note written here still prints it —
  // its whole argument, the address in it — is that note's text, not a contact too: one record per fact, as the switch
  // off holds it (its writer declines the contact, today's note kept). `\thanks{Corresponding author:
  // \href{mailto:a@x}{a@x}}` (1410.4175, 2009.03378, 2604.08687, 2410.14846), aa's `\thanks{… (\email{\tt a@x}).}`
  // (0807.0895, the paper's own class), an ORCID link (2511.20681, 2609.20386), brief FX3. A binding's e-mail call — the
  // class's in place too — makes the contact and the note written prints it nowhere: that contact stays (siamart's
  // `\thanks{…, \email{wning@vt.edu}.}`, 2511.14713; llncs's `\thanks{Corresponding author: \email{…}}`, 2406.13066).
  // Only a note on the contact's own owner ties the address to it already: a note on another creator printing it leaves
  // the contact the only tie, kept (iclr's `Antonis Antoniades$^{1*}$, Albert Örwall$^{2}$\thanks{… Correspondence to:
  // \href{mailto:antonis@ucsb.edu}{…}, …}`, 2410.20285; brief FX3b, FIDELITY_CHARTER §4 "the author it spells")
  let notes: Vec<(Owner, usize, (u16, u16))> = held
    .iter()
    .filter(|(_, key, _)| key.0 == Kind::Note)
    .filter_map(|(owner, _, note)| match note.source {
      Some((index, Some(span))) => Some((*owner, index, span)),
      _ => None,
    })
    .collect();
  let in_a_written_note = |owner: Owner, a: &Assoc| {
    let Some((index, Some(span))) = a.source else {
      return false;
    };
    let routed = event_at(
      index,
      span,
      |e| e.kind == EventKind::Contact,
      |e| e.role.as_deref() == Some("email") && e.meaning != events::EmailMeaning::Own,
    );
    a.rule.contains("ruling4:inner-contact")
      && routed == Some(false)
      && notes
        .iter()
        .any(|&(o, i, (s, t))| o == owner && i == index && s <= span.0 && span.1 <= t)
  };
  let prints_itself = |index: usize| {
    api_call(index).is_some_and(|call| {
      let own = |e: &Event| e.kind == EventKind::Contact && e.meaning == events::EmailMeaning::Own;
      call.events.iter().any(own)
    })
  };
  // (an entry an earlier item of its owner already writes, the same text from another source, is not written — the loop below
  // writes each owner's text once —, so the contact it holds is its own item, not left to it: svjour3's `\institute{Sibo
  // Cheng \at Data Science …\\ \email{sibo.cheng@…}}` under an author line printing the same affiliation, 2111.06447, brief FX5)
  let written_again = |at: usize| {
    let (owner, key, a) = &held[at];
    held[..at]
      .iter()
      .any(|(o, k, other)| o == owner && k == key && other.source != a.source)
  };
  // (a call written by its pieces — each line cut at its own mark, `item_call` writing the whole call only when it holds
  // fewer than two — prints its e-mails in none of them, so each is its own item: ws-p8's `\address{{}$^a$Dept …\\E-mail:
  // maarten@…\\{}$^b$IFAE …\\E-mail: peris@…}` whose names' marks the queue did not cut it by (`$^{\a 1}$` printing
  // "a1"), hep-ph0011101, brief KB1)
  let by_pieces = |index: usize| {
    api_call(index).is_some_and(|call| split_pieces(&call.events, &call.role).len() >= 2)
  };
  let separate: BTreeSet<usize> = held
    .iter()
    .filter(|(owner, _, a)| {
      let entry_of = |(at, (o, key, other)): (usize, &(Owner, Key, &Assoc))| {
        o == owner && key.0 == Kind::Affiliation && other.source == a.source && !written_again(at)
      };
      inner_contact(a)
        && (!held.iter().enumerate().any(entry_of)
          || a
            .source
            .is_some_and(|(index, _)| prints_itself(index) || by_pieces(index)))
    })
    .filter_map(|(_, _, a)| a.source.map(|(index, _)| index))
    .collect();
  let mut items: Vec<Item> = Vec::new();
  let mut at: BTreeMap<(Key, Option<Source>), usize> = BTreeMap::new();
  for (owner, key, assoc) in held {
    let source = assoc.source;
    let of_footnote = source.is_some_and(|(index, _)| {
      with_ledger(|ledger| matches!(ledger.entries().get(index), Some(Entry::NoteText(_))))
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
    if in_a_written_note(owner, assoc) {
      continue;
    }
    let id = match owner {
      Owner::Creator(i) => owners[i].as_str(),
      Owner::Block => "block",
    };
    let id = s!("{id}|{:?}|{}", key.0, key.1);
    // (a call a flush took from the queue and digests as it stands is off the queue: [`take_digested`])
    let taken = source
      .is_some_and(|(index, _)| lookup_mapping(EMITTED_ITEMS, &s!("taken|{index}")).is_some());
    if taken || lookup_mapping(EMITTED_ITEMS, &id).is_some() {
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
        matches!(ledger.entries().get(index), Some(Entry::Api(call)) if call.attrs.contains("=footer"))
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

/// A call's word check ([`call_checks`]): the words it prints that the reading writes nowhere, and those it writes
/// more often than they print, each with whether its names alone do.
type CallCheck = (Vec<String>, Vec<(String, bool)>);

/// The call half of the reading's word check (`resolver::words`, whose block half runs before the emit decides what it writes
/// whole), both ways, against `written`: per API call (its ledger index), the words a creator's call prints that no word the
/// reading writes holds (which the emit counts but where it keeps the call or writes it whole, the call writing its own: a
/// note an entry's call holds, hep-ex0609057); and the words the reading writes of any call more often than it prints them
/// (2607.00684's llncs `\\institute{…\\email{…, \\{changhwan.sung, mtablo, …\\}@postech.ac.kr}}`, review revZ4br2).
fn call_checks(ledger: &Ledger, written: &Written) -> BTreeMap<usize, CallCheck> {
  let calls = ledger
    .entries()
    .iter()
    .enumerate()
    .filter_map(|(index, entry)| match entry {
      Entry::Api(call) if !call.cleared => {
        let words: Vec<String> = if creator_call(&call.tag, &call.parent) {
          let lost = written.lost_inks(index, &call.events, false).into_iter();
          lost.flat_map(|ink| ink.words).collect()
        } else {
          Vec::new()
        };
        let excess = written.excess(index, &call.events);
        (!words.is_empty() || !excess.is_empty()).then_some((index, (words, excess)))
      },
      _ => None,
    });
  calls.collect()
}

/// The reading of the ledger at a flush (`reading`, as resolved; `resolved`, an entry's addresses given back) and its
/// checks: the word check's block half ([`BlockCheck`]) and call half ([`call_checks`]), from one [`Written`]; the marks
/// and the separators (`resolver::net`). Made at the ledger's `generation` ([`Ledger::generation`]), kept on the ledger while
/// it reads the same ([`Ledger::analysis`]).
#[derive(Debug)]
pub(in crate::base_utilities) struct Analysis {
  generation: u64,
  reading:    Resolved,
  resolved:   Resolved,
  checks:     BTreeMap<usize, BlockCheck>,
  calls:      BTreeMap<usize, CallCheck>,
  marks:      BTreeMap<usize, Vec<String>>,
  merged:     BTreeMap<usize, Vec<String>>,
  /// the entries the reading took addresses out of that are written cut ([`entry_cuts`]); the others given back whole
  cuts:       Vec<EntryCut>,
}

/// The ledger's [`Analysis`], made once while its entries read the same: a later flush with nothing new to read reads it as
/// the last did (1901.01540's 1,202 authors, no `\maketitle`, flushed at each section: resolved at every flush, +33 % of the
/// run's instructions).
fn analysis() -> Rc<Analysis> {
  let kept = with_ledger(|ledger| {
    let generation = ledger.generation();
    ledger
      .analysis
      .clone()
      .filter(|made| made.generation == generation)
  });
  if let Some(made) = kept {
    return made;
  }
  let (generation, inputs) = with_ledger(|ledger| (ledger.generation(), ledger_inputs(ledger)));
  let reading = resolve(&inputs);
  // (an entry's addresses, taken out, are written cut from its tokens, each address its owner's e-mail, as the switch-off writer
  // cuts today's entry (`address_cut`: 2011.02560, astro-ph0610446, 2406.02913); an entry its tokens can't be cut so is given
  // back whole, the Kind steps' to cut in place after the flush, as for a paper with no block, `insert_frontmatter` — entry by
  // entry, not the switch off's whole paper; the checks read the reading and its drops, `Resolved::drops`, 1610.05419)
  let mut resolved = reading.clone();
  let cuts = entry_cuts(&reading, &mut resolved);
  with_ledger(|ledger| {
    let written = Written::new(ledger, &reading);
    let made = Rc::new(Analysis {
      generation,
      checks: check_blocks(ledger, &written),
      calls: call_checks(ledger, &written),
      marks: uncoordinated_marks(ledger, &reading),
      merged: merged_names(ledger, &reading),
      cuts,
      reading,
      resolved,
    });
    ledger.analysis = Some(Rc::clone(&made));
    made
  })
}

/// The words the reading drops by its own rule ([`Resolved::drops`]), as the ledger's last [`Analysis`] read them:
/// what the content check counts as separators' words (`placed::content_check`).
pub(super) fn reading_drops() -> BTreeSet<RuleDrop> {
  with_ledger(|ledger| {
    ledger
      .analysis
      .as_ref()
      .map(|made| made.reading.drops.iter().cloned().collect())
      .unwrap_or_default()
  })
}

/// Whether the author block at ledger index `index` is written whole, as Perl's `\\lx@add@author` writes a block its
/// `\\lx@add@authors` can't cut (Base_Utility.pool.ltxml:692-694, its `{tabular}` fallback): where its reading would lose a
/// name's words (`lost`: an ink failing the names, `LostInk::names_fail`). Decided once, at the flush that first reads it, which a
/// later one keeps (its creators are emitted).
fn written_whole(index: usize, lost: bool) -> bool {
  let once = s!("whole|{index}");
  match lookup_mapping(EMITTED_ITEMS, &once) {
    Some(Stored::Bool(whole)) => whole,
    _ => {
      assign_mapping(EMITTED_ITEMS, &once, Some(Stored::Bool(lost)));
      lost
    },
  }
}

/// Whether the author block at ledger index `index` was written whole ([`written_whole`]), as a flush decided.
pub(super) fn kept_whole(index: usize) -> bool {
  matches!(
    lookup_mapping(EMITTED_ITEMS, &s!("whole|{index}")),
    Some(Stored::Bool(true))
  )
}

/// The creator Perl's `\\lx@add@author` makes of the author block at ledger index `index`, with its keys and `keys`
/// (`author_block::whole_block_author`, the one writer of a block kept whole), its tabular juggles and columns the line
/// breaks they print (`tabular_juggles_in_a_whole_block`: 1511.05911's vldb.cls `\\alignauthor`).
fn whole_block(index: usize, keys: &str) -> Option<Result<Tokens>> {
  let (keyvals, written) = with_ledger(|ledger| match ledger.entries().get(index) {
    Some(Entry::Author(block)) => Some((block.keyvals.clone(), block.tokens.clone())),
    _ => None,
  })?;
  let keys: Vec<&str> = keyvals.iter().map(String::as_str).chain([keys]).collect();
  let keys = keys.into_iter().filter(|keys| !keys.trim().is_empty());
  let keys = internal(&keys.collect::<Vec<_>>().join(","));
  Some(whole_block_author(
    Some(Tokens::new(keys)),
    tabular_juggles_in_a_whole_block(written),
  ))
}

/// What the emit does with the words a block or a call prints that it doesn't write ([`warn_unwritten`]).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Unwritten {
  /// the block is written whole, Perl's fallback
  Whole,
  /// the reading kept, each ink a note of the author block
  Kept,
  Lost,
}

/// The counted Warning of the words a block or a call prints that the emit doesn't write, once per source over the flushes
/// (its `lost|<index>`, which a call's writing sets too), saying what it does instead. Whether it's the first.
fn warn_unwritten(index: usize, words: &[String], then: Unwritten) -> bool {
  let once = s!("lost|{index}");
  if lookup_mapping(EMITTED_ITEMS, &once).is_some() {
    return false;
  }
  assign_mapping(EMITTED_ITEMS, &once, Some(Stored::Bool(true)));
  let then = match then {
    Unwritten::Whole => "the block is written whole, as Perl's whole-block creator",
    Unwritten::Kept => "they are kept as a note of the author block",
    Unwritten::Lost => "they are lost",
  };
  Warn!(
    "unexpected",
    "frontmatter",
    s!(
      "The frontmatter emitter's reading holds no place for words its author block or call prints ({}); {then}",
      words.join(" ")
    )
  );
  true
}

/// The counted Warning of the words the reading writes of a block more times than it prints them ([`Written::excess`]), once
/// per block over the flushes. An entry's excess is counted, not answered: whether a note's digestion keeps an address its
/// contact writes too is digestion's (an.cls's raw `\\email` in a `\\thanks` keeps it, 0807.0895; siamart's routes it out).
fn warn_excess(index: usize, words: &[(String, bool)], whole: bool) {
  let once = s!("excess|{index}");
  if lookup_mapping(EMITTED_ITEMS, &once).is_some() {
    return;
  }
  assign_mapping(EMITTED_ITEMS, &once, Some(Stored::Bool(true)));
  let then = if whole {
    "the block is written whole, as Perl's whole-block creator"
  } else {
    "they are counted"
  };
  let words: Vec<&str> = words.iter().map(|(word, _)| word.as_str()).collect();
  let what = with_ledger(|ledger| match ledger.entries().get(index) {
    Some(Entry::Api(_)) => "call",
    _ => "author block",
  });
  Warn!(
    "unexpected",
    "frontmatter",
    s!(
      "The frontmatter emitter's reading writes words of its {what} more often than it prints them ({}); {then}",
      words.join(" ")
    )
  );
}

/// The counted Warning of a check of the safety net (`resolver::net`) its reading fails, once per `key` over the flushes.
fn warn_net(key: &str, what: &str) {
  if lookup_mapping(EMITTED_ITEMS, key).is_some() {
    return;
  }
  assign_mapping(EMITTED_ITEMS, key, Some(Stored::Bool(true)));
  Warn!(
    "unexpected",
    "frontmatter",
    s!("The frontmatter safety net: {what}; they are counted")
  );
}

/// The flush's added step: `commands`' creators and their annotations replaced by calls built from the reading. Whether
/// it did.
pub(in crate::base_utilities) fn emit_commands(commands: &mut Vec<RawFrontmatter>) -> Result<bool> {
  if fm_emit_mode() == 0 || rehearsal::rehearsing() {
    return Ok(false);
  }
  if let Some(why) = stays_out() {
    DebugFeature!("frontmatter", "FRONT emit stays out: {why}");
    // (with no reader cutting at the call, a block no rehearsal read is Perl's whole-block creator, its queued entry: 2204.09614,
    // a block past the positions)
    if fm_emit_mode() == 2 && why != NO_BLOCK {
      if lookup_mapping(EMITTED_ITEMS, "unread").is_none() {
        assign_mapping(EMITTED_ITEMS, "unread", Some(Stored::Bool(true)));
        Warn!(
          "unexpected",
          "frontmatter",
          s!(
            "The frontmatter emitter can't read an author block ({why}): it is written whole, as Perl's whole-block creator"
          )
        );
      }
      // (each block's whole entry kept whole on purpose, its merge counted by that Warning, `KEPT_WHOLE`: the ledger's copy of
      // the queued call names its block, `whole_block` writes it so)
      let whole_entries: Vec<(Tokens, usize)> = with_ledger(|ledger| {
        let calls = ledger.entries().iter().filter_map(|entry| match entry {
          Entry::Api(call) if call.inside && call.tag == "ltx:creator" => {
            Some((call.command.clone()?, call.after_block?))
          },
          _ => None,
        });
        calls.collect()
      });
      for entry in commands.iter_mut() {
        if let Some(&(_, block)) = whole_entries
          .iter()
          .find(|(command, _)| *command == entry.2)
          && let Some(tokens) = whole_block(block, KEPT_WHOLE)
        {
          entry.2 = tokens?;
        }
      }
    }
    return Ok(false);
  }
  let analysis = analysis();
  // (a flush with nothing queued and nothing new read writes what this conversion's last emit wrote of the same reading:
  // nothing)
  let generation = i64::try_from(analysis.generation).unwrap_or(i64::MAX);
  if commands.is_empty() && matches!(lookup_value(EMITTED), Some(Stored::Int(g)) if g == generation)
  {
    return Ok(true);
  }
  let resolved = &analysis.resolved;
  if resolved.creators.is_empty() && resolved.block.is_empty() {
    DebugFeature!("frontmatter", "FRONT emit stays out: no reading");
    return Ok(false);
  }
  // (a call printing no word of its own stays as it is, the note it holds the document's: `\\IEEEauthorblockA{\\thanks{…}}`,
  // 1502.03038, 2005.06428, 2110.05178)
  let own_words = |call: &ApiCall| {
    let words: Vec<&Event> = call
      .events
      .iter()
      .filter(|e| e.kind != EventKind::Note)
      .collect();
    norm(&events::events_text(&words))
  };
  let (wordless, wordless_calls): (BTreeSet<usize>, Vec<Tokens>) = with_ledger(|ledger| {
    let calls = ledger.entries().iter().filter_map(|entry| match entry {
      Entry::Api(call)
        if call.events.iter().any(|e| e.kind == EventKind::Note) && own_words(call).is_empty() =>
      {
        Some((call.index, call.command.clone()?))
      },
      _ => None,
    });
    calls.unzip()
  });
  // (the reading's word check, its block half: a block whose reading loses a name's words is written whole, Perl's fallback; one
  // losing other ink keeps its reading, the ink kept as a note of the author block, as the unclaimed-text net keeps a line no
  // field holds (`insert::keep_unclaimed_net_notes`): 2605.16481, 2607.28418; each counted)
  let checks = &analysis.checks;
  let read_blocks: BTreeSet<usize> = resolved
    .creators
    .iter()
    .map(|c| c.source)
    .filter(|&source| {
      with_ledger(|ledger| matches!(ledger.entries().get(source), Some(Entry::Author(_))))
    })
    .collect();
  // (the names themselves fail: an ink among them lost, or a word of a name written more often than it prints)
  let names_fail = |source: usize| {
    checks.get(&source).is_some_and(|check| {
      for ink in check.lost.iter().filter(|ink| ink.names_fail) {
        DebugFeature!(
          "frontmatter",
          "FRONT net: a name lost at {source}: {}",
          ink.text
        );
      }

      check.lost.iter().any(|ink| ink.names_fail) || check.excess.iter().any(|(_, name)| *name)
    })
  };
  let whole: BTreeSet<usize> = read_blocks
    .into_iter()
    .filter(|&source| written_whole(source, names_fail(source)))
    .collect();
  let mut kept_inks: Vec<(usize, &LostInk)> = Vec::new();
  for (&index, check) in checks {
    let whole = whole.contains(&index);
    let words: Vec<String> = check
      .lost
      .iter()
      .flat_map(|ink| ink.words.clone())
      .collect();
    let then = if whole {
      Unwritten::Whole
    } else {
      Unwritten::Kept
    };
    if !words.is_empty() && warn_unwritten(index, &words, then) && !whole {
      kept_inks.extend(check.lost.iter().map(|ink| (index, ink)));
    }
    if !check.excess.is_empty() {
      warn_excess(index, &check.excess, whole);
    }
  }
  // (the coordination and separator checks, counted: `resolver::net`)
  for (index, labels) in analysis
    .marks
    .iter()
    .filter(|(index, _)| !whole.contains(index))
  {
    warn_net(
      &s!("marks|{index}"),
      &s!(
        "a mark its author block prints on a name reaches no contact carrying its label, or a labelled contact no \
         name's mark reaches ({})",
        labels.join(" ")
      ),
    );
  }
  for (index, pairs) in analysis
    .merged
    .iter()
    .filter(|(index, _)| !whole.contains(index))
  {
    warn_net(
      &s!("merged|{index}"),
      &s!(
        "a separator its author block prints between names parts no creators ({})",
        pairs.join("; ")
      ),
    );
  }
  // (the queue: what the emitter takes, and what it leaves, in place; a taken call no recorded call is — a stand-in
  // binding's copy's, `ledger::records_now` — kept for the content check, `Ledger::unrecorded`)
  let (mut before, mut after, mut unprinted): (Vec<_>, Vec<_>, Vec<RawFrontmatter>) =
    Default::default();
  let mut seen = false;
  // (the recorded calls' commands by a fingerprint as their equality reads them, a space by its catcode)
  let fingerprint = |command: &Tokens| {
    command.unlist_ref().iter().fold(0u64, |h, token| {
      (h.rotate_left(5) ^ token.cycle_fingerprint()).wrapping_mul(0x9E37_79B9_7F4A_7C15)
    })
  };
  let mut recorded: Option<BTreeMap<u64, Vec<Tokens>>> = None;
  let mut unrecorded = Vec::new();
  for entry in std::mem::take(commands) {
    let parent = annotated_parent(&entry.2);
    if entry.0 == "ltx:rdf" && parent == "ltx:creator" {
      unprinted.push(entry);
    } else if creator_call(&entry.0, &parent) && !wordless_calls.contains(&entry.2) {
      seen = true;
      let recorded = recorded.get_or_insert_with(|| {
        with_ledger(|ledger| {
          let mut recorded: BTreeMap<u64, Vec<Tokens>> = BTreeMap::new();
          for entry in ledger.entries() {
            if let Entry::Api(call) = entry
              && let Some(command) = &call.command
            {
              recorded
                .entry(fingerprint(command))
                .or_default()
                .push(command.clone());
            }
          }
          recorded
        })
      });
      if !recorded
        .get(&fingerprint(&entry.2))
        .is_some_and(|commands| commands.contains(&entry.2))
      {
        unrecorded.push(content_text(&queued_affiliation_text(&last_group(
          &entry.2,
        ))));
      }
    } else if seen {
      after.push(entry);
    } else {
      before.push(entry);
    }
  }
  if !unrecorded.is_empty() {
    with_ledger(|ledger| ledger.unrecorded.extend(unrecorded));
  }
  // (the creators: each its Perl label, `author:N`, new or emitted at an earlier flush: as `num_<tag>` carries a flush's counts.
  // A block written whole is one creator, its first name's, which its other names' items go to)
  let mut owners: Vec<String> = Vec::with_capacity(resolved.creators.len());
  let mut fresh: Vec<usize> = Vec::new();
  let mut role_counts: BTreeMap<String, i64> = BTreeMap::new();
  let mut first_of: BTreeMap<usize, usize> = BTreeMap::new();
  let mut rep: Vec<usize> = (0..resolved.creators.len()).collect();
  for (i, c) in resolved.creators.iter().enumerate() {
    if whole.contains(&c.source) {
      if let Some(&first) = first_of.get(&c.source) {
        rep[i] = first;
        owners.push(owners[first].clone());
        continue;
      }
      first_of.insert(c.source, i);
    }
    let id = if whole.contains(&c.source) {
      s!("{}#whole", c.source)
    } else {
      s!("{}#{}#{i}", c.source, norm(&c.name))
    };
    owners.push(match lookup_mapping(EMITTED_CREATORS, &id) {
      Some(owner) => owner.to_string(),
      None => {
        let role = creator_role(c);
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
    !whole.contains(source)
      && with_ledger(|ledger| matches!(ledger.entries().get(*source), Some(Entry::Author(_))))
  });
  let keeps_marks: BTreeMap<usize, bool> = by_block
    .keys()
    .map(|&source| (source, per_author_marks(source)))
    .collect();
  let keeps = |source: usize| keeps_marks.get(&source).copied().unwrap_or(false);
  let mut names: BTreeMap<usize, (Option<Vec<Token>>, Vec<Token>)> = BTreeMap::new();
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
  let (mut items, separate) = items_to_emit(resolved, &owners, &on_names, &mut kept);
  // (a block written whole prints its own items; its names' others are its creator's)
  items.retain(|item| !item.assoc.source.is_some_and(|(at, _)| whole.contains(&at)));
  for item in &mut items {
    let mut seen_owners = BTreeSet::new();
    item.owners = item
      .owners
      .iter()
      .map(|owner| match owner {
        Owner::Creator(i) => Owner::Creator(rep[*i]),
        Owner::Block => Owner::Block,
      })
      .filter(|owner| seen_owners.insert(*owner))
      .collect();
  }
  in_perl_order(&mut items);
  // (the labels: an item's own on every new owner; an owner made at an earlier flush holds its own copy under its Perl
  // `author:N`, its annotations long digested)
  // (by creator index, read once per owner of every item: no set lookup, no list per item)
  let mut is_fresh = vec![false; resolved.creators.len()];
  for &i in &fresh {
    is_fresh[i] = true;
  }
  let fresh_owner = |owner: &Owner| matches!(owner, Owner::Creator(i) if is_fresh[*i]);
  let mut annotations: Vec<Vec<String>> = vec![Vec::new(); resolved.creators.len()];
  let mut calls: Vec<(String, &Item)> = Vec::new();
  let mut block_pubnotes: Vec<Vec<Token>> = Vec::new();
  for (n, item) in items.iter().enumerate() {
    if let Some(note) = block_pubnote(item) {
      block_pubnotes.push(note);
      continue;
    }
    if item.owners.contains(&Owner::Block) {
      calls.push(("addresses:block".to_string(), item));
    }
    for owner in &item.owners {
      if let Owner::Creator(i) = owner
        && !is_fresh[*i]
      {
        calls.push((owners[*i].clone(), item));
      }
    }
    if item.owners.iter().any(fresh_owner) {
      let label = s!("item:{}:{n}", lookup_int(FRONTMATTER_FLUSHES));
      for owner in item.owners.iter().filter(|owner| fresh_owner(owner)) {
        if let Owner::Creator(i) = owner {
          annotations[*i].push(label.clone());
        }
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
  for &i in fresh
    .iter()
    .filter(|&&i| !whole.contains(&resolved.creators[i].source))
  {
    let c = &resolved.creators[i];
    let keeps = keeps(c.source);
    let inst_marks = inst_marks.get(&c.source).map_or(&[][..], Vec::as_slice);
    // (a creator call's own labels, which its keys write, print nothing: `creator_keys`)
    let own = creator_call_of(c)
      .map(|call| call.labels())
      .unwrap_or_default();
    for request in c
      .requests
      .iter()
      .filter(|request| (!keeps || inst_marks.contains(request)) && !own.contains(request))
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
    // (kept whole on purpose, as a tabular block is: no merge to flag beside its Warning, `flag_merged_creators`)
    if whole.contains(&c.source)
      && let Some(tokens) = whole_block(c.source, &s!("{KEPT_WHOLE},{keys}"))
    {
      let tokens = tokens?;
      out.push(entry("ltx:creator", tokens.unlist()));
      continue;
    }
    // (the call, where it is this creator alone)
    let sole = per_source.get(&c.source) == Some(&1);
    let call = (!by_block.contains_key(&c.source) && sole)
      .then(|| api_call(c.source).and_then(|call| call.command))
      .flatten()
      .and_then(|command| rekeyed(&command, &s!("{keys},"), false, true, true));
    let tokens = call.unwrap_or_else(|| {
      let (name, cites) = names.remove(&i).unwrap_or_default();
      let name = name.unwrap_or_else(|| {
        let mut name = by_text(
          Some(&s!("a name its events don't spell [{}]", c.rule)),
          &c.name,
        );
        name.extend(cites);
        name
      });
      let content = cs_group("\\lx@frontmatter@item", &[&name]);
      let content = cs_group("\\lx@personname", &[&cs_group("\\lx@author@withinst", &[
        &content,
      ])]);
      let mut head = vec![add_now()];
      head.extend(cs_group_args(&["ltx:creator"]));
      with_key_tokens(head, creator_keys(c, &annotations[i]), &content)
    });
    out.push(entry("ltx:creator", tokens.unlist()));
  }
  // (the block's own creator goes below this flush's names, where an item or a kept ink is the block's: `block_creator`)
  let block_at = out.len();
  let of_the_block = calls.iter().any(|(label, _)| label == "addresses:block")
    || kept_inks.iter().any(|(_, ink)| ink.owner.is_none());
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
  // (the calls written whole, and the ones kept as they are)
  let mut written = wordless.clone();
  let base = lookup_int(QUEUE_ORDINAL_COUNTER);
  for (label, item) in calls {
    while let Some((at, rdf)) =
      unprinted.next_if(|(at, _)| item.assoc.source.is_some_and(|(index, _)| *at <= index))
    {
      out.push(rdf_entry(at, rdf));
    }
    let tokens = item_call(
      &item.key,
      item.assoc,
      &label,
      &separate,
      &mut pieces,
      &mut written,
      &analysis.cuts,
    )?;
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
  // (the block's notes Perl prints as the document's: `block_pubnote`)
  out.extend(
    block_pubnotes
      .into_iter()
      .map(|note| entry("ltx:pubnote", note)),
  );
  // (an ink the reading holds no place for, as a note: of the author whose line it is, by the reading's own rule
  // (`LostInk::owner`: 2605.16481's Aiden Yiliu Li, 2110.03506's Kerianne L. Hobbs), else of the author block. Its tokens where
  // every word of it is lost; else its lost words' own text, its written addresses cut out (`LostInk::note_text`), in its
  // font, as the words written elsewhere stay there (`Correo electrónico: ann@a.org`, review revZ4br2; `POC: K. Hobbs
  // (kerianne.hobbs@us.af.mil)`, review revN1); a note no font holds says its markup is lost)
  for (index, ink) in kept_inks {
    let content = if ink.whole() {
      ink.span.and_then(|span| span_tokens_of(index, span))
    } else {
      None
    };
    let text = ink.note_text().to_string();
    let content = match (content, &ink.font) {
      (Some(tokens), _) => tokens.unlist(),
      (None, Some(font)) => in_its_font(Some(font.clone()), by_text(None, &text)).unlist(),
      (None, None) => by_text(Some("a note of ink the reading holds no place for"), &text),
    };
    let label = ink
      .owner
      .map_or_else(|| "addresses:block".to_string(), |one| owners[one].clone());
    let note = annotate_now(
      "ltx:note",
      &s!("role=note,label={{{label}}}"),
      &item_only(&content),
    );
    out.push(entry("ltx:note", note.unlist()));
  }
  // (a call written, at this flush or an earlier one, is never counted lost at a later one: its fact over the flushes, its
  // `lost|<index>` false, 2303.02975)
  for index in written {
    let once = s!("lost|{index}");
    if lookup_mapping(EMITTED_ITEMS, &once).is_none() {
      assign_mapping(EMITTED_ITEMS, &once, Some(Stored::Bool(false)));
    }
  }
  for (&index, (words, excess)) in &analysis.calls {
    if !words.is_empty() {
      warn_unwritten(index, words, Unwritten::Lost);
    }
    if !excess.is_empty() {
      warn_excess(index, excess, false);
    }
  }
  out.extend(kept);
  out.extend(after);
  if let Some(creator) = block_creator(of_the_block) {
    out.insert(block_at, entry("ltx:creator", creator.unlist()));
  }
  // (a document's note's own `\\email` is the reading's item already: `api:note-spelled`, 0910.1640 — but in a note making a
  // creator, which no reading answers, each contact is the newest creator's as the note is digested, as Perl's:
  // 0807.4720's `\\thanks{… \\author{…}\\address{…}\\email{…}}`, `resolver::is_creator_call`)
  let makes_a_creator = |entry: &RawFrontmatter| {
    with_ledger(|ledger| {
      ledger.entries().iter().any(|e| {
        matches!(e, Entry::Api(call) if call.command.as_ref() == Some(&entry.2)
          && call.events.iter().any(is_creator_call))
      })
    })
  };
  for entry in out
    .iter_mut()
    .filter(|entry| entry.0 == "ltx:pubnote" && !makes_a_creator(entry))
  {
    entry.2 = Tokens::new(cs_group("\\lx@frontmatter@itemonly", &[entry
      .2
      .unlist_ref()]));
  }
  *commands = out;
  assign_value(EMITTED, Stored::Int(generation), Some(Scope::Global));
  with_ledger(|ledger| {
    ledger.writer = Some("written".to_string());
    if ledger.compared.as_ref() != Some(&analysis.reading) {
      ledger.compared = Some(analysis.reading.clone());
    }
  });
  Ok(true)
}

/// The queue entries a flush digests as they stand (the emitter's rewrite done), each taken: the ledger call it is marked
/// `taken|<index>` before its digestion, so a later flush's emitter — the nested one of a block the entry's own digestion
/// calls late (`\\lx@frontmatter@late`) — writes none of its items again. memsait's `\\@opening` note, which the safety net
/// queues for a flush with no author block yet and whose digestion runs `\\title` and `\\author` (astro-ph9808358).
pub(in crate::base_utilities) fn take_digested(commands: &[RawFrontmatter]) {
  if fm_emit_mode() != 2 || rehearsal::rehearsing() {
    return;
  }
  let taken: Vec<usize> = with_ledger(|ledger| {
    let calls = ledger.entries().iter().filter_map(|entry| match entry {
      Entry::Api(call)
        if call
          .command
          .as_ref()
          .is_some_and(|c| commands.iter().any(|e| e.2 == *c)) =>
      {
        Some(call.index)
      },
      _ => None,
    });
    calls.collect()
  });
  for index in taken {
    assign_mapping(
      EMITTED_ITEMS,
      &s!("taken|{index}"),
      Some(Stored::Bool(true)),
    );
  }
}

/// The author block's own creator, the first time a flush writes an item of the block (`label=addresses:block`: `of_the_block`):
/// Perl's one nameless creator below the names, holding only the block's contacts (OXIDIZED_DESIGN #159,
/// FIDELITY_CHARTER §4), into which `relocate_annotations`'s orphan procedure shares where it would promote a stub — a
/// block item still a sole author's, a named owner's, or no orphan where an author holds a copy under another label —
/// placing it below the last named author, or removing it where nothing reached it
/// (`\lx@add@frontmatter@now{ltx:creator}[role=author,annotations={addresses:block},_blockcreator=true]{}`; uncounted
/// among the names and no annotation's parent by place, [`BLOCK_CREATOR`]). One for the conversion: a later flush's block items find it — not one
/// `\lx@add@frontmatter@now` turns away as a supplement's ([`frontmatter_entry_sealed`]), which a later flush's block
/// items would not find. Witnesses 1012.4847, 1106.3045, 2402.19043, 2609.15094, 2009.10916.
fn block_creator(of_the_block: bool) -> Option<Tokens> {
  if !of_the_block
    || lookup_value(BLOCK_CREATOR_WRITTEN).is_some()
    || frontmatter_entry_sealed("ltx:creator", "author")
  {
    return None;
  }
  assign_value(
    BLOCK_CREATOR_WRITTEN,
    Stored::Bool(true),
    Some(Scope::Global),
  );
  let mut head = vec![add_now()];
  head.extend(cs_group_args(&["ltx:creator"]));
  Some(with_keys(
    head,
    &s!("role=author,annotations={{addresses:block}},{BLOCK_CREATOR}=true"),
    &[],
  ))
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::base_utilities::frontmatter::observer::{ledger::TitleStamp, reset_fm_shadow};

  /// No reading outlives its conversion: the conversion's reset drops the ledger's reading and starts its count again, so a
  /// later conversion's first entries, counted from 0 as these were, read afresh (review revN1r3 5; the cortex worker resets the
  /// frontmatter per paper, never the engine). The count changes with the entries alone (`Ledger::entries_mut`).
  #[test]
  fn the_reset_drops_the_reading_and_its_count() {
    reset_fm_shadow();
    with_ledger(|ledger| {
      ledger.push(Entry::Title(TitleStamp {
        index:  0,
        length: 1,
        range:  None,
      }));
      assert_eq!(ledger.generation(), 1);
      ledger.analysis = Some(Rc::new(Analysis {
        generation: ledger.generation(),
        reading:    Resolved::default(),
        resolved:   Resolved::default(),
        checks:     BTreeMap::new(),
        calls:      BTreeMap::new(),
        marks:      BTreeMap::new(),
        merged:     BTreeMap::new(),
        cuts:       Vec::new(),
      }));
    });
    reset_fm_shadow();
    with_ledger(|ledger| {
      assert!(ledger.analysis.is_none());
      assert_eq!(ledger.generation(), 0);
    });
  }

  /// A queued call a flush digests as it stands is taken ([`take_digested`]): a later flush's emitter — the nested flush of a
  /// late block its digestion makes — writes none of its items again, and still writes a call no flush took. memsait's
  /// `\\@opening` note the safety net queues, digested at a flush with no author block, its `\\author` late (astro-ph9808358:
  /// 12 creators → 6). A unit guard: the paper's shape needs memsait's own readers of `\\@opening`, which no pdflatex-clean
  /// synthetic input reproduces.
  #[test]
  fn a_digested_queue_entry_is_never_written_again() {
    reset_fm_shadow();
    set_fm_emit_override(Some(2));
    let call = |index: usize, text: &'static str| {
      let command = TokenizeInternal!(text);
      Entry::Api(ApiCall {
        index,
        tag: "ltx:note".to_string(),
        role: "note".to_string(),
        epoch: 1,
        queue_position: index,
        after_block: None,
        inside: false,
        binding: None,
        attrs: "role=note".to_string(),
        parent: "ltx:creator".to_string(),
        content: Tokens::new(Vec::new()),
        events: Vec::new(),
        inside_events: Box::default(),
        unread: None,
        rehearsed: true,
        cleared: false,
        errors: 0,
        command: Some(command),
      })
    };
    let (taken, kept) = (
      call(
        0,
        r"\lx@annotate@frontmatter@now{ltx:creator}{ltx:note}{Taken note}",
      ),
      call(
        1,
        r"\lx@annotate@frontmatter@now{ltx:creator}{ltx:note}{Kept note}",
      ),
    );
    let digested = match &taken {
      Entry::Api(call) => call.command.clone().unwrap(),
      _ => unreachable!(),
    };
    with_ledger(|ledger| {
      ledger.push(taken);
      ledger.push(kept);
    });
    take_digested(&[("ltx:note".to_string(), TagAttrs::default(), digested)]);
    let note = |index: usize, text: &str| Assoc {
      role:   "note".to_string(),
      text:   text.to_string(),
      label:  None,
      rule:   "test".to_string(),
      source: Some((index, None)),
      target: None,
    };
    let resolved = Resolved {
      block: vec![note(0, "Taken note"), note(1, "Kept note")],
      ..Resolved::default()
    };
    let (items, _) = items_to_emit(&resolved, &[], &BTreeSet::new(), &mut Vec::new());
    let texts: Vec<&str> = items.iter().map(|item| item.assoc.text.as_str()).collect();
    set_fm_emit_override(None);
    reset_fm_shadow();
    assert_eq!(texts, ["Kept note"]);
  }
}
