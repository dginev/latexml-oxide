//! The mark ledger: the name each `\footnotemark` or note printed beside spacing goes to — the name before it, at a line's
//! or cell's start the name after, between two names the name after where the block's edge proof holds or the name before
//! carries its key already (DG 2026-11-17) —, read once per block for the reading and the writer
//! (OXIDIZED_DESIGN_DIVERGENCES #1057; 2412.20504, 1911.13299, 1807.02791, 1703.08733, 1111.1979).

use super::*;

/// The name a mark or note of the ledger goes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::base_utilities::frontmatter::observer) enum Side {
  /// the name before it on its line
  Before,
  /// the name after it on its line
  After,
}

/// The glue on one side of a mark out to a name's ink: its natural width (em of the box's font), whether a fill
/// stretches it, and whether there is any (a Gap or an interword space, whatever its width).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::base_utilities::frontmatter::observer) struct Reach {
  pub width: f64,
  pub fill:  bool,
  pub space: bool,
}

impl Reach {
  const NONE: Reach = Reach {
    width: 0.0,
    fill:  false,
    space: false,
  };

  fn widest(self, other: Reach) -> Reach {
    Reach {
      width: self.width.max(other.width),
      fill:  self.fill || other.fill,
      space: self.space || other.space,
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

impl MarkRecord {
  /// Whether its decided side gives it to the Name event at `name`.
  fn goes_to(&self, name: usize) -> bool {
    match self.side {
      Some(Side::Before) => self.before.is_some_and(|(at, _)| at == name),
      Some(Side::After) => self.after.is_some_and(|(at, _)| at == name),
      None => false,
    }
  }
}

/// The ledger of a block's marks and notes set beside spacing ([`mark_ledger`]).
#[derive(Debug, Clone, Default, PartialEq)]
pub(in crate::base_utilities::frontmatter::observer) struct MarkLedger {
  pub records:    Vec<MarkRecord>,
  /// each mark printed on top of the title mark before it, of the same glyph ([`overprints`]): its span and the glyph
  pub overprints: Vec<((u16, u16), String)>,
}

impl MarkLedger {
  /// The glyph the mark or note with span `span` prints on top of the same glyph ([`overprints`]).
  pub fn overprint_of(&self, span: Option<(u16, u16)>) -> Option<&str> {
    let span = span?;
    self
      .overprints
      .iter()
      .find(|(at, _)| *at == span)
      .map(|(_, glyph)| glyph.as_str())
  }

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
  /// the block's start
  Start,
  /// a `\\`'s line start: no name before it on its line
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
  let mut opening = Opening::Start;
  // (the records between two names, which the block's edge proof gives the name after)
  let mut between: Vec<usize> = Vec::new();
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
    glue.space |= space;
    if !gap_seen && space {
      glue.width += 1.0 / 3.0;
    }
  };
  for (k, e) in events.iter().enumerate() {
    match &e.kind {
      EventKind::Sep(SepKind::Gap { width, fill, .. }) => {
        glue.width += width;
        glue.fill |= fill;
        glue.space = true;
        gap_seen = true;
      },
      // (an alignment's cell opens as `&` does: `\begin{tabular}{cc}Ann Able & \footnotemark[2]Bob Baker`)
      EventKind::Layout if e.role.as_deref() == Some("cell") => {
        glues.push(glue);
        decide(
          &mut ledger,
          &mut between,
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
          &mut between,
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
        // (a comma glued to a name's run is that name's ink: what follows it is that name's — `Igor Pikovski,$^{1, 2
        // }$\footnotemark $\, \,$ Michael`, Igor's (COMMAMARK: 1111.1979, 1912.01737, 1811.08622);
        // `Lenk\footnotemark[1]\textsuperscript{,}\footnotemark[2] \and`, Lenk's (2407.04435))
        let name_ink = matches!(opening, Opening::Name(_))
          && *kind == SepKind::Comma
          && !space_after
          && !e.space_before
          && glues.iter().all(|g| g.width == 0.0 && !g.fill);
        decide(
          &mut ledger,
          &mut between,
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
          &mut between,
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
    &mut between,
    events,
    opening,
    None,
    &pieces,
    &glues,
  );
  // (the edge proof: the block's first piece before its first name and none after its last, every piece between two
  // names is the name's after — 2406.12338, math0606768; both edges, the default, counted: 1511.05393, 0906.1223. DG
  // 2026-11-17: the first piece sits in the stretch of the first name of the names' line, not before a line above it,
  // maybe an affiliation's (S2); a piece glued to the name before it, no space between, proves the other side (S1))
  let is_name = |k: usize| events[k].kind == EventKind::Name && !norm(&events[k].text).is_empty();
  let last_name = (0..events.len()).rev().find(|&k| is_name(k));
  let last_piece = (0..events.len()).rev().find(|&k| movable(&events[k]));
  if let Some(&first_between) = between.first() {
    let at = ledger.records[first_between].at;
    let line = events[..at]
      .iter()
      .rposition(|e| e.kind == EventKind::Break)
      .map_or(0, |k| k + 1);
    let line_name = (line..events.len()).find(|&k| is_name(k));
    if ledger.records[0].after.map(|(name, _)| name) == line_name {
      let glued = ledger
        .records
        .iter()
        .any(|r| r.before.is_some_and(|(_, reach)| !reach.space));
      if last_piece < last_name && !glued {
        for &at in &between {
          ledger.records[at].side = Some(Side::After);
        }
      } else {
        Info!(
          "frontmatter",
          "mark_convention",
          s!(
            "An author block sets notes both before its names and after them: {} between two names given the name \
             before",
            between.len()
          )
        );
      }
    }
  }
  // (DG 2026-11-17 S3: a piece between two names whose key, its footnote number, the name before carries already is the
  // next name's, with the pieces after it on its stretch — a name printing one key twice contradicts itself: `Ann
  // Able\footnotemark[2] \quad \footnotemark[2]Bob Baker`, "Ann Able† †Bob Baker". One glued to the piece of its key
  // prints over it, once: `\footnotemark[1]\thanks{…}`, 2103.01544)
  let apart = |from: usize, to: usize| {
    events[from].space_after
      || events[to].space_before
      || events[from + 1..to]
        .iter()
        .any(|e| matches!(e.kind, EventKind::Sep(_)) || e.space_before || e.space_after)
  };
  for &i in &between {
    let r = &ledger.records[i];
    let (Some(Side::Before), Some((name, _)), Some(key)) = (r.side, r.before, events[r.at].number)
    else {
      continue;
    };
    let carried = ledger.records[..i]
      .iter()
      .rfind(|o| o.goes_to(name) && events[o.at].number == Some(key));
    if carried.is_some_and(|o| apart(o.at, r.at)) {
      let next = r.after.map(|(at, _)| at);
      for o in &mut ledger.records[i..] {
        if o.before.map(|(at, _)| at) == Some(name) && o.after.map(|(at, _)| at) == next {
          o.side = Some(Side::After);
        }
      }
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
  between: &mut Vec<usize>,
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
  // (DG 2026-11-17: "nearly all notes attach on the right side of an author name and on the left side of an affiliation
  // name" — between two names the name's before, whatever the glue; at a line's or cell's start the name's after)
  let side = match (before, next) {
    (Some(_), _) => Some(Side::Before),
    (None, Some(_)) if matches!(opening, Opening::Start | Opening::Line | Opening::Cell) => {
      Some(Side::After)
    },
    _ => None,
  };
  for (i, &(at, movable)) in pieces.iter().enumerate() {
    if !movable {
      continue;
    }
    if next.is_some() && matches!(opening, Opening::Name(_) | Opening::Other) {
      between.push(ledger.records.len());
    }
    let reach = |range: &[Reach]| range.iter().copied().fold(Reach::NONE, Reach::widest);
    ledger.records.push(MarkRecord {
      at,
      span: events[at].span,
      before: before.map(|b| (b, reach(&glues[..=i]))),
      after: next.map(|n| (n, reach(&glues[i + 1..]))),
      side,
    });
  }
}

/// The marks of `events` (a block's read events) TeX prints on top of the mark before them, of the same glyph: where the
/// title's footnote mark takes no width (`zero_width`: article.cls:206, report.cls:212, book.cls:218 `\rlap` it), a mark
/// or note set after it with no glue between them prints over it — `Ward Struyve\footnote{A}$^{*}$\footnote{B}` reads
/// "Struyve∗†", its two ∗ at one place (2012.08663), `Aaditya Singh\footnotemark[1]\thanks{…}` one ∗ (2103.01544) —, so
/// the page shows that glyph once. A mark set after an inked one stands beside it (`Wei Fan$^{*}$\thanks{…}` "∗∗"),
/// and a mark of another glyph is ink of its own. Decided on the glue the ledger reads (a `Gap`, an interword space);
/// the glyphs compared as one asterisk.
#[cfg_attr(
  not(test),
  allow(dead_code, reason = "no reader until MARKKEY R2b's display (OD #1062)")
)]
pub(in crate::base_utilities::frontmatter::observer) fn overprints(
  events: &[Event],
  zero_width: bool,
) -> Vec<((u16, u16), String)> {
  let mut out = Vec::new();
  if !zero_width {
    return out;
  }
  // (the glyph a mark or note prints on the name, and whether it is the title's own footnote mark, of no width)
  let printed = |e: &Event| -> Option<(String, bool)> {
    match e.kind {
      EventKind::Note if !events::is_picture_note(e) => {
        note_shows(e).map(|glyph| (glyph.to_string(), true))
      },
      EventKind::Mark => {
        let glyph = e.text.trim();
        (!glyph.is_empty()).then(|| (glyph.to_string(), printed_by_footnotemark(e)))
      },
      _ => None,
    }
  };
  let form = |glyph: &str| one_asterisk(glyph.trim());
  // (the zero-width mark before, its glyph and whether a space follows it)
  let mut under: Option<(String, bool)> = None;
  for e in events {
    match &e.kind {
      // (what prints nothing between them: a font change, a recorded contact)
      EventKind::Layout if !is_image(e) => continue,
      EventKind::Declaration | EventKind::ApiCall => continue,
      EventKind::Contact if e.unprinted.is_some() => continue,
      _ => {},
    }
    let Some((glyph, zero)) = printed(e) else {
      under = None;
      continue;
    };
    if let Some((before, space_after)) = &under
      && !space_after
      && !e.space_before
      && form(before) == form(&glyph)
      && let Some(span) = e.span
    {
      out.push((span, glyph.clone()));
    }
    under = zero.then_some((glyph, e.space_after));
  }
  out
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

  /// A mark set with no glue on a zero-width title mark of its glyph prints over it (article.cls:206): `Ann
  /// Able\thanks{…}$^{*}$` (revPUREd P2), `\footnotemark[1]\thanks{…}` (2103.01544); not after an inked mark
  /// (`$^{*}$\thanks{…}`), past a space (a note's own too), of another glyph, or where the title's marks take width.
  #[test]
  fn a_mark_glued_on_a_zero_width_title_mark_of_its_glyph_overprints_it() {
    let note = |glyph: &str| {
      let mut e = Event::new(EventKind::Note);
      e.role = Some(format!("thanks shows={glyph}"));
      e.span = Some((3, 4));
      e
    };
    let typed = |glyph: &str, space_before: bool| {
      let mut e = Event::new(EventKind::Mark);
      e.role = Some("math".to_string());
      e.text = glyph.to_string();
      e.space_before = space_before;
      e.span = Some((5, 6));
      e
    };
    let mut footnotemark = mark();
    footnotemark.text = "∗".to_string();
    footnotemark.span = Some((1, 2));
    let ann = name("Ann Able");
    let star = vec![((5, 6), "∗".to_string())];
    assert_eq!(
      overprints(&[ann.clone(), note("∗"), typed("∗", false)], true),
      star
    );
    assert_eq!(
      overprints(&[ann.clone(), footnotemark.clone(), note("*")], true),
      vec![((3, 4), "*".to_string())]
    );
    // (a note's own spaces, recorded by the lexer: `\thanks{…} $^{*}$`, `$^{*}$ \thanks{…}` print apart — WRONGFIXb)
    let spaced = |before: bool, after: bool| {
      let mut e = note("∗");
      (e.space_before, e.space_after) = (before, after);
      e
    };
    for none in [
      vec![ann.clone(), spaced(false, true), typed("∗", false)],
      vec![ann.clone(), footnotemark.clone(), spaced(true, false)],
      vec![ann.clone(), typed("∗", false), note("∗")],
      vec![ann.clone(), note("∗"), typed("∗", true)],
      vec![ann.clone(), note("∗"), typed("†", false)],
      vec![ann.clone(), note("∗"), gap(0.33, false), typed("∗", false)],
    ] {
      assert_eq!(overprints(&none, true), []);
    }
    assert_eq!(overprints(&[ann, note("∗"), typed("∗", false)], false), []);
  }

  fn sides(events: &[Event]) -> Vec<Option<Side>> {
    mark_ledger(events).records.iter().map(|r| r.side).collect()
  }

  fn note(space_before: bool, space_after: bool) -> Event {
    let mut e = Event::new(EventKind::Note);
    e.role = Some("thanks".to_string());
    (e.space_before, e.space_after) = (space_before, space_after);
    e
  }

  /// Between two names a mark or note is the name's before, whatever the glue (DG 2026-11-17, superseding the nearer
  /// name by the glue of 2026-11-16): a wider blank before it, a tie, a fill, glued to the next name, beside an ORCID, a
  /// note's own spaces (`Ann Able \thanks{…}⏎Bob Baker`, revWRONGFIX B1: 1405.7029, 2109.12466, 1811.10030, 2605.31402,
  /// 1412.3142, 1705.11098, 1711.04571, 2002.10402; hep-th9401012, 2101.04647). Past a separator set off from it by a
  /// space it stays undecided, the reading's adjacency the name's before, a bare mark the counted loss.
  #[test]
  fn a_piece_between_two_names_is_the_name_befores() {
    use Side::*;
    let (ann, bob) = (name("Ann Able"), name("Bob Baker"));
    let mut orcid = Event::new(EventKind::Contact);
    orcid.role = Some("orcid".to_string());
    for run in [
      vec![gap(2.7, false), mark(), gap(0.67, false)],
      vec![gap(1.33, false), mark(), gap(1.33, false)],
      vec![gap(0.67, true), mark()],
      vec![gap(2.33, false), mark(), gap(0.33, false), orcid],
      vec![note(true, true)],
    ] {
      let mut events = vec![ann.clone()];
      events.extend(run);
      events.push(bob.clone());
      assert_eq!(sides(&events), [Some(Before)], "{events:?}");
    }
    // (two pieces: both the name's before)
    let two = [
      ann.clone(),
      gap(0.78, false),
      mark(),
      gap(7.4, false),
      mark(),
      gap(1.2, false),
      bob.clone(),
    ];
    assert_eq!(sides(&two), [Some(Before), Some(Before)]);
    // (no name after it on its line: the name's)
    let mut past = vec![ann.clone(), gap(1.33, false), mark(), gap(1.33, false)];
    past.push(Event::new(EventKind::Sep(SepKind::And("\\and"))));
    past.push(bob.clone());
    assert_eq!(sides(&past), [Some(Before)]);
    // (a comma glued to the name is its ink: past it, the name's — `Igor Pikovski,$^{1, 2 }$\footnotemark $\, \,$
    // Michael`, COMMAMARK 1111.1979; MNRAS's `Ann Able,$^{1}$⏎\thanks{…}⏎Bob`)
    let comma = Event::new(EventKind::Sep(SepKind::Comma));
    let mut labels = Event::new(EventKind::Mark);
    labels.role = Some("math".to_string());
    let glued = [
      ann.clone(),
      comma.clone(),
      labels.clone(),
      mark(),
      gap(1.0, false),
      bob.clone(),
    ];
    assert_eq!(sides(&glued), [Some(Before)]);
    let mnras = [
      ann.clone(),
      comma.clone(),
      labels,
      note(true, true),
      bob.clone(),
    ];
    assert_eq!(sides(&mnras), [Some(Before)]);
    // (set off from the comma, glued to the next name or not: undecided — `Alice One, \footnotemark Bob Two`)
    for run in [vec![gap(0.33, false), mark()], vec![note(true, true)]] {
      let mut events = vec![ann.clone(), comma.clone()];
      events.extend(run);
      events.push(bob.clone());
      assert_eq!(sides(&events), [None], "{events:?}");
    }
  }

  /// At a line's or a cell's start, no name before it, a piece is the name's after; between two names, the name's after
  /// only under the block's edge proof — its first piece before its first name and none after its last (2406.12338's
  /// `\thanks{…} {Carla Schenker,} \thanks{…} {Xiulin Wang,}`, math0606768, 1511.05393, 0906.1223). A block with pieces
  /// at both edges mixes the conventions: the default. A `\\` line opening with a mark, maybe an affiliation's, proves
  /// nothing (revWRONGFIX S2).
  #[test]
  fn the_edge_proof_gives_the_pieces_between_two_names_to_the_name_after() {
    use Side::*;
    let (ann, bob) = (name("Ann Able"), name("Bob Baker"));
    let comma = Event::new(EventKind::Sep(SepKind::Comma));
    assert_eq!(sides(&[mark(), ann.clone()]), [Some(After)]);
    let mut cell = vec![
      ann.clone(),
      Event::new(EventKind::Sep(SepKind::And("\\and"))),
    ];
    cell.extend([mark(), bob.clone()]);
    assert_eq!(sides(&cell), [Some(After)]);
    // (the proof: past a spaced comma and glued to a name's comma alike)
    let prefix = [
      note(false, true),
      ann.clone(),
      comma.clone(),
      note(true, true),
      bob.clone(),
    ];
    assert_eq!(sides(&prefix), [Some(After), Some(After)]);
    // (re-pinned, DG 2026-11-17 S1: a note glued to the name before it, its comma's or its own, is the after-the-name
    // convention's evidence — the block mixes the two, the default; After, After before)
    let glued = [
      note(false, false),
      ann.clone(),
      comma.clone(),
      note(false, false),
      bob.clone(),
    ];
    assert_eq!(sides(&glued), [Some(After), Some(Before)]);
    let carl = name("Carl Cole");
    let suffixes = [
      note(false, true),
      ann.clone(),
      note(false, true),
      bob.clone(),
      note(false, true),
      carl.clone(),
    ];
    assert_eq!(sides(&suffixes), [Some(After), Some(Before), Some(Before)]);
    // (an affiliation line after the names, its own start: the proof holds)
    let mut lines = prefix.to_vec();
    lines.push(Event::new(EventKind::Break));
    lines.extend([mark(), name("University of Somewhere")]);
    assert_eq!(sides(&lines), [Some(After), Some(After), Some(After)]);
    // (both edges: the default)
    let mut mixed = prefix.to_vec();
    mixed.push(note(true, false));
    assert_eq!(sides(&mixed), [Some(After), None, Some(Before)]);
    // (no piece before the first name: the default, a line start after it proving nothing — `Ann Able, \thanks{A} Bob
    // Baker\\ \footnotemark[1]University`)
    let mut affil = vec![ann.clone(), comma.clone(), note(true, true), bob.clone()];
    affil.push(Event::new(EventKind::Break));
    affil.extend([mark(), name("University of Somewhere")]);
    assert_eq!(sides(&affil), [None, Some(After)]);
    // (DG 2026-11-17 S2: the first piece sits in the stretch of the first name of the names' line, not before a line
    // above it, maybe an affiliation's — `\footnotemark[1]University of X\\ Ann Able \thanks{…} Bob Baker`)
    let mut above = vec![mark(), name("University of X")];
    above.push(Event::new(EventKind::Break));
    above.extend([ann.clone(), note(true, true), bob.clone()]);
    assert_eq!(sides(&above), [Some(After), Some(Before)]);
    // (the names' line its own start's: the proof holds — `University of X\\ \thanks{A}Ann, \thanks{B}Bob`)
    let mut below = vec![name("University of X"), Event::new(EventKind::Break)];
    below.extend(prefix.iter().cloned());
    assert_eq!(sides(&below), [Some(After), Some(After)]);
  }

  /// A piece between two names whose key, the footnote number, the name before carries already is not that name's again
  /// — a name printing one key twice contradicts itself — but the next name's, with the pieces after it on the stretch
  /// (DG 2026-11-17 S3: `Ann Able\footnotemark[2] \quad \footnotemark[2]Bob Baker`, "Ann Able† †Bob Baker"). Another
  /// key stays the name before's (`Ann Able\thanks{…} \quad Bob Baker\footnotemark[1]`, the mark Bob's own).
  #[test]
  fn a_key_the_name_before_carries_goes_to_the_next_name() {
    use Side::*;
    let (ann, bob) = (name("Ann Able"), name("Bob Baker"));
    let keyed = |number: i64, space_before: bool| {
      let mut e = mark();
      (e.number, e.space_before) = (Some(number), space_before);
      e
    };
    let repeat = [
      ann.clone(),
      keyed(2, false),
      gap(1.333, false),
      keyed(2, true),
      bob.clone(),
    ];
    assert_eq!(sides(&repeat), [Some(Before), Some(After)]);
    let then = [
      ann.clone(),
      keyed(2, false),
      gap(1.333, false),
      keyed(2, true),
      keyed(3, false),
      bob.clone(),
    ];
    assert_eq!(sides(&then), [Some(Before), Some(After), Some(After)]);
    let other = [
      ann.clone(),
      keyed(1, false),
      gap(1.333, false),
      keyed(2, true),
      bob.clone(),
    ];
    assert_eq!(sides(&other), [Some(Before), Some(Before)]);
    // (glued to the piece before it, the repeat prints over it, once: `\footnotemark[1]\thanks{…}`, 2103.01544)
    let over = [
      ann.clone(),
      keyed(1, false),
      keyed(1, false),
      gap(1.333, false),
      bob.clone(),
    ];
    assert_eq!(sides(&over), [Some(Before), Some(Before)]);
    let spaced_over = [
      ann.clone(),
      keyed(1, true),
      keyed(1, false),
      gap(1.333, false),
      bob.clone(),
    ];
    assert_eq!(sides(&spaced_over), [Some(Before), Some(Before)]);
  }
}
