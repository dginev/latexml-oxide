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
//!   fill glue `\hfil`/`\hfill`: its item count, its natural width in em of the box's font, and whether it fills); a
//!   control space or a tie is its font's word space at any width in em (`\tiny`, typewriter);
//! - **Break**: a row (`\\`, `\newline`, `\linebreak`, `\par`; the kernel's `\lx@newline` a `\centering` or a box
//!   rebinds `\\` to; an alignment's row end, role `row`; one inside a braced group or argument, role `group`), and what
//!   made it ([`BreakKind`]);
//! - **Contact**: `\email`, `\url`, `\href`, `\orcidlink`, an in-block `\affil`/`\address`, the frontmatter API;
//! - **Note**: `\thanks`/`\footnote`/`\footnotetext`, with its label (a `\label` inside it), its inner contacts and its
//!   Marks (the one it opens with `line_first`);
//! - **Layout**: a box or a styling container (its command), an alignment cell, a change of font between runs (a
//!   run's first font against the previous run's: a switch inside a run shows at the next run, not where it was);
//! - **Declaration**: a definition made in the block that prints nothing ([`declaration_events`], beside the block's
//!   other events: `reading.rs`).
//!
//! A Mark inside a formula or a group says so ([`MarkFormula`]: what encloses it, its base, the letter or digit after
//! it); a contact that prints nothing ([`Event::unprinted`]) and an event an error was held at ([`Event::held`]) too. So
//! do a Break or comma inside printed braces ([`Event::braces`]), an ordinal's suffix ([`Event::ordinal`]) and a Name
//! holding labels printed by hand ([`Event::printed_labels`]).
//!
//! Each event carries its span of the author field's positions (`fieldProv`/`fieldSerial` of the boxes it is made of,
//! core `gullet::field_cover_of`), its ink text and its layout nesting. ApiCall events (setters called outside the
//! field) are the ledger's, added to the block by the reading's JSON.

use latexml_core::{
  document::{FIELD_FORMULA_START_PROPERTY, FIELD_PROV_PROPERTY, FIELD_SERIAL_PROPERTY},
  token::{field_ledger_entry, field_prov_pos},
};

use super::{
  field_notes::{FieldBreak, FieldCreator, FieldDeclaration},
  marks::has_word,
  *,
};

mod fonts;
mod lexer;
pub(in crate::base_utilities) use fonts::{
  FontScope, InkFonts, Wrap, changes_font, entry_font, font_changes, font_declaration, switch_free,
  text_font_command,
};
use lexer::{Lexer, is_mark_glyph};
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
  Declaration,
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
      EventKind::Declaration => "Declaration",
    }
  }
}

/// Whose meaning an e-mail call has when its note is rehearsed (at the flush, `\maketitle`; `ledger::email_meaning`,
/// the rehearsal's binding of `\email` and `\emailAdd`): what a document note's e-mail call is read by
/// (`Resolver::api`, review rev92a_r3 B1; review rev92a_r4 B1 on the timing, OD #913), and an author's note's
/// (`Resolver::note`: a routed address is its owner's, LO4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmailMeaning {
  /// a binding's, whose code routes the address to the current author (amsart's, REVTeX's, jheppub's `\emailAdd`;
  /// OmniBus's, a guess its fallback class locks: 2511.14713)
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
  /// Its name in the reading's JSON (`"meaning"`; a binding's, the default, is not written).
  pub fn name(self) -> &'static str {
    match self {
      EmailMeaning::Binding => "binding",
      EmailMeaning::InPlace => "in-place",
      EmailMeaning::Own => "own",
    }
  }

  /// The meaning a dump's name gives (the deleted `ab_audit fmshadow`); a binding's for any other.
  pub fn from_name(name: &str) -> Self {
    match name {
      "in-place" => EmailMeaning::InPlace,
      "own" => EmailMeaning::Own,
      _ => EmailMeaning::Binding,
    }
  }
}

/// What a mark's script is set on in its formula, by what prints before it there, as `marks::read_formula` reads the
/// XMath (OXIDIZED_DESIGN_DIVERGENCES #911): spacing, a blank box, a font switch and an empty group are no base, and past
/// a formula's first script neither is punctuation or a symbol (`$^{1},^{2}$`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkBase {
  /// nothing printed before it (`$^{1}$`, `${}^{3}$`, `$\,^{a}$`, `$\rm ^{a}$`, `$\mbox{ }^{1}$`)
  Floating,
  /// on ink (`$x^{1}$`, `$\text{Ann}^{1}$`, `$^{1}x^{2}$`'s second)
  Posted,
}

impl MarkBase {
  /// Its name in the reading's JSON (`"base"`).
  pub fn name(self) -> &'static str {
    match self {
      MarkBase::Floating => "floating",
      MarkBase::Posted => "posted",
    }
  }

  /// The base a dump's name gives (the deleted `ab_audit fmshadow`).
  pub fn from_name(name: &str) -> Option<Self> {
    match name {
      "floating" => Some(MarkBase::Floating),
      "posted" => Some(MarkBase::Posted),
      _ => None,
    }
  }
}

/// What encloses a mark, from the digested boxes around it: the input of the fail-safe for a prescript and a mark, which
/// print the same (`${}^{3}$He`: floating, ending its formula, as a mark `${}^{3}$` is; the principle's point 2,
/// `FRONTMATTER_SINGLE_ENGINE_PLAN.md`), never a decision itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkFormula {
  /// the whole formula's span, or the group's or box's it is in, `None` where the content has no positions (an API
  /// call's). A brace closing a group is covered by no box, so a group's span stops at the last box in it; a formula's
  /// holds its closing shift
  pub span: Option<(u16, u16)>,
  /// in a formula, what it is set on ([`MarkBase`]); `None` in a group
  pub base: Option<MarkBase>,
  /// no letter or digit prints after it in the formula or group: spacing, punctuation, symbols, another script and a
  /// subscript do not count (`$^{a})$`, `$^{1,2}\dagger$`; `marks::read_formula`'s test of a base after a script)
  pub ends: bool,
}

/// What made a Break other than a line break (`\\`, `\newline`, `\linebreak`, an alignment's row end, none).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakKind {
  /// `\par`, a blank line: a line of its own in the class's own title page, nothing at all in a `c` column, whose
  /// restricted horizontal mode ignores it (tex.web §1094)
  Par,
  /// a section the binding declares (`Policy::block_breaks`: ijcai's `\affiliations`, `\emails`), which the title page
  /// starts on a line of its own (ijcai19.sty:124-130); beside the events
  Section,
  /// a column the binding declares (`Policy::block_breaks`: OmniBus's `\alignauthor`); beside the events
  Column,
}

impl BreakKind {
  /// Its name in the reading's JSON (`"break"`).
  pub fn name(self) -> &'static str {
    match self {
      BreakKind::Par => "par",
      BreakKind::Section => "section",
      BreakKind::Column => "column",
    }
  }

  /// The kind a dump's name gives (the deleted `ab_audit fmshadow`).
  pub fn from_name(name: &str) -> Option<Self> {
    match name {
      "par" => Some(BreakKind::Par),
      "section" => Some(BreakKind::Section),
      "column" => Some(BreakKind::Column),
      _ => None,
    }
  }
}

/// A label printed by hand in a Name's words: `a)`, `12)`, its own word with no bracket open before it (hep-ex0106079's
/// `a) also at UCEH b) on leave…`, the notes `$^{6,a}$` requests, which only `a)` leads a line of; a `(a)` or a word
/// of a bracket is none).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintedLabel {
  /// its label: `a` of `a)`
  pub label:      String,
  /// which word of the Name it is (from 0)
  pub word:       usize,
  /// it is its line's first ink
  pub line_first: bool,
  /// its positions
  pub span:       Option<(u16, u16)>,
}

/// The role of a picture's Note event (a pgf/TikZ picture in the block: `Lexer::walk_whatsit`), which the writer writes as
/// the picture's tokens whole (`emit::item_call`), never as an argument or a text.
pub const PICTURE_NOTE: &str = "picture";

/// Whether `event` is a picture's Note ([`PICTURE_NOTE`]).
pub(in crate::base_utilities) fn is_picture_note(event: &Event) -> bool {
  event.kind == EventKind::Note && event.role.as_deref() == Some(PICTURE_NOTE)
}

