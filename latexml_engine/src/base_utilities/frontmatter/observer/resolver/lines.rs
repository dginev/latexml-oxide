//! The resolver's lines and names (OXIDIZED_DESIGN_DIVERGENCES #837; the module's rules 1-4): a block's events
//! into lines and entries, the names line split at its separators with each mark on the name beside it, and the name
//! helpers (gaps, initials, collaborations, placement letters).

use super::*;

/// A line of a block: its column, its events, and whether a blank line (a row of no ink) comes before it in its column.
pub(super) struct Line<'a> {
  pub(super) column:      usize,
  pub(super) events:      Vec<&'a Event>,
  pub(super) after_blank: bool,
  /// the break before it is inside a braced group or argument (role `group`): one print unit with the line before
  pub(super) group_break: bool,
}

/// A labelled entry of a labelled line: its labels, its events, whether a symbol leads it.
pub(super) struct Entry<'a> {
  pub(super) labels:   Vec<String>,
  pub(super) symbol:   bool,
  /// its labels are the symbols of one mark printing them together (`$^{\dagger\ddagger}$`, no separator): a compound
  /// mark, which names the names carrying them all
  pub(super) compound: bool,
  pub(super) events:   Vec<&'a Event>,
}

/// The block's lines (rule 1): a `Break` ends a row, a column separator (unless `\and` is inline) a column.
pub(super) fn lines_of<'a>(events: &'a [Event], and_inline: bool) -> Vec<Line<'a>> {
  let new = |column| Line {
    column,
    events: Vec::new(),
    after_blank: false,
    group_break: false,
  };
  let mut lines = vec![new(0)];
  let mut column = 0usize;
  for event in events {
    match &event.kind {
      EventKind::Break => lines.push(Line {
        group_break: event.role.as_deref() == Some("group"),
        ..new(column)
      }),
      EventKind::Sep(SepKind::And(_) | SepKind::Column(_)) if !and_inline => {
        column += 1;
        lines.push(new(column));
      },
      // (an inline `\and` after a line of contacts alone opens the next names of the row, as the binding's old split
      // cut its `\and` pieces (`\lx@llncs@author` until Z18e): `Ann Able\\ \email{ann@x.org} \and Bob Baker\\ University
      // of Y`, Bob Baker a name, not words of Ann Able's e-mail line — rank 110, review probe L3)
      EventKind::Sep(SepKind::And(_))
        if lines.last().is_some_and(|line| {
          line.events.iter().any(|e| e.kind == EventKind::Contact)
            && line.events.iter().all(|e| {
              matches!(
                e.kind,
                EventKind::Contact | EventKind::Layout | EventKind::Sep(_)
              )
            })
        }) =>
      {
        lines.push(new(column));
      },
      EventKind::ApiCall => {},
      _ => lines.last_mut().expect("a line").events.push(event),
    }
  }
  // (a row of no ink between two inked rows of a column is a blank line)
  let mut inked_before: Vec<usize> = Vec::new();
  let mut blank = false;
  for line in &mut lines {
    let has_ink = line.events.iter().any(|e| inked(e));
    if !has_ink {
      blank = inked_before.contains(&line.column);
      continue;
    }
    line.after_blank = blank && inked_before.contains(&line.column);
    blank = false;
    inked_before.push(line.column);
  }
  lines.retain(|line| line.events.iter().any(|e| inked(e)));
  lines
}

// (the observer's, shared with the in-block contact reading: `events::mark_led_pieces`)
pub(super) use crate::base_utilities::frontmatter::observer::events::{
  inked, item_led_pieces, list_items, mark_led_pieces,
};

/// The block's events with the ink one command draws inside a word read as the word's letters (row A2): a box a command
/// draws right against the letters before or after it (no space between them in the source), whose cover holds a comma
/// or a row break it draws as ink inside its box, parts no names — combelow's `\cb{t}` stacks a `t` over a comma in an
/// alignment (combelow.sty:16), so `Bogdan Rai\cb{t}\u{a}` is one name, "Bogdan Raiță" as pdflatex prints it
/// (2507.13598), not "Bogdan Rai", "t", "ă" at a comma. The cover is the events the invocation printed, each span
/// opening at the command's one token (its letters' the widest), a row break or a cell of its own alignment no span at
/// all; one Name with the letters before and after it, their texts joined as the source joins them ([`printed_join`]).
/// A row break or a comma the command prints at its own level, after its box, parts what it ends as any does
/// (`\newcommand\nlx{\mbox{Jr.}\\}`, `Ann Able~\nlx Bob Baker`: "Ann Able Jr." and "Bob Baker", review revNL1 S2).
pub(super) fn words_through_one_command(events: &[Event]) -> Cow<'_, [Event]> {
  // (a comma drawn inside the command's box, or a row of an alignment it opens there: deeper than the command's own
  // event — not one it prints after its box)
  let drawn_inside = |command: &Event, e: &Event| {
    e.nest > command.nest
      && match e.kind {
        EventKind::Break => e.role.as_deref() == Some("row"),
        EventKind::Sep(SepKind::Comma) => true,
        _ => false,
      }
  };
  // (most blocks hold no such cover: read as they are, uncopied — the old writer reads a block's events per creator)
  let parted_cover = events.iter().enumerate().any(|(i, e)| {
    e.kind == EventKind::Layout
      && e.span.is_some_and(|(s, t)| {
        t == s + 1
          && events[i + 1..]
            .iter()
            .take_while(|e| e.span.is_none_or(|(a, _)| a == s))
            .any(|inner| drawn_inside(e, inner))
      })
  });
  if !parted_cover {
    return Cow::Borrowed(events);
  }
  let mut out: Vec<Event> = Vec::with_capacity(events.len());
  let mut i = 0;
  while i < events.len() {
    // (the command's own token, one: its box's span)
    let (Some((s, one)), EventKind::Layout) = (events[i].span, &events[i].kind) else {
      out.push(events[i].clone());
      i += 1;
      continue;
    };
    if one != s + 1 {
      out.push(events[i].clone());
      i += 1;
      continue;
    }
    let mut end = one;
    let mut last = i;
    let mut j = i + 1;
    while j < events.len() {
      let e = &events[j];
      // (an event its invocation printed: its span opens at the command)
      let inside = match e.span {
        Some((a, b)) if a == s => {
          end = end.max(b);
          true
        },
        Some(_) => false,
        None => true,
      };
      if !inside
        || !matches!(
          e.kind,
          EventKind::Layout | EventKind::Name | EventKind::Break | EventKind::Sep(SepKind::Comma)
        )
      {
        break;
      }
      if e.span.is_some() {
        last = j;
      }
      j += 1;
    }
    let cover = &events[i..=last];
    let parts = cover.iter().any(|e| drawn_inside(&events[i], e));
    let named = |e: &Event| e.kind == EventKind::Name;
    let before = out
      .last()
      .filter(|e| e.kind == EventKind::Name && e.span.is_some_and(|(_, b)| b == s));
    let after = events
      .get(last + 1)
      .filter(|e| e.kind == EventKind::Name && e.span.is_some_and(|(a, _)| a == end));
    if !parts || !cover.iter().any(named) || (before.is_none() && after.is_none()) {
      out.push(events[i].clone());
      i += 1;
      continue;
    }
    let mut word = match before {
      Some(_) => out.pop().expect("the letters before"),
      None => Event {
        text: String::new(),
        span: Some((s, end)),
        ..cover.iter().find(|e| named(e)).expect("a letter").clone()
      },
    };
    let (through, to) = match after {
      Some(after) => (last + 1, after.span.map_or(end, |(_, b)| b)),
      None => (last, end),
    };
    // (the letters before end at the command's token, so nothing parts them from its own)
    word
      .text
      .push_str(&printed_join(&events[i..=through], named));
    i = through + 1;
    // (its own tokens too, to the last letter: the emitter's run of the name, `emitter::emit::event_run_at`)
    word.span = word.span.map(|(a, _)| (a, to));
    word.marks = word.marks.map(|(a, _)| (a, to));
    out.push(word);
  }
  Cow::Owned(out)
}

/// The text of a run of events' pieces (`piece`) joined as the source joins them: a space between two pieces only where
/// the source has one, a token none of the run's events covers between them — `(for the` and `\babar\ collaboration)`
/// parted by the space before `\babar` (0711.1544); the letters before a command's box, its own and those after it, no
/// space (`Rai\cb{t}\u{a}`, 2507.13598; [`words_through_one_command`]); a note reference between two pieces covering
/// the tokens it prints, the space after it the source's (`BNL-Bielefeld\fnref{col1} Collaboration)`, 1212.4278). An
/// event of no span (a cell of a command's alignment) parts nothing. Each piece's text trimmed; the reading's one join
/// of a name's pieces ([`remark_pieces_joined`]) and of a word's ink.
pub(super) fn printed_join(run: &[Event], piece: impl Fn(&Event) -> bool) -> String {
  let mut text = String::new();
  // (where the source covered so far ends, and whether a token no event covers lies since the last piece)
  let mut covered: Option<u16> = None;
  let mut spaced = false;
  for e in run {
    let is_piece = piece(e);
    if let Some((a, b)) = e.span {
      if covered.is_some_and(|c| a > c) {
        spaced = true;
      }
      covered = Some(covered.map_or(b, |c| c.max(b)));
    }
    if !is_piece {
      continue;
    }
    let t = e.text.trim();
    if t.is_empty() {
      continue;
    }
    if spaced && !text.is_empty() {
      text.push(' ');
    }
    text.push_str(t);
    spaced = false;
  }
  text
}

/// The block's events with an IEEE template's ordinal before a name (`1\textsuperscript{st} Sertac Kilickaya`,
/// 2412.10792, 2202.06596) one Name with it, as today's personname "1 Sertac Kilickaya": the ordinal's superscript is
/// no request. On a column's first line and on no line a mark leads: anywhere else the ordinal is its line's text
/// (`Event::ordinal`, `events_text`), `5\textsuperscript{th} Floor` under a name.
pub(super) fn join_ordinals(events: &[Event]) -> Vec<Event> {
  let mut out: Vec<Event> = Vec::with_capacity(events.len());
  let mut i = 0;
  let mut names_line = true;
  let mut line_inked = false;
  while i < events.len() {
    let event = &events[i];
    match event.kind {
      EventKind::Break => {
        names_line = false;
        line_inked = false;
      },
      EventKind::Sep(SepKind::And(_) | SepKind::Column(_)) => {
        names_line = true;
        line_inked = false;
      },
      EventKind::Mark if !line_inked => {
        names_line = false;
        line_inked = true;
      },
      _ if inked(event) => line_inked = true,
      _ => {},
    }
    if names_line
      && event.kind == EventKind::Name
      && !event.text.is_empty()
      && event.text.chars().all(|c| c.is_ascii_digit())
      && let Some(mark) = events.get(i + 1)
      && mark.ordinal
      && let Some(name) = events.get(i + 2).filter(|e| e.kind == EventKind::Name)
    {
      out.push(Event {
        text: s!("{} {}", event.text, name.text),
        span: match (event.span, name.span) {
          (Some((a, _)), Some((_, b))) => Some((a, b)),
          (a, b) => a.or(b),
        },
        ..name.clone()
      });
      i += 3;
      continue;
    }
    out.push(event.clone());
    i += 1;
  }
  out
}

