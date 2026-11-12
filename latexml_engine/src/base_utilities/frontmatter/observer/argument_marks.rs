//! A query on an argument's marks as the rehearsal digests them (Option C F0, OXIDIZED_DESIGN_DIVERGENCES #995): the
//! Mark events of [`read_argument`], the one rehearsal [`super::separators`] reads too. `None` answers "unread", never
//! "no marks"; a caller falls back to its token reading, counted ([`count_unread`]).

use latexml_core::definition::PrimitiveBody;

use super::{
  events::{self, Event, EventKind, is_mark_char, opening_ink},
  rehearsal,
  separators::{ArgumentReading, read_argument},
  *,
};
use crate::setup_binding_language::is_accent_inner;

/// Count an argument the marks query could not read, which `caller` reads by tokens: a `--debug=frontmatter_marks` line
/// each, the A/B's fallback count until P6 decides.
pub(in crate::base_utilities) fn count_unread(caller: &str) {
  DebugFeature!(
    "frontmatter_marks",
    "FRONT marks query unread ({caller}): read by tokens"
  );
}

/// Whether `tokens` can print a mark at all: a command or an active character that may ([`prints`]), read through
/// parameterless macros' bodies, or a superscript; a character typed in the argument is its text. The marks queries ask
/// it before their rehearsal (2103.04797's 1,161 `\affiliation{\Amsterdam}`, an institute's name; 1901.01540's 1,204
/// names, `\author{M.~Soares-Santos}`: not rehearsed).
fn may_mark(tokens: &[Token]) -> bool { marks_in(tokens, true, MACRO_DEPTH, &mut Vec::new()) }

/// How many macros deep the gates read a body.
const MACRO_DEPTH: u8 = 8;

/// [`may_mark`] on `tokens`, the argument's own (`typed`) or a body's, `depth` macros deep; `clean` the commands found
/// to print no mark.
fn marks_in(tokens: &[Token], typed: bool, depth: u8, clean: &mut Vec<Token>) -> bool {
  tokens.iter().any(|t| match t.get_catcode() {
    Catcode::SPACE | Catcode::BEGIN | Catcode::END => false,
    Catcode::LETTER | Catcode::OTHER => !typed && t.with_str(|s| s.chars().any(is_mark_char)),
    Catcode::CS | Catcode::ACTIVE => {
      if clean.contains(t) {
        return false;
      }
      let marks = match prints(t) {
        Prints::Body(body) if depth > 0 => marks_in(body.unlist_ref(), false, depth - 1, clean),
        Prints::Nothing | Prints::NoMark => false,
        _ => true,
      };
      if !marks {
        clean.push(*t);
      }
      marks
    },
    Catcode::SUPER => true,
    // (a formula, a `&`: a body's read in the rehearsal)
    _ => !typed,
  })
}

/// What a command or an active character prints, as the marks gates read it.
enum Prints {
  /// a parameterless macro's body
  Body(Tokens),
  /// nothing: a macro of no expansion (`\newcommand\affilfont{}`)
  Nothing,
  /// no mark, the tokens after it read on: the kernel's `\relax` and tie (`~`'s `\lx@NBSP`), an accent
  /// (`accent_inner_name`, on the argument after it), a primitive or a `\chardef` printing no mark character (`\ss`,
  /// `\&`)
  NoMark,
  /// anything: a command the rehearsal binds ([`rehearsal::rebinds`]), a macro taking arguments, any other primitive
  Any,
}

/// What `t` prints ([`Prints`]), by its meaning here.
fn prints(t: &Token) -> Prints {
  if rehearsal::rebinds(t) {
    return Prints::Any;
  }
  match lookup_meaning(t) {
    Some(Stored::Expandable(defn)) => {
      if defn.get_origin().is_latexml_owned() && defn.get_cs().with_str(is_accent_inner) {
        return Prints::NoMark;
      }
      if defn
        .get_parameters()
        .is_some_and(|p| !p.get_parameters().is_empty())
      {
        return Prints::Any;
      }
      match defn.get_expansion() {
        Some(ExpansionBody::Tokens(body)) => Prints::Body(body.clone()),
        None => Prints::Nothing,
        _ => Prints::Any,
      }
    },
    Some(Stored::Primitive(defn)) if defn.origin.is_latexml_owned() => {
      // (`\relax`, a robust command's `\protect`; the tie, `~`, `\nobreakspace`)
      let blank = [
        T_CS!("\\relax"),
        T_CS!("\\lx@NBSP"),
        T_CS!("\\lx@nobreakspace"),
      ]
      .contains(&defn.cs);
      match &defn.replacement {
        _ if blank => Prints::NoMark,
        Some(PrimitiveBody::String(text)) if !with(*text, |s| s.chars().any(is_mark_char)) => {
          Prints::NoMark
        },
        _ => Prints::Any,
      }
    },
    // (a `\chardef`, `\&`)
    Some(Stored::Register(reg)) => match reg.chardef_glyph() {
      Some(glyph) if !with(glyph, |g| g.chars().any(is_mark_char)) => Prints::NoMark,
      _ => Prints::Any,
    },
    _ => Prints::Any,
  }
}

