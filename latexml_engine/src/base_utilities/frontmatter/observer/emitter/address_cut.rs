//! An entry the reading took printed addresses out of (`Resolved::taken`, `Resolver::take_out_entry_addresses`), as the
//! emitter writes it under `LATEXML_FM_EMIT=2` (OXIDIZED_DESIGN_DIVERGENCES #980 P1b): the switch-off writer's cut of
//! today's XML (`plan::cut_entry_addresses`, `dom::address_cut`, `Op::CutAddresses`, `Op::Address`), translated onto the
//! entry's tokens. The reading's runs and cut are bytes of its text; the lexer recorded where each character of an ink
//! printing an `@` prints (`events::InkTable`, from its box's stamp), so each byte the cut takes is a position of the field,
//! and the entry is its own tokens without the runs those positions cover (`Field::content_cut`). As the DOM cut: what
//! prints nothing between a separator the cut takes and the ink beside it — a line break, a mark (`$^*\,$a@x`, a whole line
//! with its mark: 2506.02272) — goes with it; a cut that would divide a box or take a note declines; the cut is the entry's
//! own, so it declines entry by entry, never the paper (the switch off's `declined`). Each address is its owner's e-mail,
//! as printed: its own tokens, in their font or wrapper (`\texttt{a@x}`, `{\tt\small a@x; b@x}`), else its text in its
//! ink's font, named by the run's label word (`E-mail: `). Witnesses: ieeeconf's `\thanks{… {\tt\small a@x; b@x}}`
//! (2011.02560, 2104.05078, 2204.02371), svmult's `\institute{… \texttt{a@x}}` (astro-ph0610446, cond-mat0610708),
//! authblk's `\affil[1]{\texttt{\{a,b\}@x}, Univ}` (2406.02913), IEEEtran's `… - Email: a@x / b@x` (1705.06453).

use std::cell::Cell;

use super::{
  content::Field,
  emit::{annotate_now, in_its_font, item_only, last_argument, rehearsed, text_tokens},
  give_back_entry,
  plan::{api_call, spells_one_of},
  *,
};
use crate::base_utilities::frontmatter::{
  classify::names_a_contact,
  observer::{
    events::{EventKind, InkAt, InkTable, TextAt, events_texts_at},
    resolver::{Taken, address_runs},
  },
};

/// One e-mail a cut made: its address, its tokens as printed, the label its run took.
type Mail = (String, Vec<Token>, Option<String>);

/// The cut of one taken entry ([`entry_cut`]).
#[derive(Debug)]
pub(super) struct EntryCut {
  source:       Option<(usize, Option<(u16, u16)>)>,
  pub original: String,
  stripped:     String,
  /// the runs the entry's tokens lose: indices of the block's tokens (a block's entry, a note's), or of the call's content
  pub cuts:     Vec<(usize, usize)>,
  /// an API call's command, its content cut
  pub command:  Option<Tokens>,
  /// each address: its tokens as printed, and the label word its run took
  mails:        Vec<Mail>,
  /// the entry was written cut (a path that writes it otherwise leaves it unset: written by its text, counted)
  pub applied:  Cell<bool>,
}

impl EntryCut {
  /// Whether `assoc` is this cut's entry as the reading keeps it, its addresses out.
  pub(super) fn is_cut_of(&self, assoc: &Assoc) -> bool {
    self.source == assoc.source && self.stripped == assoc.text
  }

  /// Whether `assoc` is this cut's entry as it printed, before the take-out.
  pub(super) fn is_entry(&self, assoc: &Assoc) -> bool {
    self.source == assoc.source && self.original == assoc.text
  }

  /// The e-mail `address` of `assoc`'s source this cut made: its tokens, and the `name=` its label gives (`E-mail: `, as the
  /// switch off's `email_contact` names it).
  pub(super) fn mail(&self, assoc: &Assoc) -> Option<(Vec<Token>, Option<String>)> {
    if self.source != assoc.source {
      return None;
    }
    let (_, tokens, label) = self.mails.iter().find(|(a, ..)| *a == assoc.text)?;
    Some((tokens.clone(), label.as_ref().map(|l| s!("{l} "))))
  }
}

