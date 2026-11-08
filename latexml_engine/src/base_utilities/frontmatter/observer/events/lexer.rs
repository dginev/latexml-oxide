//! The lexer's second pass (OXIDIZED_DESIGN_DIVERGENCES #837): the walk's atoms — ink, spacing items, finished
//! events — into the block's events: name runs split at the words "and" and `&` and at a separator glyph standing alone
//! (`is_separator_word`), footnote-symbol characters and the other mark glyphs (`is_mark_glyph`) a Mark, commas a
//! separator, a spacing run a Gap when it separates, a font change between runs a Layout event.

use super::*;

/// The lexer's second pass: the atoms into events.
#[derive(Default)]
pub(super) struct Lexer {
  pub(super) events:   Vec<Event>,
  /// the name run being read: its characters with their spans, its font, its nest
  run:                 Vec<(char, Option<(u16, u16)>)>,
  /// each of its characters' own token position, beside `run`: a box's cover is wider than what it prints (`Event::marks`)
  run_own:             Vec<Option<u16>>,
  /// the font of the `@` it prints
  address_font:        Option<Rc<Font>>,
  run_font:            Option<FontKey>,
  run_nest:            u8,
  /// the spacing run since the last ink: items, width, fill, span
  gap_items:           usize,
  gap_width:           f64,
  gap_fill:            bool,
  gap_span:            Option<(u16, u16)>,
  /// the spacing character the run opened with, when it is one printed character so far (a space or a tie)
  gap_char:            Option<char>,
  /// the line has shown ink (a name, a mark, a contact) since its last break
  line_inked:          bool,
  /// the font of the last name run
  last_font:           Option<FontKey>,
  /// the symbol mark being read (consecutive footnote-symbol characters and mark glyphs)
  symbol:              String,
  symbol_span:         Option<(u16, u16)>,
  /// a printed space before the symbol mark being read
  symbol_space:        bool,
  /// what encloses the symbol mark being read: its first character's base, its last's end
  symbol_formula:      Option<MarkFormula>,
  /// the brackets `(`, `[` printed and not yet closed since the line's start, and how many were open where the name run
  /// being read began: inside one a comma or the word "and" is the bracket's text, no separator (`Laurent Schoeffel (on
  /// behalf of the H1 and ZEUS collaborations)`, 0811.3142; review rev94a B2)
  open:                u8,
  run_open:            u8,
  /// the braces printed and not yet closed since the column's start ([`Event::braces`])
  braces:              u8,
  /// which lines (between Break events) print an `@`: their names' characters' positions are recorded (`inks`; a list
  /// part before the address, `jhansen, msandeep,dudek@cim.mcgill.ca`, 1811.10103) — no other's, a thousand-author names line
  /// none (1901.01540)
  pub(super) at_lines: Vec<bool>,
  line:                usize,
  /// where the names on those lines print, and the font of each `@` they print ([`InkTable`]): the name run's `@`s, each at
  /// its token's position, kept when the run is flushed on such a line
  pub(super) inks:     InkTable,
  run_at_fonts:        Vec<(u16, Rc<Font>)>,
}

impl Lexer {
  fn gap_is_separator(&self) -> bool {
    self.gap_fill || self.gap_items >= 2 || (self.gap_items == 1 && self.gap_width > INTERWORD)
  }

  /// Whether the pending gap is the word space of letter-spaced text: one spacing item narrower than a quad after a
  /// run of letters each set apart (two or more single letters, the run's last word one letter, with or without its
  /// period) — jfm's `F\ls I\ls R\ls S\ls T\ns A.\ns L\ls A\ls S\ls T` (jfm.cls:1024-1025, `\ls` `\kern.15em`, `\ns`
  /// `\kern.5em`) spells one name, its `\ns` the space between its words (1510.04254, 1403.3574, astro-ph0406083: each
  /// name read as two or three). Initials (`A. B. C.`) are no letter-spaced run: their letters carry periods.
  fn letter_spaced_word_space(&self) -> bool {
    if self.gap_fill || self.gap_items != 1 || self.gap_width >= 1.0 {
      return false;
    }
    let text: String = self.run.iter().map(|(c, _)| *c).collect();
    let words: Vec<&str> = text.split(' ').filter(|w| !w.is_empty()).collect();
    let letter = |w: &str| w.chars().count() == 1 && w.chars().all(char::is_alphabetic);
    let Some(last) = words.last() else {
      return false;
    };
    let last_letter = letter(last) || letter(last.trim_end_matches('.'));
    last_letter && words.iter().filter(|w| letter(w)).count() >= 2
  }