/// The events of `arg`, or none read where it cannot print a mark.
fn mark_events(arg: &Tokens) -> Result<Option<Vec<Event>>> {
  if !may_mark(arg.unlist_ref()) {
    return Ok(Some(Vec::new()));
  }
  Ok(read_argument(arg)?.map(|ArgumentReading { events, .. }| events))
}

/// The level `events` print their text at: the outermost a word of them stands at, inside the box when one box holds
/// it all (IEEEtran's `\IEEEauthorblockN{\parbox{…}{Ann\IEEEauthorrefmark{4}, …}}`, 2508.17623; acmart's
/// `\institution{$^1$Univ A …}`; apa6's `$^{1}$\mbox{Univ A}`, 2508.05776).
fn own_level(events: &[Event]) -> u8 {
  events
    .iter()
    .filter(|e| e.kind == EventKind::Name)
    .map(|e| e.nest)
    .min()
    .unwrap_or(0)
}

/// Which of `events` print at the argument's own level: at or above its words' ([`own_level`]), or in a box of their own
/// there, one holding no word (`\raisebox{0pt}[0pt][0pt]{\textsuperscript{1}}`, 2307.12121, 2311.01786); a mark in a box
/// that also holds words is none.
fn at_own_level(events: &[Event]) -> Vec<bool> {
  let level = own_level(events);
  let mut own = vec![true; events.len()];
  let mut at = 0;
  while at < events.len() {
    if events[at].nest <= level {
      at += 1;
      continue;
    }
    // (the run of deeper events a box at the words' level holds)
    let end = events[at..]
      .iter()
      .position(|e| e.nest <= level)
      .map_or(events.len(), |k| at + k);
    let wordless = events[at..end].iter().all(|e| e.kind != EventKind::Name);
    own[at..end].fill(wordless);
    at = end;
  }
  own
}

/// The marks `arg` prints at its own level (`at_own_level`), outside its notes (a note's marks are its `inner`), in
/// order, each with its printed `label`; a number's suffix (`5\textsuperscript{th}`, `Event::ordinal`) is text, no mark.
/// A mark a document macro prints is one (`\newcommand\ua{\textsuperscript{1}}`), as is a `\let` copy's.
pub fn argument_marks(arg: &Tokens) -> Result<Option<Vec<Event>>> {
  Ok(mark_events(arg)?.map(|events| {
    let own = at_own_level(&events);
    events
      .into_iter()
      .zip(own)
      .filter(|(e, own)| *own && e.kind == EventKind::Mark && !e.ordinal)
      .map(|(e, _)| e)
      .collect()
  }))
}

/// Whether `arg` prints a mark at any level (in a box beside its words too, `Ann\mbox{$^{1}$}`), and whether one is a
/// label ([`is_label_mark`]), a `\footnotemark` and a number's suffix none; unread, by its tokens
/// (`author_mark_position`, `leads_with_mark` from each mark), counted for `caller`.
pub fn prints_marks_and_labels(arg: &Tokens, caller: &str) -> Result<(bool, bool)> {
  if let Some(events) = mark_events(arg)? {
    let marks: Vec<&Event> = events.iter().filter(|e| own_mark(e)).collect();
    return Ok((!marks.is_empty(), marks.into_iter().any(is_label_mark)));
  }
  count_unread(caller);
  let v = arg.unlist_ref();
  let any = author_mark_position(arg).is_some();
  let mut from = 0;
  while let Some(at) = author_mark_position(&Tokens::new(v[from..].to_vec())) {
    let at = from + at - 1;
    if leads_with_mark(&Tokens::new(v[at..].to_vec())) {
      return Ok((any, true));
    }
    from = at + 1;
  }
  Ok((any, false))
}