/// One event of an author block.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
  pub kind:           EventKind,
  /// the positions of the author field it covers, `start..end`
  pub span:           Option<(u16, u16)>,
  /// its ink: a name's text, a mark's printed text, a contact's or a note's shown text
  pub text:           String,
  /// a mark's label, a note's `\label`, a contact's optional label
  pub label:          Option<String>,
  /// a mark printed as a footnote symbol
  pub symbol:         bool,
  /// a mark that is its line's first ink
  pub line_first:     bool,
  /// a contact's role, a mark's or a layout's source (`sup`, `math`, `inst`, `text`; the container's command); a Name's,
  /// the role of the creator call that printed it (PoS's `\speaker`, `role=speaker`: `creator_roles`)
  pub role:           Option<String>,
  /// a link's target
  pub target:         Option<String>,
  /// a note's inner contacts and Marks (the one its text opens with `line_first`, the others not)
  pub inner:          Vec<Event>,
  /// how many layout containers enclose it
  pub nest:           u8,
  /// a mark's run of footnote symbols that a line of its block leads with and a name shows, written together (`§†`):
  /// one label, as `††` is (the resolver's `glue_shown_runs`; main-session decision 2 on FM S2a.2 r7, 0907.1243)
  pub glued:          bool,
  /// a mark set off by a printed space from the ink or separator before it, and from the name after it: which of the
  /// two TeX's output sets it next to (the resolver's `nearer_next`; `Han,${}^1$ Thibault` prints "Han,¹ Thibault")
  pub space_before:   bool,
  pub space_after:    bool,
  /// an e-mail contact's: whose meaning made the call ([`EmailMeaning`])
  pub meaning:        EmailMeaning,
  /// a note's: nothing in it prints ([`leaves_no_ink`] on its digested body, the test the XML's note is dropped by:
  /// `\thanks{}`, `\thanks{~}`, OXIDIZED_DESIGN_DIVERGENCES #916)
  pub inkless:        bool,
  /// an in-block annotation's (`\lx@annotate@frontmatter`) `annotate=` option: the authors today's API gives it (`run`,
  /// `new`, a number, `all`; Perl Base_Utility.pool.ltxml:520-536)
  pub annotate:       Option<String>,
  /// a cut in-block contact entry's: the font changes in effect where the writer's cut of it starts, against its call's
  /// (`fonts::entry_font`; `Font::default()` for none), which the writer digests it under — `None` when not read. A Name's: the
  /// font of its first character as digested, for the emitter
  pub font:           Option<Rc<Font>>,
  /// a cut in-block contact entry's: the containers holding it and another entry, which the writer wraps it in again
  /// (`fonts::Wrap`, `emitter::container_wrap`)
  pub wrap:           Option<Rc<Wrap>>,
  /// a mark's formula or group ([`MarkFormula`]), `None` for one at the block's top level
  pub formula:        Option<MarkFormula>,
  /// a contact's that prints nothing, what it records (`ltx:rdf`'s `content`: AASTeX 7's `\email` without `[show]`,
  /// 2506.18556, 2507.12222; OXIDIZED_DESIGN #479): its `text` holds no ink, as the XML holds none. Such a call stands
  /// outside the reading's events (`ledger::UnprintedCall`)
  pub unprinted:      Option<String>,
  /// an error of the block's rehearsal was held at a position it covers (`mark_held`): what it reads may be the
  /// error's recovery, not the ink it stands for
  pub held:           bool,
  /// a Name's: an undefined command its rehearsal held an Error at glues ink onto an address's local part (`mark_held`,
  /// `glues_an_address`): the same word has ink before it and ink after it, both before its `@` ("Bob|b@") — it prints
  /// nothing, so `Bob\email{b@x.org}` (`\email` undefined in article) reads "Bobb@x.org", an address no word of the source
  /// spells, which no reading of addresses may take: Perl's creator holds it in the personname,
  /// `Bob<ERROR>\email</ERROR>b@x.org`. A Name whose characters the ink table cannot align (a mark glued into it,
  /// "Bob Baker1b@x.org") fails closed on its span: an Error strictly inside it, an `@` token of it past the Error. One
  /// past the `@` (`ann@x.org\IEEEmembership{Member}`), before the word (`Bob \foo b@x.org`), in another word
  /// (`\texttt{ann@x.org}\foo{} Univ X`, `(ann@x.org\foo)`) or with the whole local part before it (`ann\foo@x.org`)
  /// glues no name into an address and is none (OXIDIZED_DESIGN_DIVERGENCES #983)
  pub glued_undef:    bool,
  /// a Break's: what made it, `None` for a line break ([`BreakKind`])
  pub break_kind:     Option<BreakKind>,
  /// a Break's or a comma's: the braces printed and not yet closed before it in its column (`\{aable, bbaker,\\ ccole\}@x.edu`:
  /// both inside one group of local parts; ijcai's `\emails` list), which a TeX group is none of
  pub braces:         u8,
  /// a Mark's: the suffix of the number the Name before it ends in, `5\textsuperscript{th}`, `$21^{st}$`: printed
  /// `st`, `nd`, `rd`, `th`, a space between them or not (2412.10792's `1\textsuperscript{st} Sertac`; iopart's
  /// `\address{$^1$Univ A, 5 \textsuperscript{th} floor}`; `10$^{a}$` is a mark)
  pub ordinal:        bool,
  /// a Name's: the labels its words print by hand ([`PrintedLabel`])
  pub printed_labels: Vec<PrintedLabel>,
  /// a `\textsuperscript` Mark's: the call's own position, its `span` being the wrapper's when it is the first box in
  /// one (`\textbf{\textsuperscript{2}Univ B}`, `emitter::Field::own_print`). Not in a dump: only the content reads it
  pub own_at:         Option<u16>,
  /// a cut in-block contact entry's: where its leading marks print, first to last, whole (`emitter::span_tokens`). A Name's: its
  /// boxes' own tokens' positions, first to last, where `span` is their covers': the first box in a wrapper takes the cover of the
  /// call around it, `\\textbf{Ann, Bob}`'s Ann the whole group (the emitter's). Not in a dump: only the content reads it
  pub marks:          Option<(u16, u16)>,
  /// a Note's: a `\footnote`, whose meaning in an author's content is no `\thanks`'s (`digest_frontmatter_item`). Not in
  /// a dump: only the emitter reads it
  pub footnote:       bool,
  /// a footnote Mark's: it is given its number (`\footnotemark[1]`, a document's `\samethanks[1]`, 1912.01737,
  /// 1811.08622), latex.ltx's `\@xfootnotemark`, which sets the counter in a group of its own and steps nothing — it
  /// re-references a note written already; a bare `\footnotemark` steps the counter (`\stepcounter\@mpfn`). Not in a
  /// dump: only the emitter's dropped-mark check reads it (`emitter::emit::mark_dropped`)
  pub given_number:   bool,
  /// a Name's: the block's creator call that printed it, its place among the rehearsal's (`creator_roles`;
  /// `ledger::Outcome::Events::creators`). Not in a dump: only the emitter reads it
  pub creator_call:   Option<u16>,
  /// a contact command's note that prints its text under a mark of its own (its call's `_mark` key, jheppub's
  /// `\lx@add@contact[role=note,_mark={#1}]`): that mark, empty for the footnote counter's next number
  /// (jheppub.sty:84-89 `\note[*]{…}`, `\note{…}`); `None` for any other. Not in a dump: only the reading's
  /// corresponding-author step reads it (`Resolver::corresponding`)
  pub note_mark:      Option<String>,
  /// a Note's lexed text, whole — its names, separators and marks, where `inner` keeps its contacts and marks alone: what
  /// a `\thanks` of several marked entries is cut by, before each top-level mark (`mark_led_pieces`). Not in a dump:
  /// only the reading's note step reads it (`Resolver::keep_note`)
  pub body:           Vec<Event>,
  /// an annotated contact's (`\lx@annotate@frontmatter`'s `name=` key, a nested `\lx@add@email[name=…]`): the label its
  /// class prints with it — authblk's `\affil{Univ A\thanks{Corresponding author e-mail: …}}`, the Kind step's
  /// `\lx@add@email[name={Corresponding author e-mail:~}]{…}` (cond-mat0202268) —, as a call's own label is its `name=`
  /// attribute (`resolver::placed::call_label`). Not in a dump: only the content check reads it
  pub name:           Option<String>,
}

/// Where one character of an ink prints ([`InkTable`]): its box's cover (`span`, a wrapper's call for the first box in
/// one) and the position of the token that printed it (`own`, `None` where the box has none of its own).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InkAt {
  pub cover: (u16, u16),
  pub own:   Option<u16>,
}

