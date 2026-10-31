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
//!   rebinds `\\` to; an alignment's row end, role `row`; one inside a braced group or argument, role `group`);
//! - **Contact**: `\email`, `\url`, `\href`, `\orcidlink`, an in-block `\affil`/`\address`, the frontmatter API;
//! - **Note**: `\thanks`/`\footnote`/`\footnotetext`, with its label (a `\label` inside it), its inner contacts and its
//!   Marks (the one it opens with `line_first`);
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

mod fonts;
mod lexer;
pub(in crate::base_utilities) use fonts::{
  FontScope, InkFonts, Wrap, changes_font, entry_font, font_declaration, switch_free,
  text_font_command,
};
use lexer::Lexer;
pub(in crate::base_utilities) use lexer::{is_fnsymbol, is_separator_word};

/// The kind of a separator event.
#[derive(Debug, Clone, PartialEq)]
pub enum SepKind {
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
pub enum EventKind {
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
  pub fn name(&self) -> &'static str {
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

/// Whose meaning an e-mail call has when its note is rehearsed (at the flush, `\maketitle`; `ledger::email_meaning`,
/// the rehearsal's binding of `\email` and `\emailAdd`): what a document note's e-mail call is read by
/// (`Resolver::api`, review rev92a_r3 B1; review rev92a_r4 B1 on the timing, OD #913).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmailMeaning {
  /// a binding's, whose code routes the address to the current author (amsart's, REVTeX's, jheppub's `\emailAdd`)
  #[default]
  Binding,
  /// the document class's binding's, which prints the address where it is called (`Policy::email_in_place`: llncs,
  /// aa, svjour)
  InPlace,
  /// no binding's (the document's own `\renewcommand{\email}`, a file read raw): it prints wherever its definition
  /// prints, so a note's call stays the note's text
  Own,
}

impl EmailMeaning {
  /// Its name in the shadow dump (`"meaning"`; a binding's, the default, is not written).
  pub fn name(self) -> &'static str {
    match self {
      EmailMeaning::Binding => "binding",
      EmailMeaning::InPlace => "in-place",
      EmailMeaning::Own => "own",
    }
  }

  /// The meaning a dump's name gives (`ab_audit fmshadow`); a binding's for any other.
  pub fn from_name(name: &str) -> Self {
    match name {
      "in-place" => EmailMeaning::InPlace,
      "own" => EmailMeaning::Own,
      _ => EmailMeaning::Binding,
    }
  }
}

/// One event of an author block.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
  pub kind:         EventKind,
  /// the positions of the author field it covers, `start..end`
  pub span:         Option<(u16, u16)>,
  /// its ink: a name's text, a mark's printed text, a contact's or a note's shown text
  pub text:         String,
  /// a mark's label, a note's `\label`, a contact's optional label
  pub label:        Option<String>,
  /// a mark printed as a footnote symbol
  pub symbol:       bool,
  /// a mark that is its line's first ink
  pub line_first:   bool,
  /// a contact's role, a mark's or a layout's source (`sup`, `math`, `inst`, `text`; the container's command)
  pub role:         Option<String>,
  /// a link's target
  pub target:       Option<String>,
  /// a note's inner contacts and Marks (the one its text opens with `line_first`, the others not)
  pub inner:        Vec<Event>,
  /// how many layout containers enclose it
  pub nest:         u8,
  /// a mark's run of footnote symbols that a line of its block leads with and a name shows, written together (`§†`):
  /// one label, as `††` is (the resolver's `glue_shown_runs`; main-session decision 2 on FM S2a.2 r7, 0907.1243)
  pub glued:        bool,
  /// a mark set off by a printed space from the ink or separator before it, and from the name after it: which of the
  /// two TeX's output sets it next to (the resolver's `nearer_next`; `Han,${}^1$ Thibault` prints "Han,¹ Thibault")
  pub space_before: bool,
  pub space_after:  bool,
  /// an e-mail contact's: whose meaning made the call ([`EmailMeaning`])
  pub meaning:      EmailMeaning,
  /// a note's: nothing in it prints ([`leaves_no_ink`] on its digested body, the test the XML's note is dropped by:
  /// `\thanks{}`, `\thanks{~}`, OXIDIZED_DESIGN_DIVERGENCES #916)
  pub inkless:      bool,
  /// an in-block annotation's (`\lx@annotate@frontmatter`) `annotate=` option: the authors today's API gives it (`run`,
  /// `new`, a number, `all`; Perl Base_Utility.pool.ltxml:520-536)
  pub annotate:     Option<String>,
  /// a cut in-block contact entry's: the font changes in effect where the writer's cut of it starts, against its call's
  /// (`fonts::entry_font`; `Font::default()` for none), which the writer digests it under — `None` when not read
  pub font:         Option<Rc<Font>>,
  /// a cut in-block contact entry's: the containers holding it and another entry, which the writer wraps it in again
  /// (`fonts::Wrap`, `writer::container_wrap`)
  pub wrap:         Option<Rc<Wrap>>,
}

impl Event {
  pub fn new(kind: EventKind) -> Self {
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
      glued: false,
      space_before: false,
      space_after: false,
      meaning: EmailMeaning::Binding,
      inkless: false,
      annotate: None,
      font: None,
      wrap: None,
    }
  }
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
/// `rehearsal::load`'s `\lx@fm@obs@fnmark`), empty where that failed (`\@fnsymbol` past 9, "Counter too
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

/// The cross-reference commands whose key a mark can print ([`printless_ref_key`]): sect11.rs `\ref OptionalMatch:*
/// Semiverbatim` (`\pageref` is `\let` to it, sect11.rs:164, so it reads as one), hyperref's `\autoref`, amsmath's
/// `\eqref`, cleveref's `\lx@cref` (its `\cref`/`\Cref`, cleveref_sty.rs) — each with the key its last argument. Not
/// `\href` (a link, its last argument the text printed), nor `\nameref` (a title, never a label's mark). cleveref's
/// `\namecref{x}` is `\lx@cref{creftype}{x}` too (cleveref_sty.rs:312), a type's name, not a number: the key is read
/// only off a reference printing nothing at digestion ([`script_mark`]), and in the rehearsal `\lx@cref` prints its
/// show list ("creftype", "creftype~creflabel"), so a `\namecref` mark is that text, never `ref:x`.
const REF_COMMANDS: &[&str] = &["\\ref", "\\autoref", "\\eqref", "\\lx@cref"];