/// The cut of the entry `taken`, `None` where its tokens can't be cut at the positions its address runs print at: given back
/// whole, the Kind steps' after the flush, as before.
fn entry_cut(taken: &Taken, names: &[&str]) -> Result<Option<EntryCut>> {
  let spells = &spells_one_of(names);
  let Some(source) = taken.source else {
    return Ok(None);
  };
  let made = match source.1 {
    Some(span) => with_ledger(|ledger| {
      let Some(Entry::Author(block)) = ledger.entries().get(source.0) else {
        return None;
      };
      let Outcome::Events {
        events,
        formulas,
        environments,
        inks,
        held,
        ..
      } = &block.outcome
      else {
        return None;
      };
      let field = Field {
        tokens: block.tokens.unlist_ref(),
        base: block.range.0,
        formulas,
        environments,
        held,
        aligned: aligned(events),
        groups: Default::default(),
      };
      // (a note's entry is its text; any other the text of the events its span covers, or of those from it on, the lines the
      // reading read on with it: 2308.10621)
      let note = events
        .iter()
        .find(|e| e.kind == EventKind::Note && e.span == Some(span));
      match note {
        Some(note) => {
          let texts = [(note.text.clone(), inks.of(note)?.to_vec())];
          // (the inks its body prints, whose fonts its addresses print in: the note's own is the text's around it)
          let inner: Vec<&Event> = note.inner.iter().collect();
          let around = Around {
            inks:     &inner,
            stops:    events
              .iter()
              .chain(&note.inner)
              .filter(|e| stops(e) && e.span != Some(span))
              .collect(),
            apart:    Vec::new(),
            marks:    note
              .inner
              .iter()
              .filter(|e| e.kind == EventKind::Mark)
              .collect(),
            contacts: note.inner.iter().filter(|e| contact_command(e)).collect(),
          };
          cut_at(&field, &texts, &around, inks, (taken, spells))
        },
        None => {
          let from: Vec<&Event> = events
            .iter()
            .filter(|e| e.span.is_some_and(|s| s.0 >= span.0))
            .collect();
          let within: Vec<&Event> = from
            .iter()
            .copied()
            .filter(|e| e.span.is_some_and(|s| s.1 <= span.1))
            .collect();
          let [text, lined] = events_texts_at(&within, inks);
          let [_, read_on] = events_texts_at(&from, inks);
          let around = Around {
            inks:     &from,
            stops:    events.iter().filter(|e| stops(e)).collect(),
            apart:    Vec::new(),
            marks:    events
              .iter()
              .filter(|e| e.kind == EventKind::Mark)
              .collect(),
            contacts: events.iter().filter(|e| contact_command(e)).collect(),
          };
          cut_at(
            &field,
            &[text, lined, read_on],
            &around,
            inks,
            (taken, spells),
          )
        },
      }
    })
    .map(|(cuts, mails)| (cuts, None, mails)),
    None => api_cut(source.0, (taken, spells))?,
  };
  if made.is_some() {
    DebugFeature!(
      "frontmatter",
      "FRONT emit: an entry cut: {:?} -> {:?}",
      taken.lined,
      taken.stripped
    );
  }
  Ok(made.map(|(cuts, command, mails)| EntryCut {
    source: taken.source,
    original: taken.original.clone(),
    stripped: taken.stripped.clone(),
    cuts,
    command,
    mails,
    applied: Cell::new(false),
  }))
}

/// A cut's runs, its command (an API call's), its e-mails.
type Made = (Vec<(usize, usize)>, Option<Tokens>, Vec<Mail>);

/// An API call's entry: its command's last argument rehearsed with its positions (the call's own, from 0), its text without
/// the contacts the reading reads apart (an `\email`, an ORCID: `Resolver::api`), cut, in the command.
fn api_cut(index: usize, (taken, spells): (&Taken, &dyn Fn(&str) -> bool)) -> Result<Option<Made>> {
  let Some(call) = api_call(index) else {
    return Ok(None);
  };
  let Some(command) = call.command.clone() else {
    return Ok(None);
  };
  let Some((before, content, after)) = last_argument(&command) else {
    return Ok(None);
  };
  let Some((events, formulas, environments, inks)) = rehearsed(&content)? else {
    return Ok(None);
  };
  let field = Field {
    tokens:       &content,
    base:         0,
    formulas:     &formulas,
    environments: &environments,
    held:         &[],
    aligned:      aligned(&events),
    groups:       Default::default(),
  };
  let apart = |e: &Event| {
    e.kind == EventKind::Contact && matches!(e.role.as_deref(), Some("email" | "orcid"))
  };
  let refs: Vec<&Event> = events.iter().filter(|e| !apart(e)).collect();
  let around = Around {
    inks:     &refs,
    stops:    events.iter().filter(|e| stops(e)).collect(),
    apart:    events.iter().filter(|e| apart(e)).collect(),
    marks:    events
      .iter()
      .filter(|e| e.kind == EventKind::Mark)
      .collect(),
    contacts: events.iter().filter(|e| contact_command(e)).collect(),
  };
  let Some((cuts, mails)) = cut_at(
    &field,
    &events_texts_at(&refs, &inks),
    &around,
    &inks,
    (taken, spells),
  ) else {
    return Ok(None);
  };
  let Some((cut, _)) = field.content_cut((0, content.len()), 0, 0, &cuts) else {
    return Ok(None);
  };
  let mut tokens = before;
  tokens.extend(cut.unlist());
  tokens.extend(after);
  Ok(Some((cuts, Some(Tokens::new(tokens)), mails)))
}