/// The block's events with each link printing a name read as that name. A link to an ORCID record that prints a name is that
/// name and its ORCID, the link after it (`contact_parts`): `\href{https://orcid.org/0000-0002-9970-3034}{E. L. Gagnon}$^{*,1,2}$,
/// …`, 2312.16289; orcidlink's `\orcidlinki{José-María Martín-Olalla}{0000-0002-3750-9113}`, 2602.03244; 2301.02927. Any other link
/// that prints other text than its own target and fills a name slot ([`name_slots`]) is that text, a Name carrying the link,
/// its span the link's call (written whole, with it): `\href{http://inspirehep.net/record/1021757}{A.~Cerri}$^{1}$`, 304 of them in
/// 1812.07638 (longtail3k), whose url contacts were one creator's affiliation under the emitter's switch; authblk's
/// `\author[1,2]{\href{kaizheng.wang@ntu.edu.sg}{Kaizheng Wang}\thanks{…}}`, 2602.22747, and an unmarked `\href{…}{Bob Baker}` between
/// two names' commas, each a nameless creator's url contact before. A link printing its own target is an address (`\url{…}`,
/// `\href{mailto:ann@x.org}{ann@x.org}`), one sharing its slot with other ink or on a later line carrying nothing a contact
/// (`Ann Able \\ \href{…}{Project Page}`).
pub(super) fn link_names(events: &[Event]) -> Vec<Event> {
  let slots = name_slots(events);
  let mut out = Vec::with_capacity(events.len());
  for (at, event) in events.iter().enumerate() {
    let link = event.kind == EventKind::Contact && !event.text.trim().is_empty();
    let orcid = link && contact_parts(event).0 == "orcid";
    let named = (orcid && reads_as_names(&event.text))
      || (link
        && !orcid
        && event.role.as_deref() == Some("url")
        && slots[at]
        && !event
          .target
          .as_deref()
          .is_some_and(|target| prints_its_target(&event.text, target)));
    if !named {
      out.push(event.clone());
      continue;
    }
    let mut name = Event::new(EventKind::Name);
    name.text = event.text.trim().to_string();
    name.span = event.span;
    name.nest = event.nest;
    out.push(name);
    if orcid {
      out.push(Event {
        text: String::new(),
        ..event.clone()
      });
    }
  }
  out
}

/// Per event, whether it is a link filling a name slot: the only ink of the piece between its line's separators (a comma, a
/// word, a gap, `\and`, an alignment tab, a break, the block's ends) but for the marks and notes a name carries, an inkless
/// link (a logo) and punctuation, on a line no mark leads (an entry, `$^{1}$\href{…}{Univ A}`, is no name), which is its
/// column's first line or a names line by its marks: the slot carries a mark or a note after the link, as a later line's
/// names do (`later_line_is_names`; 1812.07638's `Editors: \\ \href{…}{A.~Cerri}$^{1}$, …`).
fn name_slots(events: &[Event]) -> Vec<bool> {
  let mut out = vec![false; events.len()];
  let line_end = |e: &Event| {
    matches!(
      e.kind,
      EventKind::Break | EventKind::Sep(SepKind::And(_) | SepKind::Column(_))
    )
  };
  let link = |e: &Event| e.kind == EventKind::Contact && !e.text.trim().is_empty();
  let mut column_first = true;
  let mut start = 0;
  while start <= events.len() {
    let end = events[start..]
      .iter()
      .position(line_end)
      .map_or(events.len(), |p| start + p);
    let line = &events[start..end];
    let mark_led = line
      .iter()
      .find(|e| {
        inked(e) && !(e.kind == EventKind::Name && !e.text.chars().any(char::is_alphanumeric))
      })
      .is_some_and(|e| e.kind == EventKind::Mark);
    if !mark_led {
      let mut from = 0;
      while from <= line.len() {
        let to = line[from..]
          .iter()
          .position(|e| matches!(e.kind, EventKind::Sep(_)))
          .map_or(line.len(), |p| from + p);
        let slot = &line[from..to];
        let other_ink = |e: &&Event| match e.kind {
          EventKind::Name => e.text.chars().any(char::is_alphanumeric),
          EventKind::Contact => link(e),
          _ => false,
        };
        if let [at] = slot
          .iter()
          .enumerate()
          .filter(|(_, e)| other_ink(e))
          .map(|(i, _)| i)
          .collect::<Vec<_>>()
          .as_slice()
          && link(&slot[*at])
        {
          let carries = slot[at + 1..]
            .iter()
            .any(|e| (e.kind == EventKind::Mark && !e.line_first) || e.kind == EventKind::Note);
          out[start + from + at] = column_first || carries;
        }
        from = to + 1;
      }
    }
    if let Some(sep) = events.get(end) {
      column_first = sep.kind != EventKind::Break;
    }
    start = end + 1;
  }
  out
}

/// Whether a link's printed text is its own target, its address: the target, its scheme and a closing slash aside, in any
/// case (`\href{https://x.ai/}{X.ai}`, 2609.25082; `\href{mailto:ann@x.org}{ann@x.org}`).
fn prints_its_target(text: &str, target: &str) -> bool {
  let (text, target) = (
    text.trim().trim_end_matches('/'),
    target.trim().trim_end_matches('/'),
  );
  let past_scheme = target
    .split_once(':')
    .filter(|(scheme, _)| {
      !scheme.is_empty()
        && scheme
          .chars()
          .all(|c| c.is_ascii_alphanumeric() || "+.-".contains(c))
    })
    .map(|(_, rest)| rest.trim_start_matches('/'));
  text.eq_ignore_ascii_case(target)
    || past_scheme.is_some_and(|rest| text.eq_ignore_ascii_case(rest))
}

/// The block's events with a mark printed as text where a line opens: a Name whose first word is a symbol some name
/// shows as its mark (`\small\em$\sharp$ - CBCL, …` under `Mroueh$^{\sharp}$`, 1209.1360 — `♯` is no `\@fnsymbol`
/// character, so the lexer read it as ink) parted into that Mark and the rest.
pub(super) fn split_text_marks(events: &[Event]) -> Vec<Event> {
  let shown: Vec<String> = events
    .iter()
    .filter(|e| e.kind == EventKind::Mark && !e.line_first)
    .flat_map(|e| mark_labels(e.label.as_deref().unwrap_or("")))
    .filter(|l| !l.chars().any(char::is_alphanumeric))
    .collect();
  if shown.is_empty() {
    return events.to_vec();
  }
  let mut out = Vec::with_capacity(events.len());
  let mut line_start = true;
  for event in events {
    match event.kind {
      EventKind::Break | EventKind::Sep(SepKind::And(_) | SepKind::Column(_)) => line_start = true,
      EventKind::Name if line_start => {
        line_start = false;
        let (first, rest) = event
          .text
          .split_once(' ')
          .unwrap_or((event.text.as_str(), ""));
        if shown.iter().any(|l| l == first) {
          let mut mark = Event::new(EventKind::Mark);
          mark.label = Some(first.to_string());
          mark.text = first.to_string();
          // (a footnote symbol only when every character is one, as the lexer reads a mark: `#` and a `★` set in text
          // are marks of no `\@fnsymbol` kind — main-session decision 3 on FM S2a.2 r7, review revfms2a2_7 finding 6)
          mark.symbol = first.chars().all(is_symbol_char);
          mark.line_first = true;
          mark.role = Some("text".to_string());
          mark.span = event.span;
          mark.nest = event.nest;
          out.push(mark);
          let rest = rest.trim_start_matches(|c: char| c == '-' || c.is_whitespace());
          if !rest.is_empty() {
            out.push(Event {
              text: rest.to_string(),
              ..event.clone()
            });
          }
          continue;
        }
      },
      EventKind::Name | EventKind::Mark | EventKind::Contact | EventKind::Note => {
        line_start = false
      },
      _ => {},
    }
    out.push(event.clone());
  }
  out
}

/// The block's events with each run of footnote symbols written together (`§†`) that a line leads with and a name
/// shows read as one mark ([`Event::glued`]), as a run of one symbol (`††`) is: 0907.1243's `W.Y.~Ching$^{\S\dagger}$`
/// under the line `$^{\S\dagger}$Department of Physics, University of Missouri …` is that line's alone, not every `§`
/// and `†` carrier's, and the lines `$^{\S}$DuPont …`, `$^{\dagger}$Laboratory …` are not Ching's (main-session
/// decision 2 on FM S2a.2 r7). A run no line leads with stays the marks it is written of (`R.~Podgornik$^{\dagger\star}$`).
pub(super) fn glue_shown_runs(events: &[Event]) -> Vec<Event> {
  let run_of = |e: &Event| -> Option<String> {
    if e.kind != EventKind::Mark {
      return None;
    }
    let run: String = mark_text_symbols(e.label.as_deref().unwrap_or(""));
    let first = run.chars().next()?;
    (run.chars().count() >= 2 && run.chars().all(is_symbol_char) && run.chars().any(|c| c != first))
      .then_some(run)
  };
  let led: Vec<String> = events
    .iter()
    .filter(|e| e.line_first)
    .filter_map(run_of)
    .collect();
  let glued: Vec<String> = events
    .iter()
    .filter(|e| !e.line_first)
    .filter_map(run_of)
    .filter(|run| led.contains(run))
    .collect();
  events
    .iter()
    .map(|e| {
      let mut e = e.clone();
      e.glued = run_of(&e).is_some_and(|run| glued.contains(&run));
      e
    })
    .collect()
}

/// A mark's label as [`mark_labels`] reads its characters (trimmed, every asterisk form as `*`, the formula `\|`'s `∥`
/// as `‖`).
fn mark_text_symbols(label: &str) -> String {
  label
    .trim()
    .chars()
    .map(|c| match c {
      // (every asterisk form one key, user ruling 2026-11-02: `queue_digest::one_asterisk`)
      c if ASTERISK_FORMS.contains(&c) => '*',
      '\u{2225}' => '‖',
      c => c,
    })
    .collect()
}

/// Whether a Mark is the class's own request (`Resolver::class_requests`): its `\inst` (role `inst`, `inst parted`) or
/// a binding's request command (`request`) — not a mark printed by hand (`math`, `sup`, a footnote symbol).
pub(super) fn is_class_request(event: &Event) -> bool {
  event
    .role
    .as_deref()
    .is_some_and(|r| r.starts_with("inst") || r == "request")
}