/// The key of a cross-reference whatsit printing nothing at digestion — one of [`REF_COMMANDS`], whose reference
/// resolves in post —, its last argument (Semiverbatim), if `w` is one.
fn printless_ref_key(w: &Whatsit) -> Option<String> {
  if !REF_COMMANDS.contains(&whatsit_cs(w).trim_end()) {
    return None;
  }
  let mut printed = String::new();
  for (arg, shown) in w.get_args().iter().zip(shown_arguments(w)) {
    if shown && let Some(arg) = arg {
      ink_text(arg, &mut printed);
    }
  }
  if let Ok(Some(body)) = w.get_body() {
    ink_text(&body, &mut printed);
  }
  if !printed.trim().is_empty() {
    return None;
  }
  let key = ink_of(w.get_args().iter().rev().flatten().next()?)
    .trim()
    .to_string();
  (!key.is_empty()).then_some(key)
}

/// Whether `d` holds a cross-reference printing nothing at digestion ([`printless_ref_key`]), at any depth.
fn holds_printless_ref(d: &Digested) -> bool {
  match d.data() {
    DigestedData::List(l) => l.borrow().boxes.iter().any(holds_printless_ref),
    DigestedData::Whatsit(w) => {
      let w = w.borrow();
      printless_ref_key(&w).is_some()
        || w.get_args().iter().flatten().any(holds_printless_ref)
        || w
          .get_body()
          .ok()
          .flatten()
          .as_ref()
          .is_some_and(holds_printless_ref)
    },
    _ => false,
  }
}

