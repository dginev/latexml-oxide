//! A query on an argument's marks as the rehearsal digests them (Option C F0, OXIDIZED_DESIGN_DIVERGENCES #995): the
//! Mark events of [`read_argument`], the one rehearsal [`super::separators`] reads too. `None` answers "unread", never
//! "no marks"; a caller falls back to its token reading, counted ([`count_unread`]).

use super::{
  events::{Event, EventKind},
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