/// A Mark event's labels: [`mark_labels`] of its label, a glued run ([`glue_shown_runs`]) one label.
pub(super) fn event_labels(event: &Event) -> Vec<String> {
  if event.glued {
    vec![mark_text_symbols(event.label.as_deref().unwrap_or(""))]
  } else {
    mark_labels(event.label.as_deref().unwrap_or(""))
  }
}

/// The labelled entries of a line whose marks trail their text (`Nanyang Technological University$^1$, Amazon$^2$`,
/// 2507.05750): each mark after a Name closes an entry of the ink since the entry before; the line must end in a mark.
pub(super) fn trailing_entries<'a>(line: &[&'a Event]) -> Vec<Entry<'a>> {
  let mut entries: Vec<Entry<'a>> = Vec::new();
  let mut run: Vec<&'a Event> = Vec::new();
  let mut closed = false;
  for event in line {
    match &event.kind {
      EventKind::Mark => {
        let labels = event_labels(event);
        if run.iter().any(|e| e.kind == EventKind::Name) {
          while run
            .last()
            .is_some_and(|e| matches!(e.kind, EventKind::Sep(SepKind::Comma)))
          {
            run.pop();
          }
          let symbol = event.symbol;
          entries.push(Entry {
            labels,
            symbol,
            compound: false,
            events: std::mem::take(&mut run),
          });
          closed = true;
        } else if let Some(entry) = entries.last_mut().filter(|_| closed) {
          entry.labels.extend(labels);
        } else {
          return Vec::new();
        }
      },
      EventKind::Layout | EventKind::Sep(SepKind::Gap { .. }) => {},
      EventKind::Sep(SepKind::Comma) if run.is_empty() => {},
      _ => {
        closed = false;
        run.push(event);
      },
    }
  }
  if run.iter().any(|e| inked(e)) {
    return Vec::new();
  }
  entries
}

/// The labelled entries of a mark-led line: each opened by a mark at the line's start or after a separator (a gap, a
/// comma), and the ink up to the next such mark; a mark right after an entry's ink is shown on it (no label of it).
pub(super) fn entries_of<'a>(line: &[&'a Event]) -> Vec<Entry<'a>> {
  let mut entries: Vec<Entry<'a>> = Vec::new();
  let mut after_sep = true;
  let mut last_ink: Option<&Event> = None;
  for event in line {
    match &event.kind {
      // (a number's suffix is the entry's text, `Event::ordinal`)
      EventKind::Mark if event.ordinal => {
        if let Some(entry) = entries.last_mut() {
          entry.events.push(event);
        }
        after_sep = false;
      },
      EventKind::Mark => {
        // (a mark set off from the ink before it by a space — the source positions between them — opens an entry of
        // its own: `$^1$Arizona State University $^2$Rider University`, 2601.20069; one glued to it is shown on it)
        if let (Some(prev), Some((start, _))) = (last_ink.and_then(|e| e.span), event.span)
          && start > prev.1
        {
          after_sep = true;
        }
        let labels = event_labels(event);
        let symbol =
          event.symbol || (!labels.is_empty() && labels.iter().all(|l| is_symbol_label(l)));
        // (one mark printing distinct symbols together, no separator between them: `$^{\dagger\ddagger}$`)
        let compound = labels.len() > 1
          && event
            .label
            .as_deref()
            .is_some_and(|l| l.trim() == labels.concat());
        let open = entries.last().is_some_and(|e| e.events.is_empty());
        if open {
          if let Some(entry) = entries.last_mut() {
            entry.labels.extend(labels);
            entry.symbol &= symbol;
            entry.compound = false;
          }
        } else if after_sep || entries.is_empty() {
          entries.push(Entry {
            labels,
            symbol,
            compound,
            events: Vec::new(),
          });
        }
        after_sep = false;
      },
      EventKind::Sep(SepKind::Gap { .. }) => after_sep = true,
      EventKind::Sep(SepKind::Comma) => {
        after_sep = true;
        if let Some(entry) = entries.last_mut() {
          entry.events.push(event);
        }
      },
      EventKind::Layout => {},
      // (punctuation opening an entry is no text of it: the `)` of `($^1$) Univ A`, 1004.4809)
      EventKind::Name
        if !event.text.chars().any(char::is_alphanumeric)
          && entries.last().is_some_and(|e| e.events.is_empty()) => {},
      _ => {
        after_sep = false;
        last_ink = Some(event);
        if let Some(entry) = entries.last_mut() {
          entry.events.push(event);
        }
      },
    }
    if event.kind == EventKind::Mark {
      last_ink = Some(event);
    }
  }
  let count = entries.len();
  for (i, entry) in entries.iter_mut().enumerate() {
    // (the separator before the next entry is no entry's: `Berkeley and $^2$National Bureau …`, 1810.07216; the line's
    // last entry keeps a word the next line continues, `Biocomputación and \\ Física …`, 1107.1186)
    while entry.events.last().is_some_and(|e| match e.kind {
      EventKind::Sep(SepKind::Comma) => true,
      EventKind::Sep(SepKind::Word(_)) => i + 1 < count,
      _ => false,
    }) {
      entry.events.pop();
    }
  }
  entries
}

/// Whether a text reads as names (the classify helpers: name-shaped parts, no institution word, no address).
pub(in crate::base_utilities) fn reads_as_names(text: &str) -> bool {
  !text.contains('@') && name_count(text) > 0 && !is_institution(text)
}

/// The lines of a numbered entry (`labelseq=`: one `\institute` with no `\and`) when every line's first ink is a Mark
/// — at the start, or after a Break event: aa's `\institute{\inst{1} Max-Planck-Institut … \\ \inst{2} University of
/// Illinois … \\ …}`, each line printed after its mark, 1003.3020 (aa class D of the stage-6 gate study, 5 papers;
/// 0904.2180) —, each line an entry of its own its mark labels, a line of contacts alone the entry's above it; empty
/// when fewer than two such entries, or a line opens with other ink (`Univ X \\ \email{…}`: one entry, its lines
/// continued). A mark mid-line opens nothing here
/// ([`mark_led_pieces`] reads those for an entry no class numbers).
pub(super) fn line_led_pieces(events: &[Event]) -> Vec<Vec<Event>> {
  let lines: Vec<Vec<Event>> = events
    .split(|e| e.kind == EventKind::Break)
    .map(<[Event]>::to_vec)
    .filter(|line| line.iter().any(inked))
    .collect();
  let led = |line: &[Event]| {
    line
      .iter()
      .find(|e| {
        inked(e) && !(e.kind == EventKind::Name && !e.text.chars().any(char::is_alphanumeric))
      })
      .is_some_and(|e| {
        e.kind == EventKind::Mark && e.label.as_deref().is_some_and(|l| !l.is_empty())
      })
  };
  // (a line of contacts alone continues the entry above it, as an unmarked line under a labelled one does (R12): the
  // institute's `\\ \email{cessen@phys.au.dk}` closing its last entry, 1507.05963)
  let contacts_only = |line: &[Event]| {
    line
      .iter()
      .filter(|e| {
        inked(e) && !(e.kind == EventKind::Name && !e.text.chars().any(char::is_alphanumeric))
      })
      .all(|e| e.kind == EventKind::Contact)
  };
  let mut pieces: Vec<Vec<Event>> = Vec::new();
  for line in lines {
    if led(&line) {
      pieces.push(line);
    } else if contacts_only(&line)
      && let Some(last) = pieces.last_mut()
    {
      last.push(Event::new(EventKind::Break));
      last.extend(line);
    } else {
      return Vec::new();
    }
  }
  if pieces.len() >= 2 {
    pieces
  } else {
    Vec::new()
  }
}

/// A names line's events with the e-mail addresses its Name runs print taken out (the run's words before its first
/// address stay the name, its label word no name's; a run of addresses alone goes), and those addresses. A group of
/// local parts is read first, whole ([`address_groups_whole`]).
pub(super) fn names_without_addresses(events: &[&Event]) -> (Vec<Event>, Vec<String>) {
  let events = address_groups_whole(events);
  let mut out = Vec::with_capacity(events.len());
  let mut addresses = Vec::new();
  for event in events {
    // (not a run an undefined command glued an address from, which no word of the source spells: `Bob\email{b@x.org}`,
    // a name as Perl's creator holds it, `Event::glued_undef`)
    if event.kind == EventKind::Name && !event.glued_undef && !addresses_in(&event.text).is_empty()
    {
      // (where its first run starts, the label word with it; else, or where a word before that run prints an address
      // the run reader does not part from its neighbours, at that word: `Kristiaan Pelckmans |kp@it.uu.se|`, OT1's
      // `|` an em dash glued to the address, 1006.4832 — the name had taken the address)
      let word = event
        .text
        .split_whitespace()
        .find(|w| !plain_addresses(w).is_empty())
        .and_then(|w| event.text.find(w))
        .unwrap_or(event.text.len());
      let run = address_runs(&event.text, &|_| false)
        .first()
        .map_or(event.text.len(), |run| run.start);
      let before = event.text[..word.min(run)].trim();
      addresses.extend(addresses_in(&event.text));
      if !before.is_empty() {
        out.push(Event {
          text: before.to_string(),
          ..event
        });
      }
      continue;
    }
    out.push(event);
  }
  (out, addresses)
}

/// A names line's events with each group of addresses whose printed braces hold the line's commas one Name run again,
/// as an address reader reads it: `\IEEEauthorblockN{Ann Able, … E-mails: \{ann, bob\}@a.org, cat@b.org}` parts at those
/// commas into `E-mails: {ann` and `bob}@a.org`, no names (2003.02512). Commas in braces that print no address part names
/// as they did.
fn address_groups_whole(events: &[&Event]) -> Vec<Event> {
  let mut out: Vec<Event> = Vec::with_capacity(events.len());
  let mut at = 0;
  while at < events.len() {
    let event = events[at];
    let mut joined = event.text.clone();
    let mut end = at;
    let in_braces = |e: &Event| e.kind == EventKind::Sep(SepKind::Comma) && e.braces > 0;
    while event.kind == EventKind::Name
      && let [comma, name, ..] = &events[end + 1..]
      && in_braces(comma)
      && name.kind == EventKind::Name
    {
      joined = s!("{joined}, {}", name.text);
      end += 2;
    }
    if end > at && !addresses_in(&joined).is_empty() {
      out.push(Event {
        text: joined,
        span: event
          .span
          .zip(events[end].span)
          .map(|((start, _), (_, stop))| (start, stop)),
        ..event.clone()
      });
      at = end + 1;
    } else {
      out.push(event.clone());
      at += 1;
    }
  }
  out
}

