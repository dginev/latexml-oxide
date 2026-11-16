//! The mark ledger (advisor T2; DG rulings 2026-11-16): which name a `\footnotemark` or a note printed beside spacing
//! belongs to, read once from a block's events for every consumer — the reading's requests and notes
//! (`Resolver::names_line`) and the writer's runs (`emitter::emit::block_names`).
//!
//! The rule: a mark between two names belongs to the nearer one, measured by the glue TeX sets between inks, a tie to
//! the following name. The marks and notes between two names on a line are parted where the widest glue between them
//! is (a mark is as near a name as the widest blank on the way to it): `Ann (7.8pt) ∗ (23.1pt) Bob (27.3pt) † (4.0pt)
//! Carl` gives ∗ to Ann and † to Carl, `Ann (7.8) ∗ (73.8) † (11.9) Bob` parts the run, `Ann (27) ∗ (6.7) † (23.5) Bob`
//! gives both to Bob (repros `fmmarks_marks_*`, `ne2h_*`). Past a name with no name after it on its line (a separator,
//! `\\`, the block's end) they are that name's (1911.13299's `\ \footnotemark[3]\and`); in a cell an `\and` or `&`
//! opens, the next name's. A fill's width is the line's: it is read at its natural width and at any wider, and decides
//! only where both part the run alike (`Ann Able\thanks{X} \hfill \footnotemark Bob`: the mark glued to Bob is his,
//! whatever the fill). Anything else is undecided, the writer's counted loss: a run holding other ink (an ORCID, a
//! `\cite`, a formula's mark) set apart from both names, glue nowhere, two fills, a run after a comma or opening a line.

use super::*;

/// The name a mark or note of the ledger goes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::base_utilities::frontmatter::observer) enum Side {
  /// the name before it on its line
  Before,
  /// the name after it on its line
  After,
}

/// The glue on one side of a mark out to a name's ink: its natural width (em of the box's font) and whether a fill
/// stretches it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::base_utilities::frontmatter::observer) struct Reach {
  pub width: f64,
  pub fill:  bool,
}

impl Reach {
  const NONE: Reach = Reach { width: 0.0, fill: false };

  fn widest(self, other: Reach) -> Reach {
    Reach {
      width: self.width.max(other.width),
      fill:  self.fill || other.fill,
    }
  }
}

/// One `\footnotemark` or note set beside a name's spacing, and the name it goes to.
#[derive(Debug, Clone, PartialEq)]
pub(in crate::base_utilities::frontmatter::observer) struct MarkRecord {
  /// its place among the block's read events (`resolver::read_events`)
  pub at:     usize,
  pub span:   Option<(u16, u16)>,
  /// the Name event before it on its line, and the widest glue out to that name's last ink; `None` past a cell's start
  pub before: Option<(usize, Reach)>,
  /// the Name event after it on its line, and the widest glue out to its first ink; `None` before a separator, `\\` or
  /// the block's end
  pub after:  Option<(usize, Reach)>,
  /// `None`: undecided
  pub side:   Option<Side>,
}

/// The ledger of a block's marks and notes set beside spacing ([`mark_ledger`]).
#[derive(Debug, Clone, Default, PartialEq)]
pub(in crate::base_utilities::frontmatter::observer) struct MarkLedger {
  pub records: Vec<MarkRecord>,
}

impl MarkLedger {
  /// The decided side of the mark or note with span `span`.
  pub fn side_of(&self, span: Option<(u16, u16)>) -> Option<Side> {
    span.and_then(|span| {
      self
        .records
        .iter()
        .find(|r| r.span == Some(span))
        .and_then(|r| r.side)
    })
  }

  /// The records past the Name event at `name`, before the next name, each with its side.
  pub fn past(&self, name: usize) -> impl Iterator<Item = &MarkRecord> {
    self
      .records
      .iter()
      .filter(move |r| r.before.is_some_and(|(at, _)| at == name))
  }

  /// The records the Name event at `name` takes as its prefix, in print order.
  pub fn prefix_of(&self, name: usize) -> impl Iterator<Item = &MarkRecord> {
    self
      .records
      .iter()
      .filter(move |r| r.side == Some(Side::After) && r.after.is_some_and(|(at, _)| at == name))
  }
}

/// What a stretch of a line between two inks opened at.
#[derive(Clone, Copy, PartialEq)]
enum Opening {
  /// a name (the Name event's place)
  Name(usize),
  /// a cell (`\and`, `&`): no name before it on its line
  Cell,
  /// anything else: a line's start, a comma, the word "and", other ink
  Other,
}

/// A movable piece: a text `\footnotemark` or a note (a mark the reading writes apart from the name).
fn movable(e: &Event) -> bool {
  match e.kind {
    EventKind::Mark => {
      e.role.as_deref() == Some("footnotemark") && !e.formula.is_some_and(|f| f.base.is_some())
    },
    EventKind::Note => !events::is_picture_note(e),
    _ => false,
  }
}

