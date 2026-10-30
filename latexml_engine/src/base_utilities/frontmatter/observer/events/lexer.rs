//! The lexer's second pass (OXIDIZED_DESIGN_DIVERGENCES #837): the walk's atoms — ink, spacing items, finished
//! events — into the block's events: name runs split at the words "and" and `&` and at a separator glyph standing alone
//! (`is_separator_word`), footnote-symbol characters and the other mark glyphs (`is_mark_glyph`) a Mark, commas a
//! separator, a spacing run a Gap when it separates, a font change between runs a Layout event.

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
  /// the symbol mark being read (consecutive footnote-symbol characters and mark glyphs)
  symbol:            String,
  symbol_span:       Option<(u16, u16)>,
  /// a printed space before the symbol mark being read
  symbol_space:      bool,
  /// the brackets `(`, `[` printed and not yet closed since the line's start, and how many were open where the name run
  /// being read began: inside one a comma or the word "and" is the bracket's text, no separator (`Laurent Schoeffel (on
  /// behalf of the H1 and ZEUS collaborations)`, 0811.3142; review rev94a B2)
  open:              u8,
  run_open:          u8,
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
    // (a footnote symbol only when every character is one: a mark glyph, `⋄`, is a mark of no `\@fnsymbol` kind)
    mark.symbol = mark.text.chars().all(is_fnsymbol);
    mark.role = Some("text".to_string());
    mark.span = self.symbol_span.take();
    mark.space_before = std::mem::take(&mut self.symbol_space);
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
    // (the brackets open at each word's start: a word inside one is the bracket's, no separator)
    let mut open = self.run_open;
    let mut opens = Vec::with_capacity(words.len());
    for (word, _) in &words {
      opens.push(open);
      open = word.chars().fold(open, bracket_depth);
    }
    let mut name: Option<Event> = None;
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
        self.open = 0;
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
      },
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
        // (a printed space between a mark and the ink after it: `space_after`)
        if self.run.is_empty()
          && let Some(last) = self.events.last_mut()
          && last.kind == EventKind::Mark
        {
          last.space_after = true;
        }
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
        self.run_open = self.open;
      }
      self.open = bracket_depth(self.open, c);
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
    for event in glue_in_word_marks(self.events) {
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
/// letters set with no space before or after it, and a Name are one Name.
fn glue_in_word_marks(events: Vec<Event>) -> Vec<Event> {
  let mut out: Vec<Event> = Vec::with_capacity(events.len());
  let mut events = events.into_iter().peekable();
  while let Some(event) = events.next() {
    let letters = event.label.as_deref().unwrap_or("");
    let in_word = event.kind == EventKind::Mark
      && !event.symbol
      && !event.space_before
      && !event.space_after
      && (1..=2).contains(&letters.chars().count())
      && letters.chars().all(char::is_lowercase)
      && out.last().is_some_and(|e| {
        e.kind == EventKind::Name && e.text.chars().next_back().is_some_and(char::is_alphabetic)
      })
      && events.peek().is_some_and(|e| {
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

/// A character printing nothing: the zero-width space, non-joiner and joiner, the word joiner and the byte-order mark.
fn is_zero_width(c: char) -> bool {
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