/// A names line's events with a byline's "by" (in any case) that a line opens with, a Name of its own the lexer keeps
/// as printed, read as the separator it is when the line's next Name reads as a name: the one owner of the rule since
/// the old reader's `without_leading_by` was deleted with its names split in Z18k2 ("`line` without the byline word
/// "by" (in any case) that opens it, when names follow it", `name_count(…) >= 1`, applied to every names line and only
/// there). A line opens after an `\and` or an alignment tab too, each author's lines as the old reader splits them
/// (`\and BY~~Max Moss~~and~~Ned Nash`). In the line or a run of its own, the spacing or the font after it parting it
/// from the names: `\author{by David Harari and Tam\'as Szamuely}`, 0808.2556; `By~~{\scshape
/// Vitaly~Bergelson}~~and~~…`, 2603.25922, 2604.07186, 2604.12864; `By~~{\scshape Bryna Kra}`, 2603.27258. A line the
/// reading reads as no names keeps it: `Ann Able \\ By\quad Lee Hall, \textit{Univ X}`'s affiliation, review revG1 1.
/// With the spans of the words it read so, which the reading records with their words
/// ([`Resolved::drops`](super::Resolved::drops)): the content check counts them as the separators' words they are, no
/// word lost.
fn byline_dropped(index: usize, mut events: Vec<Event>) -> (Vec<Event>, Vec<RuleDrop>) {
  let mut dropped = Vec::new();
  let ends_line =
    |e: &Event| matches!(e.kind, EventKind::Sep(SepKind::And(_) | SepKind::Column(_)));
  let mut line_inked = false;
  for at in 0..events.len() {
    let event = &events[at];
    if ends_line(event) {
      line_inked = false;
      continue;
    }
    if !inked(event) {
      continue;
    }
    let opens = !std::mem::replace(&mut line_inked, true);
    if !opens || event.kind != EventKind::Name || !event.text.trim().eq_ignore_ascii_case("by") {
      continue;
    }
    let named = events[at + 1..]
      .iter()
      .take_while(|e| !ends_line(e))
      .find(|e| e.kind == EventKind::Name && e.text.chars().any(char::is_alphanumeric))
      .is_some_and(|e| name_count(&e.text) >= 1);
    if named {
      let by = &events[at];
      let mut sep = Event::new(EventKind::Sep(SepKind::Word(by.text.trim().to_string())));
      sep.span = by.span;
      sep.nest = by.nest;
      dropped.extend(
        by.span
          .map(|span| (index, Some(span), by.text.trim().to_lowercase())),
      );
      events[at] = sep;
    }
  }
  (events, dropped)
}

/// A per-author class's names line as one Name when its pieces, parted only by commas and words, do not read as two
/// or more names (`name_count` on the line's text: `Sanhueza, Patricio` one name, `Ann Able, Bob Baker` two) and none
/// reads as an institution (that one is the name's affiliation, `classify:institution`).
pub(super) fn joined_single_name(events: Vec<Event>) -> Vec<Event> {
  let names: Vec<usize> = (0..events.len())
    .filter(|&i| {
      events[i].kind == EventKind::Name && events[i].text.chars().any(char::is_alphanumeric)
    })
    .collect();
  let (Some(&first), Some(&last)) = (names.first(), names.last()) else {
    return events;
  };
  if names.len() < 2
    || events[first..=last].iter().any(|e| {
      !matches!(
        e.kind,
        EventKind::Name | EventKind::Sep(SepKind::Comma | SepKind::Word(_))
      )
    })
    || names.iter().any(|&i| is_institution(&events[i].text))
  {
    return events;
  }
  let refs: Vec<&Event> = events[first..=last].iter().collect();
  let text = events_text(&refs);
  if name_count(&text) >= 2 {
    return events;
  }
  let joined = Event {
    text,
    span: match (events[first].span, events[last].span) {
      (Some((a, _)), Some((_, b))) => Some((a, b)),
      (a, b) => a.or(b),
    },
    ..events[first].clone()
  };
  let mut out = events[..first].to_vec();
  out.push(joined);
  out.extend_from_slice(&events[last + 1..]);
  out
}

/// Whether a Mark is a name annotation a remark holds, moved onto the name as the old reader's `name_takes_annotations`
/// moves every one of `NAME_ANNOTATIONS` (`classify/notes.rs`) and a `\footnotemark` (`classify::is_footnote_mark`):
/// a request (`\altaffilmark`, `\thanksref`, `\corref`, `\fnref`, `\tnoteref`), an `\inst` or `\IEEEauthorrefmark`
/// (`inst`), a `\textsuperscript` (`sup`), a `\cite`, a footnote mark, and a footnote symbol's superscript (`$^{*}$`,
/// `math`, the old reader's `\lx@frontmatter@keepsup`); not a number's formula superscript (`$^{1}$`) nor a glyph
/// printed as text, which the old reader keeps in the remark.
fn name_annotation(e: &Event) -> bool {
  e.kind == EventKind::Mark
    && e
      .role
      .as_deref()
      .is_some_and(|role| match role.split(' ').next().unwrap_or(role) {
        "request" | "inst" | "sup" | "cite" | "footnotemark" => true,
        "math" => e.symbol,
        _ => false,
      })
}

/// Whether a comma is the separator between two marks a mark command printed, `\fnmsep`'s, `\fnsep`'s or `\unskip$^,$`
/// (`events::mark_event`, its role the printing one's), which the old reader's `name_takes_annotations` moves onto the
/// name with the marks: `Ann Able (for the\inst{1}\fnmsep\thanks{x} ATLAS Collaboration)`, llncs, review revG1 2.
fn mark_separator(e: &Event) -> bool {
  matches!(e.kind, EventKind::Sep(SepKind::Comma)) && e.role.is_some()
}

/// A names line's pieces with a name's collaboration remark that notes, layout or a command's own gap part read whole
/// (rows C2-C4 of the old reader's names split, `defs/frontmatter_fields/names.rs` `name_and_remark` and
/// `name_takes_annotations`): from a piece a remark opens in ([`remark_start`]), the Name events of one run, parted
/// only by notes, name annotations ([`name_annotation`]), layout and gaps inside one command ([`gap_inside_one_command`]),
/// are one Name where their text is a name and its remark naming a collaboration — or, after a name, such a remark —
/// and the first alone is neither; their text joined as the source joins them ([`printed_join`]), the notes after it
/// the name's. A remark after the name's own event opens a run of its own, so the name keeps its event and the remark
/// a span of its own, which the emitter writes in its markup (`Kalanand Mishra\footnote{…} (for the \babar\
/// collaboration)`, the remark in a box, 0711.1544; `Ann Able\thanks{…} (for the \textsc{Atlas} Collaboration)`,
/// review revNL1 S1); a name whose own event opens the remark joins with it (`Christian Schmidt (for the
/// BNL-Bielefeld\fnref{col1} Collaboration)`, elsarticle, 1212.4278). A remark reads as a collaboration as
/// `reads_as_collaboration` reads it, or after the name's notes or marks as `speaks_for_after_notes` does (`for
/// CAPP/IBS`, 1702.03664).
pub(super) fn remark_pieces_joined(events: Vec<Event>) -> Vec<Event> {
  let refs: Vec<&Event> = events.iter().collect();
  let named = |e: &Event| e.kind == EventKind::Name && e.text.chars().any(char::is_alphanumeric);
  // (the remark a text holds after the name it reads as, or is after a name; `annotated` its name's notes or marks)
  let remark = |text: &str, after_name: bool, annotated: bool| {
    let reads = |rest: &str| {
      reads_as_collaboration(rest, false) || (annotated && speaks_for_after_notes(rest))
    };
    name_before_remark(text).is_some_and(|name| reads(text.trim()[name.len()..].trim()))
      || (after_name && is_name_remark(text) && reads(text))
      || (after_name && annotated && speaks_for_after_notes(text))
  };
  let mut out: Vec<Event> = Vec::with_capacity(events.len());
  let mut i = 0;
  while i < events.len() {
    // (a run starts where a remark opens: a piece no remark opens in is read as it is)
    if !named(&events[i]) || remark_start(&events[i].text).is_none() {
      out.push(events[i].clone());
      i += 1;
      continue;
    }
    // (a name before it in the line, and its notes or marks between)
    let before = refs[..i]
      .iter()
      .rposition(|e| named(e) || matches!(e.kind, EventKind::Sep(_) | EventKind::Break));
    let after_name = before.is_some_and(|b| named(refs[b]));
    let annotated = before.is_some_and(|b| {
      refs[b + 1..i]
        .iter()
        .any(|e| matches!(e.kind, EventKind::Note | EventKind::Mark))
    });
    let mut last = i;
    let mut j = i + 1;
    while j < events.len() {
      match &events[j].kind {
        EventKind::Layout | EventKind::Note => j += 1,
        EventKind::Mark if name_annotation(&events[j]) => j += 1,
        // (a mark separator, `\fnmsep`'s and `\fnsep`'s comma, moves with the marks it parts; a mark no annotation, a
        // number's `$^{1}$`, stays in the remark, which goes on past it: the old reader's `name_takes_annotations`, review
        // revG1 2)
        EventKind::Sep(SepKind::Comma) if mark_separator(&events[j]) => j += 1,
        EventKind::Mark => j += 1,
        EventKind::Sep(SepKind::Gap { .. })
          if j + 1 < events.len() && gap_inside_one_command(&refs, j + 1, refs[j]) =>
        {
          j += 1;
        },
        _ if named(&events[j]) => {
          last = j;
          j += 1;
        },
        _ => break,
      }
    }
    let run = &events[i..=last];
    let pieces = run.iter().filter(|e| named(e)).count();
    let alone = remark(&events[i].text, after_name, annotated);
    let text = printed_join(run, named);
    // (the notes inside it count as the name's when they part the name from its remark)
    let inside = run
      .iter()
      .any(|e| e.kind == EventKind::Note || name_annotation(e));
    if pieces < 2 || alone || !remark(&text, after_name, annotated || inside) {
      out.push(events[i].clone());
      i += 1;
      continue;
    }
    out.push(Event {
      text,
      span: match (events[i].span, events[last].span) {
        (Some((a, _)), Some((_, b))) => Some((a, b)),
        (a, b) => a.or(b),
      },
      ..events[i].clone()
    });
    out.extend(
      run
        .iter()
        .filter(|e| e.kind == EventKind::Note || name_annotation(e) || mark_separator(e))
        .cloned(),
    );
    i = last + 1;
  }
  out
}

/// Whether the Name event at `i` follows a Name with only a gap (and layout) between them.
pub(super) fn only_gap_between(events: &[&Event], i: usize) -> bool {
  let mut before = events[..i]
    .iter()
    .rev()
    .filter(|e| e.kind != EventKind::Layout);
  matches!(
    before.next().map(|e| &e.kind),
    Some(EventKind::Sep(SepKind::Gap { .. }))
  ) && before.next().is_some_and(|e| e.kind == EventKind::Name)
}

/// Whether a name read so far is no whole name yet: one word (a given name, `Takumi`), or ending in an initial
/// (`James B.`, `Sanhan M.S.`); a qualifier is whole.
pub(super) fn name_continues_at_gap(name: &str) -> bool {
  let words: Vec<&str> = name.split_whitespace().collect();
  match words.as_slice() {
    [one] => one.chars().any(char::is_alphabetic) && !is_name_qualifier(one),
    [.., last] => initials_only(last),
    [] => false,
  }
}

