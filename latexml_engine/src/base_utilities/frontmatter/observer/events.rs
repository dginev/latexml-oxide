//! The observer's event lexer: a rehearsal's digested list (`rehearsal::rehearse`) read into the events of an author
//! block, in print order (OXIDIZED_DESIGN_DIVERGENCES #837; the design note's §2 event model):
//!
//! - **Name**: a run of ink between the other events (one interword space or tie inside it);
//! - **Mark**: a superscript (`\textsuperscript`, a formula's script, `\inst`, `\IEEEauthorrefmark`, `\footnotemark`,
//!   whatever user macro printed one), or a footnote-symbol character in the text (`\dag`, `$\dagger$`): its label,
//!   whether it is a footnote symbol (LaTeX's `\@fnsymbol` list, `*` `†` `‡` `§` `¶` `‖` and their doubles, by the
//!   character printed, whatever command printed it), and whether it is its line's first ink;
//! - **Sep**: a column (`\and`, `\And`, `\AND`: which one; an alignment tab `&`), a comma, the word "and" (or a
//!   printed `\&`), a gap (a spacing run of two or more items, of a space wider than an interword one, or holding
//!   fill glue `\hfil`/`\hfill`: its item count, its natural width in em of the box's font, and whether it fills);
//! - **Break**: a row (`\\`, `\newline`, `\linebreak`, `\par`; the kernel's `\lx@newline` a `\centering` or a box
//!   rebinds `\\` to; an alignment's row end);
//! - **Contact**: `\email`, `\url`, `\href`, `\orcidlink`, an in-block `\affil`/`\address`, the frontmatter API;
//! - **Note**: `\thanks`/`\footnote`/`\footnotetext`, with its label (a `\label` inside it) and its inner contacts;
//! - **Layout**: a box or a styling container (its command), an alignment cell, a change of font between runs (a
//!   run's first font against the previous run's: a switch inside a run shows at the next run, not where it was).
//!
//! Each event carries its span of the author field's positions (`fieldProv`/`fieldSerial` of the boxes it is made of,
//! core `gullet::field_cover_of`), its ink text and its layout nesting. ApiCall events (setters called outside the
//! field) are the ledger's, added to the block by the shadow dump.

use latexml_core::{
  document::{FIELD_PROV_PROPERTY, FIELD_SERIAL_PROPERTY},
  token::{field_ledger_entry, field_prov_pos},
};

use super::*;