/// The ledger of `events`, a block's read events ([`MarkLedger`]).
pub(in crate::base_utilities::frontmatter::observer) fn mark_ledger(
  events: &[Event],
) -> MarkLedger {
  let mut ledger = MarkLedger::default();
  let mut opening = Opening::Other;
  // (the stretch's pieces: (place, movable), the glue before each, the glue since the last ink)
  let mut pieces: Vec<(usize, bool)> = Vec::new();
  let mut glues: Vec<Reach> = Vec::new();
  let mut glue = Reach::NONE;
  let mut gap_seen = false;
  let mut space_after = false;
  // (a single interword space no Gap event holds, the lexer's `space_before`/`space_after`: its width, `\fontdimen2`
  // of cmr10, 1/3 em)
  let hidden = |glue: &mut Reach, gap_seen: bool, space: bool| {
    if !gap_seen && space {
      glue.width += 1.0 / 3.0;
    }
  };
  for (k, e) in events.iter().enumerate() {
    match &e.kind {
      EventKind::Sep(SepKind::Gap { width, fill, .. }) => {
        glue.width += width;
        glue.fill |= fill;
        gap_seen = true;
      },
      // (an alignment's cell opens as `&` does: `\begin{tabular}{cc}Ann Able & \footnotemark[2]Bob Baker`)
      EventKind::Layout if e.role.as_deref() == Some("cell") => {
        glues.push(glue);
        decide(&mut ledger, events, opening, None, &pieces, &glues);
        opening = Opening::Cell;
        (pieces, glues, glue, gap_seen, space_after) =
          (Vec::new(), Vec::new(), Reach::NONE, false, false);
      },
      EventKind::Layout if !is_image(e) => {},
      // (what prints nothing: a contact recorded only, AASTeX 7's `\email`)
      EventKind::Declaration | EventKind::ApiCall => {},
      EventKind::Contact if e.unprinted.is_some() => {},
      EventKind::Name if !norm(&e.text).is_empty() => {
        hidden(&mut glue, gap_seen, space_after);
        glues.push(glue);
        decide(&mut ledger, events, opening, Some(k), &pieces, &glues);
        opening = Opening::Name(k);
        (pieces, glues, glue, gap_seen, space_after) =
          (Vec::new(), Vec::new(), Reach::NONE, false, false);
      },
      EventKind::Mark
      | EventKind::Note
      | EventKind::Contact
      | EventKind::Layout
      | EventKind::Name => {
        hidden(&mut glue, gap_seen, space_after || e.space_before);
        glues.push(glue);
        pieces.push((k, movable(e)));
        (glue, gap_seen, space_after) = (Reach::NONE, false, e.space_after);
      },
      EventKind::Sep(kind) => {
        glues.push(glue);
        // (a formula's separator between marks, `$^,$`, is ink of the run: 1703.08733)
        if e.role.as_deref() == Some("math") {
          pieces.push((k, false));
          (glue, gap_seen, space_after) = (Reach::NONE, false, false);
          continue;
        }
        decide(&mut ledger, events, opening, None, &pieces, &glues);
        opening = match kind {
          SepKind::And(_) | SepKind::Column(_) => Opening::Cell,
          _ => Opening::Other,
        };
        (pieces, glues, glue, gap_seen, space_after) =
          (Vec::new(), Vec::new(), Reach::NONE, false, false);
      },
      EventKind::Break => {
        glues.push(glue);
        decide(&mut ledger, events, opening, None, &pieces, &glues);
        opening = Opening::Other;
        (pieces, glues, glue, gap_seen, space_after) =
          (Vec::new(), Vec::new(), Reach::NONE, false, false);
      },
    }
  }
  glues.push(glue);
  decide(&mut ledger, events, opening, None, &pieces, &glues);
  ledger
}