/// Whether a printed mark is a label: a letter or a number first, no footnote symbol (`$^{1,*}$`; not `$^{*}$`).
pub fn is_label_mark(mark: &Event) -> bool {
  !mark.symbol && mark.text.chars().next().is_some_and(char::is_alphanumeric)
}

/// Whether `arg` prints a mark of its own ([`argument_marks`]), a `\footnotemark` none; unread, its tokens' marks
/// (`author_mark_position`), counted for `caller` (`count_unread`). A class's numbered store (Option C P4d).
pub fn prints_marks(arg: &Tokens, caller: &str) -> Result<bool> {
  Ok(match argument_marks(arg)? {
    Some(marks) => marks.iter().any(own_mark),
    None => {
      count_unread(caller);
      author_mark_position(arg).is_some()
    },
  })
}

/// Whether a list `arg` prints a mark: one at its own level ([`argument_marks`]), or the label a note of it opens with,
/// which TeX prints at the note's head (IEEEtran's `\thanks{\IEEEauthorrefmark{1} …}` in a block, 1504.01072); a
/// `\footnotemark` none. Unread, its tokens' marks, counted for `caller`.
pub fn prints_list_marks(arg: &Tokens, caller: &str) -> Result<bool> {
  let Some(events) = mark_events(arg)? else {
    count_unread(caller);
    return Ok(author_mark_position(arg).is_some());
  };
  let own = at_own_level(&events);
  Ok(
    events
      .iter()
      .zip(own)
      .filter(|(_, own)| *own)
      .any(|(e, _)| {
        own_mark(e)
          || (e.kind == EventKind::Note && e.inner.iter().any(|m| m.line_first && own_mark(m)))
      }),
  )
}

/// A mark an argument prints ([`marks_as_printed`]).
pub struct PrintedMark {
  /// the affiliation labels it sets (`printed_label_keys`)
  pub labels:     Vec<String>,
  /// its line's first ink (`Event::line_first`)
  pub line_first: bool,
  /// at the argument's own level (`at_own_level`), not in a box beside its words
  pub own_level:  bool,
  /// a label ([`is_label_mark`]) that opens what the argument prints ([`leads_with_label`])
  pub leads:      bool,
}

/// The marks `arg` prints at any level outside its notes (in a box beside its words too, `\parbox{\linewidth}{$^{1}$Univ
/// A\\ $^{2}$Univ B}`), a number's suffix and a `\footnotemark` none. Unread, `None`, counted for `caller`.
pub fn marks_as_printed(arg: &Tokens, caller: &str) -> Result<Option<Vec<PrintedMark>>> {
  let Some(events) = mark_events(arg)? else {
    count_unread(caller);
    return Ok(None);
  };
  let own = at_own_level(&events);
  let opening = opening_mark(&events).filter(|e| is_label_mark(e));
  Ok(Some(
    events
      .iter()
      .zip(own)
      .filter(|(e, _)| own_mark(e))
      .map(|(mark, own_level)| PrintedMark {
        labels: mark_request_text(mark)
          .map(|text| printed_label_keys(&text, "affiliation"))
          .unwrap_or_default(),
        line_first: mark.line_first,
        own_level,
        leads: opening.is_some_and(|o| std::ptr::eq(o, mark)),
      })
      .collect(),
  ))
}

/// Whether `arg` prints a mark of its own ([`argument_marks`]) whose label a queued name requests (`requested_labels`);
/// unread, its tokens' marks, counted for `caller`. A class's numbered store declines its numbers for its own marks
/// only when the names ask for them (revP4d S1: `\ma` printing `\textsuperscript{a}` under `\inst{1}`, numbered 1
/// and 2).
pub fn prints_requested_marks(arg: &Tokens, caller: &str) -> Result<bool> {
  let Some(marks) = argument_marks(arg)? else {
    count_unread(caller);
    return Ok(author_mark_position(arg).is_some());
  };
  // (the queued names' marks read first, so their requests are current: [`requested_entry_labels`])
  queued_author_marks()?;
  let requested = events::names_requested(&[]);
  Ok(
    marks
      .iter()
      .any(|mark| own_mark(mark) && events::is_requested(mark, &requested)),
  )
}

