//! The mark ledger: the name each `\footnotemark` or note printed beside spacing goes to, the nearer by TeX's glue, a tie
//! the next's, read once per block for the reading and the writer (OXIDIZED_DESIGN_DIVERGENCES #1057; 2412.20504,
//! 1911.13299, 1807.02791, 1703.08733, 1111.1979).

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
  /// The record of the mark or note with span `span`.
  pub fn record_of(&self, span: Option<(u16, u16)>) -> Option<&MarkRecord> {
    span.and_then(|span| self.records.iter().find(|r| r.span == Some(span)))
  }

  /// The decided side of the mark or note with span `span`.
  pub fn side_of(&self, span: Option<(u16, u16)>) -> Option<Side> {
    self.record_of(span).and_then(|r| r.side)
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
  /// a line's start (the block's, a `\\`'s): no name before it on its line
  Line,
  /// anything else: a comma, the word "and", other ink
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
  let mut opening = Opening::Line;
  // (the records past a separator as near the next name as the separator, which the block's convention decides)
  let mut ties: Vec<usize> = Vec::new();
  // (the stretch's pieces: (place, movable), the glue before each, the glue since the last ink)
  let mut pieces: Vec<(usize, bool)> = Vec::new();
  let mut glues: Vec<Reach> = Vec::new();
  let mut glue = Reach::NONE;
  let mut gap_seen = false;
  let mut space_after = false;
  // (a comma glued to a name's run, read as its ink: the name opens what follows only when its first piece is glued to
  // the comma too — a mark set off from the comma by a space is between the comma and the next name, as before)
  let mut past_comma = false;
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
        decide(
          &mut ledger,
          &mut ties,
          events,
          opening,
          None,
          &pieces,
          &glues,
        );
        opening = Opening::Cell;
        past_comma = false;
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
        decide(
          &mut ledger,
          &mut ties,
          events,
          opening,
          Some(k),
          &pieces,
          &glues,
        );
        opening = Opening::Name(k);
        past_comma = false;
        (pieces, glues, glue, gap_seen, space_after) =
          (Vec::new(), Vec::new(), Reach::NONE, false, false);
      },
      EventKind::Mark
      | EventKind::Note
      | EventKind::Contact
      | EventKind::Layout
      | EventKind::Name => {
        hidden(&mut glue, gap_seen, space_after || e.space_before);
        if std::mem::take(&mut past_comma) && (glue.width != 0.0 || glue.fill) {
          opening = Opening::Other;
        }
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
        // (a comma glued to a name's run is that name's ink: what follows is measured from it, the nearer name by the
        // glue past it — `Igor Pikovski,$^{1, 2 }$\footnotemark $\, \,$ Michael`, Igor's (DG 2026-11-16, COMMAMARK:
        // 1111.1979, 1912.01737, 1811.08622); `Lenk\footnotemark[1]\textsuperscript{,}\footnotemark[2] \and`, Lenk's
        // (2407.04435))
        let name_ink = matches!(opening, Opening::Name(_))
          && *kind == SepKind::Comma
          && !space_after
          && !e.space_before
          && glues.iter().all(|g| g.width == 0.0 && !g.fill);
        decide(
          &mut ledger,
          &mut ties,
          events,
          opening,
          None,
          &pieces,
          &glues,
        );
        past_comma = name_ink;
        opening = match kind {
          _ if name_ink => opening,
          SepKind::And(_) | SepKind::Column(_) => Opening::Cell,
          _ => Opening::Other,
        };
        (pieces, glues, glue, gap_seen, space_after) =
          (Vec::new(), Vec::new(), Reach::NONE, false, false);
      },
      EventKind::Break => {
        glues.push(glue);
        decide(
          &mut ledger,
          &mut ties,
          events,
          opening,
          None,
          &pieces,
          &glues,
        );
        opening = Opening::Line;
        past_comma = false;
        (pieces, glues, glue, gap_seen, space_after) =
          (Vec::new(), Vec::new(), Reach::NONE, false, false);
      },
    }
  }
  glues.push(glue);
  decide(
    &mut ledger,
    &mut ties,
    events,
    opening,
    None,
    &pieces,
    &glues,
  );
  // (a tie past a separator is the next name's where the block sets its pieces before their names, every piece the
  // ledger decided the next name's: 2406.12338's `\thanks{…} {Carla Schenker,} \thanks{…} {Xiulin Wang,}`, its first
  // note at the block's start; where any is the name's before, as hep-th9401012's `Eiji Tanaka \thanks{…} and`, it stays
  // undecided, the name's before by the reading's adjacency — `Kei-Ichi Kondo, \thanks{…} Takuya Iizuka`)
  let decided = ledger.records.iter().filter_map(|r| r.side);
  if !ties.is_empty()
    && decided.clone().next().is_some()
    && decided.into_iter().all(|side| side == Side::After)
  {
    for &at in &ties {
      ledger.records[at].side = Some(Side::After);
    }
  }
  // (the block's decided sides beside its undecided marks: what a block's convention would decide, measured — MARKKEY K8)
  if ledger.records.iter().any(|r| r.side.is_none()) {
    let count = |side: Option<Side>| ledger.records.iter().filter(|r| r.side == side).count();
    DebugFeature!(
      "frontmatter_ledger",
      "FRONT mark ledger: {} before, {} after, {} undecided",
      count(Some(Side::Before)),
      count(Some(Side::After)),
      count(None)
    );
  }
  ledger
}

