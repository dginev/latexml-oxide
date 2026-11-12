//! A query on an argument's marks as the rehearsal digests them (Option C F0, OXIDIZED_DESIGN_DIVERGENCES #995): the
//! Mark events of [`read_argument`], the one rehearsal [`super::separators`] reads too. `None` answers "unread", never
//! "no marks"; a caller falls back to its token reading, counted ([`count_unread`]).

use super::{
  events::{Event, EventKind, mark_led_pieces, opening_ink},
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

/// The marks `arg` prints at its own level (nest 0), outside its notes (a note's marks are its `inner`), in order, each
/// with its printed `label`; a number's suffix (`5\textsuperscript{th}`, `Event::ordinal`) is text, no mark. A mark a
/// document macro prints is one (`\newcommand\ua{\textsuperscript{1}}`), as is a `\let` copy's.
pub fn argument_marks(arg: &Tokens) -> Result<Option<Vec<Event>>> {
  Ok(mark_events(arg)?.map(|events| {
    events
      .into_iter()
      .filter(|e| e.kind == EventKind::Mark && e.nest == 0 && !e.ordinal)
      .collect()
  }))
}

/// Whether a mark opens what `arg` prints (`events::opens_with_mark`); with `non_symbol`, a mark that is no footnote
/// symbol (`Event::symbol`).
pub fn argument_opens_with_mark(arg: &Tokens, non_symbol: bool) -> Result<Option<bool>> {
  Ok(mark_events(arg)?.map(|events| {
    let refs: Vec<&Event> = events.iter().collect();
    opening_ink(&refs).is_some_and(|e| e.kind == EventKind::Mark && !(non_symbol && e.symbol))
  }))
}

/// `arg`'s entries, each led by a mark (`events::mark_led_pieces`); empty when it is one entry.
pub fn argument_mark_led_pieces(arg: &Tokens) -> Result<Option<Vec<Vec<Event>>>> {
  Ok(mark_events(arg)?.map(|events| mark_led_pieces(&events)))
}