/// Whether a label — a mark printing a letter or a number first, not a footnote symbol — opens what `arg` prints
/// (`$^1$Univ A`, `\ua Univ A` with `\newcommand\ua{$^{1}$}`, a `\let` copy's, one in a box of its own); a `\footnotemark`
/// or a number's suffix none. Unread, its tokens (`notes::leads_with_mark`), counted for `caller`.
pub fn leads_with_label(arg: &Tokens, caller: &str) -> Result<bool> {
  if !may_open_with_mark(arg) {
    return Ok(false);
  }
  let Some(events) = mark_events(arg)? else {
    count_unread(caller);
    return Ok(leads_with_mark(arg));
  };
  Ok(opening_mark(&events).is_some_and(is_label_mark))
}

/// The mark the first ink of `events` is, a `\footnotemark` or a number's suffix none.
fn opening_mark(events: &[Event]) -> Option<&Event> {
  let all: Vec<&Event> = events.iter().collect();
  opening_ink(&all).filter(|e| own_mark(e))
}

/// Whether `event` is a mark of its own: not a number's suffix (`Event::ordinal`), not a `\footnotemark`.
fn own_mark(event: &Event) -> bool {
  event.kind == EventKind::Mark && !event.ordinal && event.role.as_deref() != Some("footnotemark")
}

/// Whether what `arg` prints may open with a mark: its first token past spaces and braces, read through parameterless
/// macros' bodies (8 deep) and on past a body that prints nothing (`\newcommand\affilfont{}`), is no letter or digit,
/// which TeX prints first. The readers of an opening mark ask it before their rehearsal (2103.04797's 1,161
/// `\affiliation{\Amsterdam}`, each a macro of an institute's name, not rehearsed).
fn may_open_with_mark(arg: &Tokens) -> bool { first_may_be_mark(arg.unlist_ref(), MACRO_DEPTH) }

/// [`may_open_with_mark`] on `tokens`, `depth` macros deep.
fn first_may_be_mark(tokens: &[Token], depth: u8) -> bool {
  let Some(at) = tokens.iter().position(|t| {
    !matches!(
      t.get_catcode(),
      Catcode::SPACE | Catcode::BEGIN | Catcode::END
    )
  }) else {
    return false;
  };
  let first = &tokens[at];
  match first.get_catcode() {
    Catcode::LETTER | Catcode::OTHER => !first.with_str(|s| s.chars().all(char::is_alphanumeric)),
    Catcode::CS | Catcode::ACTIVE if depth > 0 => match prints(first) {
      Prints::Body(body) => {
        let mut next = body.unlist();
        next.extend_from_slice(&tokens[at + 1..]);
        first_may_be_mark(&next, depth - 1)
      },
      Prints::Nothing => first_may_be_mark(&tokens[at + 1..], depth - 1),
      _ => true,
    },
    _ => true,
  }
}

/// The labels, distinct, set by the marks that open the entries of a list `arg`, its entries as the observer cuts an API
/// call (`events::mark_led_pieces`), each with the queue positions of the names requesting it, a label no name
/// requests left out: the API Cut's decision and its copy rule (`cut::cut_list`). Unread, none — the list stays whole for
/// the observer to read —, counted for `caller`.
pub fn requested_entry_labels(arg: &Tokens, caller: &str) -> Result<Vec<(String, Vec<usize>)>> {
  if !may_open_with_mark(arg) {
    return Ok(Vec::new());
  }
  let Some(events) = mark_events(arg)? else {
    count_unread(caller);
    return Ok(Vec::new());
  };
  if !events.iter().any(own_mark) {
    return Ok(Vec::new());
  }
  // (the queued names' marks read first: the pieces part at the marks they request, `events::names_requested`)
  queued_author_marks()?;
  let mut labels: Vec<(String, Vec<usize>)> = Vec::new();
  for piece in events::mark_led_pieces(&events) {
    // (the label `\lx@sup@setlabel@affiliation` sets: the first of the mark's printed labels, `set_label`)
    let Some(label) = piece
      .iter()
      .find(|e| e.kind == EventKind::Mark)
      .filter(|mark| own_mark(mark))
      .and_then(mark_request_text)
      .and_then(|text| {
        clean_frontmatter_labels(&text, "affiliation")
          .into_iter()
          .next()
      })
    else {
      continue;
    };
    let requesters = queued_label_requesters(&label)?;
    if !requesters.is_empty() && labels.iter().all(|(seen, _)| *seen != label) {
      labels.push((label, requesters));
    }
  }
  Ok(labels)
}