  fn clear_gap(&mut self) {
    self.gap_items = 0;
    self.gap_width = 0.0;
    self.gap_fill = false;
    self.gap_span = None;
    self.gap_char = None;
  }

  /// Emit the pending gap when it separates (else it is dropped: an interword space at a boundary).
  fn flush_gap(&mut self) {
    if self.gap_is_separator() {
      let mut gap = Event::new(EventKind::Sep(SepKind::Gap {
        items: self.gap_items,
        width: (self.gap_width * 1000.0).round() / 1000.0,
        fill:  self.gap_fill,
      }));
      gap.span = self.gap_span;
      self.events.push(gap);
    }
    self.clear_gap();
  }

  fn flush_symbol(&mut self) {
    if self.symbol.is_empty() {
      return;
    }
    let mut mark = Event::new(EventKind::Mark);
    mark.text = std::mem::take(&mut self.symbol);
    mark.label = Some(mark.text.clone());
    // (a footnote symbol only when every character is one: a mark glyph, `⋄`, is a mark of no `\@fnsymbol` kind)
    mark.symbol = mark.text.chars().all(is_fnsymbol);
    mark.role = Some("text".to_string());
    mark.span = self.symbol_span.take();
    mark.space_before = std::mem::take(&mut self.symbol_space);
    mark.formula = self.symbol_formula.take();
    mark.line_first = !self.line_inked;
    self.line_inked = true;
    self.events.push(mark);
  }