/// Where the inks of a lexed list printing an e-mail address's `@` print, beside its events (the block's ledger entry,
/// `ledger::Outcome::Events::inks`; a call's rehearsal): each such ink's characters (a Name on a line printing an `@`, a
/// Note's text), one [`InkAt`] per character of its text, `None` for a space the lexer wrote — keyed by the event's span
/// and text —, and the font each `@` prints in, at its token's position (its own, else its box's cover's start): where the
/// reading's address runs print and in what, which the emitter cuts the entry's tokens at and writes each address in
/// (`emitter::address_cut`, OXIDIZED_DESIGN_DIVERGENCES #980 P1b). Beside the events, not on each: an `Event` is moved
/// and cloned throughout the reading, its size paid at each (1901.01540's thousand-author line: +0.35 % instructions for
/// 24 bytes more). Not in a dump.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InkTable {
  pub at:    Vec<InkOf>,
  pub fonts: Vec<(u16, Rc<Font>)>,
}

/// One ink's characters in an [`InkTable`]: the event's span and text, where each character prints.
pub type InkOf = ((u16, u16), String, Vec<Option<InkAt>>);

impl InkTable {
  /// Where each character of `event`'s text prints, if recorded.
  pub fn of(&self, event: &Event) -> Option<&[Option<InkAt>]> {
    let span = event.span?;
    self
      .at
      .iter()
      .find(|(s, text, _)| *s == span && *text == event.text)
      .map(|(_, _, at)| at.as_slice())
  }

  /// The font the `@` whose token is at `position` prints in.
  pub fn font_at(&self, position: u16) -> Option<&Rc<Font>> {
    self
      .fonts
      .iter()
      .find(|(at, _)| *at == position)
      .map(|(_, font)| font)
  }
}

impl Event {
  /// The positions it prints over: its own span, and a mark's whole formula or group (`$^{1}$`, its shifts too).
  pub fn extent(&self) -> Option<(u16, u16)> {
    join_span(self.span, self.formula.and_then(|f| f.span))
  }

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
      formula: None,
      unprinted: None,
      held: false,
      glued_undef: false,
      break_kind: None,
      braces: 0,
      ordinal: false,
      printed_labels: Vec::new(),
      own_at: None,
      marks: None,
      footnote: false,
      given_number: false,
      creator_call: None,
      note_mark: None,
      body: Vec::new(),
      name: None,
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
  natural_width_em(d, width).map(|em| em.max(0.0))
}