/// Whether `e` is a contact a command prints whose call an e-mail's item would mute (an `\\email`, an ORCID), no link: a
/// `\\href{mailto:a@x}{a@x}` (a `url` contact) prints inline, the e-mail's own wrapper (2608.15371).
fn contact_command(e: &Event) -> bool {
  e.kind == EventKind::Contact && e.role.as_deref() != Some("url")
}

/// Whether a cut may not take `e`: a note (the DOM cut's `Seg::Note`), a line's break.
fn stops(e: &Event) -> bool { matches!(e.kind, EventKind::Note | EventKind::Break) }

/// `None`, the cut declined for `why` (`--debug=frontmatter`).
fn declined<T>(why: &str) -> Option<T> {
  DebugFeature!("frontmatter", "FRONT emit: an entry's cut declined: {why}");
  None
}

/// The span a character prints at: its box's cover with its own token.
fn reach(ink: &InkAt) -> (usize, usize) {
  let (s, t) = (usize::from(ink.cover.0), usize::from(ink.cover.1));
  match ink.own.map(usize::from) {
    Some(own) => (s.min(own), t.max(own + 1)),
    None => (s, t),
  }
}

/// Whether a kept character prints inside `range`: its own token, or its whole cover where it has none.
fn prints_in(ink: &InkAt, (s, t): (usize, usize)) -> bool {
  match ink.own.map(usize::from) {
    Some(own) => s <= own && own < t,
    None => s <= usize::from(ink.cover.0) && usize::from(ink.cover.1) <= t,
  }
}

/// The events of an entry's source a cut reads: those printing it (`inks`, for their fonts), those it may not take (`stops`: a
/// note; a line's break, but the one between the lines a cut takes whole), and the contacts the text leaves out, which a cut is
/// parted around (`apart`).
struct Around<'a> {
  inks:     &'a [&'a Event],
  stops:    Vec<&'a Event>,
  apart:    Vec<&'a Event>,
  /// the marks the source prints, which no cut takes past the separator it takes ([`cut_at`])
  marks:    Vec<&'a Event>,
  /// the contacts it prints, whose command no e-mail is written as (its item would mute the call: 1109.3369's
  /// `Email:~$^a$\email{…}`)
  contacts: Vec<&'a Event>,
}