  /// End the name run: a Name, split at the words "and" and `&` and at a separator glyph standing alone (Sep events:
  /// `Chiara Simeoni -- \url{…}`'s `--`, 2511.07187), each piece with its own span. A dash inside a word stays as typed
  /// (`F.~M\"uller--S\'anchez`, 2003.01448; the XML prints the ligature's `–`): the classify helpers today's split
  /// shares read `--` as a hyphen and an en dash as no letter — `H. Strecker and H. V. Klapdor--Kleingrothaus`, an
  /// elsarticle per-author line, read as one name with the en dash (hep-ex9811040, FM S2a.2 r7's census).
  fn flush_run(&mut self) {
    let run = std::mem::take(&mut self.run);
    let run_own = std::mem::take(&mut self.run_own);
    let font = self.run_font.take();
    // the run's words, each with the span of its characters, and where each prints on a line printing an `@` (`InkTable`)
    let record = self.at_lines.get(self.line).copied().unwrap_or(false);
    let at_fonts = std::mem::take(&mut self.run_at_fonts);
    if record {
      self.inks.fonts.extend(at_fonts);
    }
    let mut words: Vec<(String, Option<(u16, u16)>)> = Vec::new();
    let mut owns: Vec<Option<(u16, u16)>> = Vec::new();
    let mut inks: Vec<Vec<Option<InkAt>>> = Vec::new();
    let mut word = String::new();
    let mut word_span = None;
    let mut word_own = None;
    let mut word_ink = Vec::new();
    for ((c, span), own) in run
      .into_iter()
      .zip(run_own)
      .chain(std::iter::once(((' ', None), None)))
    {
      if c == ' ' {
        if !word.is_empty() {
          words.push((std::mem::take(&mut word), word_span.take()));
          owns.push(word_own.take());
          if record {
            inks.push(std::mem::take(&mut word_ink));
          }
        }
      } else {
        word.push(c);
        word_span = join_span(word_span, span);
        word_own = join_span(word_own, own.map(|own| (own, own + 1)));
        if record {
          word_ink.push(span.map(|cover| InkAt { cover, own }));
        }
      }
    }
    if words.is_empty() {
      return;
    }
    if font.is_some() && self.last_font.is_some() && font != self.last_font {
      let mut layout = Event::new(EventKind::Layout);
      layout.role = Some("font".to_string());
      layout.nest = self.run_nest;
      self.events.push(layout);
    }
    // (the font of its first character, or of the address it prints behind a label word at most, `E-mail: \\texttt{a@x}`: the
    // emitter's, 0802.3384)
    let address_font = self.address_font.take().filter(|_| words.len() <= 3);
    let ink_font = address_font.or(font.as_ref().map(|font| Rc::clone(&font.0)));
    if font.is_some() {
      self.last_font = font;
    }
    // (the brackets open at each word's start: a word inside one is the bracket's, no separator)
    let mut open = self.run_open;
    let mut opens = Vec::with_capacity(words.len());
    for (word, _) in &words {
      opens.push(open);
      open = word.chars().fold(open, bracket_depth);
    }
    let mut name: Option<Event> = None;
    // (where its characters print, on a line printing an `@`)
    let mut name_ink: Vec<Option<InkAt>> = Vec::new();
    // (a byline's "by", in any case, opening a line before names is no name — today's `without_leading_by`:
    // `\author{by David Harari and Tam\'as Szamuely}`, 0808.2556)
    let byline = !self.line_inked && words.len() > 1 && words[0].0.eq_ignore_ascii_case("by");
    for (at, (word, span)) in words.into_iter().enumerate() {
      if opens[at] == 0
        && (word.eq_ignore_ascii_case("and")
          || word == "&"
          || is_separator_word(&word)
          || (at == 0 && byline))
      {
        self.push_name(name.take(), std::mem::take(&mut name_ink));
        let mut sep = Event::new(EventKind::Sep(SepKind::Word(word)));
        sep.span = span;
        sep.nest = self.run_nest;
        self.events.push(sep);
        continue;
      }
      let name = name.get_or_insert_with(|| {
        let mut name = Event::new(EventKind::Name);
        name.nest = self.run_nest;
        name.font.clone_from(&ink_font);
        name
      });
      if opens[at] == 0
        && let Some(label) = printed_label(&word)
      {
        name.printed_labels.push(PrintedLabel {
          label: label.to_string(),
          word: name.text.split(' ').filter(|w| !w.is_empty()).count(),
          line_first: at == 0 && !self.line_inked,
          span,
        });
      }
      if !name.text.is_empty() {
        name.text.push(' ');
        if record {
          name_ink.push(None);
        }
      }
      name.text.push_str(&word);
      if let Some(ink) = inks.get_mut(at) {
        name_ink.append(ink);
      }
      name.span = join_span(name.span, span);
      name.marks = join_span(name.marks, owns[at]);
    }
    self.push_name(name, name_ink);
    self.line_inked = true;
  }

  /// A name read, and where its characters print ([`InkTable`]: recorded on a line printing an `@` only).
  fn push_name(&mut self, name: Option<Event>, ink: Vec<Option<InkAt>>) {
    let Some(name) = name else {
      return;
    };
    if !ink.is_empty()
      && let Some(span) = name.span
    {
      self.inks.at.push((span, name.text.clone(), ink));
    }
    self.events.push(name);
  }

  pub(super) fn push_event(&mut self, mut event: Event) {
    self.flush_symbol();
    self.flush_run();
    match event.kind {
      EventKind::Break => {
        self.clear_gap();
        self.line += 1;
        self.line_inked = false;
        self.open = 0;
        event.braces = self.braces;
      },
      EventKind::Mark => {
        event.space_before = self.gap_items > 0;
        self.flush_gap();
        event.line_first = !self.line_inked;
        self.line_inked = true;
      },
      EventKind::Contact | EventKind::Note => {
        self.flush_gap();
        self.line_inked = true;
      },
      EventKind::Sep(SepKind::And(_) | SepKind::Column(_)) => {
        self.clear_gap();
        self.open = 0;
        self.braces = 0;
      },
      _ => self.flush_gap(),
    }
    self.events.push(event);
  }