/// Record the stretch `pieces` between `opening` and the Name event `next` (`None`: a separator, `\\` or the block's
/// end), `glues[i]` the glue before piece `i` and its last the glue out to `next`'s ink.
fn decide(
  ledger: &mut MarkLedger,
  events: &[Event],
  opening: Opening,
  next: Option<usize>,
  pieces: &[(usize, bool)],
  glues: &[Reach],
) {
  if !pieces.iter().any(|&(_, movable)| movable) {
    return;
  }
  let before = match opening {
    Opening::Name(at) => Some(at),
    _ => None,
  };
  // (other ink glued to a name is that name's — `Ann Able$^{1}$ \qquad \footnotemark[2]`, `\ $^{b}$Bob`; set apart from
  // both it is no movable piece's to cross)
  let glued_before = glues
    .iter()
    .take_while(|g| g.width == 0.0 && !g.fill)
    .count();
  let glued_after = glues
    .iter()
    .rev()
    .take_while(|g| g.width == 0.0 && !g.fill)
    .count();
  let glued_to_next = |i: usize| next.is_some() && i + glued_after >= pieces.len();
  let stray = pieces
    .iter()
    .enumerate()
    .any(|(i, &(_, movable))| !movable && i >= glued_before && !glued_to_next(i));
  // (the place of the widest glue, the first of equal ones: a tie the next name's; with the run's one fill read at its
  // natural width and at any wider, `None` where the two differ)
  let parted_at = || {
    let widest = |fill_wins: bool| {
      let width = |g: &Reach| {
        if fill_wins && g.fill {
          f64::INFINITY
        } else {
          g.width
        }
      };
      let max = glues.iter().map(width).fold(0.0, f64::max);
      (max > 0.0)
        .then(|| glues.iter().position(|g| width(g) == max))
        .flatten()
    };
    match glues.iter().filter(|g| g.fill).count() {
      0 => widest(false),
      1 => widest(false).filter(|&at| Some(at) == widest(true)),
      _ => None,
    }
  };
  let side_from =
    |part: Option<usize>, i: usize| part.map(|p| if i < p { Side::Before } else { Side::After });
  let part = match (before, next) {
    _ if stray => None,
    (Some(_), Some(_)) => parted_at(),
    // (past a name, no name after it on the line: all the name's)
    (Some(_), None) => Some(pieces.len()),
    // (in a cell an `\and` or `&` opens, before its name: all that name's)
    (None, Some(_)) if opening == Opening::Cell => Some(0),
    _ => None,
  };
  for (i, &(at, movable)) in pieces.iter().enumerate() {
    if !movable {
      continue;
    }
    let reach = |range: &[Reach]| range.iter().copied().fold(Reach::NONE, Reach::widest);
    ledger.records.push(MarkRecord {
      at,
      span: events[at].span,
      before: before.map(|b| (b, reach(&glues[..=i]))),
      after: next.map(|n| (n, reach(&glues[i + 1..]))),
      side: side_from(part, i),
    });
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn name(text: &str) -> Event {
    let mut e = Event::new(EventKind::Name);
    e.text = text.to_string();
    e
  }

  fn gap(width: f64, fill: bool) -> Event {
    Event::new(EventKind::Sep(SepKind::Gap { items: 2, width, fill }))
  }

  fn mark() -> Event {
    let mut e = Event::new(EventKind::Mark);
    e.role = Some("footnotemark".to_string());
    e
  }

  fn sides(events: &[Event]) -> Vec<Option<Side>> {
    mark_ledger(events).records.iter().map(|r| r.side).collect()
  }

  /// The widest glue parts a run between two names: each mark the nearer name's, a tie the next name's, a fill
  /// deciding only where its width cannot change the part.
  #[test]
  fn a_run_between_two_names_is_parted_at_its_widest_glue() {
    use Side::*;
    let (ann, bob) = (name("Ann Able"), name("Bob Baker"));
    // (Ann (7.8) ∗ (73.8) † (11.9) Bob: parted)
    let split = [
      ann.clone(),
      gap(0.78, false),
      mark(),
      gap(7.4, false),
      mark(),
      gap(1.2, false),
      bob.clone(),
    ];
    assert_eq!(sides(&split), [Some(Before), Some(After)]);
    // (Ann (27) ∗ (6.7) † (23.5) Bob: both Bob's)
    let wider_before = [
      ann.clone(),
      gap(2.7, false),
      mark(),
      gap(0.67, false),
      mark(),
      gap(2.35, false),
      bob.clone(),
    ];
    assert_eq!(sides(&wider_before), [Some(After), Some(After)]);
    // (a tie: the next name's)
    let tie = [
      ann.clone(),
      gap(1.33, false),
      mark(),
      gap(1.33, false),
      bob.clone(),
    ];
    assert_eq!(sides(&tie), [Some(After)]);
    // (a fill before a mark glued to Bob: Bob's whatever its width; before a mark nearer Ann at its natural width:
    // undecided)
    let glued = [ann.clone(), gap(0.67, true), mark(), bob.clone()];
    assert_eq!(sides(&glued), [Some(After)]);
    let fill = [
      ann.clone(),
      gap(0.67, true),
      mark(),
      gap(1.33, false),
      bob.clone(),
    ];
    assert_eq!(sides(&fill), [None]);
    // (no name after it on its line: the name's)
    let mut past = vec![ann.clone(), gap(1.33, false), mark(), gap(1.33, false)];
    past.push(Event::new(EventKind::Sep(SepKind::And("\\and"))));
    past.push(bob.clone());
    assert_eq!(sides(&past), [Some(Before)]);
    // (an ORCID apart from both names: undecided)
    let mut orcid = Event::new(EventKind::Contact);
    orcid.role = Some("orcid".to_string());
    let other = [
      ann,
      gap(2.33, false),
      mark(),
      gap(0.33, false),
      orcid,
      gap(0.33, false),
      bob,
    ];
    assert_eq!(sides(&other), [None]);
  }
}