/// The cut of `taken` in the field, where one of `texts` (each with where its characters print) holds its text once: each
/// cut of the reading (`Taken::cuts`, its bytes) the positions its characters print at, grown over what prints nothing between
/// a separator it takes and the ink beside it, parted around a contact read apart, each part a balanced run; and each
/// address's tokens. `None` where none holds it, a cut takes a kept character's token, a note or a line's break, or no
/// balanced run holds it.
fn cut_at(
  field: &Field,
  texts: &[TextAt],
  around: &Around,
  inks: &InkTable,
  (taken, spells): (&Taken, &dyn Fn(&str) -> bool),
) -> Option<CutAt> {
  let lined = taken.lined.as_str();
  let Some(((text, at), map)) = texts.iter().find_map(|t| {
    (t.1.len() == t.0.chars().count())
      .then(|| locate(&t.0, lined))
      .flatten()
      .map(|map| (t, map))
  }) else {
    DebugFeature!(
      "frontmatter",
      "FRONT emit: the reading's {lined:?}, its events' {:?}",
      texts.iter().map(|t| &t.0).collect::<Vec<_>>()
    );
    return declined("no text of its events holds the reading's once");
  };
  // (each character of the entry's text where it prints, and whether the cut takes it; what the text holds besides, kept)
  let chars: Vec<(usize, char)> = lined.char_indices().collect();
  let ink: Vec<Option<InkAt>> = map.iter().map(|j| j.and_then(|j| at[j])).collect();
  let taken_at = |byte: usize| taken.cuts.iter().any(|&(s, t)| s <= byte && byte < t);
  let base = field.base;
  let index = |p: usize| p.checked_sub(base);
  let cut_chars: Vec<usize> = (0..chars.len())
    .filter(|&i| taken_at(chars[i].0))
    .filter_map(|i| map[i])
    .collect();
  let kept: Vec<InkAt> = (0..at.len())
    .filter(|j| !cut_chars.contains(j))
    .filter_map(|j| at[j])
    .collect();
  // (the runs the text prints, its spaces as printed — the switch off's `mailed_runs` on today's layout: each one the cut
  // touches every address of which the reading takes, else no cut; `kwb,flynn,walter.scheirer@nd.edu` is one run of three, the
  // reading's text, spaced after each comma, one address after prose: 1903.10019)
  let letters: Vec<char> = text.chars().collect();
  let mut printed = String::new();
  let mut printed_at: Vec<usize> = Vec::new();
  // (the entry's own stretch of the text: a later entry the lines read on hold is another item)
  let first = map.iter().flatten().min().copied().unwrap_or(0);
  let last = map.iter().flatten().max().copied().unwrap_or(0);
  for (j, &c) in letters.iter().enumerate().take(last + 1).skip(first) {
    let glued = || {
      let (Some(Some(before)), Some(Some(after))) =
        (j.checked_sub(1).map(|b| at[b]), at.get(j + 1))
      else {
        return false;
      };
      reach(&before).1 >= reach(after).0
    };
    if at[j].is_none() && c == ' ' && glued() {
      continue;
    }
    printed_at.extend(std::iter::repeat_n(j, c.len_utf8()));
    printed.push(c);
  }
  let mailed = |a: &String| {
    taken
      .runs
      .iter()
      .flat_map(|run| &run.addresses)
      .any(|m| m.eq_ignore_ascii_case(a))
  };
  for run in address_runs(&printed, spells) {
    let touched = printed_at
      .get(run.start..run.end)
      .is_some_and(|js| js.iter().any(|j| cut_chars.contains(j)));
    if touched && !run.addresses.iter().all(mailed) {
      return declined("a printed run not every address of which the reading takes");
    }
    // (or glued to a list's comma before it, a local part the reading leaves — `kwb,flynn,walter.scheirer@nd.edu`, a dotted
    // part ending no list (`ends_a_listed_run`): the switch off's block reader reads the line as the list and its writer
    // declines the paper, 1903.10019; here the entry alone, given back to the Kind steps. A word that is no local part
    // before it is the entry's, `México, D.F.,octavio@astro.unam.mx`, 1310.7021)
    let local = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || "._-".contains(c);
    if touched
      && let Some(before) = printed[..run.start].strip_suffix(',')
      && before
        .rsplit(|c: char| c.is_whitespace() || c == ',')
        .next()
        .is_some_and(|part| !part.is_empty() && part.chars().all(local))
    {
      return declined("a run glued to a list's comma");
    }
  }
  let span_of = |e: &Event| e.span.map(|(a, b)| (usize::from(a), usize::from(b)));
  // (where a mark prints: its formula, else its span)
  let mark_of = |e: &Event| {
    e.formula
      .and_then(|f| f.span)
      .or(e.span)
      .map(|(a, b)| (usize::from(a), usize::from(b)))
  };
  // (a position something kept prints at, a stop or a mark: what no cut grows over)
  let prints = |p: usize| {
    kept.iter().any(|ink| prints_in(ink, (p, p + 1)))
      || around
        .stops
        .iter()
        .chain(&around.apart)
        .filter_map(|e| span_of(e))
        .chain(around.marks.iter().filter_map(|e| mark_of(e)))
        .any(|(a, b)| a <= p && p < b)
  };
  let mut cuts: Vec<(usize, usize)> = Vec::new();
  for &(start, end) in &taken.cuts {
    let inside: Vec<usize> = (0..chars.len())
      .filter(|&i| start <= chars[i].0 && chars[i].0 < end)
      .collect();
    let (Some(&first), Some(&last)) = (inside.first(), inside.last()) else {
      return declined("a cut of no character");
    };
    let reaches: Vec<(usize, usize)> = inside
      .iter()
      .filter_map(|&i| ink[i].as_ref().map(reach))
      .collect();
    let (Some(mut from), Some(mut to)) = (
      reaches.iter().map(|r| r.0).min(),
      reaches.iter().map(|r| r.1).max(),
    ) else {
      return declined("a cut printing at no position");
    };
    let own = (from, to);
    // (a separator the cut takes at an end takes what prints nothing up to the kept ink there: the line's break, a mark)
    if ink[first].is_none()
      && let Some(before) = (0..first)
        .rev()
        .find_map(|i| ink[i].filter(|_| !taken_at(chars[i].0)))
    {
      from = from.min(reach(&before).1);
    }
    // (but never past a mark: one before the kept ink after it leads that ink, `Univ X, {\tt a@x}, $^1$Dept`, as the Kind
    // step leaves it — a mark leading the address side, `\\$^*\,$a@x`, goes with the cut: 2506.02272; review revP1b S1)
    if ink[last].is_none()
      && let Some(after) =
        (last + 1..chars.len()).find_map(|i| ink[i].filter(|_| !taken_at(chars[i].0)))
    {
      let ink_at = reach(&after).0;
      let mark = around
        .marks
        .iter()
        .filter_map(|e| mark_of(e))
        .map(|(a, _)| a)
        .filter(|&a| a >= to && a < ink_at)
        .min();
      to = to.max(mark.unwrap_or(ink_at));
    }
    // (parted around a contact the text leaves out, which stays where it prints: `(\email{a@x}; b@x)`, 1106.3045)
    let mut parts = vec![(from, to)];
    for (a, b) in around.apart.iter().filter_map(|e| span_of(e)) {
      parts = parts
        .into_iter()
        .flat_map(|(s, t)| {
          if a >= s && b <= t {
            vec![(s, a), (b, t)]
          } else {
            vec![(s, t)]
          }
        })
        .filter(|(s, t)| s < t)
        .collect();
    }
    // (what prints nothing a separator took at an end, as few tokens of it as a balanced run needs: the brace closing the
    // kept ink's group before the break a whole line's cut takes, `\textit{… China}\\ \textit{*Corresponding author: a@x,
    // b@x}`, 2405.13751)
    for (from, to) in parts {
      let starts = from..=from.max(own.0.min(to));
      let Some(run) = starts
        .take(8)
        .flat_map(|s| {
          (s.max(own.1.min(to))..=to)
            .rev()
            .take(8)
            .map(move |t| (s, t))
        })
        .find_map(|(from, to)| {
          let (from, to) = (index(from)?, index(to)?);
          grown(field, from, to, &|i| prints(i + base))
        })
      else {
        return declined("no balanced run holds a cut");
      };
      cuts.push(run);
    }
  }
  cuts.sort_unstable();
  cuts.dedup();
  let overlaps =
    |e: &Event, (s, t): (usize, usize)| span_of(e).is_some_and(|(a, b)| a < t && b > s);
  for (s, t) in cuts.iter().map(|&(s, t)| (s + base, t + base)) {
    if kept.iter().any(|ink| prints_in(ink, (s, t))) {
      return declined("a cut takes a kept character's token");
    }
    if around
      .stops
      .iter()
      .any(|e| e.kind == EventKind::Note && overlaps(e, (s, t)))
    {
      return declined("a cut takes a note");
    }
  }
  // (each address as printed — RT1 row 56, the switch off's wrapper (`dom::address_cut`): the call wrapping it alone, whole, its
  // cover's tokens where nothing else prints there and the cut takes it — `\href{mailto:a@x}{a@x}` (2608.15371), `\textit{a@x}`
  // —; else its characters' own tokens, with the switches of the group the run's first address opens (`{\tt\small a@x; b@x}`'s
  // second), else in the font of the ink printing it — `{\it Contact e-mail: a@x}` (2008.08137), `\texttt{a@x, b@y}`'s each
  // (2608.14026), a `\thanks` body's (its inner inks: 2405.06709); a list's written-out address, which prints no tokens of its
  // own, its text in that font (RP1's `EntryCut::mail`: `\texttt{\{a,b\}@x}`; one a contact command prints, whose call the
  // e-mail's item would mute, 1109.3369's `Email:~$^a$\email{…}`))
  let mut mails = Vec::new();
  let reach_of = |from: usize, to: usize| {
    let reaches: Vec<(usize, usize)> = (0..chars.len())
      .filter(|&i| from <= chars[i].0 && chars[i].0 < to)
      .filter_map(|i| ink[i].as_ref().map(reach))
      .collect();
    Some((
      reaches.iter().map(|r| r.0).min()?,
      reaches.iter().map(|r| r.1).max()?,
    ))
  };
  // (whether an event prints inside `range`: what a wrapper taken whole would take with the address — a contact command its
  // item would mute among them)
  let holds_event = |(s, t): (usize, usize)| {
    around
      .stops
      .iter()
      .chain(&around.apart)
      .chain(&around.contacts)
      .filter_map(|e| span_of(e))
      .chain(around.marks.iter().filter_map(|e| mark_of(e)))
      .any(|(a, b)| a < t && b > s)
  };
  // (whether a formula the field prints overlaps `range`)
  let in_a_formula = |(s, t): (usize, usize)| {
    field
      .formulas
      .iter()
      .any(|f| usize::from(f.start) < t && usize::from(f.end) > s)
  };
  for run in &taken.runs {
    let run_at = reach_of(run.bytes.0, run.bytes.1);
    let bytes_of = |address: &str| {
      lined
        .get(run.bytes.0..run.bytes.1)
        .and_then(|r| r.find(address))
        .map(|at| (run.bytes.0 + at, run.bytes.0 + at + address.len()))
    };
    // (where the run's first address prints: the switches before it are its font's, a label or a bracket before it none)
    let first_at = run
      .addresses
      .iter()
      .filter_map(|a| bytes_of(a))
      .min()
      .and_then(|(s, t)| reach_of(s, t));
    for address in &run.addresses {
      let bytes = bytes_of(address);
      // (the font its `@` prints in (`InkTable::fonts`: a font switched inside the ink, `\thanks{… \texttt{E-Mail: a@x}}`,
      // 2405.06709), else the ink's printing it)
      let at_font = bytes
        .and_then(|(s, t)| {
          (0..chars.len()).find(|&i| s <= chars[i].0 && chars[i].0 < t && chars[i].1 == '@')
        })
        .or_else(|| {
          (0..chars.len())
            .find(|&i| run.bytes.0 <= chars[i].0 && chars[i].0 < run.bytes.1 && chars[i].1 == '@')
        })
        .and_then(|i| ink[i])
        .and_then(|ink| inks.font_at(ink.own.unwrap_or(ink.cover.0)))
        .cloned();
      let font = || {
        at_font.clone().or_else(|| {
          around
            .inks
            .iter()
            .filter(|e| e.font.is_some() && e.text.contains('@'))
            .find(|e| e.text.contains(address.as_str()) || run_at.is_some_and(|r| overlaps(e, r)))
            .and_then(|e| e.font.clone())
        })
      };
      let printed = bytes.zip(first_at.or(run_at)).and_then(|((s, t), run_at)| {
        let mine: Vec<usize> = (0..chars.len())
          .filter(|&i| s <= chars[i].0 && chars[i].0 < t)
          .collect();
        mine.iter().all(|&i| ink[i].is_some()).then_some(())?;
        let owns: Option<Vec<usize>> = mine
          .iter()
          .map(|&i| ink[i].and_then(|ink| ink.own).map(usize::from))
          .collect();
        let own = owns.and_then(|owns| Some((*owns.iter().min()?, owns.iter().max()? + 1)));
        let range = reach_of(s, t)?;
        let others_print = |range: (usize, usize)| {
          (0..chars.len())
            .filter(|i| !mine.contains(i))
            .filter_map(|i| ink[i])
            .any(|ink| prints_in(&ink, range))
        };
        let (from, to, first) = (index(range.0)?, index(range.1)?, index(run_at.0)?);
        let taken_whole = cuts.iter().any(|&(s, t)| s <= from && to <= t);
        if Some(range) != own
          && taken_whole
          && !others_print(range)
          && !holds_event(range)
          && !in_a_formula(range)
        {
          let (tokens, _) = field.content_of((from, to), 0, from)?;
          return Some((tokens.unlist(), "its wrapper"));
        }
        let own = own?;
        // (never its printed formula, `dcook12$@$uwyo.edu` (1409.0847): its text, as the switch off writes it)
        if others_print(own) || in_a_formula(own) {
          return None;
        }
        let (own_from, own_to) = (index(own.0)?, index(own.1)?);
        let (tokens, _) = field.content_of((own_from, own_to), 0, first.min(own_from))?;
        let tokens = tokens.unlist();
        Some(if tokens.len() > own_to - own_from {
          (tokens, "its group's switches")
        } else {
          (in_its_font(font(), tokens).unlist(), "its ink's font")
        })
      });
      let (tokens, how) = printed.unwrap_or_else(|| {
        (
          in_its_font(font(), text_tokens(address)).unlist(),
          "its text",
        )
      });
      DebugFeature!(
        "frontmatter",
        "FRONT emit: a cut address {address:?}: {how}"
      );
      mails.push((address.clone(), tokens, run.label.clone()));
    }
  }
  Some((cuts, mails))
}