  pub(super) fn ink(
    &mut self,
    text: &str,
    span: Option<(u16, u16)>,
    font: Option<FontKey>,
    nest: u8,
    formula: Option<MarkFormula>,
    own: Option<u16>,
  ) {
    for (at, c) in text.char_indices() {
      // (a zero-width character prints no ink: the zero-width space `{\,\!}` leaves in `${\,\!}^2$`, 2112.13115 —
      // it was read into the name before the mark and into the entry after it)
      if is_zero_width(c) {
        continue;
      }
      // (a combining mark is part of the symbol it is on, `\={\dag}`'s `†̄`: no `†` mark and a mark-led name run —
      // its line read as "̄Univ One" — review rev92b_r3 B1's sibling; `labels::mark_labels` reads a script so)
      if !self.symbol.is_empty() && is_combining_mark(c) {
        self.symbol.push(c);
        self.symbol_span = join_span(self.symbol_span, span);
        continue;
      }
      if is_fnsymbol(c) || is_mark_glyph(c) {
        self.flush_run();
        if self.symbol.is_empty() {
          self.symbol_space = self.gap_items > 0;
        }
        self.flush_gap();
        // (a letter or digit later in this box prints after it too)
        let here = formula.map(|f| MarkFormula {
          ends: f.ends && !has_word(&text[at + c.len_utf8()..]),
          ..f
        });
        self.symbol_formula = match (self.symbol_formula, here) {
          (Some(first), Some(here)) if !self.symbol.is_empty() => {
            Some(MarkFormula { ends: here.ends, ..first })
          },
          (_, here) => here,
        };
        self.symbol.push(c);
        self.symbol_span = join_span(self.symbol_span, span);
        continue;
      }
      self.flush_symbol();
      if (c == ',' || c == ';') && self.open == 0 {
        self.flush_run();
        self.clear_gap();
        let mut comma = Event::new(EventKind::Sep(SepKind::Comma));
        comma.span = span;
        comma.nest = nest;
        comma.braces = self.braces;
        self.events.push(comma);
        continue;
      }
      if c.is_whitespace() {
        let word = match c {
          ' ' | '\n' => Some(' '),
          '\u{a0}' => Some(c),
          _ => None,
        };
        if self.ties_the_gap(word) {
          self.gap_span = join_span(self.gap_span, span);
          continue;
        }
        self.gap_char = if self.gap_items == 0 { word } else { None };
        self.gap_items += 1;
        self.gap_width += space_width(c);
        self.gap_span = join_span(self.gap_span, span);
        continue;
      }
      if self.gap_items > 0 {
        // (a printed space between a mark and the ink after it: `space_after`)
        if self.run.is_empty()
          && let Some(last) = self.events.last_mut()
          && last.kind == EventKind::Mark
        {
          last.space_after = true;
        }
        if (self.gap_is_separator() && !self.letter_spaced_word_space()) || self.run.is_empty() {
          self.flush_run();
          self.flush_gap();
        } else {
          self.run.push((' ', None));
          self.run_own.push(None);
          self.clear_gap();
        }
      }
      // (a font switched inside the run does not end it: the run is one Name whatever fonts its characters are in —
      // gretsi's `first \textsc{last}`, 2508.03934 —, and the Layout event, `flush_run`, compares the runs' first
      // fonts: OXIDIZED_DESIGN_DIVERGENCES #837's departures)
      if self.run.is_empty() {
        self.run_font = font.clone();
        self.run_nest = nest;
        self.run_open = self.open;
      }
      self.open = bracket_depth(self.open, c);
      match c {
        '{' => self.braces = self.braces.saturating_add(1),
        '}' => self.braces = self.braces.saturating_sub(1),
        _ => {},
      }
      if c == '@'
        && let Some(font) = &font
      {
        self.address_font = Some(Rc::clone(&font.0));
        if let Some(at) = own.or(span.map(|s| s.0)) {
          self.run_at_fonts.push((at, Rc::clone(&font.0)));
        }
      }
      self.run.push((c, span));
      self.run_own.push(own);
    }
  }

  /// Whether the run so far is no whole name: one word at most, or words ending in an initial (`N. B.`, `V.G.`, `J.-P.`,
  /// `Yu.`) — the name's surname is still to come.
  fn run_awaits_its_name(&self) -> bool {
    let text: String = self.run.iter().map(|(c, _)| *c).collect();
    let words: Vec<&str> = text.split(' ').filter(|w| !w.is_empty()).collect();
    words.len() < 2 || words.last().is_some_and(|word| word.ends_with('.'))
  }