/// The kind of a separator event.
#[derive(Debug, Clone, PartialEq)]
pub(in crate::base_utilities) enum SepKind {
  /// `\and`, `\And`, `\AND`: a column (the command: NeurIPS's and ICLR's `\AND` starts a new row)
  And(&'static str),
  /// an alignment tab `&` read in the block (a column end of LaTeX's `\begin{tabular}[t]{c}` author line): the
  /// character
  Column(&'static str),
  /// `,` between names
  Comma,
  /// the word "and" (any case) or a printed `&` (`\&`)
  Word(String),
  /// a spacing run: its items, its natural width (em of the box's font), and whether it holds fill glue
  Gap {
    items: usize,
    width: f64,
    fill:  bool,
  },
}

/// The kind of an event.
#[derive(Debug, Clone, PartialEq)]
pub(in crate::base_utilities) enum EventKind {
  Name,
  Mark,
  Sep(SepKind),
  Break,
  Contact,
  Note,
  Layout,
  ApiCall,
}

impl EventKind {
  pub(in crate::base_utilities) fn name(&self) -> &'static str {
    match self {
      EventKind::Name => "Name",
      EventKind::Mark => "Mark",
      EventKind::Sep(_) => "Sep",
      EventKind::Break => "Break",
      EventKind::Contact => "Contact",
      EventKind::Note => "Note",
      EventKind::Layout => "Layout",
      EventKind::ApiCall => "ApiCall",
    }
  }
}

/// One event of an author block.
#[derive(Debug, Clone, PartialEq)]
pub(in crate::base_utilities) struct Event {
  pub kind:       EventKind,
  /// the positions of the author field it covers, `start..end`
  pub span:       Option<(u16, u16)>,
  /// its ink: a name's text, a mark's printed text, a contact's or a note's shown text
  pub text:       String,
  /// a mark's label, a note's `\label`, a contact's optional label
  pub label:      Option<String>,
  /// a mark printed as a footnote symbol
  pub symbol:     bool,
  /// a mark that is its line's first ink
  pub line_first: bool,
  /// a contact's role, a mark's or a layout's source (`sup`, `math`, `inst`, `text`; the container's command)
  pub role:       Option<String>,
  /// a link's target
  pub target:     Option<String>,
  /// a note's inner contacts
  pub inner:      Vec<Event>,
  /// how many layout containers enclose it
  pub nest:       u8,
}

impl Event {
  fn new(kind: EventKind) -> Self {
    Event {
      kind,
      span: None,
      text: String::new(),
      label: None,
      symbol: false,
      line_first: false,
      role: None,
      target: None,
      inner: Vec::new(),
      nest: 0,
    }
  }
}

/// LaTeX's `\@fnsymbol` characters (latex.ltx `\@fnsymbol`: `*`, `\dagger`, `\ddagger`, `\mathsection`,
/// `\mathparagraph`, `\|`, and their doubles), with the math asterisk.
fn is_fnsymbol(c: char) -> bool {
  matches!(c, '*' | '\u{2217}' | '†' | '‡' | '§' | '¶' | '‖')
}

/// Whether a mark's printed text is a footnote symbol (only `\@fnsymbol` characters, commas and spaces aside).
fn is_symbol_label(text: &str) -> bool {
  let mut any = false;
  for c in text.chars() {
    if is_fnsymbol(c) {
      any = true;
    } else if !(c.is_whitespace() || c == ',') {
      return false;
    }
  }
  any
}

/// The event of a printed mark (`text`, made by `role`: `sup`, `math`, `inst`, `footnotemark`): a Mark labelled with
/// its text; a mark printing only commas is the separator between two marks (llncs's and aa's `\fnmsep`,
/// `\unskip$^,$`), a comma; an empty superscript prints nothing and is no mark (`\textsuperscript{}`), but a
/// `\footnotemark` is one whatever it printed: its text is the `\thefootnote` it prints (the `printed` property of
/// `rehearsal::load`'s `\lx@fm@obs@footnotemark`), empty where that failed (`\@fnsymbol` past 9, "Counter too
/// large").
fn mark_event(text: String, role: &str) -> Option<Event> {
  if text.is_empty() && role != "footnotemark" {
    return None;
  }
  let mut event = if !text.is_empty() && text.chars().all(|c| c == ',' || c.is_whitespace()) {
    Event::new(EventKind::Sep(SepKind::Comma))
  } else {
    let mut mark = Event::new(EventKind::Mark);
    mark.symbol = is_symbol_label(&text);
    mark.label = Some(text.clone());
    mark.text = text;
    mark
  };
  event.role = Some(role.to_string());
  Some(event)
}

/// The width in em of a spacing character (an interword space, a tie, the Unicode spaces `\quad`, `\enspace`, `\,`
/// print).
fn space_width(c: char) -> f64 {
  match c {
    '\u{2003}' => 1.0,
    '\u{2002}' => 0.5,
    '\u{2004}' => 0.333,
    '\u{2005}' => 0.25,
    '\u{2006}' | '\u{2009}' => 0.167,
    '\u{200A}' => 0.1,
    '\u{202F}' => 0.2,
    '\u{2001}' => 2.0,
    _ => 0.333,
  }
}

/// The widest an interword space or a tie is (em): a single spacing item up to it stays inside a name.
const INTERWORD: f64 = 0.34;

/// The em of `d`'s font in points (its quad: `Font::get_em_width`), 10pt when it has none.
fn em_pt(d: &Digested) -> f64 {
  d.get_font()
    .ok()
    .flatten()
    .map(|font| font.get_em_width() as f64 / 65536.0)
    .filter(|em| *em > 0.0)
    .unwrap_or(10.0)
}

/// The natural width in em of a spacing box with a `width` property (`\hspace`'s, `\hskip`'s), in its font's em.
fn width_property_em(d: &Digested, width: &Stored) -> Option<f64> {
  let pt = match width {
    Stored::Dimension(d) => d.pt_value(None),
    Stored::Glue(g) => g.pt_value(None),
    _ => return None,
  };
  Some((pt / em_pt(d)).max(0.0))
}

/// The span of the author field a digested box covers: its `fieldSerial`'s cover in the field ledger, else its own
/// position (`fieldProv`).
fn box_span(d: &Digested) -> Option<(u16, u16)> {
  let prov = match d.get_property(FIELD_PROV_PROPERTY).as_deref() {
    Some(Stored::Int(p)) => u16::try_from(*p).ok()?,
    _ => return None,
  };
  if let Some(Stored::Int(serial)) = d.get_property(FIELD_SERIAL_PROPERTY).as_deref()
    && let Ok(serial) = u32::try_from(*serial)
    && let Some(entry) = field_ledger_entry(serial)
  {
    return Some((
      entry.cover.start,
      entry.cover.end.max(entry.cover.start + 1),
    ));
  }
  let pos = field_prov_pos(prov);
  Some((pos, pos + 1))
}

/// The union of two spans.
fn join_span(a: Option<(u16, u16)>, b: Option<(u16, u16)>) -> Option<(u16, u16)> {
  match (a, b) {
    (Some((s1, e1)), Some((s2, e2))) => Some((s1.min(s2), e1.max(e2))),
    (a, None) => a,
    (None, b) => b,
  }
}

/// The span of every box of `d` (its own, and what it holds).
fn deep_span(d: &Digested) -> Option<(u16, u16)> {
  let mut span = box_span(d);
  match d.data() {
    DigestedData::List(l) => {
      for child in l.borrow().boxes.iter() {
        span = join_span(span, deep_span(child));
      }
    },
    DigestedData::Whatsit(w) => {
      let w = w.borrow();
      for arg in w.get_args().iter().flatten() {
        span = join_span(span, deep_span(arg));
      }
      if let Ok(Some(body)) = w.get_body() {
        span = join_span(span, deep_span(&body));
      }
    },
    _ => {},
  }
  span
}

/// The ink of `d`, whitespace collapsed.
fn ink_of(d: &Digested) -> String {
  let mut text = String::new();
  ink_text(d, &mut text);
  text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A piece of the walk: ink, a spacing item, or a finished event.
#[derive(Debug)]
enum Atom {
  Ink {
    text: String,
    span: Option<(u16, u16)>,
    font: Option<String>,
    nest: u8,
  },
  Space {
    width: f64,
    fill:  bool,
    span:  Option<(u16, u16)>,
  },
  Event(Event),
}

/// The command a whatsit was made by.
fn whatsit_cs(w: &Whatsit) -> String { w.get_definition().get_cs_name().into_owned() }

/// The font of a text box, as a key telling fonts apart (its family, series, shape and size).
fn font_key(d: &Digested) -> Option<String> {
  d.get_font().ok().flatten().map(|font| {
    format!(
      "{:?}/{:?}/{:?}/{:?}",
      font.get_family(),
      font.get_series(),
      font.get_shape(),
      font.get_size()
    )
  })
}

/// Walk `d` in print order, collecting atoms.
#[derive(Default)]
struct Walker {
  atoms: Vec<Atom>,
}

impl Walker {
  fn walk(&mut self, d: &Digested, nest: u8) {
    match d.data() {
      DigestedData::TBox(b) => {
        let text = b
          .borrow()
          .get_string()
          .map(|s| s.into_owned())
          .unwrap_or_default();
        if text.is_empty() {
          return;
        }
        let span = box_span(d);
        if text.chars().all(char::is_whitespace) {
          // fill glue (`\hfil`, `\hfill`: tex_glue.rs, natural width 0, tex.web §1058) always separates; a skip's
          // width is its `width` property (`\hspace{3em}`), else the width of the space characters printed
          let fill = d.get_property_bool("isFill");
          let width = match d.get_property("width").as_deref() {
            Some(width) => width_property_em(d, width),
            None => None,
          }
          .unwrap_or_else(|| {
            if fill {
              0.0
            } else {
              text.chars().map(space_width).sum()
            }
          });
          self.atoms.push(Atom::Space { width, fill, span });
        } else {
          self.atoms.push(Atom::Ink {
            text,
            span,
            font: font_key(d),
            nest,
          });
        }
      },
      DigestedData::List(l) => {
        for child in l.borrow().boxes.iter() {
          self.walk(child, nest);
        }
      },
      DigestedData::Whatsit(w) => self.walk_whatsit(d, &w.borrow(), nest),
      DigestedData::Alignment(a) => {
        let a = a.borrow();
        let rows = a.rows();
        for (r, row) in rows.iter().enumerate() {
          for (c, cell) in row.get_columns().iter().enumerate() {
            let mut layout = Event::new(EventKind::Layout);
            layout.role = Some("cell".to_string());
            layout.label = Some(s!("{r},{c}"));
            layout.nest = nest;
            self.atoms.push(Atom::Event(layout));
            if let Some(boxes) = cell.boxes.as_ref() {
              self.walk(boxes, nest.saturating_add(1));
            }
          }
          if r + 1 < rows.len() {
            let mut row_end = Event::new(EventKind::Break);
            row_end.role = Some("row".to_string());
            row_end.nest = nest;
            self.atoms.push(Atom::Event(row_end));
          }
        }
      },
      _ => {},
    }
  }

  fn event(&mut self, mut event: Event, d: &Digested, nest: u8) {
    event.span = deep_span(d);
    event.nest = nest;
    self.atoms.push(Atom::Event(event));
  }

  fn walk_whatsit(&mut self, d: &Digested, w: &Whatsit, nest: u8) {
    let cs = whatsit_cs(w);
    let arg = |n: usize| w.get_arg(n).map(ink_of).unwrap_or_default();
    let raw = |n: usize| w.get_arg(n).map(|a| a.to_string()).unwrap_or_default();
    match cs.as_str() {
      "\\lx@fm@obs@and" => self.event(Event::new(EventKind::Sep(SepKind::And("\\and"))), d, nest),
      "\\lx@fm@obs@And" => self.event(Event::new(EventKind::Sep(SepKind::And("\\And"))), d, nest),
      "\\lx@fm@obs@AND" => self.event(Event::new(EventKind::Sep(SepKind::And("\\AND"))), d, nest),
      "\\lx@fm@obs@break" => self.event(Event::new(EventKind::Break), d, nest),
      // The kernel's own line breaks, where the block rebinds `\\` past the rehearsal's binding: `\centering`,
      // `\raggedright`, `\raggedleft` let it be `\@normalcr` (sect06.rs, latex.ltx `\let\\\@centercr`), a
      // minipage `\lx@newline` (sect12.rs `{minipage}`), a parbox's body `\lx@parboxnewline` (sect12.rs), each the
      // `\lx@newline` constructor (sect01.rs, `\@normalcr` and `\lx@parboxnewline` its `\let` copies).
      "\\lx@newline" | "\\lx@parboxnewline" | "\\@normalcr" => {
        self.event(Event::new(EventKind::Break), d, nest)
      },
      // A stray alignment tab (article's `\author` line is LaTeX's `\begin{tabular}[t]{c}`, so pdflatex reads it as a
      // column end): the `&` constructor errs only when it is built into the document (tex_tables.rs), so the
      // rehearsal holds nothing for it; it separates as a column. (Its definition is the alignment-tab catcode's,
      // named `Align`: matched by the catcode.)
      _ if w.get_definition().get_cs().get_catcode() == Catcode::ALIGN => {
        self.event(Event::new(EventKind::Sep(SepKind::Column("&"))), d, nest)
      },
      "\\lx@fm@obs@note" | "\\lx@fm@obs@notetext" => {
        let mut note = Event::new(EventKind::Note);
        if let Some(body) = w.get_arg(2) {
          note.text = ink_of(body);
          for inner in lex_events(body) {
            match inner.kind {
              EventKind::Contact => note.inner.push(inner),
              EventKind::Layout if inner.role.as_deref() == Some("label") => {
                note.label = inner.label
              },
              _ => {},
            }
          }
        }
        let mark = arg(1);
        let source = if cs.ends_with("notetext") {
          "footnotetext"
        } else {
          "thanks"
        };
        note.role = Some(if mark.is_empty() {
          source.to_string()
        } else {
          s!("{source} mark={mark}")
        });
        self.event(note, d, nest);
      },
      "\\lx@fm@obs@label" => {
        let mut label = Event::new(EventKind::Layout);
        label.role = Some("label".to_string());
        label.label = Some(raw(1).trim().to_string());
        self.event(label, d, nest);
      },
      "\\lx@fm@obs@footnotemark" => {
        let printed = match w.get_property("printed").as_deref() {
          Some(Stored::String(printed)) => to_string(*printed),
          _ => String::new(),
        };
        if let Some(mark) = mark_event(printed, "footnotemark") {
          self.event(mark, d, nest);
        }
      },
      "\\lx@fm@obs@mark" => {
        if let Some(mark) = mark_event(arg(1), "inst") {
          self.event(mark, d, nest);
        }
      },
      "\\textsuperscript" | "\\@textsuperscript" => {
        if let Some(mark) = mark_event(arg(1), "sup") {
          self.event(mark, d, nest);
        }
      },
      "\\lx@fm@obs@email" | "\\lx@fm@obs@orcid" | "\\lx@fm@obs@affil" => {
        let mut contact = Event::new(EventKind::Contact);
        contact.role = Some(
          match cs.as_str() {
            "\\lx@fm@obs@email" => "email",
            "\\lx@fm@obs@orcid" => "orcid",
            _ => "affiliation",
          }
          .to_string(),
        );
        let label = arg(1);
        contact.label = (!label.is_empty()).then_some(label);
        contact.text = if cs == "\\lx@fm@obs@affil" {
          arg(2)
        } else {
          raw(2).trim().to_string()
        };
        self.event(contact, d, nest);
      },
      "\\lx@fm@obs@url" => {
        let mut contact = Event::new(EventKind::Contact);
        contact.role = Some("url".to_string());
        contact.text = raw(1).trim().to_string();
        contact.target = Some(contact.text.clone());
        self.event(contact, d, nest);
      },
      "\\lx@fm@obs@href" => {
        let mut contact = Event::new(EventKind::Contact);
        contact.role = Some("url".to_string());
        contact.target = Some(raw(1).trim().to_string());
        contact.text = arg(2);
        self.event(contact, d, nest);
      },
      "\\lx@fm@obs@annotate" => {
        let tag = raw(2);
        let options = raw(3);
        let role = options
          .split(',')
          .find_map(|kv| {
            kv.trim()
              .strip_prefix("role=")
              .map(|r| r.trim().to_string())
          })
          .unwrap_or_default();
        let mut event = Event::new(if tag.trim() == "ltx:note" {
          EventKind::Note
        } else {
          EventKind::Contact
        });
        event.role = Some(role);
        event.text = arg(4);
        self.event(event, d, nest);
      },
      _ if w.is_math() || cs.contains("math") && w.get_body().ok().flatten().is_some() => {
        self.walk_math(d, w, nest)
      },
      _ => self.walk_container(d, w, &cs, nest),
    }
  }

  /// A formula: its scripts are marks (`$^{1}$`, `Szabo$^3$`); its other ink is text (`$\dagger$` a symbol).
  fn walk_math(&mut self, d: &Digested, w: &Whatsit, nest: u8) {
    let Ok(Some(body)) = w.get_body() else {
      self.walk_container(d, w, &whatsit_cs(w), nest);
      return;
    };
    self.walk_math_list(&body, nest);
  }

  fn walk_math_list(&mut self, d: &Digested, nest: u8) {
    match d.data() {
      DigestedData::List(l) => {
        for child in l.borrow().boxes.iter() {
          self.walk_math_list(child, nest);
        }
      },
      DigestedData::Whatsit(w) => {
        let w = w.borrow();
        let cs = whatsit_cs(&w);
        if cs.contains("superscript") {
          // (the script is the last argument: `\lx@post@superscript`'s, `\lx@floating@superscript`'s)
          let script = w
            .get_args()
            .iter()
            .rev()
            .flatten()
            .next()
            .map(ink_of)
            .unwrap_or_default();
          if let Some(mark) = mark_event(script, "math") {
            self.event(mark, d, nest);
          }
        } else if cs.contains("subscript") {
          // a subscript is the formula's ink, not a mark
          self.atoms.push(Atom::Ink {
            text: ink_of(d),
            span: deep_span(d),
            font: None,
            nest,
          });
        } else {
          self.walk_whatsit(d, &w, nest);
        }
      },
      _ => self.walk(d, nest),
    }
  }

  /// Any other whatsit: a container of what it shows (a box, a styling command, an environment) is a Layout event
  /// followed by its content one level deeper; a spacing command (`\hspace`, `\hskip`, `\quad`) a spacing item; what
  /// shows nothing, nothing.
  fn walk_container(&mut self, d: &Digested, w: &Whatsit, cs: &str, nest: u8) {
    let mut contents: Vec<Digested> = Vec::new();
    if let Some(Stored::Digested(a)) = w.get_property("alignment").as_deref() {
      contents.push(a.clone());
    }
    // (only the arguments it shows, as `typesets_content` reads them: not a box's or an alignment's placement — the
    // `[t]` of `\begin{tabular}[t]{c}`, `\parbox[t]`, `\begin{minipage}[b]`, the `[l]` of `\makebox[3cm][l]`: a
    // `Position` parameter, util_text.rs `shown_arguments` —, nor keyvals or a verbatim key)
    let args = w.get_args();
    for (arg, shown) in args.iter().zip(shown_arguments(w)) {
      if let Some(arg) = arg
        && shown
        && typesets_content(arg)
      {
        contents.push(arg.clone());
      }
    }
    if let Ok(Some(body)) = w.get_body() {
      contents.push(body);
    }
    // an image shows none of its arguments (its file and options) but is ink: a Layout event of its own, between the
    // spaces around it (2503.08161's `$^{1}$ \includegraphics[width=0.36cm]{polyu-logo.png} The Hong Kong …`)
    if contents.is_empty()
      && (w.get_property("graphic").is_some() || w.get_property("candidates").is_some())
    {
      let mut layout = Event::new(EventKind::Layout);
      layout.role = Some(cs.to_string());
      self.event(layout, d, nest);
      return;
    }
    if contents.is_empty() {
      let mut text = String::new();
      ink_text(d, &mut text);
      if !text.trim().is_empty() {
        // (a whatsit printing text of its own: an ERROR box holding an undefined command's name)
        self.atoms.push(Atom::Ink {
          text,
          span: deep_span(d),
          font: None,
          nest,
        });
      } else if let Some(width) = whatsit_width_em(d, w) {
        self.atoms.push(Atom::Space {
          width,
          fill: false,
          span: box_span(d),
        });
      }
      return;
    }
    let mut layout = Event::new(EventKind::Layout);
    layout.role = Some(cs.to_string());
    self.event(layout, d, nest);
    for content in contents {
      self.walk(&content, nest.saturating_add(1));
    }
  }
}

/// The natural width in em (of its font) of a spacing whatsit (`\hskip`, `\hspace`, `\kern`, `\quad` made as a
/// skip), when it has one.
fn whatsit_width_em(d: &Digested, w: &Whatsit) -> Option<f64> {
  let cs = whatsit_cs(w);
  if !(cs.contains("skip") || cs.contains("space") || cs.contains("kern") || cs.contains("quad")) {
    return None;
  }
  let pt = match w.get_property("width").as_deref() {
    Some(width @ (Stored::Dimension(_) | Stored::Glue(_))) => return width_property_em(d, width),
    _ => match w.get_args().iter().flatten().next() {
      Some(arg) => arg.pt_value(None),
      None => return Some(1.0),
    },
  };
  Some((pt / em_pt(d)).max(0.0))
}

/// The lexer's second pass: the atoms into events.
#[derive(Default)]
struct Lexer {
  events:      Vec<Event>,
  /// the name run being read: its characters with their spans, its font, its nest
  run:         Vec<(char, Option<(u16, u16)>)>,
  run_font:    Option<String>,
  run_nest:    u8,
  /// the spacing run since the last ink: items, width, fill, span
  gap_items:   usize,
  gap_width:   f64,
  gap_fill:    bool,
  gap_span:    Option<(u16, u16)>,
  /// the line has shown ink (a name, a mark, a contact) since its last break
  line_inked:  bool,
  /// the font of the last name run
  last_font:   Option<String>,
  /// the symbol mark being read (consecutive footnote-symbol characters)
  symbol:      String,
  symbol_span: Option<(u16, u16)>,
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

  fn push_event(&mut self, mut event: Event) {
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

  fn ink(&mut self, text: &str, span: Option<(u16, u16)>, font: Option<String>, nest: u8) {
    for c in text.chars() {
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

  fn space(&mut self, width: f64, fill: bool, span: Option<(u16, u16)>) {
    self.flush_symbol();
    self.gap_items = self.gap_items.saturating_add(1);
    self.gap_width += width;
    self.gap_fill |= fill;
    self.gap_span = join_span(self.gap_span, span);
  }

  fn finish(mut self) -> Vec<Event> {
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

/// The events of a digested author block (a rehearsal's list), in print order.
pub(in crate::base_utilities) fn lex_events(digested: &Digested) -> Vec<Event> {
  let mut walker = Walker::default();
  walker.walk(digested, 0);
  let mut lexer = Lexer::default();
  for atom in walker.atoms {
    match atom {
      Atom::Ink { text, span, font, nest } => lexer.ink(&text, span, font, nest),
      Atom::Space { width, fill, span } => lexer.space(width, fill, span),
      Atom::Event(event) => lexer.push_event(event),
    }
  }
  lexer.finish()
}