/// Where each character of the reading's `lined` text is in `text` (an index of its characters; `None` for a line's end):
/// `lined` once in it, or each of its lines once in a line of it, in order — the reading's lines are an entry's, trimmed of
/// the separator a line of the block ends in (`Dept. of X, Univ Y,\\ Town`, 1203.0253, 2405.03617).
fn locate(text: &str, lined: &str) -> Option<Vec<Option<usize>>> {
  let once = |hay: &str, needle: &str| {
    hay
      .find(needle)
      .filter(|&at| hay.rfind(needle) == Some(at))
      .map(|at| hay[..at].chars().count())
  };
  let mut map = Vec::new();
  if !lined.contains('\n') {
    let at = once(text, lined)?;
    map.extend((0..lined.chars().count()).map(|i| Some(at + i)));
    return Some(map);
  }
  let rows: Vec<(usize, &str)> = {
    let mut from = 0;
    text
      .split('\n')
      .map(|row| {
        let start = from;
        from += row.chars().count() + 1;
        (start, row)
      })
      .collect()
  };
  let lines: Vec<&str> = lined.split('\n').collect();
  let fits = |first: usize| {
    lines.iter().enumerate().all(|(k, line)| {
      rows
        .get(first + k)
        .is_some_and(|(_, row)| once(row, line).is_some())
    })
  };
  let mut starts = (0..rows.len()).filter(|&first| fits(first));
  let (Some(first), None) = (starts.next(), starts.next()) else {
    return None;
  };
  for (k, line) in lines.iter().enumerate() {
    if k > 0 {
      map.push(None);
    }
    let (start, row) = rows[first + k];
    let at = start + once(row, line)?;
    map.extend((0..line.chars().count()).map(|i| Some(at + i)));
  }
  Some(map)
}