/// Whether a names line's piece is a collaboration listed among the authors (`and the MAST Team`, 1609.09534; `and the
/// VVDS team`, astro-ph0509500): "the" before a team, collaboration, consortium or group's name — no institution of the
/// name before it; the names line reads it as their collaboration (`reads_as_collaboration`, OXIDIZED_DESIGN_DIVERGENCES
/// #946; an author before batch 97d).
pub(super) fn is_collaboration(text: &str) -> bool {
  let t = text.trim().to_lowercase();
  let t = t.strip_prefix("and ").unwrap_or(&t).trim();
  t.starts_with("the ")
    && [
      "team",
      "collaboration",
      "consortium",
      "group",
      "collaborations",
    ]
    .iter()
    .any(|w| t.ends_with(w))
}

/// Where a remark on a name opens in a piece's text: at a bracket (`(on behalf of the H1 and ZEUS collaborations)`,
/// 0811.3142), or at the words of whom the name speaks for ([`SPEAKS_FOR`]) opening it or a word of it (`for the Hall A
/// DVCS collaboration`, 1502.03246; `Christian Schmidt (for the`, 1212.4278) — the byte of its first; `None` where no
/// remark opens. Where [`remark_pieces_joined`]'s runs start.
pub(super) fn remark_start(text: &str) -> Option<usize> {
  let t = text.trim_start();
  let skipped = text.len() - t.len();
  // (ASCII lowercase keeps every byte where it was)
  let lower = t.to_ascii_lowercase();
  let bracket = t.find(['(', '[']);
  let said = SPEAKS_FOR
    .iter()
    .filter_map(|w| match lower.strip_prefix(w) {
      Some(_) => Some(0),
      None => lower.find(&s!(" {w}")).map(|at| at + 1),
    })
    .min();
  [bracket, said]
    .into_iter()
    .flatten()
    .min()
    .map(|at| at + skipped)
}

/// Whether a names line's piece is a remark on the name before it, no name of its own: a bracket (`(on behalf of the H1
/// and ZEUS collaborations)`, 0811.3142), or the collaboration the name speaks for (`for the Hall A DVCS
/// collaboration`, 1502.03246; `on behalf of the ATLAS Collaboration`) — review rev94a B2, S6: a bracket at its first
/// byte, or "for" or "on behalf of" there before a collaboration ([`speaks_for`]).
pub(super) fn is_name_remark(text: &str) -> bool {
  let t = text.trim();
  t.starts_with(['(', '[']) || speaks_for(t)
}

/// Whether a text is one bracket and nothing after it prints a word: `(deceased)`, `[corresponding author]`, not
/// `[ver3]S. Curry`, `(Dee) Dorn` (a name after its remark; review revZ18k3 R1). A bracket never closed spans the text.
pub(super) fn bracket_spans_whole(text: &str) -> bool {
  let t = text.trim();
  let (open, close) = match t.chars().next() {
    Some('(') => ('(', ')'),
    Some('[') => ('[', ']'),
    _ => return false,
  };
  let mut depth = 0usize;
  for (at, c) in t.char_indices() {
    if c == open {
      depth += 1;
    } else if c == close {
      depth -= 1;
      if depth == 0 {
        return !t[at + c.len_utf8()..].chars().any(char::is_alphanumeric);
      }
    }
  }
  true
}

/// Whether a text names a collaboration framed as a remark on names printed before it ([`Framing::Framed`]: "for", "on
/// behalf of", "representing", brackets, dashes), as the class prints it (`On behalf of the KM3NeT Consortium`, 1201.5050).
pub(super) fn frames_the_names(text: &str) -> bool {
  !collaboration_heads(text) && collaboration_line_framing(text) == Some(Framing::Framed)
}

/// Whether `text`, on a line of `events`, is a collaboration framed for the names before it ([`frames_the_names`]) and the
/// line carries no mark, note or contact of its own (`[ATLAS Collaboration]\thanks{…}` stays a creator, review revP5b).
fn frames_the_names_alone(events: &[&Event], text: &str) -> bool {
  !events.iter().any(|e| {
    matches!(
      e.kind,
      EventKind::Mark | EventKind::Note | EventKind::Contact
    )
  }) && frames_the_names(text)
}

/// The text of a line printing a collaboration framed for the names before it and nothing else
/// ([`frames_the_names_alone`]): its names and separators as printed.
fn framed_collaboration(events: &[&Event]) -> Option<String> {
  let text = events_text(events);
  frames_the_names_alone(events, &text).then_some(text)
}

/// The source span from a line's first event to its last.
fn events_span(events: &[&Event]) -> Option<(u16, u16)> {
  let mut spans = events.iter().filter_map(|e| e.span);
  let first = spans.next()?;
  Some(spans.fold(first, |(a, b), (c, d)| (a.min(c), b.max(d))))
}

/// Whether a text is the collaboration a name speaks for: "for" or "on behalf of" ([`SPEAKS_FOR`]) before a
/// collaboration ([`is_collaboration`]: `for the Hall A DVCS collaboration`, 1502.03246; the article left out, `on
/// behalf of ATLAS Collaboration`).
pub(super) fn speaks_for(text: &str) -> bool {
  let lower = text.trim().to_lowercase();
  SPEAKS_FOR
    .iter()
    .find_map(|p| lower.strip_prefix(p))
    .is_some_and(|rest| is_collaboration(rest) || is_collaboration(&s!("the {rest}")))
}

/// The name a piece holds before a remark on it, where one follows it: a bracket (`Bob Baker (MIT)`) or the
/// collaboration it speaks for ([`speaks_for`]: `Bob Baker for the ATLAS Collaboration`, `… on behalf of the CMS
/// Team`) — `None` when no remark follows a name. A name with its remark is a name: the classify helpers read the
/// collaboration's word as an institution's, so `Ann Able and Bob Baker for the ATLAS Collaboration` gave Bob Baker to
/// Ann Able as her affiliation (review rev94a r2 B-1; 1502.03246's `for the Hall A DVCS collaboration`).
pub(in crate::base_utilities::frontmatter::observer) fn name_before_remark(
  text: &str,
) -> Option<&str> {
  let t = text.trim();
  // (ASCII lowercase keeps every byte where it was)
  let lower = t.to_ascii_lowercase();
  let bracket = t.find(['(', '[']);
  let said = [" for ", " on behalf of "]
    .iter()
    .flat_map(|p| lower.match_indices(p).map(|(at, _)| at))
    .filter(|&at| speaks_for(&t[at..]))
    .min();
  let at = [bracket, said].into_iter().flatten().min()?;
  let name = t[..at].trim_end();
  (!name.is_empty() && reads_as_names(name)).then_some(name)
}

/// Whether a piece may name a person, so that the reading continues no institution into it (review rev94a r2
/// B-1): it reads as names, or does once the remark after its name is set aside ([`name_before_remark`]), or its words
/// before its first institution word are a full name of two words or more (`Bob Baker University` — a person and a
/// place, or one place as `Johns Hopkins University` is). A word reading: it shapes the reading's `classify:institution`
/// items only, which license no writer operation (`writer::repart::by_words` (deleted), review rev94a r3); the writer reads it only
/// to decline (`emitter::repart::plan`'s person made an item, review rev94a r4).
pub(in crate::base_utilities::frontmatter::observer) fn may_name_a_person(text: &str) -> bool {
  if reads_as_names(text) || name_before_remark(text).is_some() {
    return true;
  }
  let words: Vec<&str> = text.split_whitespace().collect();
  let Some(first) = words.iter().position(|w| non_name_word(w)) else {
    return false;
  };
  let head = words[..first].join(" ");
  first >= 2 && name_count(&head) > 0
}

/// Whether a name is initials alone (`V.G.`, `A. B.`): each word a capital with at most one small letter and a dot.
pub(super) fn initials_only(name: &str) -> bool {
  let words: Vec<&str> = name.split_whitespace().collect();
  !words.is_empty()
    && words.iter().all(|w| {
      w.split_inclusive('.').all(|part| {
        let letters: Vec<char> = part.trim_end_matches('.').chars().collect();
        part.ends_with('.')
          && letters.first().is_some_and(|c| c.is_uppercase())
          && letters.len() <= 2
          && letters.iter().skip(1).all(|c| c.is_lowercase())
      })
    })
}

/// Whether the Name event at `i` continues the Name before it: only layout events (a container, a font change)
/// between them.
pub(super) fn joins_previous(events: &[&Event], i: usize) -> bool {
  let between: Vec<&&Event> = events[..i]
    .iter()
    .rev()
    .take_while(|e| e.kind == EventKind::Layout)
    .collect();
  !between.is_empty()
    && !between.iter().any(|e| is_image(e))
    && events[..i - between.len()]
      .last()
      .is_some_and(|e| e.kind == EventKind::Name)
}

/// Whether a Name joins the initials at `i` as the next piece of their name: past a Layout ([`joins_previous`]), or past a
/// gap, where initials alone do not end a name (the gap rule: `V.G.~ Kagramanova`, 0802.4349; `F. M.\, Munteanu`,
/// cond-mat9905290).
fn next_name_joins(events: &[&Event], i: usize) -> bool {
  let mut next = (i + 1..events.len()).filter(|&j| events[j].kind != EventKind::Layout);
  match next.next() {
    Some(j) if events[j].kind == EventKind::Name => j > i + 1 && joins_previous(events, j),
    Some(j) if matches!(events[j].kind, EventKind::Sep(SepKind::Gap { .. })) => next
      .next()
      .is_some_and(|k| events[k].kind == EventKind::Name),
    _ => false,
  }
}

/// Whether `gap`, the separator before the Name at `i`, is inside a word: one source token (a command) printed it with
/// the end of the Name before and the start of the Name at `i`, each of which runs on past that token — a glyph a
/// document builds of boxes in the middle of a name (`S\OW N`, 1706.00234). A gap of two calls' spaces (`\redact{Ann
/// Able} \redact{Bob Baker}`, 2605.30896) or a macro printing two whole names (`Ann Able\quad Bob Baker` from one
/// `\authorlist`) is a separator.
pub(super) fn gap_inside_one_command(events: &[&Event], i: usize, gap: &Event) -> bool {
  let EventKind::Sep(SepKind::Gap { .. }) = gap.kind else {
    return false;
  };
  let before = events[..i]
    .iter()
    .rev()
    .find(|e| !matches!(e.kind, EventKind::Layout | EventKind::Sep(_)))
    .filter(|e| e.kind == EventKind::Name);
  match (before.and_then(|e| e.span), gap.span, events[i].span) {
    (Some((before_start, before_end)), Some((gap_start, gap_end)), Some((start, end))) => {
      gap_end == gap_start + 1
        && before_start < gap_start
        && before_end == gap_end
        && start == gap_start
        && end > gap_end
    },
    _ => false,
  }
}