/// The event of a mark's script (`\inst{…}`, `\textsuperscript{…}`, a formula's `^{…}`, made by `role`): the notes it
/// holds aside ([`notes_in`]), a script holding a cross-reference that prints nothing at digestion is a Mark whose
/// label is the script in print order, each such reference written as its key prefixed `ref:` — the resolver matches
/// it against the `\label`s inside the labelled lines and affiliations (S2a.2, review revfms2a1_2's open question (a)):
/// A&A's `\inst{\ref{ipag}}` is `ref:ipag` (1809.08354), `\inst{\ref{ia},\ref{ib}}` one Mark `ref:ia,ref:ib` carrying
/// both keys (1707.06238, 1302.1442: the comma between them was read as the separator between two marks, and the mark
/// lost), `\inst{1,\ref{ib}}` `1,ref:ib`; any other script is [`mark_event`]'s, by what it prints.
fn script_mark(script: Option<&Digested>, role: &'static str, notes: &[Digested]) -> Option<Event> {
  let role = parted_role(role, notes);
  let Some(script) = script else {
    return mark_event(String::new(), role);
  };
  if !holds_printless_ref(script) {
    return mark_event(ink_of_without_notes(script), role);
  }
  let mut mark = Event::new(EventKind::Mark);
  mark.label = Some(script_ink(script, true));
  mark.text = ink_of_without_notes(script);
  mark.role = Some(s!("{role} ref"));
  Some(mark)
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

/// Whether a whatsit is one of the rehearsal's notes (`\thanks`, `\footnotetext`), footnote marks (`\footnotemark`) or
/// ORCID links (the rehearsal's `\orcidlink`/`\orcid`, the kernel's `\lx@orcidlink`, a link to an ORCID record): inside
/// a mark's script, each is an event of its own after the mark, not the mark's text
/// (`Martin Bachratý$^{\orcidlink{0000-0002-4300-7507},1,\ast}$`, 2506.11626: the ORCID was lost;
/// `\inst{\ref{inst1} \orcidicon{…}}`, `\inst{1,2,\orcidicon{…}}`, `\orcidicon` a paper's
/// `\href{https://orcid.org/#1}{…tikzpicture…}`, 2407.07678, 2605.02769, 2608.18287: the logo's path data was read as
/// the mark's labels, its digits parted onto institutes 1-7).
fn is_note_whatsit(w: &Whatsit) -> bool {
  match whatsit_cs(w).as_str() {
    "\\lx@fm@obs@note"
    | "\\lx@fm@obs@notetext"
    | "\\lx@fm@obs@fnmark"
    | "\\lx@fm@obs@orcid"
    | "\\lx@orcidlink" => true,
    "\\lx@fm@obs@href" => href_target(w).contains("orcid.org/"),
    _ => false,
  }
}

/// A whatsit's argument `n` as its tokens read (a link's target, a contact's role), empty when it has none: the one
/// reader of the walker's `raw` and [`is_note_whatsit`] (review rev92a_r2 N2).
fn raw_arg(w: &Whatsit, n: usize) -> String {
  w.get_arg(n).map(|a| a.to_string()).unwrap_or_default()
}

/// An e-mail call's argument as it prints (`raw_arg`), each line break it holds — the rehearsal's `\lx@fm@obs@break`,
/// `\\` inside the verbatim argument of llncs's `\email{a@x.org \\ b@y.org}`, `{\tt#1}` printing one address a line —
/// a space between its lines: their addresses are several, each written out (2608.29307's three addresses under one
/// `\email` were one address, `…@gmail.com\lx@fm@obs@breakn.van.stein@…`, which spelled its first author). A link in
/// it is what it prints: an `\href`'s text, never its target (`\email{\href{mailto:ann@a.org}{ann@a.org}}` printed one
/// address, read twice from the target and the text — two contacts written for one print: aa 2210.16066, 1907.11241,
/// 2007.06573, 2103.15844, llncs 2405.04152, 2603.13900; `\href{mailto:ann@a.org}{Ann}` prints no address), a `\url`'s
/// argument (review rev92c B1).
fn raw_lines(w: &Whatsit, n: usize) -> String {
  fn walk(d: &Digested, out: &mut String) {
    match d.data() {
      DigestedData::List(l) => {
        for child in l.borrow().boxes.iter() {
          walk(child, out);
        }
      },
      DigestedData::Whatsit(w) => {
        let w = w.borrow();
        match whatsit_cs(&w).as_str() {
          "\\lx@fm@obs@break" => out.push(' '),
          "\\lx@fm@obs@href" => {
            if let Some(text) = w.get_arg(2) {
              walk(text, out);
            }
          },
          "\\lx@fm@obs@url" => {
            if let Some(text) = w.get_arg(1) {
              walk(text, out);
            }
          },
          _ => out.push_str(&d.to_string()),
        }
      },
      _ => out.push_str(&d.to_string()),
    }
  }
  let mut out = String::new();
  if let Some(arg) = w.get_arg(n) {
    walk(arg, &mut out);
  }
  out
}

/// The target of the rehearsal's link (`\lx@fm@obs@href{target}{text}`), trimmed: the contact's target the walker
/// writes, and what [`is_note_whatsit`] tests for an ORCID record.
fn href_target(w: &Whatsit) -> String { raw_arg(w, 1).trim().to_string() }

/// The notes and footnote marks a mark's script holds (`$^{1,\thanks{…}}$`, `\inst{3\footnotemark[1]}`), outermost
/// first.
fn notes_in(d: &Digested) -> Vec<Digested> {
  fn collect(d: &Digested, out: &mut Vec<Digested>) {
    match d.data() {
      DigestedData::List(l) => {
        for child in l.borrow().boxes.iter() {
          collect(child, out);
        }
      },
      DigestedData::Whatsit(w) => {
        let w = w.borrow();
        if is_note_whatsit(&w) {
          out.push(d.clone());
          return;
        }
        for arg in w.get_args().iter().flatten() {
          collect(arg, out);
        }
        if let Ok(Some(body)) = w.get_body() {
          collect(&body, out);
        }
      },
      _ => {},
    }
  }
  let mut out = Vec::new();
  collect(d, &mut out);
  out
}

/// A script mark's role: `role`, with ` parted` when the script held a footnote mark the lexer parted from it
/// (`\inst{1, 2\footnotemark[1]}`, 2404.09857).
fn parted_role(role: &'static str, notes: &[Digested]) -> &'static str {
  let parted = notes.iter().any(|note| match note.data() {
    DigestedData::Whatsit(w) => whatsit_cs(&w.borrow()) == "\\lx@fm@obs@fnmark",
    _ => false,
  });
  match (role, parted) {
    ("inst", true) => "inst parted",
    ("sup", true) => "sup parted",
    ("math", true) => "math parted",
    _ => role,
  }
}

/// [`ink_of`] a mark's script without the notes it holds ([`notes_in`]).
fn ink_of_without_notes(d: &Digested) -> String { script_ink(d, false) }

/// A mark's script as [`ink_of_without_notes`] reads it, and with `refs` each cross-reference printing nothing at
/// digestion written as its key, prefixed `ref:` ([`printless_ref_key`], [`script_mark`]), a list item of its own: a
/// comma parts it from the ink or the reference beside it, whatever printed between them (`\ref{ia}\ref{ib}`
/// `ref:ia,ref:ib`; `1\ref{b}` `1,ref:b`; `\ref{a}--\ref{c}` `ref:a,–,ref:c`; review rev92a_r1 B3, where the keys
/// written back to back were one label, `ref:iaref:ib`, that matched no entry). Spacing (`is_spacing_item`, the reader
/// `first_ink` and the kernel's label keys use: `$^{1\,2}$`'s glue, `\!`'s zero-width-space box, a `\kern`) is a space
/// between labels (`$^{1\kern2pt2}$` was read "12", one label), a format character none and a combining mark part of
/// the character before it (`keeps_label_char`: `\up{\!1,2}` was a zero-width-space label of its own, 2209.14855 —
/// review rev92b_r1 B1; `\={x}` is `x̄`, not `x`, review rev92b_r2 B1), and a pgf/TikZ picture prints only its text
/// nodes (`picture_text`: a circled `a` its `a`, an SVG path's data nothing — review rev92b_r2 B2), apart
/// (`queue_digest::label_ink` reads a label so).
fn script_ink(d: &Digested, refs: bool) -> String {
  fn ink(d: &Digested, refs: bool, out: &mut String) {
    if is_spacing_item(d) {
      out.push(' ');
      return;
    }
    match d.data() {
      DigestedData::List(l) => {
        for child in l.borrow().boxes.iter() {
          ink(child, refs, out);
        }
      },
      DigestedData::Whatsit(w) if is_note_whatsit(&w.borrow()) => {},
      // (a pgf picture prints only its text nodes, apart: `picture_text`)
      DigestedData::Whatsit(w) if let Some(text) = picture_text(&w.borrow()) => {
        out.push(' ');
        for node in &text {
          ink(node, refs, out);
        }
        out.push(' ');
      },
      DigestedData::Whatsit(w) if refs && let Some(key) = printless_ref_key(&w.borrow()) => {
        out.push_str(",ref:");
        out.push_str(&key);
        out.push(',');
      },
      // (what any other whatsit shows, walked as `ink_text` walks it, its notes and printless references read here)
      DigestedData::Whatsit(w) => {
        let w = w.borrow();
        if let Some(Stored::Digested(a)) = w.get_property("alignment").as_deref() {
          ink(a, refs, out);
        }
        for (arg, shown) in w.get_args().iter().zip(shown_arguments(&w)) {
          if shown && let Some(arg) = arg {
            ink(arg, refs, out);
          }
        }
        if let Ok(Some(body)) = w.get_body() {
          ink(&body, refs, out);
        }
        // (a math accent its glyph on its base, `accent_combining_form`: `\bar{\dag}` is `†̄`)
        out.push_str(&accent_combining_form(&w).unwrap_or_default());
      },
      _ => ink_text(d, out),
    }
  }
  let mut text = String::new();
  ink(d, refs, &mut text);
  let text = text.chars().fold(String::new(), |mut kept, c| {
    if keeps_label_char(c, kept.chars().next_back()) {
      kept.push(c);
    }
    kept
  });
  let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
  if !refs {
    return text;
  }
  // (the list's items, each trimmed, the empty ones a reference's own commas made beside a printed one dropped)
  text
    .split(',')
    .map(str::trim)
    .filter(|item| !item.is_empty())
    .collect::<Vec<_>>()
    .join(",")
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
  /// how many lists enclose the box being walked (the block's own is one; a braced group or argument inside it more)
  lists: usize,
  /// each in-block contact cut into its marked entries (`labelled_entries`): the entries' spans, and the one entry its
  /// leading marks label, read when a cut entry's marks are no name's ([`lex_events`])
  cuts:  Vec<(Vec<(u16, u16)>, Event)>,
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
        self.lists += 1;
        for child in l.borrow().boxes.iter() {
          self.walk(child, nest);
        }
        self.lists -= 1;
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
    let raw = |n: usize| raw_arg(w, n);
    match cs.as_str() {
      "\\lx@fm@obs@and" => self.event(Event::new(EventKind::Sep(SepKind::And("\\and"))), d, nest),
      "\\lx@fm@obs@And" => self.event(Event::new(EventKind::Sep(SepKind::And("\\And"))), d, nest),
      "\\lx@fm@obs@AND" => self.event(Event::new(EventKind::Sep(SepKind::And("\\AND"))), d, nest),
      // (a break inside a braced group or argument — `\\ {\em College …,\\ City}`, `\textsl{Pure Mathematics\\University
      // of Waterloo\\…}` — is role `group`: the lines it parts are one print unit, as today's reads them, 1710.07902,
      // math0610612)
      "\\lx@fm@obs@break" => {
        let mut row = Event::new(EventKind::Break);
        row.role = (self.lists > 1).then(|| "group".to_string());
        self.event(row, d, nest)
      },
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
          note.inkless = leaves_no_ink(body);
          let events = lex_events(body);
          // (its text as printed, its contacts' too: the rehearsal's contact constructors print nothing, so the ink
          // of "available at \url{…}" lacks the address — 2105.10596)
          note.text = printed_text(&events);
          let mut inked = false;
          for inner in events {
            let first = !inked;
            inked |= matches!(
              inner.kind,
              EventKind::Name | EventKind::Mark | EventKind::Contact | EventKind::Note
            );
            match inner.kind {
              EventKind::Contact => note.inner.push(inner),
              // (the Mark its text opens with, the legend's label: `\thanks{$^{1}$Unit…}`, S2a.2 r2; its later Marks
              // too, never leading — a note of several marked entries is no one label's legend, IEEEtran's
              // `\IEEEcompsocthanksitem \Mark{1}College … \IEEEcompsocthanksitem \Mark{2}College …`, 1802.03043)
              EventKind::Mark => {
                let mut mark = inner;
                mark.line_first = first;
                note.inner.push(mark);
              },
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
        let mut role = if mark.is_empty() {
          source.to_string()
        } else {
          s!("{source} mark={mark}")
        };
        // (the mark a `\thanks` or `\footnote` shows on the name, last: `thanks mark=1 shows=∗` — FM S2a.2 r7,
        // 1607.07280; the resolver's `note_shows`)
        if source == "thanks"
          && let Some(Stored::String(printed)) = w.get_property("printed").as_deref()
        {
          let printed = to_string(*printed);
          if !printed.trim().is_empty() {
            role.push_str(" shows=");
            role.push_str(printed.trim());
          }
        }
        note.role = Some(role);
        self.event(note, d, nest);
      },
      "\\lx@fm@obs@label" => {
        let mut label = Event::new(EventKind::Layout);
        label.role = Some("label".to_string());
        label.label = Some(raw(1).trim().to_string());
        self.event(label, d, nest);
      },
      "\\lx@fm@obs@fnmark" => {
        let printed = match w.get_property("printed").as_deref() {
          Some(Stored::String(printed)) => to_string(*printed),
          _ => String::new(),
        };
        if let Some(mark) = mark_event(printed, "footnotemark") {
          self.event(mark, d, nest);
        }
      },
      // (a note mark of another kind — endnotes.sty's `\endnotemark[1]`, the kernel's `\lx@notemark{endnote}[1]` — is a
      // mark showing its printed tag, its type argument no ink: 1703.08733's names read "Adel Alahmadi endnote")
      "\\lx@notemark" => {
        let printed = match w.get_property("mark").as_deref() {
          Some(Stored::Digested(mark)) => ink_of(mark),
          _ => String::new(),
        };
        if let Some(mark) = mark_event(printed, "footnotemark") {
          self.event(mark, d, nest);
        }
      },
      "\\lx@fm@obs@mark" => {
        // (a `\footnotemark` inside the mark is a mark of its own after it, its footnote's: `Kui Wu\inst{3\footnotemark[1]}`
        // printed "31", parted into "3" and "1" — llncs, 2404.09857)
        let notes = w.get_arg(1).map(notes_in).unwrap_or_default();
        if let Some(mark) = script_mark(w.get_arg(1), "inst", &notes) {
          self.event(mark, d, nest);
        }
        self.walk_notes(&notes, nest);
      },
      "\\lx@fm@obs@request" => {
        let prefix = raw(1).trim().to_string();
        let labels: Vec<String> = raw(2)
          .split(',')
          .map(str::trim)
          .filter(|label| !label.is_empty())
          .map(|label| {
            if prefix.is_empty() {
              label.to_string()
            } else {
              s!("{prefix}:{label}")
            }
          })
          .collect();
        // (a request of a separator alone is still a mark, of no label: `J. Edelstein\altaffilmark, E. Korpela` hands
        // the comma to `\altaffilmark`, which prints it superscripted after the name — astro-ph0601584)
        if !labels.is_empty() || !raw(2).trim().is_empty() {
          let mut mark = Event::new(EventKind::Mark);
          mark.label = (!labels.is_empty()).then(|| labels.join(","));
          mark.role = Some("request".to_string());
          self.event(mark, d, nest);
        }
      },
      // (a citation in a name is a mark of it, no request: revtex's `\author{Kari Enqvist\cite{byline}}` points at a
      // bibliography note, astro-ph9907221, cond-mat0305028, gr-qc9405060 — its ink "cite[]" was read into the name)
      "\\lx@@cite" | "\\lx@cite" => {
        let mut mark = Event::new(EventKind::Mark);
        mark.role = Some("cite".to_string());
        self.event(mark, d, nest);
      },
      "\\textsuperscript" | "\\@textsuperscript" => {
        let notes = w.get_arg(1).map(notes_in).unwrap_or_default();
        if let Some(mark) = script_mark(w.get_arg(1), "sup", &notes) {
          self.event(mark, d, nest);
        }
        self.walk_notes(&notes, nest);
      },
      "\\lx@fm@obs@email"
      | "\\lx@fm@obs@email@inplace"
      | "\\lx@fm@obs@email@own"
      | "\\lx@fm@obs@orcid"
      | "\\lx@fm@obs@affil"
      | "\\lx@fm@obs@affil@run" => {
        let mut contact = Event::new(EventKind::Contact);
        contact.role = Some(
          match cs.as_str() {
            "\\lx@fm@obs@orcid" => "orcid",
            "\\lx@fm@obs@affil" | "\\lx@fm@obs@affil@run" => "affiliation",
            _ => "email",
          }
          .to_string(),
        );
        // (a raw file's in-place setter the API took, `\lx@add@affiliation[annotate=run]` — the run of new authors'
        // entry, as today's attaches it: ci2018's `\affil` after `JORGE RAMIREZ and EVGENY KRIVOSHEEV`, 1805.12376)
        if cs == "\\lx@fm@obs@affil@run" {
          contact.annotate = Some("run".to_string());
        }
        contact.meaning = match cs.as_str() {
          "\\lx@fm@obs@email@inplace" => EmailMeaning::InPlace,
          "\\lx@fm@obs@email@own" => EmailMeaning::Own,
          _ => EmailMeaning::Binding,
        };
        let label = arg(1);
        contact.label = (!label.is_empty()).then_some(label);
        contact.text = if cs.starts_with("\\lx@fm@obs@affil") {
          arg(2)
        } else {
          raw_lines(w, 2).trim().to_string()
        };
        // (an in-block affiliation without a label of its own that opens with marks is the entry they label, as an API
        // entry is (`leading_marks`): ar-1col.cls's `\affil{$^1$Max Planck …}` inside `\author{}`, 1602.05576,
        // 2405.08158, the names showing `1` requesting it — its text the rest of its events, not the ink "1Max Planck"
        // given to the name beside it; one of several marked entries, `\affil{$^1$Univ A\\ $^2$Univ B}`, each entry by
        // its own leading marks, as the API's reading cuts a call (`mark_led_pieces`: jetpl's `\address{$^1$…\\
        // $^2$…}`, cond-mat0611585) — never one entry under the first mark with the second dropped (review rev94b B1).
        // Entries the writer could not cut — one of no text, one holding a mark past its leading run, entries whose
        // texts are not the contact's, a span no balanced run holds — are the one entry its leading marks label, as
        // before: today's one contact, written on the names requesting its first marks; never a nameless creator's
        // (review rev94b r2 B1: `$^1$\\ $^2$Univ B`, `Town and $^2$Univ B`, `Town$^2$`, a font a cut lost)
        if cs.starts_with("\\lx@fm@obs@affil")
          && contact.label.is_none()
          && let Some(body) = w.get_arg(2)
        {
          let events = lex_events(body);
          // (the font each entry's cut starts in, read from the call's digested ink: its first token's after the call's
          // own — the content's start —, against the call's font; review rev94b r3 B1)
          let scope = d
            .get_font()
            .ok()
            .flatten()
            .zip(box_span(d))
            .map(|(root, (at, _))| FontScope {
              ink: InkFonts::of(body),
              root,
              start: usize::from(at) + 1,
            });
          if let Some((entries, whole)) = labelled_entries(&events, scope.as_ref()) {
            let entry_event = |e: LabelledEntry| {
              let LabelledEntry { label, text, span, font, wrap } = e;
              let mut entry = contact.clone();
              entry.label = Some(label);
              entry.text = text;
              entry.font = font;
              entry.wrap = wrap;
              // (its span its own ink, which the writer makes the entry from (`plan::block_span_tokens`): not the
              // call, whose own typesetting — a document's `\newcommand{\affil}[1]{\\{\footnotesize #1}}` — is no
              // part of it)
              entry.span = span.or_else(|| deep_span(d));
              entry.nest = nest;
              entry
            };
            if entries.len() > 1 {
              let spans = entries.iter().filter_map(|e| e.span).collect();
              self.cuts.push((spans, entry_event(whole)));
            }
            for entry in entries {
              self.atoms.push(Atom::Event(entry_event(entry)));
            }
            return;
          }
        }
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
        contact.target = Some(href_target(w));
        contact.text = arg(2);
        self.event(contact, d, nest);
      },
      // (the kernel's ORCID link, frontmatter_fields.rs `\lx@orcidlink{id}{content}` — orcidlink.sty's `\orcidlink`
      // family, `\lx@add@orcid` —: a link to the iD's record, its first argument no ink; `\orcidlinki{José-María
      // Martín-Olalla}{0000-0002-3750-9113}` printed the iD into the name, 2602.03244; a link printing only the logo
      // is the iD, as `\lx@add@orcid`'s contact holds it)
      "\\lx@orcidlink" => {
        let mut contact = Event::new(EventKind::Contact);
        let id = raw(1).trim().to_string();
        contact.role = Some("url".to_string());
        contact.target = Some(s!("https://orcid.org/{id}"));
        contact.text = Some(arg(2))
          .filter(|text| !text.trim().is_empty())
          .unwrap_or(id);
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
        event.annotate = options.split(',').find_map(|kv| {
          kv.trim()
            .strip_prefix("annotate=")
            .map(|a| a.trim().to_string())
        });
        event.text = arg(4);
        self.event(event, d, nest);
      },
      // Invisible ink (`\phantom`, `\hphantom`, `\vphantom`, math_common.rs; plain.tex/latex.ltx `\finph@nt`): pdflatex
      // prints a box of its argument's size and no ink — a space as wide as the argument (a `\vphantom` none), the
      // notes set inside it still read (2406.07529's `\vphantom{\thanks{Equal contribution.}}`). Its ink was a name:
      // 0909.3056's and 1507.02405's `{R.~Caimmi}\footnote{…}\phantom{agga}` an author "agga", hep-th9806168's
      // `J.S. Dowker\thanks{…} \phantom{a}and Klaus Kirsten` one name "aand Klaus Kirsten" (review rev94a r2 B-2).
      "\\phantom" | "\\vphantom" | "\\lx@text@hphantom" | "\\lx@math@hphantom" => {
        let width = match w.get_property("width").as_deref() {
          Some(width) if cs != "\\vphantom" => width_property_em(d, width),
          _ => None,
        }
        .unwrap_or(0.0);
        let at = self.atoms.len();
        if let Some(arg) = w.get_arg(1) {
          self.walk(arg, nest.saturating_add(1));
        }
        let inside = self.atoms.split_off(at);
        if width > 0.0 {
          self.atoms.push(Atom::Space {
            width,
            fill: false,
            span: box_span(d),
          });
        }
        self.atoms.extend(inside.into_iter().filter(|atom| {
          matches!(atom, Atom::Event(e) if matches!(e.kind, EventKind::Note | EventKind::Contact))
        }));
      },
      _ if w.is_math() || cs.contains("math") && w.get_body().ok().flatten().is_some() => {
        self.walk_math(d, w, nest)
      },
      _ => self.walk_container(d, w, &cs, nest),
    }
  }

  /// The notes and footnote marks a mark's script held ([`notes_in`]), each an event of its own after the mark; a
  /// footnote mark so parted from the script says so (role `footnotemark parted`), as the script's mark does
  /// ([`parted_role`]): the PDF prints them glued, `2∗`, and ruling 1 reads them apart.
  fn walk_notes(&mut self, notes: &[Digested], nest: u8) {
    for note in notes {
      if let DigestedData::Whatsit(nw) = note.data() {
        let at = self.atoms.len();
        self.walk_whatsit(note, &nw.borrow(), nest);
        for atom in &mut self.atoms[at..] {
          if let Atom::Event(event) = atom
            && event.role.as_deref() == Some("footnotemark")
          {
            event.role = Some("footnotemark parted".to_string());
          }
        }
      }
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
          let script_box = w.get_args().iter().rev().flatten().next();
          let notes = script_box.map(notes_in).unwrap_or_default();
          if let Some(mark) = script_mark(script_box, "math", &notes) {
            self.event(mark, d, nest);
          }
          // (a `\thanks` inside the script is a note of its own, after the mark, not the mark's text:
          // `Liang-Jian Zou$^{1,\thanks{Correspondence author, …}}$` read "1,Correspondence author, …" as its label,
          // 1106.3485)
          self.walk_notes(&notes, nest);
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
    // (nor a number or a dimension it reads, which sets it: `\scalebox{.7}`, `\rotatebox{180}`, `\raisebox{1pt}` — the
    // ʻokina drawn as `Hawai\scalebox{.7}{\raisebox{…}{\rotatebox{180}{\reflectbox{s}}}}i` read ".7180.0" into the
    // affiliation, 1005.4252, 2301.02699)
    let numeric = numeric_arguments(w);
    let shown_args: Vec<bool> = KEYED_CONSTRUCTORS
      .iter()
      .find(|(name, _)| *name == cs)
      .map_or_else(|| shown_arguments(w), |(_, shown)| shown.to_vec())
      .into_iter()
      .enumerate()
      .map(|(i, shown)| shown && !numeric.get(i).copied().unwrap_or(false))
      .collect();
    // (an environment's arguments are its settings, its body the content: `\begin{CJK*}{UTF8}{gbsn}张嘉懿\end{CJK*}`
    // printed "UTF8gbsn" into the name, 2607.05550, 2503.21605, 2404.12123)
    let environment = cs.starts_with("\\begin{");
    for (arg, shown) in args.iter().zip(shown_args) {
      if let Some(arg) = arg
        && shown
        && !environment
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
    // (a keyed constructor showing none of its arguments prints nothing: `\lx@frontmatter@mark`)
    if contents.is_empty() && KEYED_CONSTRUCTORS.iter().any(|(name, _)| *name == cs) {
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

/// Which arguments of a whatsit its definition reads as a number or a dimension (`Float`, `Number`, `Dimension`,
/// `Glue`, `TempboxaDimension`): settings of the box, never ink. All false when the parameters and the arguments do
/// not pair up.
fn numeric_arguments(w: &Whatsit) -> Vec<bool> {
  let types: Vec<SymStr> = w
    .get_definition()
    .get_parameters()
    .map(|params| {
      params
        .get_parameters()
        .into_iter()
        .filter(|p| !p.novalue)
        .map(|p| p.name)
        .collect()
    })
    .unwrap_or_default();
  if types.len() != w.get_args().len() {
    return Vec::new();
  }
  types
    .iter()
    .map(|t| {
      with(*t, |name| {
        ["Float", "Number", "Dimension", "Glue"]
          .iter()
          .any(|kind| name.contains(kind))
      })
    })
    .collect()
}

/// Constructors whose argument is a key the element carries, not ink (`shown_arguments` cannot read a compiled
/// replacement): acmart's address parts, `\lx@acm@addresspart@{institution}{SAS Institute Inc.}` an
/// `ltx_affiliation_institution` text of the second (acmart_cls.rs; 1907.07713); `\scalebox`'s scales; the frontmatter's position marker
/// `\lx@frontmatter@mark{ltx:abstract}` (frontmatter_fields.rs), which prints nothing — an abstract begun inside a
/// revtex `\address` read "ltx:abstract" as an affiliation, cond-mat9801109.
const KEYED_CONSTRUCTORS: &[(&str, &[bool])] = &[
  ("\\lx@acm@addresspart@", &[false, true]),
  ("\\lx@frontmatter@mark", &[false]),
  // (graphics.sty's `\scalebox{h-scale}[v-scale]{text}`, graphics_sty.rs: its scales are plain arguments)
  ("\\scalebox", &[false, false, true]),
];

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

/// The text events print, in order: names, contacts and inner notes as their text, a comma `,`, a separating word its
/// word, the rest a space, the marks aside (a note's text, `\lx@fm@obs@note`).
pub(in crate::base_utilities) fn printed_text(events: &[Event]) -> String {
  let mut out = String::new();
  for event in events {
    let (piece, glued) = match &event.kind {
      EventKind::Name | EventKind::Contact | EventKind::Note => (event.text.as_str(), false),
      // (a mark is no text of the note, as the XML's `ltx:sup` is none: CDF's `With visitors from $^a$Istituto …`,
      // 1203.3894, 1307.5202; its leading one the legend's label, `leading_mark`)
      EventKind::Mark => continue,
      EventKind::Sep(SepKind::Comma) => (",", true),
      EventKind::Sep(SepKind::Word(word)) => (word.as_str(), false),
      _ => {
        if !out.ends_with(' ') && !out.is_empty() {
          out.push(' ');
        }
        continue;
      },
    };
    if piece.is_empty() {
      continue;
    }
    if !glued && !out.is_empty() && !out.ends_with(' ') {
      out.push(' ');
    }
    out.push_str(piece);
  }
  out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The text of events, as printed: names and contacts, a comma's `,`, a word's word, a gap's space; marks aside.
pub(in crate::base_utilities) fn events_text(events: &[&Event]) -> String {
  let mut out = String::new();
  for event in events {
    let piece = match &event.kind {
      EventKind::Name | EventKind::Contact => event.text.as_str(),
      EventKind::Sep(SepKind::Comma) => ",",
      EventKind::Sep(SepKind::Word(word)) => word.as_str(),
      _ => continue,
    };
    if piece == "," {
      out.push(',');
    } else {
      if !out.is_empty() {
        out.push(' ');
      }
      out.push_str(piece);
    }
  }
  out.trim().to_string()
}

/// The run of marks an entry's events lead with — `\affil{$^1$Univ X}`, `\address{$\dagger,\;\ast$\;Univ A}` (one run,
/// its separators no text: math-ph0404036) —, as `start..end` of `events`: what comes before its first ink (a space or a
/// break, a layout, an unprinted piece of punctuation: `\affiliation{ $^a$Fermilab, …}`, 0902.0006; an acmart address
/// part's brace, `{$^1$Gaoling School …`, 2211.15148), then its marks and what parts them. `start == end` when it leads
/// with no mark. The labelled-entry reading of a frontmatter API entry (`resolver/api.rs`) and of an in-block contact
/// (`\lx@fm@obs@affil`) alike.
pub(in crate::base_utilities) fn leading_marks(events: &[Event]) -> (usize, usize) {
  let start = events
    .iter()
    .position(|e| {
      !(matches!(
        e.kind,
        EventKind::Sep(SepKind::Gap { .. }) | EventKind::Break | EventKind::Layout
      ) || unprinted_piece(e))
    })
    .unwrap_or(events.len());
  let mut end = start;
  for (at, e) in events.iter().enumerate().skip(start) {
    if matches!(e.kind, EventKind::Mark | EventKind::Layout) {
      end = at + 1;
    } else if !between_leading_marks(e) {
      break;
    }
  }
  (start, end)
}

/// A Name event printing no letter or digit (punctuation alone): no text of an entry before or between its marks.
pub(in crate::base_utilities) fn unprinted_piece(e: &Event) -> bool {
  e.kind == EventKind::Name && !e.text.chars().any(char::is_alphanumeric)
}

/// What parts two marks of an entry's leading run and is no text of it ([`leading_marks`]).
pub(in crate::base_utilities) fn between_leading_marks(e: &Event) -> bool {
  unprinted_piece(e) || matches!(e.kind, EventKind::Sep(SepKind::Comma | SepKind::Gap { .. }))
}

/// Whether an event is ink a line's type is read from (not layout, not a separator).
pub(in crate::base_utilities) fn inked(event: &Event) -> bool {
  matches!(
    event.kind,
    EventKind::Name | EventKind::Mark | EventKind::Contact | EventKind::Note
  )
}

/// The entries of an API call's or an in-block contact's content whose first ink is a mark, when two or more marks open one — a mark at its
/// start, after a line break or after a separator (a comma, a gap, a word) with ink since the entry before (`$^{*}$Univ
/// A,\\$^{+}$Univ B`, cond-mat0510562; `$^1$SJTU, $^2$Beihang`, 1809.03738): each entry its events up to the next,
/// the separators between them dropped. A mark glued to the ink before it (`Univ A$^{1}$`) opens none. Empty when the
/// content is one entry.
pub(in crate::base_utilities) fn mark_led_pieces(events: &[Event]) -> Vec<Vec<Event>> {
  mark_led_pieces_keeping(events, false)
}

/// [`mark_led_pieces`], each piece keeping, with `punctuation`, the punctuation it prints at the end of its line
/// (`$^1$Univ A, Town;\\ $^2$Univ B` keeps its `;` — review rev94b r2 N4; a comma between two entries of one line,
/// `Town, $^2$Univ B`, parts them, as between an API call's): an in-block contact's entries are its printed lines
/// (`labelled_entries`).
fn mark_led_pieces_keeping(events: &[Event], punctuation: bool) -> Vec<Vec<Event>> {
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
    let labels = resolver::mark_labels(e.label.as_deref().unwrap_or(""));
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
    // (punctuation printed at the end of its line is the line's; between two entries of one line, none)
    let keep = punctuation && {
      let ink = piece
        .iter()
        .rposition(|e| inked(e) && e.text.chars().any(char::is_alphanumeric))
        .unwrap_or(0);
      piece[ink..].iter().any(|e| e.kind == EventKind::Break)
    };
    while piece.last().is_some_and(|e| match &e.kind {
      EventKind::Sep(SepKind::Comma) => !keep,
      EventKind::Break | EventKind::Sep(_) => true,
      EventKind::Name => !keep && !e.text.chars().any(char::is_alphanumeric),
      _ => false,
    }) {
      piece.pop();
    }
  }
  if pieces.len() < 2 { Vec::new() } else { pieces }
}

/// An in-block contact's entry read from its leading marks: its labels, its text, its span, and for an entry of a cut
/// the font its cut starts in and the containers it is wrapped in again ([`labelled_entries`]).
#[derive(Clone)]
struct LabelledEntry {
  label: String,
  text:  String,
  span:  Option<(u16, u16)>,
  font:  Option<Rc<Font>>,
  wrap:  Option<Rc<Wrap>>,
}

/// An author block's first position in the author field and its tokens ([`with_block_content`]).
type BlockContent = Option<(usize, Tokens)>;

thread_local! {
  /// The author block being rehearsed, while it is: what the writer cuts an in-block contact's entries from, which
  /// [`labelled_entries`] checks them by.
  static BLOCK_CONTENT: std::cell::RefCell<BlockContent> = const { std::cell::RefCell::new(None) };
}

/// Run `f` (the rehearsal of an author block, `shadow.rs`) with the block's `tokens`, its first at position `base`,
/// as the content its in-block contacts' entries are cut from ([`labelled_entries`]).
pub(in crate::base_utilities) fn with_block_content<R>(
  base: usize,
  tokens: Tokens,
  f: impl FnOnce() -> R,
) -> R {
  let saved = BLOCK_CONTENT.with(|b| b.replace(Some((base, tokens))));
  let out = f();
  BLOCK_CONTENT.with(|b| *b.borrow_mut() = saved);
  out
}

/// The entry `events` lead with marks for ([`leading_marks`]): its labels, its text (the rest, `events_text`) and its
/// span, its own ink past them (the writer makes an entry of the block from it, the mark left out,
/// `plan::block_span_tokens`). `None` when it leads with no labelled mark.
fn labelled_entry(events: &[Event]) -> Option<LabelledEntry> {
  let (start, end) = leading_marks(events);
  let labels: Vec<&str> = events[start..end]
    .iter()
    .filter(|e| e.kind == EventKind::Mark)
    .filter_map(|e| e.label.as_deref().filter(|l| !l.is_empty()))
    .collect();
  if labels.is_empty() {
    return None;
  }
  let rest: Vec<&Event> = events
    .iter()
    .enumerate()
    .filter(|&(at, e)| !((start..end).contains(&at) && between_leading_marks(e)))
    .map(|(_, e)| e)
    .collect();
  let span = events[end..]
    .iter()
    .filter_map(|e| e.span)
    .reduce(|(a, b), (s, e)| (a.min(s), b.max(e)));
  Some(LabelledEntry {
    label: labels.join(","),
    text: events_text(&rest),
    span,
    font: None,
    wrap: None,
  })
}

/// The entries of an in-block contact's content each led by marks ([`mark_led_pieces_keeping`], its printed
/// punctuation kept), each by [`labelled_entry`]; else the content one entry its leading marks label (today's one
/// contact: written on the names requesting them, as before the cut) — whenever an entry is one the
/// writer could not cut from the block: one leading with no labelled mark, one of no text (`$^1$\\ $^2$Univ B`), one
/// holding a mark past its leading run, which its text would drop (`$^1$Univ A$^2$`), entries whose texts joined are
/// not the content's (`Town and $^2$Univ B`: the word between them no entry's), or a span the writer's cut
/// (`plan::span_tokens`, on the block being rehearsed: [`with_block_content`]) holds in no balanced run of the block
/// or reads otherwise — review rev94b r2 B1: such a cut left today's contact on a nameless creator. `None` when the
/// content leads with no labelled mark.
fn labelled_entries(
  events: &[Event],
  scope: Option<&FontScope>,
) -> Option<(Vec<LabelledEntry>, LabelledEntry)> {
  let whole = labelled_entry(events)?;
  let norm = |text: &str| -> String {
    text
      .chars()
      .filter(|c| c.is_alphanumeric())
      .flat_map(char::to_lowercase)
      .collect()
  };
  let pieces = mark_led_pieces_keeping(events, true);
  let entries: Option<Vec<LabelledEntry>> = pieces
    .iter()
    .map(|piece| {
      let (_, end) = leading_marks(piece);
      if piece[end..].iter().any(|e| e.kind == EventKind::Mark) {
        return None;
      }
      labelled_entry(piece).filter(|e| e.span.is_some() && !norm(&e.text).is_empty())
    })
    .collect();
  // (each entry with the font its cut starts in, `fonts::entry_font`, and the containers holding it and another entry
  // it is wrapped in again, `writer::container_wrap` — review rev94b r3 B1: a font open from earlier in the block was
  // lost from every entry after the first. A font the boxes cannot tell, a container the writer cannot wrap again
  // exactly, no font of the call or no block content to read it from never undo the cut: the entry is cut as before
  // fonts were read, the switches of the group it sits in its own — review rev94b r4 B1: `$^1$\textbf{Univ A}\\
  // $^2$\textbf{Univ B}` was one contact on the first name. Only what the writer could not cut at all stays one entry.)
  let cut = |entries: Vec<LabelledEntry>| -> Option<Vec<LabelledEntry>> {
    if entries.len() < 2
      || entries.iter().map(|e| norm(&e.text)).collect::<String>() != norm(&whole.text)
    {
      return None;
    }
    let spans: Vec<(u16, u16)> = entries.iter().filter_map(|e| e.span).collect();
    if spans.len() != entries.len() {
      return None;
    }
    BLOCK_CONTENT.with(|block| {
      let block = block.borrow();
      let (base, tokens) = block.as_ref()?;
      let content = tokens.unlist_ref();
      // (where each entry's cut starts: no such start is no cut the writer could make, `writer::span_tokens`)
      let starts: Vec<(usize, (u16, u16))> = spans
        .iter()
        .map(|&span| Some((writer::labelled_start(content, *base, span)?, span)))
        .collect::<Option<_>>()?;
      let forms = match scope {
        Some(scope) => writer::entry_forms(content, *base, &starts, scope, events),
        None => vec![(None, None); entries.len()],
      };
      entries
        .into_iter()
        .zip(forms)
        .map(|(entry, (font, wrap))| {
          writer::span_tokens(
            content,
            *base,
            entry.span?,
            Some(&norm(&entry.text)),
            None,
            true,
            false,
            font.is_some(),
          )?;
          Some(LabelledEntry { font, wrap, ..entry })
        })
        .collect()
    })
  };
  Some(match entries.and_then(cut) {
    Some(entries) => (entries, whole),
    None => (vec![whole.clone()], whole),
  })
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
  let mut events = lexer.finish();
  // (an in-block contact cut into entries one of which no name's mark requests is the one entry its leading marks
  // label: the writer makes no entry of the block's own, and today's contact stays whole on the names its first marks
  // request — not on a nameless creator, review rev94b r2 B1: `\affil{$^1$Univ A\\ $^2$Univ B\\ $^3$Lab C}` under names
  // showing 1 and 2)
  let shown: Vec<String> = events
    .iter()
    .filter(|e| e.kind == EventKind::Mark)
    .filter_map(|e| e.label.as_deref())
    .flat_map(resolver::mark_labels)
    .collect();
  for (spans, whole) in walker.cuts {
    let at: Vec<usize> = events
      .iter()
      .enumerate()
      .filter(|(_, e)| e.kind == EventKind::Contact && e.span.is_some_and(|s| spans.contains(&s)))
      .map(|(at, _)| at)
      .collect();
    let requested = at.iter().all(|&i| {
      resolver::mark_labels(events[i].label.as_deref().unwrap_or(""))
        .iter()
        .all(|label| shown.contains(label))
    });
    if requested || at.len() != spans.len() {
      continue;
    }
    for &i in at.iter().skip(1).rev() {
      events.remove(i);
    }
    events[at[0]] = whole;
  }
  events
}