  /// Whether the word space or tie `word` and the one spacing character the pending gap holds are a space and a tie,
  /// read as one word space of a name, the tie (the gap's character then spent): `N.~B. ~Li$^{22}$` in the BESIII list
  /// of 340 (1202.2048) — read as a gap of two items, "N. B." was initials after `Lei~Li$^{1}$,` and both one name
  /// written surname first, which declined the writer's reading of the whole block, its sub-lettered marks with it (the
  /// rev107b review's B2); a tie and a space, `V.G.~ Kagramanova` (0802.4349), alike. Only inside one name
  /// ([`Lexer::run_awaits_its_name`]): after a whole name they are a gap of two, the gap between two names
  /// (`Gasnikov~\IEEEmembership{}⏎ Nikolay Malkovsky`, 1805.06045; `Ann Able ~Bob Baker`).
  fn ties_the_gap(&mut self, word: Option<char>) -> bool {
    let tie = self.gap_items == 1
      && !self.gap_fill
      && word.is_some_and(|c| self.gap_char.is_some_and(|first| first != c))
      && self.run_awaits_its_name();
    if tie {
      self.gap_char = None;
    }
    tie
  }

  pub(super) fn space(
    &mut self,
    width: f64,
    fill: bool,
    span: Option<(u16, u16)>,
    word: Option<char>,
  ) {
    self.flush_symbol();
    if self.ties_the_gap(word) {
      self.gap_span = join_span(self.gap_span, span);
      return;
    }
    self.gap_char = if self.gap_items == 0 { word } else { None };
    self.gap_items = self.gap_items.saturating_add(1);
    self.gap_width += width;
    self.gap_fill |= fill;
    self.gap_span = join_span(self.gap_span, span);
  }

  pub(super) fn finish(mut self) -> (Vec<Event>, InkTable) {
    self.flush_symbol();
    self.flush_run();
    self.clear_gap();
    let inks = std::mem::take(&mut self.inks);
    // a `\label` outside a note labels the note before it
    let mut events: Vec<Event> = Vec::with_capacity(self.events.len());
    let glued = glue_in_word_marks(self.events);
    let glued = mark_ordinals(byline_runs(glued));
    for event in glued {
      if event.kind == EventKind::Layout && event.role.as_deref() == Some("label") {
        if let Some(note) = events.iter_mut().rev().find(|e| e.kind == EventKind::Note)
          && note.label.is_none()
        {
          note.label = event.label;
          continue;
        }
        events.push(event);
        continue;
      }
      events.push(event);
    }
    (events, inks)
  }
}

/// The events with a byline's "by" that a line opens with as a run of its own — the spacing or the font after it parts it
/// from the names (`By~~{\scshape Vitaly~Bergelson}~~and~~…`, 2603.25922, 2604.07186, 2604.12864; `By~~{\scshape Bryna
/// Kra}`, 2603.27258; `BY~~Max Moss`) — the separator [`Lexer::flush_run`] makes of one that opens a run of names, when
/// the line's next Name reads as a name: the old reader's `without_leading_by` (`author_block/name_text.rs:669`, applied
/// to every names line by `split_author_line`: "`line` without the byline word "by" (in any case) that opens it, when
/// names follow it", `name_count(…) >= 1`). Alone, the gap rule joined it to the name after it ("By Bryna Kra").
fn byline_runs(mut events: Vec<Event>) -> Vec<Event> {
  // (a line opens after a break, and after an `\and` or an alignment tab, each author's lines as the old reader splits
  // them: `\and BY~~Max Moss~~and~~Ned Nash`)
  let ends_line = |e: &Event| {
    matches!(
      e.kind,
      EventKind::Break | EventKind::Sep(SepKind::And(_) | SepKind::Column(_))
    )
  };
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
      events[at] = sep;
    }
  }
  events
}

/// The label a word of a name run prints by hand ([`PrintedLabel`]): `a)`, `12)`, one or two letters or digits and its `)`.
fn printed_label(word: &str) -> Option<&str> {
  let label = word.strip_suffix(')')?;
  (matches!(label.chars().count(), 1 | 2) && label.chars().all(char::is_alphanumeric))
    .then_some(label)
}

