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

mod lexer;
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
      | "\\lx@fm@obs@affil" => {
        let mut contact = Event::new(EventKind::Contact);
        contact.role = Some(
          match cs.as_str() {
            "\\lx@fm@obs@orcid" => "orcid",
            "\\lx@fm@obs@affil" => "affiliation",
            _ => "email",
          }
          .to_string(),
        );
        contact.meaning = match cs.as_str() {
          "\\lx@fm@obs@email@inplace" => EmailMeaning::InPlace,
          "\\lx@fm@obs@email@own" => EmailMeaning::Own,
          _ => EmailMeaning::Binding,
        };
        let label = arg(1);
        contact.label = (!label.is_empty()).then_some(label);
        contact.text = if cs == "\\lx@fm@obs@affil" {
          arg(2)
        } else {
          raw_lines(w, 2).trim().to_string()
        };
        // (an in-block affiliation without a label of its own that opens with marks is the entry they label, as an API
        // entry is (`leading_marks`): ar-1col.cls's `\affil{$^1$Max Planck …}` inside `\author{}`, 1602.05576,
        // 2405.08158, the names showing `1` requesting it — its text the rest of its events, not the ink "1Max Planck"
        // given to the name beside it)
        let mut content_span: Option<(u16, u16)> = None;
        if cs == "\\lx@fm@obs@affil"
          && contact.label.is_none()
          && let Some(body) = w.get_arg(2)
        {
          let events = lex_events(body);
          let (start, end) = leading_marks(&events);
          let labels: Vec<&str> = events[start..end]
            .iter()
            .filter(|e| e.kind == EventKind::Mark)
            .filter_map(|e| e.label.as_deref().filter(|l| !l.is_empty()))
            .collect();
          if !labels.is_empty() {
            contact.label = Some(labels.join(","));
            let rest: Vec<&Event> = events
              .iter()
              .enumerate()
              .filter(|&(at, e)| !((start..end).contains(&at) && between_leading_marks(e)))
              .map(|(_, e)| e)
              .collect();
            contact.text = events_text(&rest);
            // (and its span its content's, from its marks on, which the writer makes the entry from, as a line of the
            // block from its mark (`plan::block_span_tokens`): not the call, whose own typesetting — a document's
            // `\newcommand{\affil}[1]{\\{\footnotesize #1}}` — is no part of it)
            content_span = events[start..]
              .iter()
              .filter_map(|e| e.span)
              .reduce(|(a, b), (s, e)| (a.min(s), b.max(e)));
          }
        }
        self.event(contact, d, nest);
        if let Some(span) = content_span
          && let Some(Atom::Event(event)) = self.atoms.last_mut()
        {
          event.span = Some(span);
        }
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
        event.text = arg(4);
        self.event(event, d, nest);
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