/// [`width_property_em`] as it is, a negative width (a kern backing up) below zero.
fn natural_width_em(d: &Digested, width: &Stored) -> Option<f64> {
  let pt = match width {
    Stored::Dimension(d) => d.pt_value(None),
    Stored::Glue(g) => g.pt_value(None),
    _ => return None,
  };
  Some(pt / em_pt(d))
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

/// The whole of a formula: its opening shift (`document::FIELD_FORMULA_START_PROPERTY`), and past its closing shift, whose
/// box it is. Not [`deep_span`], which the first box's cover widens to the call around it (`\textbf{$^2$Univ B}`), and
/// which reads every box in it. `None` under no rehearsal's notes.
fn formula_extent(d: &Digested) -> Option<(u16, u16)> {
  let start = match d.get_property(FIELD_FORMULA_START_PROPERTY).as_deref() {
    Some(Stored::Int(start)) => u16::try_from(*start).ok()?,
    _ => return None,
  };
  let (_, end) = box_span(d)?;
  Some((start, end.max(start + 1)))
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
  // (a pgf/TikZ picture its text nodes, never its drawing: `ink_text_printed`, 2409.13798)
  ink_text_printed(d, &mut text);
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
    | "\\lx@fm@obs@fnnote"
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

/// The value of `key` in an API call's options, as written (`role=affiliation,annotate=new`).
fn option_value(options: &str, key: &str) -> Option<String> {
  options.split(',').find_map(|kv| {
    let (k, value) = kv.split_once('=')?;
    (k.trim() == key).then(|| value.trim().to_string())
  })
}

/// An e-mail call's argument as it prints (`raw_arg`), each line break it holds — the rehearsal's `\lx@fm@obs@break`,
/// `\\` inside the verbatim argument of llncs's `\email{a@x.org \\ b@y.org}`, `{\tt#1}` printing one address a line —
/// a space between its lines: their addresses are several, each written out (2608.29307's three addresses under one
/// `\email` were one address, `…@gmail.com\lx@fm@obs@breakn.van.stein@…`, which spelled its first author). A link in
/// it is what it prints: an `\href`'s text, never its target (`\email{\href{mailto:ann@a.org}{ann@a.org}}` printed one
/// address, read twice from the target and the text — two contacts written for one print: aa 2210.16066, 1907.11241,
/// 2007.06573, 2103.15844, llncs 2405.04152, 2603.13900; `\href{mailto:ann@a.org}{Ann}` prints no address), a `\url`'s
/// argument (review rev92c B1). A separator it holds parts its addresses as a break does (svjour3's `\email{yjansson@kth.se
/// \and tony@kth.se}`, 2106.06418: no `\andtony@kth.se`, review revN2).
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
          "\\lx@fm@obs@break" | "\\lx@fm@obs@par" | "\\lx@fm@obs@and" | "\\lx@fm@obs@And"
          | "\\lx@fm@obs@AND" => out.push(' '),
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
// (the lexer takes each at once, and a block's events are its cost: boxing one per name would add an allocation to save stack)
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum Atom {
  Ink {
    text:    String,
    span:    Option<(u16, u16)>,
    /// its box's own token's position
    own:     Option<u16>,
    font:    Option<FontKey>,
    nest:    u8,
    /// a formula's subscript, which a formula's reading sets aside as it does a superscript
    script:  bool,
    /// what encloses it when it holds a mark glyph ([`Walker::enclose`])
    formula: Option<MarkFormula>,
  },
  Space {
    width: f64,
    fill:  bool,
    span:  Option<(u16, u16)>,
    /// the one character it prints when it is a word space or a tie (` `, U+00A0), which the lexer reads together
    word:  Option<char>,
  },
  Event(Event),
}

/// The command a whatsit was made by.
fn whatsit_cs(w: &Whatsit) -> String { w.get_definition().get_cs_name().into_owned() }

/// The font of a text box, as a key telling fonts apart: two keys are equal when the fonts' family, series, shape and
/// size are (a size by its value: every NaN alike, `-0.0` not `0.0`), whatever else they carry. The box's own font,
/// shared (no copy, no text made per box).
#[derive(Debug, Clone)]
pub(super) struct FontKey(Rc<Font>);

impl PartialEq for FontKey {
  fn eq(&self, other: &Self) -> bool {
    let (a, b) = (&*self.0, &*other.0);
    Rc::ptr_eq(&self.0, &other.0)
      || (a.get_family() == b.get_family()
        && a.get_series() == b.get_series()
        && a.get_shape() == b.get_shape()
        && match (a.get_size(), b.get_size()) {
          (Some(x), Some(y)) => x.to_bits() == y.to_bits() || (x.is_nan() && y.is_nan()),
          (x, y) => x.is_none() && y.is_none(),
        })
  }
}

/// The font of a text box, as a key telling fonts apart ([`FontKey`]).
fn font_key(d: &Digested) -> Option<FontKey> { d.get_font().ok().flatten().map(FontKey) }

/// Walk `d` in print order, collecting atoms.
#[derive(Default)]
struct Walker {
  atoms:         Vec<Atom>,
  /// how many lists enclose the box being walked (the block's own is one; a braced group or argument inside it more)
  lists:         usize,
  /// each in-block contact cut into its marked entries (`labelled_entries`): the entries' spans, and the one entry its
  /// leading marks label, read when a cut entry's marks are no name's ([`lex_events`])
  cuts:          Vec<(Vec<(u16, u16)>, Event)>,
  /// the word space in em of the font last asked ([`Walker::is_word_space_of_its_font`]): sized once, for a block's thousand ties
  word_space:    Option<(Rc<Font>, f64)>,
  /// each formula, in the order they end ([`lex_with_formulas`]), a note's and an in-block contact's too
  formulas:      Vec<Formula>,
  /// the formulas walked here, in the order they end: the atoms each made, and the formula
  formula_atoms: Vec<(usize, usize, usize)>,
  /// where a note's text printing an `@` prints, and the fonts of its `@`s ([`lex_with_inks`])
  inks:          InkTable,
}

/// Whether `atoms` hold no word and no event: what may stand beside a mark's formula in its group.
fn no_word(atoms: &[Atom]) -> bool {
  atoms.iter().all(|atom| match atom {
    Atom::Space { .. } => true,
    Atom::Ink { text, script: false, .. } => !has_word(text),
    _ => false,
  })
}

/// A formula of a field's tokens, as the digested list says it, or the ink of a group printing symbols alone
/// ([`Walker::note_a_symbols_group`]), which a piece's run treats alike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::base_utilities) struct Formula {
  /// its opening shift, and past its closing shift ([`formula_extent`])
  pub start:         u16,
  pub end:           u16,
  /// how many of the groups around it, from the innermost, print nothing else ([`Walker::note_a_formulas_group`]): a mark's own
  /// group, whatever its tokens are
  pub alone_in:      u8,
  /// how many of the groups around it, from the innermost, it opens where a line starts ([`Walker::note_a_formulas_group`]): an entry's
  /// group, whatever follows the mark in it (`\\ \normalsize{$^{1}$ Univ X,} Moscow`, 2005.08290)
  pub opens_line_in: u8,
}

impl Walker {
  /// Whether a spacing box of one word-space character (a space, a tie) `width` em wide is its font's word space: what a
  /// control space prints, as the font's metrics size it (`Font::compute_string_size`). A skip of the same width is no
  /// different, and rare.
  fn is_word_space_of_its_font(&mut self, d: &Digested, text: &str, width: f64) -> bool {
    if !matches!(text, " " | "\n" | "\u{a0}") {
      return false;
    }
    let Some(font) = d.get_font().ok().flatten() else {
      return false;
    };
    let space = match &self.word_space {
      Some((last, space)) if Rc::ptr_eq(last, &font) => *space,
      _ => {
        let (space, ..) = font.compute_string_size(" ", SymHashMap::default());
        let space = space.pt_value(None) / em_pt(d);
        self.word_space = Some((font, space));
        space
      },
    };
    (space - width).abs() < 0.002
  }

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
          // (a word space is its font's, whatever that is in em: `M.\ de\ Naurois` at `\tiny` 0.347, aa's `~` the same,
          // 1702.08280; typewriter 0.5: a name cut at each)
          let width =
            if !fill && width > INTERWORD && self.is_word_space_of_its_font(d, &text, width) {
              space_width(' ')
            } else {
              width
            };
          let mut chars = text.chars();
          let word = match (chars.next(), chars.next()) {
            (Some(' ' | '\n'), None) if !fill && width <= INTERWORD => Some(' '),
            (Some('\u{a0}'), None) if !fill && width <= INTERWORD => Some('\u{a0}'),
            _ => None,
          };
          self.atoms.push(Atom::Space { width, fill, span, word });
        } else {
          self.atoms.push(Atom::Ink {
            text,
            span,
            own: match d.get_property(FIELD_PROV_PROPERTY).as_deref() {
              Some(Stored::Int(prov)) => u16::try_from(*prov).ok().map(field_prov_pos),
              _ => None,
            },
            font: font_key(d),
            nest,
            script: false,
            formula: None,
          });
        }
      },
      DigestedData::List(l) => {
        let at = self.atoms.len();
        self.walk_list(&l.borrow(), nest);
        // (a braced group; the block's own list is none)
        if self.lists > 0 {
          self.note_a_formulas_group(at);
          self.note_a_symbols_group(at);
          self.enclose(at, d, false, None);
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

  /// A list's boxes, walked in it.
  fn walk_list(&mut self, list: &List, nest: u8) {
    self.lists += 1;
    for child in list.boxes.iter() {
      self.walk(child, nest);
    }
    self.lists -= 1;
  }

  /// Whether the ink of the group of the atoms from `at` is its formulas' alone: spacing, a font or a color around them print nothing
  /// (`{$^1$\,}`, `{$^1$\bf}`, `{\color{red}$^1$}`, `{\fontsize{9}{9}\selectfont $^1$}`), a command printing before them does (`{\S $^1$}`,
  /// `{\LaTeX $^1$}`) and so does a word after them (`\textit{$^2$H}`); a separator or a symbol after them is no word (`{$^1$,}`, `{$^1$\dag}`).
  /// And whether its first formula opens it where a line starts, what digestion made: only spacing before the group since the field's start or a
  /// line break (`~\\ \normalsize{$^{1}$ Univ X,} Moscow`, 2005.08290), so that whatever follows the mark in it, it holds the entry's first words.
  /// After anything else it is no such fact: `Univ A, {$^{2}$H} Lab` may be an isotope (`perfect_kernel_batch107e::a_wrapped_isotope_is_entry_text`).
  fn note_a_formulas_group(&mut self, at: usize) {
    let first = self
      .formula_atoms
      .iter()
      .rposition(|run| run.0 < at)
      .map_or(0, |last_before| last_before + 1);
    let Some(&(lead_end, _, leader)) = self.formula_atoms.get(first) else {
      return;
    };
    let mut alone = self
      .atoms
      .get(at..lead_end)
      .is_some_and(|lead| lead.iter().all(|atom| matches!(atom, Atom::Space { .. })));
    if alone
      && self.atoms[..at]
        .iter()
        .rfind(|atom| !matches!(atom, Atom::Space { .. }))
        .is_none_or(|atom| matches!(atom, Atom::Event(e) if e.kind == EventKind::Break))
      && let Some(formula) = self.formulas.get_mut(leader)
    {
      formula.opens_line_in = formula.opens_line_in.saturating_add(1);
    }
    let mut from = lead_end;
    for &(start, end, _) in &self.formula_atoms[first..] {
      alone &= self.atoms.get(from.max(at)..start).is_none_or(no_word);
      from = from.max(end);
    }
    alone &= self.atoms.get(from..).is_some_and(no_word);
    if alone {
      for run in first..self.formula_atoms.len() {
        if let Some(formula) = self.formulas.get_mut(self.formula_atoms[run].2) {
          formula.alone_in = formula.alone_in.saturating_add(1);
        }
      }
    }
  }

  /// A group of no formula printing symbols alone is a mark's own group as a formula's is, when a mark starts there: `{*\,} Univ X, {**\,} Univ Y`
  /// (1605.02212). Its ink stands in the formulas, whole, so that a piece that starts at it starts at the brace (`emitter::Field::starts_at_its_brace`).
  /// A word or a digit after the symbol is no such group (`no_word`).
  fn note_a_symbols_group(&mut self, at: usize) {
    if self.formula_atoms.last().is_some_and(|run| run.0 >= at) {
      return;
    }
    let Some(atoms) = self.atoms.get(at..).filter(|atoms| no_word(atoms)) else {
      return;
    };
    let mut ink = None;
    for atom in atoms {
      if let Atom::Ink { span, .. } = atom {
        let Some((start, end)) = *span else {
          return;
        };
        ink = Some(ink.map_or((start, end), |(first, _)| (first, end)));
      }
    }
    let Some((start, end)) = ink else {
      return;
    };
    match self.formulas.last_mut() {
      Some(formula) if formula.start == start => {
        formula.alone_in = formula.alone_in.saturating_add(1)
      },
      _ => self.formulas.push(Formula {
        start,
        end,
        alone_in: 1,
        opens_line_in: 0,
      }),
    }
  }

  /// Say what encloses the marks among the atoms from `at` that nothing inside encloses: `d`, a formula (`extent`, or
  /// `formula` where it has none) or a group ([`MarkFormula`]). Only where it holds one: a formula of no mark costs nothing.
  fn enclose(&mut self, at: usize, d: &Digested, formula: bool, extent: Option<(u16, u16)>) {
    fn holds_glyph(text: &str) -> bool { text.chars().any(|c| is_fnsymbol(c) || is_mark_glyph(c)) }
    let open = |atom: &Atom| match atom {
      Atom::Event(e) => e.kind == EventKind::Mark && e.formula.is_none(),
      Atom::Ink { text, formula, .. } => formula.is_none() && holds_glyph(text),
      Atom::Space { .. } => false,
    };
    let atoms = &mut self.atoms[at..];
    if !atoms.iter().any(open) {
      return;
    }
    let span = extent.or_else(|| deep_span(d));
    // (print order: a first script's base is whatever prints before it, a later one's a letter or digit)
    let mut bases = Vec::with_capacity(atoms.len());
    let (mut has_base, mut scripted) = (false, false);
    for atom in atoms.iter() {
      bases.push(has_base);
      match atom {
        Atom::Ink { text, script: false, .. } if scripted => has_base |= has_word(text),
        Atom::Ink { text, script: false, .. } => {
          has_base |= text
            .chars()
            .any(|c| !c.is_whitespace() && !lexer::is_zero_width(c));
        },
        Atom::Ink { script: true, .. } => scripted = true,
        Atom::Event(e) if e.kind == EventKind::Mark => scripted = true,
        _ => {},
      }
    }
    let mut word_after = false;
    for (atom, has_base) in atoms.iter_mut().zip(bases).rev() {
      let enclosed = open(atom).then_some(MarkFormula {
        span,
        base: formula.then_some(if has_base {
          MarkBase::Posted
        } else {
          MarkBase::Floating
        }),
        ends: !word_after,
      });
      match atom {
        Atom::Event(event) => {
          if enclosed.is_some() {
            event.formula = enclosed;
          }
        },
        Atom::Ink { text, script, formula, .. } => {
          if enclosed.is_some() {
            *formula = enclosed;
          }
          word_after |= !*script && has_word(text);
        },
        Atom::Space { .. } => {},
      }
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
      // (a creator call an API call's content made: `rehearsal`'s `\lx@fm@obs@add`, 0807.4720)
      "\\lx@fm@obs@creatorcall" => {
        let mut event = Event::new(EventKind::ApiCall);
        event.role = Some("ltx:creator".to_string());
        self.event(event, d, nest);
      },
      "\\lx@fm@obs@and" => self.event(Event::new(EventKind::Sep(SepKind::And("\\and"))), d, nest),
      "\\lx@fm@obs@And" => self.event(Event::new(EventKind::Sep(SepKind::And("\\And"))), d, nest),
      "\\lx@fm@obs@AND" => self.event(Event::new(EventKind::Sep(SepKind::And("\\AND"))), d, nest),
      // (a break inside a braced group or argument — `\\ {\em College …,\\ City}`, `\textsl{Pure Mathematics\\University
      // of Waterloo\\…}` — is role `group`: the lines it parts are one print unit, as today's reads them, 1710.07902,
      // math0610612)
      "\\lx@fm@obs@break" | "\\lx@fm@obs@par" => {
        let mut row = Event::new(EventKind::Break);
        row.role = (self.lists > 1).then(|| "group".to_string());
        row.break_kind = (cs == "\\lx@fm@obs@par").then_some(BreakKind::Par);
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
      "\\lx@fm@obs@note" | "\\lx@fm@obs@fnnote" | "\\lx@fm@obs@notetext" => {
        let mut note = Event::new(EventKind::Note);
        note.footnote = cs.ends_with("fnnote");
        if let Some(body) = w.get_arg(2) {
          note.inkless = leaves_no_ink(body);
          let (events, formulas, inks) = lex_with_inks(body);
          self.formulas.extend(formulas);
          // (its text as printed, its contacts' too: the rehearsal's contact constructors print nothing, so the ink
          // of "available at \url{…}" lacks the address — 2105.10596)
          note.text = printed_text(&events);
          // (and where each of its characters prints, where it prints an address, and the fonts of its `@`s: `InkTable`)
          if note.text.contains('@') {
            let at;
            (note.text, at) = printed_text_at(&events, &inks);
            if let Some(span) = deep_span(d) {
              self.inks.at.push((span, note.text.clone(), at));
            }
            self.inks.fonts.extend(inks.fonts);
          }
          let mut inked = false;
          for inner in &events {
            let first = !inked;
            inked |= matches!(
              inner.kind,
              EventKind::Name | EventKind::Mark | EventKind::Contact | EventKind::Note
            );
            match inner.kind {
              EventKind::Contact => note.inner.push(inner.clone()),
              // (the Mark its text opens with, the legend's label: `\thanks{$^{1}$Unit…}`, S2a.2 r2; its later Marks
              // too, never leading — a note of several marked entries is no one label's legend, IEEEtran's
              // `\IEEEcompsocthanksitem \Mark{1}College … \IEEEcompsocthanksitem \Mark{2}College …`, 1802.03043)
              EventKind::Mark => {
                let mut mark = inner.clone();
                mark.line_first = first;
                note.inner.push(mark);
              },
              EventKind::Layout if inner.role.as_deref() == Some("label") => {
                note.label = inner.label.clone()
              },
              _ => {},
            }
          }
          note.body = events;
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
        if let Some(mut mark) = mark_event(printed, "footnotemark") {
          // (its optional argument, `\footnotemark[<n>]`: a mark that steps no counter, `Event::given_number`)
          mark.given_number = w.get_arg(1).is_some();
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
        // (each item as it prints, its control words and braces no part of it — the binding's own key,
        // `relocate::mark_key`: `\altaffilmark{1,\hbox{ }2}` requests `altaffil:2`, astro-ph0105133, review rev126a
        // B3 —; an item printing nothing else keeps its spelling, `\dag`, as the key does)
        let labels: Vec<String> = raw(2)
          .split(',')
          .map(|label| {
            let printed = mark_spelling_printed(label);
            match printed.trim() {
              "" => label.trim().to_string(),
              printed => printed.to_string(),
            }
          })
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
        if let Some(mut mark) = script_mark(w.get_arg(1), "sup", &notes) {
          mark.own_at = match d.get_property(FIELD_PROV_PROPERTY).as_deref() {
            Some(Stored::Int(prov)) => u16::try_from(*prov).ok().map(field_prov_pos),
            _ => None,
          };
          self.event(mark, d, nest);
        }
        self.walk_notes(&notes, nest);
      },
      "\\lx@fm@obs@email"
      | "\\lx@fm@obs@email@inplace"
      | "\\lx@fm@obs@email@own"
      | "\\lx@fm@obs@emailAdd@own"
      | "\\lx@fm@obs@orcid"
      | "\\lx@fm@obs@affil"
      | "\\lx@fm@obs@affil@run"
      | "\\lx@fm@obs@affil@marked" => {
        let mut contact = Event::new(EventKind::Contact);
        contact.role = Some(
          match cs.as_str() {
            "\\lx@fm@obs@orcid" => "orcid",
            _ if cs.starts_with("\\lx@fm@obs@affil") => "affiliation",
            _ => "email",
          }
          .to_string(),
        );
        // (a raw file's in-place setter the API took, `\lx@add@affiliation[annotate=run]` — the run of new authors'
        // entry, as today's attaches it: ci2018's `\affil` after `JORGE RAMIREZ and EVGENY KRIVOSHEEV`, 1805.12376)
        if cs == "\\lx@fm@obs@affil@run" {
          contact.annotate = Some("run".to_string());
        }
        // (a binding's marked list: its options the API's, no label)
        let marked = cs == "\\lx@fm@obs@affil@marked";
        if marked {
          contact.annotate = option_value(&raw(1), "annotate");
        }
        contact.meaning = match cs.as_str() {
          "\\lx@fm@obs@email@inplace" => EmailMeaning::InPlace,
          "\\lx@fm@obs@email@own" | "\\lx@fm@obs@emailAdd@own" => EmailMeaning::Own,
          _ => EmailMeaning::Binding,
        };
        // (the label a class's own command prints before the address, read as it prints (the rehearsal's
        // `own_email_properties`): jfm/jpp's "Email address for correspondence:", 2603.18265 — its `name`, as an annotated
        // contact's label is)
        if let Some(Stored::String(name)) = w.get_property("name").as_deref()
          && contact.meaning == EmailMeaning::Own
        {
          contact.name = Some(to_string(*name));
        }
        let label = if marked { String::new() } else { arg(1) };
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
          let (events, formulas) = lex_with_formulas(body);
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
          let cut = labelled_entries(&events, &formulas, scope.as_ref());
          self.formulas.extend(formulas);
          if let Some((entries, whole)) = cut {
            let entry_event = |e: LabelledEntry| {
              let LabelledEntry {
                label,
                text,
                span,
                marks,
                font,
                wrap,
              } = e;
              let mut entry = contact.clone();
              entry.label = Some(label);
              entry.text = text;
              entry.marks = marks;
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
        // (a link whose text is a binding's e-mail call is that call, the link printing what the call prints: `\thanks{…
        // E-mail: \href{mailto:samuel.mimram@cea.fr}{\email{samuel.mimram@cea.fr}}.}`, 1004.3134 — read as the link,
        // whose ink the call's rehearsal prints nothing of, the address was lost)
        let call = w.get_arg(2).filter(|text| {
          lex_with_inks(text).0.iter().any(|e| {
            e.kind == EventKind::Contact
              && e.role.as_deref() == Some("email")
              && e.meaning != EmailMeaning::Own
          })
        });
        if let Some(text) = call {
          self.walk(text, nest);
        } else {
          let mut contact = Event::new(EventKind::Contact);
          contact.role = Some("url".to_string());
          contact.target = Some(href_target(w));
          // (a note in the link is no text of it, an event of its own after it: `\href{https://orcid.org/…}{\usebox{\orcid}
          // Nils~Bochow\thanks{\texttt{contact@…}}}`, 2507.22485 — "Nils Bochowcontact@…" read as no name)
          let notes = w.get_arg(2).map(notes_in).unwrap_or_default();
          contact.text = if notes.is_empty() {
            arg(2)
          } else {
            w.get_arg(2).map(ink_of_without_notes).unwrap_or_default()
          };
          self.event(contact, d, nest);
          self.walk_notes(&notes, nest);
        }
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
        let role = option_value(&options, "role").unwrap_or_default();
        let mut event = Event::new(if tag.trim() == "ltx:note" {
          EventKind::Note
        } else {
          EventKind::Contact
        });
        if role == "note" {
          event.note_mark = option_value(&options, "_mark").map(|mark| {
            mark
              .trim_start_matches('{')
              .trim_end_matches('}')
              .trim()
              .to_string()
          });
        }
        event.role = Some(role);
        event.annotate = option_value(&options, "annotate");
        if event.kind == EventKind::Contact {
          event.name = option_value(&options, "name");
        }
        event.text = arg(4);
        // (a contact a link in it prints, no ink of the constructor's: `\IEEEauthorblockA{Dept\\ Univ\\ Email: \url{ann@x.no}}`,
        // its address read as `\url`'s argument, 2603.01171 — the e-mail was lost)
        if event.text.trim().is_empty() {
          event.text = raw_lines(w, 4).trim().to_string();
        }
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
            word: None,
          });
        }
        self.atoms.extend(inside.into_iter().filter(|atom| {
          matches!(atom, Atom::Event(e) if matches!(e.kind, EventKind::Note | EventKind::Contact))
        }));
      },
      // A pgf/TikZ picture (pgfsys-latexml.def's `\lxSVG@insertpicture`) is no ink of a name or an item: a Note of its own,
      // role `picture` ([`PICTURE_NOTE`]), its text its text nodes' ([`ink_of`], none for a drawing alone), kept whole as
      // markup by the writer — never its drawing's transforms and path data read as a name or written as text. Inside
      // `\author` it goes out of the name, a frontmatter note in source order (user ruling 2026-11-10, "a note, kept
      // whole", OXIDIZED_DESIGN_DIVERGENCES #989): 2404.13552's page-overlay copyright notice was a creator
      // "1.00.00.01.0", its 21 words lost. Inside a note it is that note's, printed in it (a `\thanks{… \tikz\draw …;}` had
      // its path data as the note's text, CR5).
      _ if picture_text(w).is_some() => {
        let mut picture = Event::new(EventKind::Note);
        picture.role = Some(PICTURE_NOTE.to_string());
        picture.text = ink_of(d);
        self.event(picture, d, nest);
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
    let at = self.atoms.len();
    self.walk_math_list(&body, nest);
    let extent = formula_extent(d);
    if let Some((start, end)) = extent {
      self
        .formula_atoms
        .push((at, self.atoms.len(), self.formulas.len()));
      self.formulas.push(Formula {
        start,
        end,
        alone_in: 0,
        opens_line_in: 0,
      });
    }
    self.enclose(at, d, true, extent);
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
            own: None,
            text: ink_of(d),
            span: deep_span(d),
            font: None,
            nest,
            script: true,
            formula: None,
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
      {
        if typesets_content(arg) {
          contents.push(arg.clone());
        } else {
          // (an argument printing nothing but an e-mail call, whose verbatim argument is no shown one: its contact. llncs
          // queues each `\email` of an `\author` line as a creator call of its own,
          // `\lx@personname{\lx@author@withinst{\email{ann@x.org}}}` (llncs.cls:897 `{\tt#1}`, printed under the name),
          // whose rehearsal read no event, and the content check no address: `perfect_kernel_batch77t::llncs_author_e_mail_line_ends_a_row`)
          let at = self.atoms.len();
          self.walk(arg, nest.saturating_add(1));
          let inside = self.atoms.split_off(at);
          self.atoms.extend(inside.into_iter().filter(|atom| {
            matches!(atom, Atom::Event(e) if e.kind == EventKind::Contact && e.role.as_deref() == Some("email"))
          }));
        }
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
          own: None,
          text,
          span: deep_span(d),
          font: None,
          nest,
          script: false,
          formula: None,
        });
      } else if let Some(width) = whatsit_width_em(d, w)
        && width >= 0.0
      {
        self.atoms.push(Atom::Space {
          width,
          fill: false,
          span: box_span(d),
          word: None,
        });
      }
      return;
    }
    let at = self.atoms.len();
    let mut layout = Event::new(EventKind::Layout);
    layout.role = Some(cs.to_string());
    self.event(layout, d, nest);
    for content in contents {
      // (the group of what it shows is the whatsit, its own command's span with it, not the argument's list)
      match content.data() {
        DigestedData::List(l) => self.walk_list(&l.borrow(), nest.saturating_add(1)),
        _ => self.walk(&content, nest.saturating_add(1)),
      }
    }
    self.enclose(at, d, false, None);
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
/// `ltx_affiliation_institution` text of the second (acmart_cls.rs; 1907.07713); `\scalebox`'s scales, `\resizebox`'s
/// height type and sizes; the frontmatter's position marker `\lx@frontmatter@mark{ltx:abstract}`
/// (frontmatter_fields.rs), which prints nothing — an abstract begun inside a revtex `\address` read "ltx:abstract" as
/// an affiliation, cond-mat9801109.
const KEYED_CONSTRUCTORS: &[(&str, &[bool])] = &[
  ("\\lx@acm@addresspart@", &[false, true]),
  ("\\lx@frontmatter@mark", &[false]),
  // (graphics.sty's `\scalebox{h-scale}[v-scale]{text}`, graphics_sty.rs: its scales are plain arguments)
  ("\\scalebox", &[false, false, true]),
  // (`\resizebox{w}{h}{text}`'s box, graphics_sty.rs `\Gscale@@box{\height}{w}{h}{text}`: its height type and sizes are
  // plain arguments — read as ink, a `\resizebox{0.5\linewidth}{!}` author block's "0pt" was a name, 2107.12423)
  ("\\Gscale@@box", &[false, false, false, true]),
];

/// The natural width in em (of its font) of a spacing whatsit (`\hskip`, `\hspace`, `\kern`, `\quad` made as a
/// skip), when it has one — below zero for one backing up, which prints no space between the letters beside it:
/// `B\kern-0.1em{\small A}` is one word "BA", as TeX sets it (a document's `\babar` logo of kerned letters, 0711.1544,
/// 0804.0020: read "B A B A R", its words missing from what the emitter wrote, review revNL1 B1).
fn whatsit_width_em(d: &Digested, w: &Whatsit) -> Option<f64> {
  let cs = whatsit_cs(w);
  if !(cs.contains("skip") || cs.contains("space") || cs.contains("kern") || cs.contains("quad")) {
    return None;
  }
  let pt = match w.get_property("width").as_deref() {
    Some(width @ (Stored::Dimension(_) | Stored::Glue(_))) => return natural_width_em(d, width),
    _ => match w.get_args().iter().flatten().next() {
      Some(arg) => arg.pt_value(None),
      None => return Some(1.0),
    },
  };
  Some(pt / em_pt(d))
}

/// The text events print, in order: names, contacts and inner notes as their text, a comma `,`, a separating word its
/// word, the rest a space, the marks aside (a note's text, `\lx@fm@obs@note`).
pub(in crate::base_utilities) fn printed_text(events: &[Event]) -> String {
  printed_text_into(events, None)
}

/// [`printed_text`], and where each of its characters prints ([`InkTable`]).
pub(in crate::base_utilities) fn printed_text_at(
  events: &[Event],
  inks: &InkTable,
) -> (String, Vec<Option<InkAt>>) {
  let mut at = Vec::new();
  let text = printed_text_into(events, Some((&mut at, inks)));
  (text, at)
}

/// Where each character of an event's `piece` prints: its ink's own record where it has one for the whole text, else the
/// event's span for each.
fn piece_at(event: &Event, piece: &str, (at, inks): (&mut Vec<Option<InkAt>>, &InkTable)) {
  if let Some(own) = inks
    .of(event)
    .filter(|own| piece == event.text && own.len() == piece.chars().count())
  {
    at.extend_from_slice(own);
  } else {
    let whole = event.span.map(|cover| InkAt { cover, own: None });
    at.extend(piece.chars().map(|_| whole));
  }
}

/// `text` with its whitespace runs one space, trimmed, and `at` (one per character, when given) with it.
fn spaced_at(text: &str, at: Option<&mut Vec<Option<InkAt>>>) -> String {
  let Some(at) = at else {
    return text.split_whitespace().collect::<Vec<_>>().join(" ");
  };
  let mut out = String::new();
  let mut kept = Vec::new();
  let mut space = false;
  for (c, ink) in text.chars().zip(at.iter()) {
    if c.is_whitespace() {
      space = !out.is_empty();
      continue;
    }
    if space {
      out.push(' ');
      kept.push(None);
      space = false;
    }
    out.push(c);
    kept.push(*ink);
  }
  *at = kept;
  out
}

fn printed_text_into(
  events: &[Event],
  mut at: Option<(&mut Vec<Option<InkAt>>, &InkTable)>,
) -> String {
  let mut out = String::new();
  for event in events {
    let (piece, glued) = match &event.kind {
      EventKind::Name | EventKind::Contact | EventKind::Note => (event.text.as_str(), false),
      // (a number's suffix is the note's text, `Event::ordinal`)
      EventKind::Mark if event.ordinal => (event.text.as_str(), false),
      // (a mark is no text of the note, as the XML's `ltx:sup` is none: CDF's `With visitors from $^a$Istituto …`,
      // 1203.3894, 1307.5202; its leading one the legend's label, `leading_mark`)
      EventKind::Mark => continue,
      EventKind::Sep(SepKind::Comma) => (",", true),
      EventKind::Sep(SepKind::Word(word)) => (word.as_str(), false),
      _ => {
        if !out.ends_with(' ') && !out.is_empty() {
          out.push(' ');
          if let Some((at, _)) = at.as_mut() {
            at.push(None);
          }
        }
        continue;
      },
    };
    if piece.is_empty() {
      continue;
    }
    if !glued && !out.is_empty() && !out.ends_with(' ') {
      out.push(' ');
      if let Some((at, _)) = at.as_mut() {
        at.push(None);
      }
    }
    out.push_str(piece);
    if let Some((at, inks)) = at.as_mut() {
      piece_at(event, piece, (at, inks));
    }
  }
  spaced_at(&out, at.map(|(at, _)| at))
}

/// The text of events, as printed: names and contacts, a comma's `,`, a word's word, a gap's space; marks aside but a
/// number's suffix (`Event::ordinal`).
pub(in crate::base_utilities) fn events_text(events: &[&Event]) -> String {
  events_text_into(events, None)
}

fn events_text_into(
  events: &[&Event],
  mut at: Option<(&mut Vec<Option<InkAt>>, &InkTable)>,
) -> String {
  let mut out = String::new();
  for event in events {
    let piece = match &event.kind {
      EventKind::Name | EventKind::Contact => event.text.as_str(),
      EventKind::Mark if event.ordinal => event.text.as_str(),
      EventKind::Sep(SepKind::Comma) => ",",
      EventKind::Sep(SepKind::Word(word)) => word.as_str(),
      _ => continue,
    };
    if piece == "," {
      out.push(',');
    } else {
      if !out.is_empty() {
        out.push(' ');
        if let Some((at, _)) = at.as_mut() {
          at.push(None);
        }
      }
      out.push_str(piece);
    }
    if let Some((at, inks)) = at.as_mut() {
      piece_at(event, piece, (at, inks));
    }
  }
  match at {
    // (trimmed, its characters with it)
    Some((at, _)) => {
      let lead = out.len() - out.trim_start().len();
      let lead_chars = out[..lead].chars().count();
      let kept = out.trim().chars().count();
      *at = at.iter().skip(lead_chars).take(kept).copied().collect();
      out.trim().to_string()
    },
    None => out.trim().to_string(),
  }
}

/// [`events_text`] with its lines kept: the text of each run of events between two `Break`s, a `\n` between them —
/// where an address printed in an affiliation stands at a line's end (`Resolver::take_out_entry_addresses`, brief 95c
/// r2 B2, N2: cond-mat0607219's revtex `\affiliation{… \\ Ph: …; FAX: …; email: rlake@… \\ Jan. 4, 2005 …}`).
pub(in crate::base_utilities) fn events_lined_text(events: &[&Event]) -> String {
  events
    .split(|e| e.kind == EventKind::Break)
    .map(events_text)
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>()
    .join("\n")
}

/// [`events_text`] and [`events_lined_text`] of `events`, each with where each of its characters prints
/// ([`InkTable`]; a space or a line's end the text writes `None`): what the emitter reads an entry's address runs at
/// (`emitter::address_cut`).
pub(in crate::base_utilities) type TextAt = (String, Vec<Option<InkAt>>);

pub(in crate::base_utilities) fn events_texts_at(
  events: &[&Event],
  inks: &InkTable,
) -> [TextAt; 2] {
  let mut at = Vec::new();
  let text = events_text_into(events, Some((&mut at, inks)));
  let (mut lined, mut lined_at) = (String::new(), Vec::new());
  for line in events.split(|e| e.kind == EventKind::Break) {
    let mut line_at = Vec::new();
    let line = events_text_into(line, Some((&mut line_at, inks)));
    if line.is_empty() {
      continue;
    }
    if !lined.is_empty() {
      lined.push('\n');
      lined_at.push(None);
    }
    lined.push_str(&line);
    lined_at.extend(line_at);
  }
  [(text, at), (lined, lined_at)]
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
    if matches!(e.kind, EventKind::Mark | EventKind::Layout) && !e.ordinal {
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

/// Whether a mark opens what `events` print (the first that prints: the Kind step's `LineText::opens_with_mark`, read on the
/// events — what parts a line's marks, a space, a break, a layout, prints nothing).
pub(in crate::base_utilities) fn opens_with_mark(events: &[&Event]) -> bool {
  events
    .iter()
    .find(|e| {
      !(matches!(
        e.kind,
        EventKind::Sep(SepKind::Gap { .. }) | EventKind::Break | EventKind::Layout
      ) || unprinted_piece(e))
    })
    .is_some_and(|e| e.kind == EventKind::Mark)
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
      // (a number's suffix is the entry's text, `Event::ordinal`)
      EventKind::Mark if event.ordinal => ink = true,
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
  /// where its leading marks print, first to last, whole
  marks: Option<(u16, u16)>,
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

/// Each Name a block's own creator call printed (`\lx@add@creator[keys]{name}`, PoS's and JHEP's `\speaker{…}`,
/// `[role=speaker]`; the rehearsal's notes, `field_notes::FieldCreator`) is that call's (`Event::creator_call`), its role
/// the call's, Perl's API argument (`Event::role`): its span inside the positions of the call's name.
pub(in crate::base_utilities) fn creator_roles(events: &mut [Event], calls: &[FieldCreator]) {
  for (call, FieldCreator { role, start, end, .. }) in calls.iter().enumerate() {
    let (start, end) = (*start, *end);
    // (a Name's span is the cover of the call around it, `\speaker{Ann Able}`'s whole: one meeting the name's positions)
    for event in events.iter_mut().filter(|e| e.kind == EventKind::Name) {
      if event.span.is_some_and(|(s, e)| s < end && start < e) {
        event.creator_call = u16::try_from(call).ok();
        event.role.clone_from(role);
      }
    }
  }
}

/// Run `f` (the rehearsal of an author block, `reading.rs`) with the block's `tokens`, its first at position `base`,
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

/// Whether an author block is being rehearsed ([`with_block_content`]), not an API call's content.
pub(in crate::base_utilities) fn in_block_content() -> bool {
  BLOCK_CONTENT.with(|b| b.borrow().is_some())
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
  // (each mark as it prints: its formula whole, whose script is its span)
  let print = |e: &Event| match e.formula {
    Some(MarkFormula {
      span: Some(formula),
      base: Some(_),
      ..
    }) => Some(formula),
    _ => e.span,
  };
  let printed = events[start..end]
    .iter()
    .filter(|e| e.kind == EventKind::Mark)
    .filter_map(print)
    .reduce(|(a, b), (s, e)| (a.min(s), b.max(e)));
  Some(LabelledEntry {
    label: labels.join(","),
    text: events_text(&rest),
    span,
    marks: printed,
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
  formulas: &[Formula],
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
      if piece[end..]
        .iter()
        .any(|e| e.kind == EventKind::Mark && !e.ordinal)
      {
        return None;
      }
      labelled_entry(piece).filter(|e| e.span.is_some() && !norm(&e.text).is_empty())
    })
    .collect();
  // (each entry with the font its cut starts in, `fonts::entry_font`, and the containers holding it and another entry
  // it is wrapped in again, `emitter::container_wrap` — review rev94b r3 B1: a font open from earlier in the block was
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
      let environments = field_notes::field_environments();
      let field = emitter::Field {
        tokens: content,
        base: *base,
        formulas,
        environments: &environments,
        held: &[],
        aligned: true,
        groups: Default::default(),
      };
      // (where each entry's cut starts: no such start is no cut the writer could make, `emitter::span_tokens`)
      let starts: Vec<(usize, (u16, u16))> = entries
        .iter()
        .map(|entry| {
          let span = entry.span?;
          Some((emitter::labelled_start(&field, span, entry.marks)?, span))
        })
        .collect::<Option<_>>()?;
      let forms = match scope {
        Some(scope) => emitter::entry_forms(content, *base, &starts, scope, events),
        None => vec![(None, None); entries.len()],
      };
      entries
        .into_iter()
        .zip(forms)
        .map(|(entry, (font, wrap))| {
          emitter::span_tokens(
            &field,
            entry.span?,
            Some(&norm(&entry.text)),
            None,
            Some(entry.marks),
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
  lex_with_formulas(digested).0
}

/// [`lex_events`], and each outermost formula's extent in order (`emitter::Field`): what a span's content is read by.
pub(in crate::base_utilities) fn lex_with_formulas(
  digested: &Digested,
) -> (Vec<Event>, Vec<Formula>) {
  let (events, formulas, _) = lex_with_inks(digested);
  (events, formulas)
}

/// [`lex_with_formulas`], and where its inks printing an `@` print ([`InkTable`]): what the emitter cuts an entry's addresses
/// at (`emitter::address_cut`).
pub(in crate::base_utilities) fn lex_with_inks(
  digested: &Digested,
) -> (Vec<Event>, Vec<Formula>, InkTable) {
  let mut walker = Walker::default();
  walker.walk(digested, 0);
  let mut lexer = Lexer::default();
  // (the lines printing an `@`, whose names' characters' positions the lexer records: `InkTable`)
  let mut line = 0;
  for atom in &walker.atoms {
    let (text, breaks) = match atom {
      Atom::Ink { text, .. } => (text.as_str(), false),
      Atom::Event(e) => (e.text.as_str(), e.kind == EventKind::Break),
      Atom::Space { .. } => ("", false),
    };
    if breaks {
      line += 1;
    } else if text.contains('@') {
      lexer
        .at_lines
        .resize(lexer.at_lines.len().max(line + 1), false);
      lexer.at_lines[line] = true;
    }
  }
  for atom in walker.atoms {
    match atom {
      Atom::Ink {
        text,
        span,
        own,
        font,
        nest,
        formula,
        ..
      } => lexer.ink(&text, span, font, nest, formula, own),
      Atom::Space { width, fill, span, word } => lexer.space(width, fill, span, word),
      Atom::Event(event) => lexer.push_event(event),
    }
  }
  let (mut events, mut inks) = lexer.finish();
  inks.at.append(&mut walker.inks.at);
  inks.fonts.append(&mut walker.inks.fonts);
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
  let mut formulas = walker.formulas;
  formulas.sort_unstable_by_key(|formula| (formula.start, std::cmp::Reverse(formula.end)));
  let mut outermost: Vec<Formula> = Vec::with_capacity(formulas.len());
  for formula in formulas {
    if outermost
      .last()
      .is_none_or(|last| last.end <= formula.start)
    {
      outermost.push(formula);
    }
  }
  (events, outermost, inks)
}

/// The Declaration events of a block's definitions (`field_notes::FieldNotes`): a definition, printing nothing, is in no box, so
/// it is read from the assignment and the position of the invocation that made it. Each is the call's span, the definees
/// in `label` (space-parted, in order), `global` its role when it defines past the block's groups, its group depth
/// `nest`. They stand beside the block's events, whose name runs a definition inside does not part (`Ann \def\x{y}Able`).
/// One inside a box the block prints (`$\def\x{1}\x$`, `\thanks{\def\x{y}\x}`, `\textbf{…}`, a `minipage`, a `center`) is
/// that box's and none of these (`field_notes::field_invocation`). A plain group's stays: a group is no box.
pub(in crate::base_utilities) fn declaration_events(
  declarations: &[FieldDeclaration],
) -> Vec<Event> {
  declarations
    .iter()
    .map(|declaration| {
      let mut event = Event::new(EventKind::Declaration);
      event.span = Some((declaration.start, declaration.end));
      event.label = Some(
        declaration
          .definees
          .iter()
          .map(|key| to_string(*key))
          .collect::<Vec<_>>()
          .join(" "),
      );
      event.role = declaration.global.then(|| "global".to_string());
      event.nest = u8::try_from(declaration.depth).unwrap_or(u8::MAX);
      event
    })
    .collect()
}

/// The Break events of the block breaks its binding declares (`field_notes::FieldBreak`), each its call's position, its
/// [`BreakKind`], `label` the command. They stand beside the block's events, which they do not part: what the command
/// prints stays whatever the class makes it, a no-op under OmniBus, a tabular's end under a raw class.
pub(in crate::base_utilities) fn break_events(breaks: &[FieldBreak]) -> Vec<Event> {
  breaks
    .iter()
    .map(|field_break| {
      let mut event = Event::new(EventKind::Break);
      event.span = Some((field_break.at, field_break.at + 1));
      event.label = Some(field_break.command.to_string());
      event.break_kind = Some(field_break.kind);
      event
    })
    .collect()
}

/// An error a rehearsal held: where (`None`, past the block's last token: its groups' end), and its log line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldError {
  pub span:    Option<(u16, u16)>,
  pub message: String,
}

/// Mark the events an error held covers ([`Event::held`]), a note's inner events too, and the Names an undefined
/// command glues an address in ([`Event::glued_undef`], told by where `inks` says their characters print). A rehearsal
/// goes on past a held error, so the events are all there, the block's whole: what the marker adds is which of them the
/// error touched.
pub(in crate::base_utilities) fn mark_held(
  events: &mut [Event],
  held: &[HeldError],
  inks: &InkTable,
) {
  for event in events {
    event.held = event.span.is_some_and(|(start, end)| {
      held
        .iter()
        .filter_map(|error| error.span)
        .any(|(from, to)| from < end && start < to)
    });
    event.glued_undef = event.kind == EventKind::Name && glues_an_address(event, held, inks);
    mark_held(&mut event.inner, held, inks);
  }
}

/// Whether an undefined command's held Error glues a word's ink onto an address's local part in `event`'s text
/// ([`Event::glued_undef`]): the same word has ink before the Error and ink after it, both before the word's first `@`
/// ("Bob|b@" — the local part is glued from ink on both sides of a command printing nothing). A local part wholly on one
/// side ("ann|@", `ann\foo@x.org`) is the address itself, read as without the Error. A Name's characters print where
/// `inks` records them, on a line printing an `@` — no line without one prints an address. Each character is before the
/// Error when its own token is (else its box's cover ends by it), after it when at or past its end. A Name whose
/// characters `inks` cannot align — no record for its span and text, or one of another length (a mark glued into it:
/// `Bob Baker\textsuperscript{1}\foo b@x.org` reads "Bob Baker1b@x.org") — fails closed on its span: an Error strictly
/// inside it, with an `@` token of it at or past the Error's end (its span end-exclusive, as the ink path's `after`:
/// `…b\foo@x.org` as `…b\foo{}@x.org`, review revERR2c finding 2), is a glue.
fn glues_an_address(event: &Event, held: &[HeldError], inks: &InkTable) -> bool {
  let errors: Vec<(u16, u16)> = held
    .iter()
    .filter(|error| error.message.starts_with("Error:undefined:"))
    .filter_map(|error| error.span)
    .collect();
  if errors.is_empty() || !event.text.contains('@') {
    return false;
  }
  let Some(at) = inks
    .of(event)
    .filter(|at| at.len() == event.text.chars().count())
  else {
    let Some((start, end)) = event.span else {
      return false;
    };
    return errors.iter().any(|&(from, to)| {
      start < from && inks.fonts.iter().any(|&(sign, _)| to <= sign && sign < end)
    });
  };
  let before = |ink: &InkAt, from: u16| ink.own.map_or(ink.cover.1 <= from, |own| own < from);
  let after = |ink: &InkAt, to: u16| ink.own.map_or(ink.cover.0 >= to, |own| own >= to);
  // (each word's characters up to its first `@`: its local part)
  let mut local: Vec<&InkAt> = Vec::new();
  let mut past_at = false;
  for (c, ink) in event.text.chars().zip(at.iter().map(Option::as_ref)) {
    if c.is_whitespace() {
      local.clear();
      past_at = false;
    } else if past_at {
      continue;
    } else if c == '@' {
      if let Some(sign) = ink
        && errors.iter().any(|&(from, to)| {
          after(sign, to)
            && local.iter().any(|ink| before(ink, from))
            && local.iter().any(|ink| after(ink, to))
        })
      {
        return true;
      }
      past_at = true;
    } else {
      local.extend(ink);
    }
  }
  false
}