/// Mark each ordinal's suffix ([`Event::ordinal`]): a Mark printing `st`, `nd`, `rd` or `th` right after a Name ending in
/// a digit.
fn mark_ordinals(mut events: Vec<Event>) -> Vec<Event> {
  for at in 1..events.len() {
    let number = events[at - 1].kind == EventKind::Name
      && events[at - 1]
        .text
        .chars()
        .next_back()
        .is_some_and(|c| c.is_ascii_digit());
    let event = &mut events[at];
    event.ordinal = number
      && event.kind == EventKind::Mark
      && matches!(event.label.as_deref(), Some("st" | "nd" | "rd" | "th"));
  }
  events
}

/// The brackets open after `c`, `open` open before it: a `(` or `[` opens one, a `)` or `]` closes one.
fn bracket_depth(open: u8, c: char) -> u8 {
  match c {
    '(' | '[' => open.saturating_add(1),
    ')' | ']' => open.saturating_sub(1),
    _ => open,
  }
}

/// A lowercase mark glued on both sides inside a word is the word's letters, not a mark between two names: `James
/// M\textsuperscript{c}Kernan` is one name, "James McKernan" (math0610203). A Name, a Mark of one or two lowercase
/// letters set with no space before or after it, and a Name are one Name. So is a mark of digits glued so, as today's
/// reads it (`rewrite_in_word_superscripts`, 77n r4: no mark list, no "and" after it): `$^1$NLP$^2$CT Lab, Univ X` is
/// the lab "NLP²CT Lab" (2603.20851, 2605.30274; `I$^2$R`, `DM$^3$L`, 2603.04869, 2604.00800), not entries "NLP" and
/// "CT Lab, Univ X" — which the writer, cutting a block entry by its whole span (OD #945), would have written — when
/// it stands glued in the source too ([`glued_in_source`]). So is a symbol on a line a mark leads, an entry, which names no
/// one: `$^1$A$^*$STAR`, `$^1$IHPC A$^*$STAR` (2401.15006's institute, cut at its star), whatever `Bob Baker$^{*}$` shows elsewhere.
/// On a line no mark leads it stays a mark: `Cher$^*$Bob Baker`.
fn glue_in_word_marks(events: Vec<Event>) -> Vec<Event> {
  let mut out: Vec<Event> = Vec::with_capacity(events.len());
  let mut events = events.into_iter().peekable();
  // (whether the line this event stands on is led by a mark; none printed yet)
  let mut mark_led: Option<bool> = None;
  while let Some(event) = events.next() {
    match event.kind {
      EventKind::Break | EventKind::Sep(SepKind::And(_) | SepKind::Column(_)) => mark_led = None,
      EventKind::Layout => {},
      _ => mark_led = mark_led.or(Some(event.kind == EventKind::Mark)),
    }
    let letters = event.label.as_deref().unwrap_or("");
    let next = events.peek();
    let number_or_letter = !event.symbol
      && ((1..=2).contains(&letters.chars().count()) && letters.chars().all(char::is_lowercase)
        || (1..=3).contains(&letters.chars().count())
          && letters.chars().all(|c| c.is_ascii_digit())
          && glued_in_source(out.last(), &event, next));
    let in_word = event.kind == EventKind::Mark
      && !event.space_before
      && !event.space_after
      && (number_or_letter
        || (mark_led == Some(true)
          && is_symbol_run(letters)
          && glued_in_source(out.last(), &event, next)))
      && out.last().is_some_and(|e| {
        e.kind == EventKind::Name && e.text.chars().next_back().is_some_and(char::is_alphabetic)
      })
      && next.is_some_and(|e| {
        e.kind == EventKind::Name && e.text.chars().next().is_some_and(char::is_alphabetic)
      });
    if in_word {
      let next = events.next().expect("peeked");
      let name = out.last_mut().expect("checked");
      name.text.push_str(letters);
      name.text.push_str(&next.text);
      name.span = join_span(join_span(name.span, event.span), next.span);
      continue;
    }
    out.push(event);
  }
  out
}

/// Whether a mark's label prints footnote symbols or mark glyphs alone.
fn is_symbol_run(label: &str) -> bool {
  !label.is_empty() && label.chars().all(|c| is_fnsymbol(c) || is_mark_glyph(c))
}