/// A cut's runs and its e-mails ([`cut_at`]).
type CutAt = (Vec<(usize, usize)>, Vec<Mail>);

/// The run `from..to` (indices) grown to a balanced one ([`Field::balanced`]), over a few tokens before or after it that print
/// nothing (`prints`): the switches of the group it starts in (`{\tt(sluys@astro.uu.nl)}`, astro-ph0610492), the space before
/// a group's closing brace (`{\tt\small a@x }`).
fn grown(
  field: &Field,
  from: usize,
  to: usize,
  prints: &dyn Fn(usize) -> bool,
) -> Option<(usize, usize)> {
  let back = (1..=4)
    .take_while(|&k| from >= k && !prints(from - k))
    .count();
  let on = (0..4)
    .take_while(|&k| to + k < field.tokens.len() && !prints(to + k))
    .count();
  (0..=back)
    .flat_map(|b| (0..=on).map(move |f| (b, f)))
    .find_map(|(b, f)| field.balanced(from - b, to + f, 0))
}

/// The tokens of the event's `span` of the block at `index`, balanced (`emit::span_tokens_of`), without the runs `cuts` (an entry's address runs, `address_cut`).
pub(super) fn span_tokens_cut(
  index: usize,
  span: (u16, u16),
  cuts: &[(usize, usize)],
) -> Option<Tokens> {
  span_tokens_within(index, span, None, cuts)
}

