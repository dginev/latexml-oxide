//! A query on an argument's marks as the rehearsal digests them (Option C F0, OXIDIZED_DESIGN_DIVERGENCES #995): the
//! Mark events of [`read_argument`], the one rehearsal [`super::separators`] reads too. `None` answers "unread", never
//! "no marks"; a caller falls back to its token reading, counted ([`count_unread`]).

use super::{
  events::{self, Event, EventKind},
  separators::{ArgumentReading, read_argument},
  *,
};

/// Count an argument the marks query could not read, which `caller` reads by tokens: a `--debug=frontmatter_marks` line
/// each, the A/B's fallback count until P6 decides.
pub(in crate::base_utilities) fn count_unread(caller: &str) {
  DebugFeature!(
    "frontmatter_marks",
    "FRONT marks query unread ({caller}): read by tokens"
  );
}

/// Whether `tokens` can print a mark at all: a command, an active character or a superscript.
fn may_mark(tokens: &[Token]) -> bool {
  tokens.iter().any(|t| {
    matches!(
      t.get_catcode(),
      Catcode::CS | Catcode::ACTIVE | Catcode::SUPER
    )
  })
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
/// `\institution{$^1$Univ A …}`); a mark in a box beside its words is none of its own (`${\mathrm Park^{1}}$`,
/// cond-mat0206329).
fn own_level(events: &[Event]) -> u8 {
  events
    .iter()
    .filter(|e| e.kind == EventKind::Name)
    .map(|e| e.nest)
    .min()
    .unwrap_or(0)
}

/// The marks `arg` prints at its own level (the level its words print at, inside the box when one box holds them all), outside its notes (a note's marks are its `inner`), in
/// order, each with its printed `label`; a number's suffix (`5\textsuperscript{th}`, `Event::ordinal`) is text, no mark.
/// A mark a document macro prints is one (`\newcommand\ua{\textsuperscript{1}}`), as is a `\let` copy's.
pub fn argument_marks(arg: &Tokens) -> Result<Option<Vec<Event>>> {
  Ok(mark_events(arg)?.map(|events| {
    let level = own_level(&events);
    events
      .into_iter()
      .filter(|e| e.kind == EventKind::Mark && e.nest == level && !e.ordinal)
      .collect()
  }))
}

/// Whether `arg` prints a mark of its own ([`argument_marks`]), a `\footnotemark` none; unread, its tokens' marks
/// (`author_mark_position`), counted for `caller` (`count_unread`). A class's numbered store (Option C P4d).
pub fn prints_marks(arg: &Tokens, caller: &str) -> Result<bool> {
  Ok(match argument_marks(arg)? {
    Some(marks) => marks
      .iter()
      .any(|mark| mark.role.as_deref() != Some("footnotemark")),
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
  let label = |e: &Event| {
    e.kind == EventKind::Mark && !e.ordinal && e.role.as_deref() != Some("footnotemark")
  };
  let level = own_level(&events);
  Ok(events.iter().filter(|e| e.nest == level).any(|e| {
    label(e) || (e.kind == EventKind::Note && e.inner.iter().any(|m| m.line_first && label(m)))
  }))
}

/// Whether `arg` prints a mark of its own ([`argument_marks`]) whose label a queued name requests (`requested_labels`);
/// unread, its tokens' marks, counted for `caller`. A class's numbered store declines its numbers for its own marks only
/// when the names ask for them (revP4d S1: `\ma` printing `\textsuperscript{a}` under `\inst{1}`, numbered 1 and 2).
pub fn prints_requested_marks(arg: &Tokens, caller: &str) -> Result<bool> {
  let Some(marks) = argument_marks(arg)? else {
    count_unread(caller);
    return Ok(author_mark_position(arg).is_some());
  };
  let requested = events::names_requested(&[]);
  Ok(marks.iter().any(|mark| {
    mark.role.as_deref() != Some("footnotemark") && events::is_requested(mark, &requested)
  }))
}
