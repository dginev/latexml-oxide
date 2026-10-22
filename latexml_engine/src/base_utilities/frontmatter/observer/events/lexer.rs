//! The lexer's second pass (OXIDIZED_DESIGN_DIVERGENCES #837): the walk's atoms — ink, spacing items, finished
//! events — into the block's events: name runs split at the words "and" and `&`, footnote-symbol characters a Mark,
//! commas a separator, a spacing run a Gap when it separates, a font change between runs a Layout event.

use super::*;

/// The lexer's second pass: the atoms into events.
#[derive(Default)]
pub(super) struct Lexer {
  pub(super) events: Vec<Event>,
  /// the name run being read: its characters with their spans, its font, its nest
  run:               Vec<(char, Option<(u16, u16)>)>,
  run_font:          Option<String>,
  run_nest:          u8,
  /// the spacing run since the last ink: items, width, fill, span
  gap_items:         usize,
  gap_width:         f64,
  gap_fill:          bool,
  gap_span:          Option<(u16, u16)>,
  /// the line has shown ink (a name, a mark, a contact) since its last break
  line_inked:        bool,
  /// the font of the last name run
  last_font:         Option<String>,
  /// the symbol mark being read (consecutive footnote-symbol characters)
  symbol:            String,
  symbol_span:       Option<(u16, u16)>,
}

impl Lexer {
  fn gap_is_separator(&self) -> bool {
    self.gap_fill || self.gap_items >= 2 || (self.gap_items == 1 && self.gap_width > INTERWORD)
  }

  fn clear_gap(&mut self) {
    self.gap_items = 0;
    self.gap_width = 0.0;
    self.gap_fill = false;
    self.gap_span = None;
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
    mark.symbol = true;
    mark.role = Some("text".to_string());
    mark.span = self.symbol_span.take();
    mark.line_first = !self.line_inked;
    self.line_inked = true;
    self.events.push(mark);
  }

  /// End the name run: a Name, split at the words "and" and `&` (Sep events), each piece with its own span.
  fn flush_run(&mut self) {
    let run = std::mem::take(&mut self.run);
    let font = self.run_font.take();
    // the run's words, each with the span of its characters
    let mut words: Vec<(String, Option<(u16, u16)>)> = Vec::new();
    let mut word = String::new();
    let mut word_span = None;
    for (c, span) in run.into_iter().chain(std::iter::once((' ', None))) {
      if c == ' ' {
        if !word.is_empty() {
          words.push((std::mem::take(&mut word), word_span.take()));
        }
      } else {
        word.push(c);
        word_span = join_span(word_span, span);
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
    if font.is_some() {
      self.last_font = font;
    }
    let mut name: Option<Event> = None;
    for (word, span) in words {
      if word.eq_ignore_ascii_case("and") || word == "&" {
        self.events.extend(name.take());
        let mut sep = Event::new(EventKind::Sep(SepKind::Word(word)));
        sep.span = span;
        sep.nest = self.run_nest;
        self.events.push(sep);
        continue;
      }
      let name = name.get_or_insert_with(|| {
        let mut name = Event::new(EventKind::Name);
        name.nest = self.run_nest;
        name
      });
      if !name.text.is_empty() {
        name.text.push(' ');
      }
      name.text.push_str(&word);
      name.span = join_span(name.span, span);
    }
    self.events.extend(name);
    self.line_inked = true;
  }

  pub(super) fn push_event(&mut self, mut event: Event) {
    self.flush_symbol();
    self.flush_run();
    match event.kind {
      EventKind::Break => {
        self.clear_gap();
        self.line_inked = false;
      },
      EventKind::Mark => {
        self.flush_gap();
        event.line_first = !self.line_inked;
        self.line_inked = true;
      },
      EventKind::Contact | EventKind::Note => {
        self.flush_gap();
        self.line_inked = true;
      },
      EventKind::Sep(SepKind::And(_) | SepKind::Column(_)) => self.clear_gap(),
      _ => self.flush_gap(),
    }
    self.events.push(event);
  }

  pub(super) fn ink(
    &mut self,
    text: &str,
    span: Option<(u16, u16)>,
    font: Option<String>,
    nest: u8,
  ) {
    for c in text.chars() {
      // (a zero-width character prints no ink: the zero-width space `{\,\!}` leaves in `${\,\!}^2$`, 2112.13115 —
      // it was read into the name before the mark and into the entry after it)
      if is_zero_width(c) {
        continue;
      }
      if is_fnsymbol(c) {
        self.flush_run();
        self.flush_gap();
        self.symbol.push(c);
        self.symbol_span = join_span(self.symbol_span, span);
        continue;
      }
      self.flush_symbol();
      if c == ',' || c == ';' {
        self.flush_run();
        self.clear_gap();
        let mut comma = Event::new(EventKind::Sep(SepKind::Comma));
        comma.span = span;
        comma.nest = nest;
        self.events.push(comma);
        continue;
      }
      if c.is_whitespace() {
        self.gap_items += 1;
        self.gap_width += space_width(c);
        self.gap_span = join_span(self.gap_span, span);
        continue;
      }
      if self.gap_items > 0 {
        if self.gap_is_separator() || self.run.is_empty() {
          self.flush_run();
          self.flush_gap();
        } else {
          self.run.push((' ', None));
          self.clear_gap();
        }
      }
      // (a font switched inside the run does not end it: the run is one Name whatever fonts its characters are in —
      // gretsi's `first \textsc{last}`, 2508.03934 —, and the Layout event, `flush_run`, compares the runs' first
      // fonts: OXIDIZED_DESIGN_DIVERGENCES #837's departures)
      if self.run.is_empty() {
        self.run_font = font.clone();
        self.run_nest = nest;
      }
      self.run.push((c, span));
    }
  }

  pub(super) fn space(&mut self, width: f64, fill: bool, span: Option<(u16, u16)>) {
    self.flush_symbol();
    self.gap_items = self.gap_items.saturating_add(1);
    self.gap_width += width;
    self.gap_fill |= fill;
    self.gap_span = join_span(self.gap_span, span);
  }

  pub(super) fn finish(mut self) -> Vec<Event> {
    self.flush_symbol();
    self.flush_run();
    self.clear_gap();
    // a `\label` outside a note labels the note before it
    let mut events: Vec<Event> = Vec::with_capacity(self.events.len());
    for event in self.events {
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
    events
  }
}

/// A character printing nothing: the zero-width space, non-joiner and joiner, the word joiner and the byte-order mark.
fn is_zero_width(c: char) -> bool {
  matches!(
    c,
    '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{2060}' | '\u{FEFF}'
  )
}