/// [`span_tokens_cut`] within the span `within` of the block (a note's, `\thanks{…}`): its tokens and the formulas inside it
/// alone, `None` where no balanced run of them holds `span`. A `\thanks` argument is typeset in the footnote (latex.ltx
/// `\thanks` → `\footnotetext`), never part of the formula around the command, so a piece of it read whole by that formula
/// is the formula, the `\thanks` itself digested once more (`Ann Able$^{1}$ and Bob Baker$^{2\thanks{$^{1}$Univ A;
/// $^{2}$Univ B}}$`, review rck2).
pub(super) fn span_tokens_within(
  index: usize,
  span: (u16, u16),
  within: Option<(u16, u16)>,
  cuts: &[(usize, usize)],
) -> Option<Tokens> {
  with_ledger(|ledger| {
    let Some(Entry::Author(block)) = ledger.entries().get(index) else {
      return None;
    };
    let Outcome::Events {
      events,
      formulas,
      environments,
      held,
      ..
    } = &block.outcome
    else {
      return None;
    };
    let at = |p: u16| usize::from(p).checked_sub(block.range.0);
    let all = block.tokens.unlist_ref();
    let Some((s, e)) = within else {
      let field = Field {
        tokens: all,
        base: block.range.0,
        formulas,
        environments,
        held,
        aligned: aligned(events),
        groups: Default::default(),
      };
      let start = at(span.0)?;
      return Some(field.content_cut((start, at(span.1)?), 0, start, cuts)?.0);
    };
    let (from, to) = (at(s)?, at(e)?.min(all.len()));
    // (the formulas inside it: outermost and in order, a run of them)
    let inside = |f: &Formula| s <= f.start && f.end <= e;
    let first = formulas.iter().position(inside).unwrap_or(formulas.len());
    let count = formulas[first..].iter().take_while(|f| inside(f)).count();
    let field = Field {
      tokens: all.get(from..to)?,
      base: block.range.0 + from,
      formulas: &formulas[first..first + count],
      environments,
      held,
      aligned: aligned(events),
      groups: Default::default(),
    };
    let cuts: Vec<(usize, usize)> = cuts
      .iter()
      .filter(|&&(s, t)| s < to && t > from)
      .map(|&(s, t)| (s.max(from) - from, t.min(to) - from))
      .collect();
    let start = at(span.0)?.checked_sub(from)?;
    let end = at(span.1)?.checked_sub(from)?;
    Some(field.content_cut((start, end), 0, start, &cuts)?.0)
  })
}

/// The call for `assoc`, an entry the reading took addresses out of (its rule ends `ruling3:address-out`), `None` for any other:
/// the entry as it printed written by `whole` with its cut, where the path it takes cuts it (`EntryCut::applied`), else its text,
/// counted (`by_text`).
pub(super) fn cut_entry_call(
  cuts: &[EntryCut],
  assoc: &Assoc,
  whole: impl FnOnce(&Assoc, &EntryCut) -> Result<Tokens>,
  by_text: impl FnOnce() -> Tokens,
) -> Option<Result<Tokens>> {
  let rule = assoc.rule.strip_suffix(" ruling3:address-out")?;
  let Some(cut) = cuts.iter().find(|cut| cut.is_cut_of(assoc)) else {
    return Some(Ok(by_text()));
  };
  let entry = Assoc {
    text: cut.original.clone(),
    rule: rule.to_string(),
    ..assoc.clone()
  };
  cut.applied.set(false);
  Some(whole(&entry, cut).map(|full| if cut.applied.get() { full } else { by_text() }))
}