/// Record the stretch `pieces` between `opening` and the Name event `next` (`None`: a separator, `\\` or the block's
/// end), `glues[i]` the glue before piece `i` and its last the glue out to `next`'s ink.
fn decide(
  ledger: &mut MarkLedger,
  ties: &mut Vec<usize>,
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
  // (a formula's separator glued between two movable pieces is ink of their run: `\footnotemark[1]$^,$\footnotemark[2]`,
  // 1703.08733)
  let in_run = |i: usize| {
    events[pieces[i].0].role.as_deref() == Some("math")
      && matches!(events[pieces[i].0].kind, EventKind::Sep(_))
      && i > 0
      && pieces.get(i + 1).is_some_and(|&(_, movable)| movable)
      && pieces[i - 1].1
      && [glues[i], glues[i + 1]]
        .iter()
        .all(|g| g.width == 0.0 && !g.fill)
  };
  let stray = pieces
    .iter()
    .enumerate()
    .any(|(i, &(_, movable))| !movable && i >= glued_before && !glued_to_next(i) && !in_run(i));
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
  let mut tie = false;
  let part = match (before, next) {
    _ if stray => None,
    (Some(_), Some(_)) => parted_at(),
    // (past a name, no name after it on the line: all the name's)
    (Some(_), None) => Some(pieces.len()),
    // (in a cell an `\and` or `&` opens, before its name: all that name's)
    (None, Some(_)) if opening == Opening::Cell => Some(0),
    // (at a line's start, before its first name: that name's)
    (None, Some(_)) if opening == Opening::Line => Some(0),
    // (past a separator, before a name: set off from the separator by the stretch's widest glue, the name's — `Alice One,
    // \thanks{…}Bob Two`, the mark `\rlap`ped into the space before Bob (article.cls:206), 2406.12338 —; as near the
    // separator, the block's convention's (`ties`); glued to it, undecided: llncs's `Ann
    // Able,\inst{1}\fnmsep\inst{2}\thanks{…} Bob`, Ann's)
    (None, Some(_)) if opening == Opening::Other => match parted_at() {
      Some(0)
        if glues[1..]
          .iter()
          .all(|g| !g.fill && g.width < glues[0].width) =>
      {
        Some(0)
      },
      Some(0) => {
        tie = true;
        None
      },
      _ => None,
    },
    _ => None,
  };
  for (i, &(at, movable)) in pieces.iter().enumerate() {
    if !movable {
      continue;
    }
    if tie {
      ties.push(ledger.records.len());
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
    // (a formula's comma glued between two marks is ink of their run, 1703.08733's shape; glued after the run alone, it
    // is other ink apart from both names: undecided)
    let mut comma = Event::new(EventKind::Sep(SepKind::Comma));
    comma.role = Some("math".to_string());
    let between = [
      ann.clone(),
      gap(2.33, false),
      mark(),
      comma.clone(),
      mark(),
      gap(0.67, false),
      bob.clone(),
    ];
    assert_eq!(sides(&between), [Some(After), Some(After)]);
    let after = [
      ann.clone(),
      gap(2.33, false),
      mark(),
      comma,
      gap(0.67, false),
      bob.clone(),
    ];
    assert_eq!(sides(&after), [None]);
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

  /// A comma glued to a name's run is its ink: a mark glued past it, its labels before, is measured from it — the nearer
  /// name by the glue, `Igor Pikovski,$^{1, 2 }$\footnotemark $\, \,$ Michael`, Igor's (DG 2026-11-16, COMMAMARK:
  /// 1111.1979) —, and one with no name after it on its line is the name's (`Lenk\footnotemark[1]\textsuperscript{,}
  /// \footnotemark[2] \and`, 2407.04435); one set off from the comma by a space and glued to the next name is that
  /// name's (2406.12338), as near both the block's convention's, glued to the comma or beside other ink set apart from
  /// both undecided (llncs's `Ann Able,\n\inst{1}\thanks{…} Bob Baker`).
  #[test]
  fn a_mark_glued_past_a_names_comma_is_measured_from_the_comma() {
    use Side::*;
    let (igor, michael) = (name("Igor Pikovski"), name("Michael Vanner"));
    let comma = Event::new(EventKind::Sep(SepKind::Comma));
    let mut labels = Event::new(EventKind::Mark);
    labels.role = Some("math".to_string());
    let glued = [
      igor.clone(),
      comma.clone(),
      labels.clone(),
      mark(),
      gap(1.0, false),
      michael.clone(),
    ];
    assert_eq!(sides(&glued), [Some(Before)]);
    let mut last = vec![igor.clone(), comma.clone(), mark()];
    last.push(Event::new(EventKind::Sep(SepKind::And("\\and"))));
    last.push(michael.clone());
    assert_eq!(sides(&last), [Some(Before)]);
    // (a space after the comma and one after the mark: as near the comma as the next name, undecided where the block
    // decides nothing else; glued to the next name, its — the `\rlap`ped mark printed into the space before it,
    // article.cls:206, 2406.12338)
    let spaced = [
      igor.clone(),
      comma.clone(),
      gap(0.33, false),
      mark(),
      gap(0.33, false),
      michael.clone(),
    ];
    assert_eq!(sides(&spaced), [None]);
    let glued_next = [
      igor.clone(),
      comma.clone(),
      gap(0.33, false),
      mark(),
      michael.clone(),
    ];
    assert_eq!(sides(&glued_next), [Some(After)]);
    // (the block's convention decides a tie: its pieces before their names, the next name's — 2406.12338's
    // `\thanks{…} {Carla Schenker,} \thanks{…} {Xiulin Wang,}` —; one after its name, the name's before, undecided —
    // hep-th9401012's `Kei-Ichi Kondo, \thanks{…} Takuya Iizuka, … Eiji Tanaka \thanks{…} and`)
    let prefix = [
      mark(),
      gap(0.33, false),
      igor.clone(),
      comma.clone(),
      gap(0.33, false),
      mark(),
      gap(0.33, false),
      michael.clone(),
    ];
    assert_eq!(sides(&prefix), [Some(After), Some(After)]);
    let mut suffix = vec![
      igor,
      comma,
      gap(0.33, false),
      mark(),
      gap(0.33, false),
      michael,
      gap(0.33, false),
      mark(),
    ];
    suffix.push(Event::new(EventKind::Sep(SepKind::Word("and".to_string()))));
    suffix.push(name("Cat Cole"));
    assert_eq!(sides(&suffix), [None, Some(Before)]);
    // (an `\inst` set apart from both names before the mark: undecided, as before)
    let mut inst = Event::new(EventKind::Mark);
    inst.role = Some("inst".to_string());
    let llncs = [
      name("Ann Able"),
      Event::new(EventKind::Sep(SepKind::Comma)),
      gap(0.33, false),
      inst,
      mark(),
      gap(0.33, false),
      name("Bob Baker"),
    ];
    assert_eq!(sides(&llncs), [None]);
    // (glued to the separator, its glue after it: undecided — llncs's `\inst{2}\thanks{…}` after `\fnmsep`)
    let glued = [
      name("Ann Able"),
      Event::new(EventKind::Sep(SepKind::Comma)),
      gap(0.33, false),
      Event::new(EventKind::Sep(SepKind::Comma)),
      mark(),
      gap(0.33, false),
      name("Bob Baker"),
    ];
    assert_eq!(sides(&glued), [None]);
    // (at the block's start: the first name's)
    assert_eq!(sides(&[mark(), name("Ann Able")]), [Some(After)]);
  }
}
