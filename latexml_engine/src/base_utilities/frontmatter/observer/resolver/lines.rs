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

/// Whether an event is ink a line's type is read from (not layout, not a separator).
pub(super) fn inked(event: &Event) -> bool {
  matches!(
    event.kind,
    EventKind::Name | EventKind::Mark | EventKind::Contact | EventKind::Note
  )
}

/// The block's events with an IEEE template's ordinal before a name (`1\textsuperscript{st} Sertac Kilickaya`,
/// 2412.10792, 2202.06596) one Name with it, as today's personname "1 Sertac Kilickaya": the ordinal's superscript is
/// no request.
pub(super) fn join_ordinals(events: &[Event]) -> Vec<Event> {
  let mut out: Vec<Event> = Vec::with_capacity(events.len());
  let mut i = 0;
  while i < events.len() {
    let event = &events[i];
    if event.kind == EventKind::Name
      && !event.text.is_empty()
      && event.text.chars().all(|c| c.is_ascii_digit())
      && let Some(mark) = events.get(i + 1)
      && mark.kind == EventKind::Mark
      && matches!(mark.label.as_deref(), Some("st" | "nd" | "rd" | "th"))
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

/// The block's events with each link to an ORCID record that prints a name read as that name and its ORCID, the link
/// after it (`contact_parts`): `\href{https://orcid.org/0000-0002-9970-3034}{E. L. Gagnon}$^{*,1,2}$, …`, 2312.16289;
/// orcidlink's `\orcidlinki{José-María Martín-Olalla}{0000-0002-3750-9113}`, 2602.03244; 2301.02927.
pub(super) fn orcid_link_names(events: &[Event]) -> Vec<Event> {
  let mut out = Vec::with_capacity(events.len());
  for event in events {
    if event.kind == EventKind::Contact
      && contact_parts(event).0 == "orcid"
      && !event.text.trim().is_empty()
      && reads_as_names(&event.text)
    {
      let mut name = Event::new(EventKind::Name);
      name.text = event.text.trim().to_string();
      name.span = event.span;
      name.nest = event.nest;
      out.push(name);
      out.push(Event {
        text: String::new(),
        ..event.clone()
      });
    } else {
      out.push(event.clone());
    }
  }
  out
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

/// A mark's label as [`mark_labels`] reads its characters (trimmed, the math asterisk `∗` as `*`, the formula `\|`'s `∥`
/// as `‖`).
fn mark_text_symbols(label: &str) -> String {
  label
    .trim()
    .chars()
    .map(|c| match c {
      '\u{2217}' => '*',
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
pub(in crate::base_utilities::frontmatter::observer) fn reads_as_names(text: &str) -> bool {
  !text.contains('@') && name_count(text) > 0 && !reads_as_affiliation(&text_tokens(text))
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

/// The entries of an API call's content whose first ink is a mark, when two or more marks open one — a mark at its
/// start, after a line break or after a separator (a comma, a gap, a word) with ink since the entry before (`$^{*}$Univ
/// A,\\$^{+}$Univ B`, cond-mat0510562; `$^1$SJTU, $^2$Beihang`, 1809.03738): each entry its events up to the next,
/// the separators between them dropped. A mark glued to the ink before it (`Univ A$^{1}$`) opens none. Empty when the
/// content is one entry.
pub(super) fn mark_led_pieces(events: &[Event]) -> Vec<Vec<Event>> {
  let first_ink = events.iter().find(|e| {
    inked(e) && !(e.kind == EventKind::Name && !e.text.chars().any(char::is_alphanumeric))
  });
  if first_ink.is_none_or(|e| e.kind != EventKind::Mark) {
    return Vec::new();
  }
  // (a mark glued to the ink before it that labels as the leading mark does — a number after numbers, a letter after
  // letters — with a name right after it opens the next entry: `\institution{$^{1}$University of Illinois …, USA
  // $^{2}$Amazon.com Inc, CA, USA}` lexed "USA" "2" "Amazon.com Inc" — the call's content has no positions to show the
  // space —, 2202.06129, given whole to the first mark's authors; `ECT$^*$`'s symbol and BaBar's `$^a$` sub-labels after
  // numbers stay the entry's)
  let kind = |e: &Event| -> Option<bool> {
    let labels = mark_labels(e.label.as_deref().unwrap_or(""));
    if e.symbol || labels.is_empty() {
      None
    } else if labels.iter().all(|l| l.chars().all(|c| c.is_ascii_digit())) {
      Some(true)
    } else if labels
      .iter()
      .all(|l| l.chars().count() <= 2 && l.chars().all(char::is_alphabetic))
    {
      Some(false)
    } else {
      None
    }
  };
  let lead_kind = first_ink.and_then(kind);
  let name_follows = |at: usize| {
    events[at + 1..]
      .iter()
      .find(|e| e.kind != EventKind::Layout)
      .is_some_and(|e| {
        e.kind == EventKind::Name && e.text.chars().next().is_some_and(char::is_alphabetic)
      })
  };
  let mut pieces: Vec<Vec<Event>> = vec![Vec::new()];
  let mut after_sep = true;
  let mut ink = false;
  for (at, event) in events.iter().enumerate() {
    match &event.kind {
      EventKind::Mark
        if ink
          && !after_sep
          && lead_kind.is_some()
          && kind(event) == lead_kind
          && name_follows(at) =>
      {
        pieces.push(vec![event.clone()]);
        ink = false;
        continue;
      },
      EventKind::Mark if after_sep && ink => {
        pieces.push(vec![event.clone()]);
        ink = false;
        after_sep = false;
        continue;
      },
      EventKind::Mark => {},
      EventKind::Break | EventKind::Sep(_) => {
        after_sep = true;
        pieces.last_mut().expect("a piece").push(event.clone());
        continue;
      },
      EventKind::Layout => {},
      // (a piece of no letter or digit, punctuation or a zero-width space, is no ink between a separator and the mark
      // after it: `\\ \and ​$^2$Department …`, 2112.13115)
      EventKind::Name if !event.text.chars().any(char::is_alphanumeric) => {
        pieces.last_mut().expect("a piece").push(event.clone());
        continue;
      },
      _ => ink = true,
    }
    after_sep = matches!(event.kind, EventKind::Layout) && after_sep;
    pieces.last_mut().expect("a piece").push(event.clone());
  }
  for piece in &mut pieces {
    while piece.last().is_some_and(|e| {
      matches!(e.kind, EventKind::Break | EventKind::Sep(_))
        || (e.kind == EventKind::Name && !e.text.chars().any(char::is_alphanumeric))
    }) {
      piece.pop();
    }
  }
  if pieces.len() < 2 { Vec::new() } else { pieces }
}

/// A names line's events with the e-mail addresses its Name runs print taken out (the run's words before its first
/// address stay the name; a run of addresses alone goes), and those addresses.
pub(super) fn names_without_addresses(events: &[&Event]) -> (Vec<Event>, Vec<String>) {
  let mut out = Vec::with_capacity(events.len());
  let mut addresses = Vec::new();
  for event in events {
    if event.kind == EventKind::Name && !addresses_in(&event.text).is_empty() {
      let words: Vec<&str> = event.text.split_whitespace().collect();
      let first = words
        .iter()
        .position(|w| !plain_addresses(w).is_empty())
        .unwrap_or(words.len());
      addresses.extend(addresses_in(&event.text));
      if first > 0 {
        out.push(Event {
          text: words[..first].join(" "),
          ..(*event).clone()
        });
      }
      continue;
    }
    out.push((*event).clone());
  }
  (out, addresses)
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
    || names
      .iter()
      .any(|&i| reads_as_affiliation(&text_tokens(&events[i].text)))
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
/// VVDS team`, astro-ph0509500): "the" before a team, collaboration, consortium or group's name — an author, as today's.
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

/// Whether a names line's piece is a remark on the name before it, no name of its own: a bracket (`(on behalf of the H1
/// and ZEUS collaborations)`, 0811.3142), or the collaboration the name speaks for (`for the Hall A DVCS
/// collaboration`, 1502.03246; `on behalf of the ATLAS Collaboration`) — review rev94a B2, S6.
pub(super) fn is_name_remark(text: &str) -> bool {
  let t = text.trim();
  t.starts_with('(') || t.starts_with('[') || speaks_for(t)
}

/// Whether a text is the collaboration a name speaks for: "for" or "on behalf of" before a collaboration
/// ([`is_collaboration`]: `for the Hall A DVCS collaboration`, 1502.03246; the article left out, `on behalf of ATLAS
/// Collaboration`).
pub(super) fn speaks_for(text: &str) -> bool {
  let lower = text.trim().to_lowercase();
  ["for ", "on behalf of "]
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

/// Whether a piece may name a person, so that the shadow reading continues no institution into it (review rev94a r2
/// B-1): it reads as names, or does once the remark after its name is set aside ([`name_before_remark`]), or its words
/// before its first institution word are a full name of two words or more (`Bob Baker University` — a person and a
/// place, or one place as `Johns Hopkins University` is). A word reading: it shapes the reading's `classify:institution`
/// items only, which license no writer operation (`writer::repart::by_words`, review rev94a r3).
pub(super) fn may_name_a_person(text: &str) -> bool {
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
    // (a per-author class's names line that does not read as two or more names is one name as written: aastex's
    // `\author{Sanhueza, Patricio}`, 2604.19097 — `split_author_line_cautious`, `name_count` on the line's text)
    let owned = if cautious {
      joined_single_name(owned)
    } else {
      owned
    };
    let line = &Line {
      column:      line.column,
      events:      owned.iter().collect(),
      after_blank: line.after_blank,
      group_break: line.group_break,
    };
    let made = self.names_line_events(index, policy, line, block_requests, group);
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
    line: &Line<'_>,
    block_requests: &[String],
    group: usize,
  ) -> NamesLine {
    let mut made: Vec<usize> = Vec::new();
    // the separator before the next name, and whether a mark after a comma waits for the next name
    let mut last_sep: Option<&Event> = None;
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
    let split_rule = |sep: Option<&Event>| match sep.map(|e| &e.kind) {
      Some(EventKind::Sep(SepKind::Comma)) => "line:names sep:comma",
      Some(EventKind::Sep(SepKind::Word(_))) => "line:names sep:word",
      Some(EventKind::Sep(SepKind::Gap { .. })) => "line:names sep:gap",
      Some(EventKind::Sep(SepKind::And(_))) => "line:names sep:and",
      _ => "line:names",
    };
    let events = &line.events;
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
          if !creator.rule.contains("name:macro-joined") {
            creator.rule.push_str(" name:macro-joined");
          }
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
          let tag = if initials_only(&creator.name) {
            "name:initials-joined"
          } else {
            "name:gap-joined"
          };
          creator.name = s!("{} {}", creator.name, event.text);
          if !creator.rule.contains(tag) {
            creator.rule.push(' ');
            creator.rule.push_str(tag);
          }
          last_sep = None;
        },
        EventKind::Name if joins_previous(events, i) && !made.is_empty() => {
          let c = *made.last().expect("a name");
          let creator = &mut self.out.creators[c];
          creator.name = s!("{} {}", creator.name, event.text);
          if !creator.rule.contains("name:layout-joined") {
            creator.rule.push_str(" name:layout-joined");
          }
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
              let before = &self.out.creators[*made.last().expect("a name")].name;
              !before.contains(',')
                && before.split_whitespace().count() <= 2
                && before.split_whitespace().all(|w| !initials_only(w))
            } =>
        {
          let c = *made.last().expect("a name");
          let creator = &mut self.out.creators[c];
          creator.name = s!("{}, {}", creator.name, event.text);
          if !creator.rule.contains("name:reversed") {
            creator.rule.push_str(" name:reversed");
          }
          last_sep = None;
        },
        // (a qualifier after a comma is the name's: `Robert W. Heath, Jr.`, `Matthew Nokleby, Student Member, IEEE`,
        // printed after the name, comma-separated — name_text.rs `is_name_qualifier`; 1106.5253, 1203.0695)
        // (and so is a remark on the name after it, a bracket or a collaboration it speaks for: `Ann Able$^1$ (on
        // behalf of the A and B teams)`, `Maxime Defurne \\ for the Hall A DVCS collaboration`, 0811.3142, 1502.03246 —
        // review rev94a B2, S6)
        EventKind::Name
          if !made.is_empty()
            && (is_name_qualifier(&event.text) || is_name_remark(&event.text)) =>
        {
          let c = *made.last().expect("a name");
          let creator = &mut self.out.creators[c];
          let glue = if last_sep.is_some_and(|s| s.kind == EventKind::Sep(SepKind::Comma)) {
            ", "
          } else {
            " "
          };
          creator.name = s!("{}{glue}{}", creator.name, event.text);
          if !creator.rule.contains("name:qualifier") {
            creator.rule.push_str(" name:qualifier");
          }
          last_sep = None;
        },
        // (an institution in a row of names is the affiliation of the name before it: `Chen Jia, SI-TECH Information
        // Technology`, 2406.02959; the classify helpers, `reads_as_affiliation`)
        // (a name with a remark after it is no institution, whatever word the remark holds: `Ann Able and Bob Baker
        // for the ATLAS Collaboration`, Bob Baker an author as today's — review rev94a r2 B-1)
        EventKind::Name
          if !made.is_empty()
            && reads_as_affiliation(&text_tokens(&event.text))
            && !is_collaboration(&event.text)
            && name_before_remark(&event.text).is_none() =>
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
          let requests = [std::mem::take(&mut waiting), block_requests.to_vec()].concat();
          let c = self.new_creator(
            event.text.clone(),
            split_rule(last_sep),
            index,
            requests,
            group,
          );
          made.push(c);
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
          let labels = spelled_ranges(event_labels(event));
          let next_is_name = events[i + 1..]
            .iter()
            .find(|e| !matches!(e.kind, EventKind::Layout | EventKind::Mark))
            .is_some_and(|e| e.kind == EventKind::Name);
          let after_comma = last_sep.is_some_and(|s| s.kind == EventKind::Sep(SepKind::Comma));
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
          // (a mark glued to the mark before it goes where that one went: `Oh,$^{1}$$^{,2}$ Nordlander`, mn2e,
          // 2306.15492 — the second no nearer the next name than the first; a math mark's span is inside its shifts,
          // so at most the two shifts `$$` lie between)
          let glued = match (last_mark, event.span) {
            (Some((end, to_next)), Some((start, _))) if start >= end && start <= end + 2 => {
              Some(to_next)
            },
            _ => None,
          };
          let to_next = glued.unwrap_or_else(|| {
            made.is_empty()
              || (last_sep.is_some()
                && next_is_name
                && nearer_next
                && !(inst && policy.inst_after_comma))
          });
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
          let owner: Vec<usize> = made.last().copied().into_iter().collect();
          self.contact_event(index, event, &owner, "owner:adjacent");
        },
        _ => {},
      }
    }
    for note in waiting_notes {
      self.note(index, note, made.last().copied(), "note:adjacent");
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
}