/// Whether a digit mark stands in the source between the names around it with nothing but its own math shifts (or
/// none) between them, as `rewrite_in_word_superscripts` reads a superscript inside a word: `NLP$^2$CT`, `I$^2$R` —
/// not `Russia\break $^2$Nuclear` (hep-ph9809222's revtex `\address`, a line break before the next entry's mark,
/// which TeX's spaces leave unseen).
fn glued_in_source(before: Option<&Event>, mark: &Event, after: Option<&Event>) -> bool {
  match (
    before.and_then(|e| e.span),
    mark.span,
    after.and_then(|e| e.span),
  ) {
    (Some((_, end)), Some((start, stop)), Some((next, _))) => {
      start <= end.saturating_add(1) && next <= stop.saturating_add(1)
    },
    _ => false,
  }
}

/// A character printing nothing: the zero-width space, non-joiner and joiner, the word joiner and the byte-order mark.
pub(super) fn is_zero_width(c: char) -> bool {
  matches!(
    c,
    '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{2060}' | '\u{FEFF}'
  )
}

/// LaTeX's `\@fnsymbol` characters (latex.ltx `\@fnsymbol`: `*`, `\dagger`, `\ddagger`, `\mathsection`,
/// `\mathparagraph`, `\|`, and their doubles), with the math asterisk; `\|` as the kernel prints it in a formula, `∥`
/// (U+2225, `$\|$`: `R.~E.~Hughes$\|$` under `\address{$\|$ The Ohio State University …}`, 0806.2911 — FM S2a.2 r7) as
/// well as `\textbardbl`'s `‖`.
pub(in crate::base_utilities) fn is_fnsymbol(c: char) -> bool {
  matches!(
    c,
    '*' | '\u{2217}' | '†' | '‡' | '§' | '¶' | '‖' | '\u{2225}'
  )
}

/// A glyph a document prints as an author's mark that is no `\@fnsymbol` character: the symbols set beside a name to
/// point at its affiliation or note, read as the marks they are, never as the name's ink (`A. \v Siber $\diamond$` under
/// `$\diamond$ Institute of Physics …`, 0907.1243; `A.~A.~Moiseev$\sharp$`, 0806.2911; `R. Podgornik $\dagger$$\star$`
/// — FM S2a.2 r7, main-session decision 2 of int66: the marks the lexer did not read as such once a name became its
/// printed text). Not a dot, a bullet or a bar (separators, [`is_separator_word`]), nor a degree sign or a ring.
pub(super) fn is_mark_glyph(c: char) -> bool {
  matches!(
    c,
    '♯'
      | '♭'
      | '♮'
      | '⋄'
      | '◇'
      | '◆'
      | '♢'
      | '♦'
      | '⋆'
      | '★'
      | '☆'
      | '♣'
      | '♠'
      | '♡'
      | '♥'
      | '⊕'
      | '⊖'
      | '⊗'
      | '⊘'
      | '⊙'
      | '⊛'
      | '⊎'
      | '△'
      | '▽'
      | '▲'
      | '▼'
      | '♀'
      | '♂'
      | '✝'
      | '✠'
      | '※'
      | '⁎'
  )
}

/// Whether a word of a name run is a separator glyph printed between two names, or between a name and its contact or
/// degree, standing alone: a bar, a colon, the math dot `\cdot` or a bullet, a dash (`Tom Y. Zhao$^{\dagger}$ | PhD`,
/// 2311.00652; `MeerKLASS Collaboration\thanks{…}: Matilde Barberi-Squarotti`, 2407.21626; `Chiara Simeoni --
/// \url{…}`, 2511.07187; `Christopher Hojny\myorcidlink{…} $\cdot$ Mathieu Besançon`, 2511.18580 — FM S2a.2 r7): the
/// lexer reads it as the word separator it is, as it reads "and" and `&`. Not the text middle dot `·`, which a name
/// prints between its parts as often (`Ann \textperiodcentered{} Able`, `perfect_kernel_batchfms2a2r6`).
pub(in crate::base_utilities) fn is_separator_word(word: &str) -> bool {
  !word.is_empty()
    && word
      .chars()
      .all(|c| matches!(c, '|' | ':' | '⋅' | '•' | '∙' | '-' | '–' | '—'))
}