/// The cuts of the entries the reading took addresses out of (`Resolved::taken`), each in `resolved`; one that can't be cut given
/// back whole there, the Kind steps' to cut in place after the flush, as for a paper with no block (`insert_frontmatter`) — entry
/// by entry, not the switch off's whole paper (RT1 row 57a).
///
/// The cut keeps `=2`'s improvements where it harms none (the standing ruling, user 2026-11-08; OD #980 KB2): an entry whose
/// cut would strand the phrase its address completes ([`strands_a_phrase`]: 2306.04726) is given back whole, where its holders
/// are exactly its addresses' owners ([`held_by_its_owners`]). The switch off's cut is unchanged.
pub(super) fn entry_cuts(reading: &Resolved, resolved: &mut Resolved) -> Vec<EntryCut> {
  let names: Vec<&str> = reading.creators.iter().map(|c| c.name.as_str()).collect();
  let mut cuts = Vec::new();
  for taken in &reading.taken {
    let whole = strands_a_phrase(taken) && held_by_its_owners(reading, taken);
    let cut = (!whole)
      .then(|| entry_cut(taken, &names).ok().flatten())
      .flatten();
    match cut {
      Some(cut) => cuts.push(cut),
      None => {
        DebugFeature!(
          "frontmatter",
          "FRONT emit: an entry given back whole: {:?}",
          taken.original
        );
        give_back_entry(resolved, taken);
      },
    }
  }
  cuts
}

/// Whether the cut of `taken` would strand the phrase an address it takes completes: a run with no label word the take-out
/// takes with it (`Taken::runs`, `resolver::labels::take_labels`) after a colon or a "to" the cut keeps, the words before it
/// on its line, from the last separator, naming a contact (`names_a_contact`) — "Author to whom correspondence should be
/// addressed: a@x" (2306.04726), "Correspondence to: a@x" (2004.11812), "Email for correspondence: a@x" (2110.05460), "…,
/// *Corresponding author: \texttt{a@x}" (2602.09324) —: the phrase is the address's own label, which the cut would leave
/// dangling in the entry, its colon lost. An address after a place's colon (`…, France: a@x`, 1103.3532) strands nothing, nor
/// does a phrase the entry holds alone: a legend's, its carriers' note (`\affil[*]{Corresponding author: a@x}`, 1812.03816).
fn strands_a_phrase(taken: &Taken) -> bool {
  let kept = |at: usize| !taken.cuts.iter().any(|&(s, t)| s <= at && at < t);
  taken
    .runs
    .iter()
    .filter(|run| run.label.is_none())
    .any(|run| {
      let Some(before) = taken.lined.get(..run.bytes.0) else {
        return false;
      };
      let before = before.trim_end_matches(|c: char| c.is_whitespace() || "([<".contains(c));
      let word = before
        .rsplit(char::is_whitespace)
        .next()
        .unwrap_or_default();
      let phrase = before.ends_with(':') || word.eq_ignore_ascii_case("to");
      let phrase_end = before.trim_end_matches(':').len();
      let head = &before[..phrase_end];
      let from = head.rfind(['\n', ',', ';', '.']).map_or(0, |p| p + 1);
      let names = head[from..]
        .split(|c: char| !c.is_alphabetic() && c != '-')
        .any(names_a_contact);
      // (a phrase the entry holds alone, `\affil[*]{Corresponding author: a@x}`, is the legend its carriers' note is made of)
      let beside = head[..from].chars().any(char::is_alphanumeric);
      phrase && names && beside && kept(phrase_end - 1)
    })
}

/// Whether the holders of `taken` are exactly the owners of the addresses the reading took out of it: the creators (the block,
/// `None`) holding an `owner:entry-address` contact from its source. An entry given back whole gives its addresses to its
/// holders, so one whose address another author owns (2004.11812's last line, "Correspondence to: \texttt{pascal.klink@…}",
/// Klink's, read onto the Aalto entry only Pajarinen holds) or that a holder shares without owning it (authblk's `\affil[1]`
/// held by two authors, one address) keeps its cut (the author an address spells owns it, FIDELITY_CHARTER §4).
fn held_by_its_owners(reading: &Resolved, taken: &Taken) -> bool {
  let owns = |contacts: &[Assoc]| {
    contacts
      .iter()
      .any(|a| a.source == taken.source && a.rule.contains("owner:entry-address"))
  };
  let mut owners: Vec<Option<usize>> = reading
    .creators
    .iter()
    .enumerate()
    .filter(|(_, c)| owns(&c.contacts))
    .map(|(i, _)| Some(i))
    .collect();
  if owns(&reading.block) {
    owners.insert(0, None);
  }
  let mut holders: Vec<Option<usize>> = taken.holders.iter().map(|(c, ..)| *c).collect();
  holders.sort_unstable();
  holders.dedup();
  !owners.is_empty() && owners == holders
}

/// The e-mail call of `assoc`, an address the reading took out of an entry (`owner:entry-address`) that a cut made, under `keys`
/// and the name its run's label gives (RT1 row 57).
pub(super) fn cut_mail_call(cuts: &[EntryCut], assoc: &Assoc, keys: &str) -> Option<Tokens> {
  if assoc.role != "email" || !assoc.rule.contains("owner:entry-address") {
    return None;
  }
  let (mail, name) = cuts.iter().find_map(|cut| cut.mail(assoc))?;
  let keys = name.map_or(keys.to_string(), |name| s!("{keys},name={{{name}}}"));
  Some(annotate_now("ltx:contact", &keys, &item_only(&mail)))
}