/// Whether a Layout event is an image (a logo, an affiliation marker: 2503.08161; review revfms2a1_3): it parts the
/// runs beside it, as a separator, never joins them into one name.
pub(super) fn is_image(event: &Event) -> bool {
  event.kind == EventKind::Layout
    && event
      .role
      .as_deref()
      .is_some_and(|r| r.contains("includegraphics") || r.contains("graphic"))
}

/// Whether the Name event at `i` is a lone placement letter (`t`, `b`, `c`, `l`, `r`) next to a box (a Layout event).
pub(super) fn is_placement_letter(events: &[&Event], i: usize) -> bool {
  matches!(events[i].text.as_str(), "t" | "b" | "c" | "l" | "r")
    && (i
      .checked_sub(1)
      .is_some_and(|j| events[j].kind == EventKind::Layout)
      || events
        .get(i + 1)
        .is_some_and(|e| e.kind == EventKind::Layout))
}

/// The names of a run of events (an entry of a prefix-marked line): its Name events' texts.
pub(super) fn names_of(events: &[&Event]) -> Vec<String> {
  events
    .iter()
    .filter(|e| e.kind == EventKind::Name)
    .map(|e| e.text.clone())
    .collect()
}

/// Whether the Mark at `i`, after a separator (`last_sep`) and a name before it, goes to the next name: it stands
/// between the separator and the next name nearer to that name.
fn mark_goes_next(events: &[&Event], i: usize, last_sep: Option<&Event>, policy: &Policy) -> bool {
  let event = events[i];
  let next_is_name = events[i + 1..]
    .iter()
    .find(|e| !matches!(e.kind, EventKind::Layout | EventKind::Mark))
    .is_some_and(|e| e.kind == EventKind::Name);
  // (a mark between a separator and the next name stands with the one it is nearer to in the source: `Ann,$^{1}$
  // Bob` the name before, as revtex writes it, 0705.3409; `Ann$^1$, $^{2}$Bob` the next, a prefix — the spans'
  // distances, the math shift counted)
  let next_start = events[i + 1..]
    .iter()
    .find(|e| e.kind == EventKind::Name)
    .and_then(|e| e.span)
    .map(|s| s.0);
  // (TeX's output decides first, where it sets the mark apart on one side only: `Han,${}^1$ Thibault` prints
  // "Han,¹ Thibault", the mark the name before's — measured in the source from the script, the empty base `{}`
  // put it nearer the next name, 1603.00635; else the source distances)
  let nearer_next = match (event.space_before, event.space_after) {
    (false, true) => false,
    (true, false) => true,
    _ => match (last_sep.and_then(|s| s.span), event.span, next_start) {
      (Some((_, sep_end)), Some((start, end)), Some(next)) => {
        next.saturating_sub(end) < start.saturating_sub(sep_end)
      },
      _ => false,
    },
  };
  let inst = event.role.as_deref().is_some_and(|r| r.starts_with("inst"));
  last_sep.is_some() && next_is_name && nearer_next && !(inst && policy.inst_after_comma)
}

/// Whether the Name at `i` of a names line is a marked name (rows N1-N3, the reading's since the old reader's names split,
/// `marked_names`, was deleted in Z18k2): it carries its own mark — the next event past
/// layout and notes, or past the comma after it a mark that stays with it ([`mark_goes_next`]: `Teng Wu,$^{1,*}$`,
/// revtex) —, and on a per-author class's line (`cautious`, the old reader's cautious split) its institution word is a
/// surname after an initial ([`surname_after_initial`]: `Gary P. Centers,$^{1}$`, 1901.10843, which no address is —
/// the old `!only_addresses` test holds in it: it refuses a text holding an `@`). The
/// generic split of any other names line never reads an institution word (`R. New$^2$`, 0904.4122; `S. Michelle
/// Consiglio,$^{1}$`, 2103.02641; `Alexander New$^2$`, 2311.16860). An unmarked piece keeps the reading's institution
/// (`John Smith, University of Toronto`).
pub(super) fn marked_name(events: &[&Event], i: usize, cautious: bool, policy: &Policy) -> bool {
  let mut after = (i + 1..events.len())
    .filter(|&j| !matches!(events[j].kind, EventKind::Layout | EventKind::Note));
  let carried = match after.next().map(|j| (j, &events[j].kind)) {
    Some((j, EventKind::Mark)) => !events[j].line_first,
    Some((comma, EventKind::Sep(SepKind::Comma))) => after.next().is_some_and(|j| {
      events[j].kind == EventKind::Mark
        && !events[j].line_first
        && !mark_goes_next(events, j, Some(events[comma]), policy)
    }),
    _ => false,
  };
  carried && (!cautious || surname_after_initial(&events[i].text))
}

impl Resolver {
  /// A names line: names split at its separators, marks attached by adjacency, notes and contacts to the name before.
  pub(super) fn names_line(
    &mut self,
    index: usize,
    policy: &'static Policy,
    cautious: bool,
    line: &Line<'_>,
    block_requests: &[String],
    group: usize,
  ) -> NamesLine {
    // (the addresses a names line prints as text are no names: `\name Shengjian Chen \email {chensj@…, chshengj@…}`,
    // jmlr2e's `\email` a font switch, 2507.03885 — each an e-mail of the line's names by the owner chain, placed once
    // the line's names are read; the words before an address in its run stay the name)
    let (owned, addresses) = names_without_addresses(&line.events);
    let (owned, dropped) = byline_dropped(index, owned);
    self.out.drops.extend(dropped);
    // (a per-author class's names line that does not read as two or more names is one name as written: aastex's
    // `\author{Sanhueza, Patricio}`, 2604.19097 — `split_author_line_cautious`, `name_count` on the line's text)
    let owned = if cautious {
      joined_single_name(owned)
    } else {
      owned
    };
    let owned = remark_pieces_joined(owned);
    let line = &Line {
      column:      line.column,
      events:      owned.iter().collect(),
      after_blank: line.after_blank,
      group_break: line.group_break,
    };
    let made = self.names_line_events(index, policy, cautious, line, block_requests, group);
    if !addresses.is_empty() {
      let source = (index, line.events.first().and_then(|e| e.span));
      self.place_line_contacts(
        &addresses.join(", "),
        "email",
        &made.creators,
        source,
        None,
        Chain::Line,
      );
    }
    made
  }

  /// [`Resolver::names_line`] over a line's events once its addresses are out.
  pub(super) fn names_line_events(
    &mut self,
    index: usize,
    policy: &'static Policy,
    cautious: bool,
    line: &Line<'_>,
    block_requests: &[String],
    group: usize,
  ) -> NamesLine {
    let mut made: Vec<usize> = Vec::new();
    // the separator before the next name, and whether a mark after a comma waits for the next name
    let mut last_sep: Option<&Event> = None;
    // (the membership contact a grade after a name's comma made: its creator, the event, the contact)
    let mut grade_at: Option<(usize, usize, usize)> = None;
    let mut waiting: Vec<String> = Vec::new();
    // (of them, the labels a `\footnotemark` printed: `fn_requests`)
    let mut waiting_fn: Vec<String> = Vec::new();
    // (and those the class's own request command made: `class_requests`)
    let mut waiting_class: Vec<String> = Vec::new();
    let mut waiting_parted: Vec<String> = Vec::new();
    let mut waiting_notes: Vec<&Event> = Vec::new();
    // the mark before, where it ends in the source and whether it went to the next name
    let mut last_mark: Option<(u16, bool)> = None;
    // the institution a names line read as the affiliation of the name before (`classify:institution`): its creator and
    // place, which the pieces after it continue
    let mut institution: Option<(usize, usize)> = None;
    // (the names a collaboration of the line covers start here in `made`, and whether the piece just read was one, whose
    // mark is no name's: OXIDIZED_DESIGN_DIVERGENCES #946)
    let mut uncovered = 0usize;
    let mut after_collaboration = false;
    // (the names the last collaboration covered, which one right after it covers too; a heading waiting for the names
    // after it, with where they start)
    let mut last_covered: Vec<usize> = Vec::new();
    let mut heading: Option<(String, usize)> = None;
    let listed = |sep: Option<&Event>| {
      sep.is_some_and(|s| matches!(&s.kind, EventKind::Sep(SepKind::Word(word)) if word.eq_ignore_ascii_case("and") || word == "&"))
    };
    let split_rule = |sep: Option<&Event>| match sep.map(|e| &e.kind) {
      Some(EventKind::Sep(SepKind::Comma)) => "line:names sep:comma",
      Some(EventKind::Sep(SepKind::Word(_))) => "line:names sep:word",
      Some(EventKind::Sep(SepKind::Gap { .. })) => "line:names sep:gap",
      Some(EventKind::Sep(SepKind::And(_))) => "line:names sep:and",
      _ => "line:names",
    };
    let events = &line.events;
    // (a line printing only a collaboration framed for the names before it is theirs, its words as digested: a macro, a
    // `\let` copy, a font group or its own "and" — `\author{\obo{} the KM3NeT Consortium}`, 1201.5050, segaudit col1)
    if let Some(text) = framed_collaboration(events)
      && !self.uncovered_creators().is_empty()
    {
      let covered = self.uncovered_creators();
      self.collaboration(index, events_span(events), &covered, &text);
      return NamesLine { creators: made };
    }
    for (i, event) in events.iter().enumerate() {
      match &event.kind {
        EventKind::Name if !event.text.chars().any(char::is_alphanumeric) => {},
        // (a mark printed as text after a name is its request, no name: `A.P.~Orlov$^1$, 2,` — the "2" a mark the
        // source typed outside the math, 1912.02075)
        EventKind::Name
          if !made.is_empty()
            && event.text.chars().any(|c| c.is_ascii_digit())
            && event
              .text
              .chars()
              .all(|c| c.is_ascii_digit() || ", ".contains(c))
            && event.text.len() <= 6 =>
        {
          let c = *made.last().expect("a name");
          self.out.creators[c]
            .requests
            .extend(mark_labels(&event.text));
          last_sep = None;
        },
        // (a gap the command printing the ink on both its sides made, inside its expansion, is no space the source put
        // between two names: Vietnamese accents drawn with `\raise\hbox{…}` by a document's `\EES`, `\OW` —
        // `TI\EES N-S\OW N PH\d{A}M`, 1706.00234, parted "TIÊ ´N-SO | . N PHẠM" at `\OW`'s spacing)
        EventKind::Name
          if !made.is_empty()
            && last_sep.is_some_and(|gap| gap_inside_one_command(events, i, gap)) =>
        {
          let c = *made.last().expect("a name");
          let creator = &mut self.out.creators[c];
          creator.name = s!("{} {}", creator.name, event.text);
          creator.join(NameJoin::Macro);
          last_sep = None;
        },
        // (a lone placement letter beside a box is no name: `shown_arguments` reads every argument as shown when a
        // definition's parameter and argument counts differ, util_text.rs:374 — review revfms2a1_3; flagged by its rule)
        EventKind::Name if is_placement_letter(events, i) => {
          if let Some(&c) = made.last() {
            let creator = &mut self.out.creators[c];
            if !creator.rule.contains("flag:placement-letter") {
              creator.rule.push_str(" flag:placement-letter");
            }
          }
        },
        // (a name run a layout container alone parts is one name: revtex's `\firstname{Francesco}
        // \surname{Parisen Toldin}`, 1303.6104; a font switch or a box inside a name never separates, decision 6d)
        // (a name of initials alone does not end at a gap: `V.G.~ Kagramanova`, 0802.4349, a tie and a space)
        // (nor does a given name alone, or a name ending in an initial, with nothing but the gap after it: `Takumi~
        // Saegusa`, 1406.5580; `Nazlı\ \ Demirer`, 1612.07367; `James B.~ Martin`, 1806.07838 — a full name is two
        // words or more; the pieces were the wide-space XML-wrong class's by mistake)
        EventKind::Name
          if !made.is_empty()
            && last_sep.is_some_and(|s| matches!(s.kind, EventKind::Sep(SepKind::Gap { .. })))
            && (initials_only(&self.out.creators[*made.last().expect("a name")].name)
              || (only_gap_between(events, i)
                && name_continues_at_gap(
                  &self.out.creators[*made.last().expect("a name")].name,
                ))) =>
        {
          let c = *made.last().expect("a name");
          let creator = &mut self.out.creators[c];
          let join = if initials_only(&creator.name) {
            NameJoin::Initials
          } else {
            NameJoin::Gap
          };
          creator.name = s!("{} {}", creator.name, event.text);
          creator.join(join);
          last_sep = None;
        },
        EventKind::Name if joins_previous(events, i) && !made.is_empty() => {
          let c = *made.last().expect("a name");
          let creator = &mut self.out.creators[c];
          creator.name = s!("{} {}", creator.name, event.text);
          creator.join(NameJoin::Layout);
        },
        // (initials after a comma and a surname are that name's, written surname first: `Santucho, V., Luparello, H.
        // E., Lares, M.`, mn2e, 2005.00053; `Blanchet, J., Glynn, P. and Thorisson, H.`, 1503.08374; `Della Valle, M.`,
        // 1609.07297 — each piece was a creator)
        EventKind::Name
          if !made.is_empty()
            && last_sep.is_some_and(|s| s.kind == EventKind::Sep(SepKind::Comma))
            && initials_only(&event.text)
            && !event.text.chars().any(char::is_lowercase)
            && {
              // (a surname alone before it, a word or two and no initial: not `X.W. Shu, D. Elbaz`, 1512.00167; nor a
              // degree after a name, `Luluk Muthoharoh, S.Si., M.Si.`, 2605.07811, whose initials hold small letters)
              // (each word read through its glued initials, as `name_shaped` reads it: `P.A.Vshivtseva$^{a,b}$,
              // E.T.\, Einiev`, 2409.13569, is two names)
              let before = &self.out.creators[*made.last().expect("a name")].name;
              let words: Vec<&str> = before
                .split_whitespace()
                .flat_map(unglued_initials)
                .collect();
              !before.contains(',') && words.len() <= 2 && words.iter().all(|w| !initials_only(w))
            }
            // (the initials end their piece: a name a gap or a layout joins to them is theirs, a name of its own after the
            // comma — `Yongmin\, Kim$^*$, F. M.\, Munteanu$^{*,\dag}$`, cond-mat9905290, two people)
            && !next_name_joins(events, i) =>
        {
          let c = *made.last().expect("a name");
          let creator = &mut self.out.creators[c];
          creator.name = s!("{}, {}", creator.name, event.text);
          creator.join(NameJoin::Reversed);
          last_sep = None;
        },
        // (a collaboration among the names, or a remark on a name naming one, is a contact on the names of the line
        // since its last collaboration, no name and no part of one — `… and the GAPS Team`, `J.~Zapatero$^{5}$ (The
        // MAGIC Collaboration)`, as today's reader reads it: `collaboration_split`, OXIDIZED_DESIGN_DIVERGENCES #946)
        // (a call or a column of its own speaking for one, under the names before it, too: `\author{on behalf of the KM3NeT
        // consortium}`, 1201.5726; `\author{for the IceCube collaboration}`, 1301.6619 — elsarticle, printed under them; the
        // names since the last collaboration, no name. An unframed one stays a creator, revtex's `\collaboration{The HERMES
        // Collaboration}` (its `\author{#1}`): 0704.3133, gr-qc0505029)
        EventKind::Name
          if (!made.is_empty()
            || ((speaks_for(&event.text) || frames_the_names_alone(events, &event.text))
              && !self.uncovered_creators().is_empty()))
            && reads_as_listed_collaboration(&event.text, listed(last_sep)) =>
        {
          let text = event
            .text
            .trim()
            .trim_end_matches(':')
            .trim_end()
            .to_string();
          if collaboration_heads(&event.text) {
            heading = Some((text, made.len()));
          } else {
            let covered = if made.is_empty() && last_covered.is_empty() {
              self.uncovered_creators()
            } else {
              made[uncovered.min(made.len())..].to_vec()
            };
            let covered = if covered.is_empty() {
              last_covered.clone()
            } else {
              covered
            };
            self.collaboration(index, event.span, &covered, &text);
            last_covered = covered;
          }
          uncovered = made.len();
          after_collaboration = true;
          last_sep = None;
        },
        // (a remark after a name's notes or marks naming whom the author speaks for is the names' collaboration, no name
        // and no part of one, row C2 (`speaks_for_after_notes`): `Ann Able\inst{1}\fnsep\thanks{…} for CAPP/IBS`, webofc,
        // 1702.03664)
        EventKind::Name
          if !made.is_empty()
            && speaks_for_after_notes(&event.text)
            && events[..i]
              .iter()
              .rev()
              .take_while(|e| e.kind != EventKind::Name)
              .any(|e| matches!(e.kind, EventKind::Note | EventKind::Mark)) =>
        {
          let covered = made[uncovered.min(made.len())..].to_vec();
          self.collaboration(index, event.span, &covered, event.text.trim());
          last_covered = covered;
          uncovered = made.len();
          after_collaboration = true;
          last_sep = None;
        },
        // (a qualifier after a comma is the name's: `Robert W. Heath, Jr.`, `Matthew Nokleby, Student Member, IEEE`,
        // printed after the name, comma-separated — name_text.rs `is_name_qualifier`; 1106.5253, 1203.0695)
        // (and so is a remark on the name after it, a bracket or a collaboration it speaks for: `Ann Able$^1$ (on
        // behalf of the A and B teams)`, `Maxime Defurne \\ for the Hall A DVCS collaboration`, 0811.3142, 1502.03246 —
        // review rev94a B2, S6)
        // (but a bracketed piece after a separator carrying its own mark is a name of its own, marks win (charter §4 R1):
        // `Y. Dong\affil{6}, \add[ver3]{S. Curry}\affil{1}` printing "[ver3]S. Curry¹", trackchanges' `\add` undefined
        // (OmniBus's `\authors`, 2308.02720); llncs's `Bob Baker\inst{2}, [v3]{Cy Cole}\inst{1}` — each joined to the
        // name before it with both affiliations; review revZ18k B1. A join of two marked names left is the safety net's
        // counted Warning, `net::merged_names`)
        // (only one printing words after its bracket closes: a piece the bracket spans whole is a remark still, whose
        // join is that Warning — llncs's `Ann Able\inst{1}, (deceased)\inst{2}, Bob Baker\inst{2} and [corresponding
        // author]\inst{1}` made "(deceased)" and "[corresponding author]" creators, review revZ18k3 R1;
        // `bracket_spans_whole`)
        EventKind::Name
          if !made.is_empty()
            && (is_name_qualifier(&event.text)
              || (is_name_remark(&event.text)
                && !(last_sep.is_some()
                  && !speaks_for(&event.text)
                  && !bracket_spans_whole(&event.text)
                  && marked_name(events, i, false, policy)))) =>
        {
          let c = *made.last().expect("a name");
          // (but a membership grade written after the comma is the author's `membership` contact, no part of the name
          // today's names print: `Tamara Munzner, \textit{Senior Member}, \textit{IEEE}`, 2009.02373 — as
          // `\lx@add@author` reads it, OXIDIZED_DESIGN_DIVERGENCES #967)
          if is_membership_part(&event.text) {
            // (its parts the commas cut, `Senior Member` and `IEEE`, one contact)
            match grade_at {
              Some((owner, at, contact)) if owner == c && i <= at + 2 => {
                let text = &mut self.out.creators[c].contacts[contact].text;
                *text = s!("{text}, {}", event.text);
              },
              _ => {
                let contact = self.out.creators[c].contacts.len();
                self.attach(
                  Some(c),
                  Assoc {
                    role:   "membership".into(),
                    text:   event.text.clone(),
                    label:  None,
                    rule:   "name:membership".into(),
                    source: Some((index, event.span)),
                    target: None,
                  },
                  false,
                );
                grade_at = Some((c, i, contact));
                last_sep = None;
                continue;
              },
            }
            grade_at = grade_at.map(|(owner, _, contact)| (owner, i, contact));
            last_sep = None;
            continue;
          }
          let creator = &mut self.out.creators[c];
          let glue = if last_sep.is_some_and(|s| s.kind == EventKind::Sep(SepKind::Comma)) {
            ", "
          } else {
            " "
          };
          creator.name = s!("{}{glue}{}", creator.name, event.text);
          creator.join(NameJoin::Qualifier);
          last_sep = None;
        },
        // (an institution in a row of names is the affiliation of the name before it: `Chen Jia, SI-TECH Information
        // Technology`, 2406.02959; the classify helpers, `reads_as_affiliation`)
        // (a name with a remark after it is no institution, whatever word the remark holds: `Ann Able and Bob Baker
        // for the ATLAS Collaboration`, Bob Baker an author as today's — review rev94a r2 B-1)
        // (a marked name is a name whatever its words, rows N1-N3: [`marked_name`])
        EventKind::Name
          if !made.is_empty()
            && is_institution(&event.text)
            && !is_collaboration(&event.text)
            && name_before_remark(&event.text).is_none()
            && !marked_name(events, i, cautious, policy) =>
        {
          let c = *made.last().expect("a name");
          institution = Some((c, self.out.creators[c].affiliations.len()));
          self.attach(
            Some(c),
            Assoc {
              role:   "affiliation".into(),
              text:   event.text.clone(),
              label:  None,
              rule:   "line:names classify:institution".into(),
              source: Some((index, event.span)),
              target: None,
            },
            false,
          );
          last_sep = None;
        },
        // (a piece after the word "and" that ends an institution and may name no person is the institution's:
        // `Department of Physics and Astronomy` — review rev94a B1; an author after it may, `Univ X and Bob Baker`, `Univ
        // X and Bob Baker for the CMS Collaboration`, r2 B-1)
        EventKind::Name
          if institution.is_some()
            && last_sep.is_some_and(|s| matches!(s.kind, EventKind::Sep(SepKind::Word(_))))
            && !may_name_a_person(&event.text) =>
        {
          if let Some((c, at)) = institution
            && let Some(affiliation) = self.out.creators[c].affiliations.get_mut(at)
            && let Some(EventKind::Sep(SepKind::Word(word))) = last_sep.map(|s| &s.kind)
          {
            affiliation.text = s!("{} {word} {}", affiliation.text, event.text);
          }
          last_sep = None;
        },
        // (a piece after an institution's comma that may name no person is the institution's address, as today's
        // one affiliation: `Frank Schadt, Pforzheim University of Applied Sciences, Germany`, 1106.0130; `Ilse Fischer,
        // Institut für Mathematik der Universität Klagenfurt, Universitätsstrasse 65-67, A-9020 Klagenfurt, Austria`,
        // math0112263 — the street, the town and the country were authors)
        EventKind::Name
          if institution.is_some()
            && last_sep.is_some_and(|s| s.kind == EventKind::Sep(SepKind::Comma))
            && !may_name_a_person(&event.text) =>
        {
          if let Some((c, at)) = institution
            && let Some(affiliation) = self.out.creators[c].affiliations.get_mut(at)
          {
            affiliation.text = s!("{}, {}", affiliation.text, event.text);
          }
          last_sep = None;
        },
        EventKind::Name => {
          institution = None;
          after_collaboration = false;
          // (and a creator call's own `annotations=`, Perl's labels of the creator it makes, which a separate call's
          // `label=` answers: `\lx@add@creator[role=editor,annotations={ed}]{Bob Baker}` and
          // `\lx@add@affiliation[label=ed]{Editorial Office}`, given to Ann Able, the block's, before)
          let own = event
            .creator_call
            .and_then(|k| self.call_labels.get(usize::from(k)))
            .cloned()
            .unwrap_or_default();
          let requests = [std::mem::take(&mut waiting), block_requests.to_vec(), own].concat();
          // (a name and the collaboration it speaks for: `Bob Baker for the ATLAS Collaboration`, the name's alone — not a
          // heading before a colon, `JLQCD Collaboration: S.~Aoki$^1$` (hep-lat0212039), which the reading cannot tell
          // from one inside a box today's keeps whole, `\parbox{\textwidth}{\Large{ MAGIC Collaboration: K.~Abe$^{1}$, …}}`
          // (2601.03873), its names the writer's to re-part)
          let remark = name_before_remark(&event.text).and_then(|name| {
            let rest = event.text.trim()[name.len()..].trim();
            reads_as_collaboration(rest, false).then(|| (name.to_string(), rest.to_string()))
          });
          let name = remark
            .as_ref()
            .map_or_else(|| event.text.clone(), |(name, _)| name.clone());
          let c = self.new_creator(name, split_rule(last_sep), index, requests, group);
          self.out.creators[c].role.clone_from(&event.role);
          self.out.creators[c].call = event.creator_call;
          made.push(c);
          if let Some((_, rest)) = &remark {
            let covered = made[uncovered.min(made.len())..].to_vec();
            self.collaboration(index, event.span, &covered, rest);
            last_covered = covered;
            uncovered = made.len();
          }
          self
            .fn_requests
            .extend(std::mem::take(&mut waiting_fn).into_iter().map(|l| (c, l)));
          self.class_requests.extend(
            std::mem::take(&mut waiting_class)
              .into_iter()
              .chain(block_requests.iter().cloned())
              .map(|l| (c, l)),
          );
          self.parted.extend(
            std::mem::take(&mut waiting_parted)
              .into_iter()
              .map(|l| (c, l)),
          );
          for note in std::mem::take(&mut waiting_notes) {
            self.note(index, note, Some(c), "note:adjacent");
          }
          last_sep = None;
        },
        EventKind::Sep(_) => last_sep = Some(event),
        EventKind::Mark => {
          let labels = mark_requests(event);
          let after_comma = last_sep.is_some_and(|s| s.kind == EventKind::Sep(SepKind::Comma));
          let inst = event.role.as_deref().is_some_and(|r| r.starts_with("inst"));
          // (a mark glued to the mark before it goes where that one went: `Oh,$^{1}$$^{,2}$ Nordlander`, mn2e,
          // 2306.15492 — the second no nearer the next name than the first; a math mark's span is inside its shifts,
          // so at most the two shifts `$$` lie between)
          let glued = match (last_mark, event.span) {
            (Some((end, to_next)), Some((start, _))) if start >= end && start <= end + 2 => {
              Some(to_next)
            },
            _ => None,
          };
          let to_next =
            glued.unwrap_or_else(|| made.is_empty() || mark_goes_next(events, i, last_sep, policy));
          last_mark = event.span.map(|(_, end)| (end, to_next));
          let footnote = event
            .role
            .as_deref()
            .is_some_and(|r| r.starts_with("footnotemark"));
          // (a mark the lexer parted from the footnote mark glued to it, `\inst{1, 2\footnotemark[1]}` printing
          // "1, 2∗": ruling 1's reading, the marks shown apart — 2404.09857)
          // — and so is a label glued to a symbol in one written piece, `\inst{1*}` printing "1*" (mark 1 and the
          // corresponding author's `*`, 2506.20360): only the labels so parted
          let parted: Vec<String> = if event
            .role
            .as_deref()
            .is_some_and(|r| r.ends_with(" parted"))
          {
            labels.clone()
          } else if event.glued {
            Vec::new()
          } else {
            glued_symbol_parts(event.label.as_deref().unwrap_or(""))
          };
          if !parted.is_empty() {
            let owner = if to_next { None } else { made.last().copied() };
            if let Some(c) = owner {
              self.parted.extend(parted.into_iter().map(|l| (c, l)));
            } else {
              waiting_parted.extend(parted);
            }
          }
          let class = is_class_request(event);
          // (a mark on a collaboration is no name's: `… and LHD experimental group$^{2}$`, 1711.07097)
          if after_collaboration && !to_next {
            continue;
          }
          if to_next {
            if footnote {
              waiting_fn.extend(labels.iter().cloned());
            }
            if class {
              waiting_class.extend(labels.iter().cloned());
            }
            waiting.extend(labels);
          } else if let Some(&c) = made.last() {
            if footnote {
              self
                .fn_requests
                .extend(labels.iter().map(|l| (c, l.clone())));
            }
            if class {
              self
                .class_requests
                .extend(labels.iter().map(|l| (c, l.clone())));
            }
            self.out.creators[c].requests.extend(labels);
            if after_comma {
              let rule = if inst && policy.inst_after_comma {
                "mark:inst-after-comma"
              } else {
                "mark:after-comma"
              };
              let creator = &mut self.out.creators[c];
              if !creator.rule.contains(rule) {
                creator.rule.push(' ');
                creator.rule.push_str(rule);
              }
            }
          }
        },
        EventKind::Note => match made.last() {
          Some(&c) => self.note(index, event, Some(c), "note:adjacent"),
          None => waiting_notes.push(event),
        },
        EventKind::Contact => {
          // (an unlabelled annotation of the run of new authors, `annotate=run` or `new` — a class's `\affil` printed
          // in place inside `\author`, rerouted to `\lx@add@affiliation[annotate=run]`: `JORGE RAMIREZ and EVGENY
          // KRIVOSHEEV \affil{University of Trento} MARCOS BAEZ and …`, ci2018, 1805.12376 —, the run's names of the
          // line as today's API attaches it (`Resolver::record_run`), not the name beside it alone)
          // (the run ends at the block's contact of the role before it, labelled or not: a labelled one waits for its
          // label, its owner holding no role yet — `ANN ABLE$^1$ \affil{$^1$Univ A} BOB BAKER \affil{Univ B}` gives
          // Univ B to Bob alone, review rev94a r6 B1; `Resolver::ends_a_run`)
          let role = contact_parts(event).0;
          let run: Vec<usize> = match event.annotate.as_deref() {
            Some(annotate)
              if event.label.is_none() && (annotate == "new" || annotate.starts_with("run")) =>
            {
              let run = if annotate == "new" {
                self.new_run(&role)
              } else {
                self.record_run(annotate, &role, false)
              };
              run.into_iter().filter(|c| made.contains(c)).collect()
            },
            _ => Vec::new(),
          };
          self.run_from.insert(role, self.out.creators.len());
          if run.len() > 1 {
            self.contact_event(index, event, &run, "owner:annotate:run");
          } else {
            let owner: Vec<usize> = made.last().copied().into_iter().collect();
            self.contact_event(index, event, &owner, "owner:adjacent");
          }
        },
        _ => {},
      }
    }
    for note in waiting_notes {
      self.note(index, note, made.last().copied(), "note:adjacent");
    }
    // (a heading over the names after it: those of the line)
    if let Some((text, from)) = heading
      && let Some(first) = events.first()
    {
      let covered = made[from.min(made.len())..].to_vec();
      self.collaboration(index, first.span, &covered, &text);
    }
    if !waiting.is_empty()
      && let Some(&c) = made.last()
    {
      self.out.creators[c].requests.extend(waiting);
      self
        .fn_requests
        .extend(waiting_fn.into_iter().map(|l| (c, l)));
      self
        .class_requests
        .extend(waiting_class.into_iter().map(|l| (c, l)));
      self
        .parted
        .extend(waiting_parted.into_iter().map(|l| (c, l)));
    }
    NamesLine { creators: made }
  }

  /// The live creators of the document no collaboration covers yet: what a call of its own naming one covers (a replaced
  /// call's creators print nothing: the class's `\author{Ann}\author{for the X Collaboration}` prints only the second).
  fn uncovered_creators(&self) -> Vec<usize> {
    (self.collaboration_from..self.out.creators.len())
      .filter(|c| !self.replaced.contains(c))
      .collect()
  }

  /// A collaboration a names line reads (`reads_as_collaboration`): a `collaboration` contact on each creator it
  /// covers, as today's reader gives it (`Collaborations`, OXIDIZED_DESIGN_DIVERGENCES #946), sourced at `span`, the
  /// ink printing it.
  pub(super) fn collaboration(
    &mut self,
    index: usize,
    span: Option<(u16, u16)>,
    covered: &[usize],
    text: &str,
  ) {
    self.collaboration_from = self.out.creators.len();
    for &c in covered {
      self.attach(
        Some(c),
        Assoc {
          role:   "collaboration".into(),
          text:   text.to_string(),
          label:  None,
          rule:   "line:names collaboration".into(),
          source: Some((index, span)),
          target: None,
        },
        false,
      );
    }
  }
}
