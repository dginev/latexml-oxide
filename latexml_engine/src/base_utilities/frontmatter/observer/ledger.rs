//! The observer's ledger: one block per `\author`-family call, the S1 stamp of each `\title` call, and the
//! contact/affiliation API calls, in call order (OXIDIZED_DESIGN_DIVERGENCES #837). Never recorded while the frontmatter
//! is frozen (a replayed class setter, a speculative title-page run, the observer's own rehearsal: those calls are not
//! the document's).

use std::{cell::RefCell, collections::BTreeSet, rc::Rc};

use latexml_core::{
  definition::{Definition, origin::DefinitionOrigin},
  gullet::{InputMark, input_mark, input_unread_since},
  state::{fallback_meaning_origin, lookup_catcode},
  token::{AUTHOR_PROV_POS_MASK, field_prov},
};

use super::{
  emitter::Analysis,
  events::{EmailMeaning, Event, EventKind, HeldError, lex_body_with_inks},
  field_notes::{EnvironmentStep, FieldCreator},
  rehearsal::{past_line_break, past_tabular_call},
  resolver::printed_words,
  *,
};

/// How a block's tokens got their positions in the author field. Positions are per call: a block's start where the
/// field's own stamps end (`lx_field_base_2`, which the document's tokens carry: a block printing `\@author` reads them
/// apart from its own), never after the blocks before it, which no token of the document carries — so a document of
/// many calls never runs out, and one call has the author field's 15 bits (`token::AUTHOR_PROV_POS_MASK`; a block of
/// 559 names, 2204.09614, 2404.06718).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::base_utilities) enum Stamp {
  /// The kernel `\author`'s `\lx@field@mark` stamped the stored value these tokens are a copy of, at these positions
  /// (S1, OXIDIZED_DESIGN #715).
  Kernel,
  /// No mark stamps them (a binding's own `\lx@add@authors*` call, a kernel value past the stamps' positions): the
  /// positions an argument stamper (`\lx@field@stamp`) would give them, from where the field's stamps end.
  Projected,
  /// Out of positions ([`out_of_positions`]): the value went unstamped.
  Overflow,
}

impl Stamp {
  pub(in crate::base_utilities) fn name(self) -> &'static str {
    match self {
      Stamp::Kernel => "kernel",
      Stamp::Projected => "projected",
      Stamp::Overflow => "overflow",
    }
  }
}

/// What the flush made of a block.
#[derive(Debug, Clone, PartialEq)]
pub(in crate::base_utilities) enum Outcome {
  /// Not yet rehearsed (no flush since the call).
  Pending,
  /// Not yet rehearsed, waiting for a later flush: at the latest no rehearsal could run, and why
  /// ([`SkipReason::JournalUnavailable`]: that flush was itself inside a speculative run).
  Deferred(SkipReason),
  /// Rehearsed and lexed: its events, the errors and warnings the rehearsal held, and beside the events the block's
  /// definitions, the breaks its binding declares and the errors by position (`rehearsal::Rehearsal::Done`).
  Events {
    events:        Vec<Event>,
    /// the extents of its formulas, outermost, in order (`events::lex_with_formulas`): what a span's content reads
    /// (`emitter::Field`). Not in a dump: nothing offline reads the content
    formulas:      Vec<events::Formula>,
    /// where its inks printing an `@` print, and in what font (`events::InkTable`): what the emitter cuts an entry's
    /// addresses at (`emitter::address_cut`). Not in a dump
    inks:          Box<events::InkTable>,
    errors:        usize,
    warnings:      usize,
    declarations:  Vec<Event>,
    breaks:        Vec<Event>,
    held:          Vec<HeldError>,
    /// the environments its tokens open and close, what a span's content reads (`emitter::Field`). Not in a dump
    environments:  Vec<EnvironmentStep>,
    /// the creator calls its own expansion made (`field_notes::FieldCreator`), the keyvals the emitter writes on the
    /// creators their Names make (`Event::creator_call`), fixed once rehearsed (a slice: the variant stays under clippy's
    /// `large_enum_variant` bound). Not in a dump
    creators:      Box<[FieldCreator]>,
    /// where the rehearsal left a group open (`rehearsal::Rehearsal::Done::left_open`), what it read past it no event
    /// says: the block's content's text ([`content_text`]), whose words the events lack the content check counts
    /// (`resolver::placed::content_check`), as a call's no rehearsal read (`ApiCall::unread`). In a dump only when set
    unread:        Option<Box<ContentText>>,
    /// where its own digestion moved the footnote counter its marks print from, no event saying so
    /// (`field_notes::FieldNotes::counter_moves`): what the emitter carries or writes whole (`emitter::emit::counter_moved`).
    /// Not in a dump: only the emitter reads it
    counter_moves: Option<Box<CounterMoves>>,
  },
  /// Not rehearsed, and why.
  Skipped(SkipReason),
}

/// Where a block's own digestion moved the footnote counter, `start..end` each (`Outcome::Events::counter_moves`): boxed,
/// none for most blocks, so the variant stays under clippy's `large_enum_variant` bound.
#[derive(Debug, Clone, PartialEq)]
pub(in crate::base_utilities) struct CounterMoves(pub Vec<(u16, u16)>);

/// Why a block was not rehearsed (or its rehearsal not kept).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::base_utilities) enum SkipReason {
  /// its value went unstamped: out of positions ([`Stamp::Overflow`])
  Overflow,
  /// its argument is empty
  Empty,
  /// no flush came after the call
  NoFlush,
  /// no assignment journal could open (one is already open: the flush is itself inside a speculative run)
  JournalUnavailable,
  /// the rehearsal ended in a Fatal error of its own (discarded with it)
  Fatal,
  /// its assignments could not be undone
  NotUndone,
  /// it read all it may, or the conversion's rehearsals had ([`rehearsal::rehearsal_budget`])
  OverBudget,
}

impl SkipReason {
  pub(in crate::base_utilities) fn name(self) -> &'static str {
    match self {
      SkipReason::Overflow => "unstamped: out of positions",
      SkipReason::Empty => "empty",
      SkipReason::NoFlush => "no flush after the call",
      SkipReason::JournalUnavailable => "journal unavailable",
      SkipReason::Fatal => "rehearsal ended in a Fatal error",
      SkipReason::NotUndone => "rehearsal could not be undone",
      SkipReason::OverBudget => "rehearsal read all it may",
    }
  }
}

/// One `\author`-family call.
#[derive(Debug, Clone)]
pub(in crate::base_utilities) struct AuthorBlock {
  /// the call's place among every ledger entry (blocks, titles, API calls)
  pub index:          usize,
  /// the macro called (`\lx@add@authors@adaptive` for the kernel `\author`, `\lx@add@authors@append`, …)
  pub hook:           &'static str,
  /// the conventions the call runs under (`policy::policy_for` of `binding` and `hook`: what that binding declared,
  /// `kernel` for every family that declared none)
  pub policy:         String,
  /// the binding the policy is keyed on ([`policy_binding`]): the `\author` owner, or the document class's binding
  /// when that is the kernel's — with `hook`, the key of its declared conventions (`policy::policy_for`; S2a.2)
  pub binding:        String,
  /// the tags its rehearsal queued besides its creators (`rehearsal::Rehearsal::Done::queued`): a frontmatter setter run inside
  /// the block, a `\date` in `\author` (2503.02741), which the emitter counts (`emitter::emit::warn_queued_unwritten`). Not in
  /// a dump: only the emitter reads it
  pub queued:         Vec<RawFrontmatter>,
  /// the binding owning the `\author` meaning at the call ([`author_binding`]): a compiled binding's file name
  /// (`revtex4_support.sty`), else `kernel`, `document`, `raw` or `fallback` by its definition origin
  pub owner:          String,
  /// the value of the policy's state counter at the call (`policy::Policy::state_counter`), when it has one defined
  pub counter:        Option<i64>,
  /// whether the call replaces the authors before it (`\lx@clear@frontmatter{ltx:creator}`)
  pub replace:        bool,
  /// whether a later creator clear in its queue removed its authors ([`record_clear`]: llncs's `\author` given twice,
  /// its `\lx@clear@creators[role=author]` — a call of its own, not the block's `replace`)
  pub cleared:        bool,
  /// whether the IJCAI sectioned split applies: a sectioned caller's block (`\lx@add@authors`, `@adaptive`, `@list`)
  /// holding `\affiliations` or `\emails` (`author_macros::ijcai_block`), not every block such a caller records
  pub sectioned:      bool,
  /// made while a flush digests its queue (`lx_frontmatter_flushing`: an `\author` in the `\title`, in an amsart
  /// `\thanks`), after that flush's reading: the block's own flush reads it (`\lx@frontmatter@late`)
  pub late:           bool,
  /// a per-author binding's cautious name split
  pub cautious:       bool,
  /// the keyvals the call hands each author (revtex `annotations=`)
  pub keyvals:        Option<String>,
  /// those keyvals as the call's tokens: what a writer hands on is these, never [`keyvals`](Self::keyvals) retokenized, where
  /// a command glues to the letters after it (`\and Jonathan` read back `\andJonathan`, 2205.13776)
  pub keyval_tokens:  Option<Tokens>,
  /// the frontmatter flushes (`digest_front_matter`) before the call: the queue the position is in (each flush
  /// empties it)
  pub epoch:          usize,
  /// the frontmatter queue's length at the call
  pub queue_position: usize,
  /// how the tokens got their positions, and the positions `start..end`
  pub stamp:          Stamp,
  pub range:          (usize, usize),
  /// the argument, as the call received it, and as text
  pub tokens:         Tokens,
  pub source:         String,
  /// the input at the call: an API call made before any of it is read comes from the call's own expansion
  pub mark:           InputMark,
  /// still the latest block whose expansion has not ended
  pub window_open:    bool,
  pub outcome:        Outcome,
  /// a binding's list setter the argument calls ([`list_setter_called`]): sv_support's `\institute` inside an author's
  /// group, `\and{Cy Coe\inst{1}\institute{Univ X \and Lab Y}}` (cond-mat0605195) — whose entries the binding numbers
  /// for the marks to point to, which the rehearsal reads as one in-block affiliation (`rehearsal::install_event_bindings`);
  /// the emitter writes the call as it is, after the block's names (`emitter::emit::list_setter_runs`, 2502.07804)
  pub list_setter:    Option<&'static str>,
}

/// The S1 stamp of one `\title` call (`\lx@field@mark`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::base_utilities) struct TitleStamp {
  pub index:  usize,
  pub length: usize,
  /// the positions `start..end`, or `None` when the value went unstamped (out of positions)
  pub range:  Option<(usize, usize)>,
}

/// One queued creator, contact or note of the frontmatter API (`\lx@add@frontmatter`'s and `\lx@annotate@frontmatter`'s
/// entries): an author, an affiliation, an e-mail, a `\thanks`.
#[derive(Debug, Clone, PartialEq)]
pub(in crate::base_utilities) struct ApiCall {
  pub index:          usize,
  pub tag:            String,
  pub role:           String,
  /// the flushes before the call, and the queue's length at it (as [`AuthorBlock`]'s)
  pub epoch:          usize,
  pub queue_position: usize,
  /// the latest author block before the call
  pub after_block:    Option<usize>,
  /// made by that block's own expansion (its calls), not by the input after it
  pub inside:         bool,
  /// the entry's attributes as the call gave them (`role=affiliation,label=a,annotate=new`)
  pub attrs:          String,
  /// the element an annotation goes into (`\lx@annotate@frontmatter{ltx:creator}…`, `ltx:title` for a title's note),
  /// empty for an entry of its own (`\lx@add@frontmatter`)
  pub parent:         String,
  /// the entry's content (its command's last argument, inside the affiliation wrappers), and its events once the
  /// flush rehearsed it (`reading::read_at_flush`; empty before)
  pub content:        Tokens,
  pub events:         Vec<Event>,
  /// whether a flush rehearsed its content
  pub rehearsed:      bool,
  /// the entries its content's rehearsal queued besides creators, as an author block's ([`AuthorBlock::queued`]): a
  /// setter run inside the call — REVTeX 3's `\address{\begin{minipage}…\begin{abstract}…\end{abstract}\pacs{…}
  /// \end{minipage}}`, cond-mat9801109 —, which the emitter queues at the flush (`emitter::emit::recover_queued`)
  pub queued:         Vec<RawFrontmatter>,
  /// the rehearsal gave up on it (`SkipReason::OverBudget`, a Fatal, `NotUndone`): no events, what it printed its
  /// content's text alone ([`content_text`]), which the content check counts (`resolver::placed::content_check`)
  pub unread:         Option<Box<ContentText>>,
  /// a call its block's own expansion made (`inside`) that annotates a creator: its content's events, rehearsed for the
  /// content check alone (`resolver::placed::content_check`), which counts its words unless its block's events print
  /// them — no reader takes it apart from its block
  pub inside_events:  Box<[Event]>,
  /// whether a later clear removed it from the queue before any flush digested it ([`record_clear`])
  pub cleared:        bool,
  /// the errors its content's rehearsal held (no positions: an API call's content has none)
  pub errors:         usize,
  /// the queued call itself, where the emitter may reuse it (`emitter::emit`, under its switch)
  pub command:        Option<Tokens>,
}

/// A contact the document records and the PDF does not print (`ltx:rdf`, `\lx@add@creator@rdf`: AASTeX 7's `\email`
/// without `[show]`, OXIDIZED_DESIGN #479), which no reading holds: nothing else in the ledger reads it, and it takes no
/// place among the [`Ledger::entries`]. The call the real digestion makes, wherever it stands: a rehearsal records none,
/// and binds `\email` before the class decides, so a block's `\email` has this record beside its printing Contact event
/// (`\author{Jane Doe, \email{x}}`, none in 373 AASTeX 7 papers: OD #479's open case).
#[derive(Debug, Clone, PartialEq)]
pub(in crate::base_utilities) struct UnprintedCall {
  /// the ledger's length at the call: the entries before it
  pub at:          usize,
  /// the latest author block before the call
  pub after_block: Option<usize>,
  /// made by that block's own expansion
  pub inside:      bool,
  /// the entry's attributes as the call gave them
  pub attrs:       String,
  /// the Contact: its role the `property`, what it records its `unprinted`
  pub event:       Event,
}

/// One `\footnotetext` the document made (the kernel's `\lx@notetext`, a footnote's text): its printed mark, the footnote
/// counter value it stands for (`_fnvalue`), and its body's events, lexed from its own digestion — the text of a
/// `\footnotemark` an author block printed (`\author{Steven Delvaux\footnotemark[1], …}` and the body's
/// `\footnotetext[1]{Department of …}`, 1108.5155, 1404.2124), which today's `relocate_footnote` sets in that name by
/// the mark (FM S2a.2 r5, review revfms2a2_4 finding 1).
#[derive(Debug, Clone, PartialEq)]
pub(in crate::base_utilities) struct NoteText {
  pub index:  usize,
  /// the frontmatter flushes before it: made before the flush that digests a block, it is built before that block's
  /// creators (today's pairs it with the first mark only), after it, after them (with every mark)
  pub epoch:  usize,
  pub mark:   String,
  /// the footnote counter value it stands for — diagnostic only (review revfms2a2_5 finding 5): no resolver input
  /// (`resolver::Input::NoteText` has none, as the PDF's marks pair a text, not the counter), written to the dump so a
  /// census can count the texts today's `relocate_footnote` pairs by counter value alone (the r5 residual: 1404.2124,
  /// 0909.2782, hep-th0302032)
  pub value:  Option<String>,
  pub events: Vec<Event>,
}

/// An entry of the ledger.
#[derive(Debug, Clone)]
pub(in crate::base_utilities) enum Entry {
  Author(Box<AuthorBlock>),
  Title(TitleStamp),
  Api(ApiCall),
  NoteText(NoteText),
}

/// The ledger of one conversion.
#[derive(Debug, Default)]
pub(in crate::base_utilities) struct Ledger {
  /// its entries, in call order: read by [`Self::entries`], changed only by [`Self::entries_mut`], which counts the change
  entries:        Vec<Entry>,
  /// the changes to `entries` so far: what the emitter's reading of them is made at ([`Self::analysis`]), counted by
  /// [`Self::entries_mut`] alone, which a caller takes only to change them
  generation:     u64,
  /// the emitter's reading of `entries` and its checks, made at a generation (`emitter::emit::analysis`): kept while they
  /// read the same, dropped with the ledger (a conversion's start: no reading outlives its conversion, review revN1r2)
  pub analysis:   Option<Rc<Analysis>>,
  /// the frontmatter flushes so far (`reading::read_at_flush`)
  pub flushes:    usize,
  /// whether a flush's emitter wrote the paper's frontmatter from the reading (`emitter::emit`)
  pub emitted:    bool,
  /// the contacts recorded that print nothing, in call order ([`UnprintedCall`])
  pub unprinted:  Vec<UnprintedCall>,
  /// the contents' texts ([`content_text`]) of the queued creator calls the emitter took out of the queue that no
  /// recorded call is (a stand-in binding's copy's, [`records_now`]), in queue order: what the content check counts them
  /// by (`emitter::emit::emit_commands`, `resolver::placed::content_check`)
  pub unrecorded: Vec<ContentText>,
  /// what a list setter call the emitter wrote replaces as its clear (`\lx@clear@frontmatter{ltx:contact}[role=affiliation]`)
  /// would at the block, which pdflatex never prints: an affiliation call queued before the block (its ledger index, no
  /// span) and an earlier setter call of a block (the block's index and the call's span) — what the content check counts
  /// no word of (`emitter::emit::replaced_by_setter`, `resolver::placed::content_check`; review revDEL3 S1)
  pub replaced:   BTreeSet<(usize, Option<(u16, u16)>)>,
  /// the entries the last binding's `\author` body made, a later `\author` clears them ([`record_author_body_clear`])
  author_body:    (usize, usize),
}

impl Ledger {
  /// Its entries, in call order.
  pub(in crate::base_utilities) fn entries(&self) -> &[Entry] { &self.entries }

  /// Its entries, to change: a change of what the emitter reads, which a reading made before it no longer is
  /// ([`Self::generation`]) — the one write path of the count, so taken only where an entry changes (a lookup reads
  /// [`Self::entries`]: review revN1r3 4).
  pub(in crate::base_utilities) fn entries_mut(&mut self) -> &mut Vec<Entry> {
    self.generation += 1;
    &mut self.entries
  }

  /// Its entries in `range` that `picks` picks, each changed by `change`, a change counted ([`Self::entries_mut`]) only
  /// where one is picked, the emitter's reading being kept while the entries read the same ([`Self::analysis`]).
  pub(in crate::base_utilities) fn change_each(
    &mut self,
    range: std::ops::Range<usize>,
    picks: impl Fn(&Entry) -> bool,
    mut change: impl FnMut(&mut Entry),
  ) {
    if !self
      .entries
      .iter()
      .take(range.end)
      .skip(range.start)
      .any(&picks)
    {
      return;
    }
    for entry in self
      .entries_mut()
      .iter_mut()
      .take(range.end)
      .skip(range.start)
    {
      if picks(entry) {
        change(entry);
      }
    }
  }

  /// A new entry, last.
  pub(in crate::base_utilities) fn push(&mut self, entry: Entry) { self.entries_mut().push(entry); }

  /// The changes to its entries so far.
  pub(in crate::base_utilities) fn generation(&self) -> u64 { self.generation }
}

thread_local! {
  static LEDGER: RefCell<Ledger> = RefCell::new(Ledger::default());
}

/// Run `f` on the ledger.
pub(in crate::base_utilities) fn with_ledger<R>(f: impl FnOnce(&mut Ledger) -> R) -> R {
  LEDGER.with(|ledger| f(&mut ledger.borrow_mut()))
}

/// Clear the ledger (a conversion's start).
pub(in crate::base_utilities) fn reset_ledger() {
  with_ledger(|ledger| *ledger = Ledger::default());
}

/// The document class's bindings in load order, the binding loads' depth now, and whether the outermost is the class's.
type ClassBindings = (Vec<String>, usize, bool);

/// The commands whose owning binding the observer tracks ([`note_binding_load`]): `\author` (the policy key's binding
/// half, [`author_binding`]), the e-mail commands the rehearsal binds (`\email`, `\emailAdd`: whose a note's call
/// is, [`email_meaning`]).
const OWNED_COMMANDS: [&str; 3] = ["\\author", "\\email", "\\emailAdd"];
const AUTHOR: usize = 0;

/// The frontmatter setters a block may call a binding's list setter by ([`list_setter_called`]): the e-mail commands,
/// the brief's `\institute`, `\address` and `\affil` (95b r3), and the contact setters of a raw class's stores, which
/// the store reroute makes API calls of (`latexml_core::binding::store_setters::STORE_SETTERS`: `\affiliation`, `\ead`,
/// …) — not `\inst`, a binding's mark request (inst_support.sty's, which aa loads). The core's watch of their
/// expansions, which keyed an API call on the setter's binding (`ApiCall::binding`, never switched on since every family
/// was written), was deleted with that field (DEL5).
const FRONTMATTER_SETTERS: [&str; 11] = [
  "\\email",
  "\\emailAdd",
  "\\institute",
  "\\address",
  "\\affil",
  "\\affiliation",
  "\\ead",
  "\\emailaddr",
  "\\affaddr",
  "\\department",
  "\\institution",
];

/// A tracked command's owning binding and the meaning it made.
type Owner = Option<(String, Rc<dyn Definition>)>;
const OWNED: usize = OWNED_COMMANDS.len();

thread_local! {
  /// For each of [`OWNED_COMMANDS`], the compiled binding that defined its current meaning, with that meaning itself
  /// ([`note_binding_load`]; held, so its identity is `Rc::ptr_eq`'s and no later definition can take its address),.
  static OWNERS: RefCell<[Owner; OWNED]> = const { RefCell::new([const { None }; OWNED]) };
  /// The document class's compiled bindings: the first class binding loaded outside any other binding's load, and every
  /// binding its load loaded (`report.cls`; iopart.cls's `iopart_support.sty`), in load order; and how deep the loads
  /// nest now, with whether the outermost is that class's.
  static CLASS_BINDINGS: RefCell<ClassBindings> = const { RefCell::new((Vec::new(), 0, false)) };
}

/// The current meaning of the tracked command `n` ([`OWNED_COMMANDS`]), if any (what [`note_binding_load`] compares,
/// by `Rc::ptr_eq`).
fn tracked_meaning(n: usize) -> Option<Rc<dyn Definition>> {
  lookup_definition(&T_CS!(OWNED_COMMANDS[n])).ok().flatten()
}

/// The meaning the tracked command `n`'s noted owner made.
fn owned_meaning(n: usize) -> Option<Rc<dyn Definition>> {
  OWNERS.with(|o| o.borrow()[n].as_ref().map(|(_, defn)| Rc::clone(defn)))
}

/// Whether two meanings are the same definition (or both absent).
fn same_meaning(a: Option<&Rc<dyn Definition>>, b: Option<&Rc<dyn Definition>>) -> bool {
  match (a, b) {
    (Some(a), Some(b)) => Rc::ptr_eq(a, b),
    (None, None) => true,
    _ => false,
  }
}

/// Run a compiled binding's loader (`name`.`extension`, the binding's file: `revtex4_support.sty`), noting it as the
/// owner of each tracked command's meaning (`OWNED_COMMANDS`) when its load changed it and no binding it loaded did, or
/// its own code redefined it after one did (revtex4-2.cls's binding loads revtex4_support.sty, whose `\author` it is;
/// aa.cls's loads aa_support.sty, whose `\email` it is, and defines its own `\author` after inst_support.sty's): the binding half of the policy key (`author_binding`), and an e-mail call's binding
/// (`email_meaning`).
pub fn note_binding_load<R>(name: &str, extension: &str, load: impl FnOnce() -> R) -> R {
  // (a speculative run's load is undone with it: the document's own, if any, is the one noted)
  if in_speculative_digestion() {
    return load();
  }
  let before: [Option<Rc<dyn Definition>>; OWNED] = std::array::from_fn(tracked_meaning);
  let owners_before: [Option<Rc<dyn Definition>>; OWNED] = std::array::from_fn(owned_meaning);
  CLASS_BINDINGS.with(|c| {
    let (names, depth, in_class) = &mut *c.borrow_mut();
    if *depth == 0 && extension == "cls" && names.is_empty() {
      *in_class = true;
    }
    if *in_class && extension != "pool" {
      names.push(s!("{name}.{extension}"));
    }
    *depth += 1;
  });
  let value = load();
  CLASS_BINDINGS.with(|c| {
    let (_, depth, in_class) = &mut *c.borrow_mut();
    *depth = depth.saturating_sub(1);
    if *depth == 0 {
      *in_class = false;
    }
  });
  for n in 0..OWNED_COMMANDS.len() {
    // (a binding it loaded that defined the meaning first owns it unless this binding's own code redefined it after:
    // aa.cls's `\author`, defined after its aa_support.sty's inst_support.sty defined one, is aa.cls's, OD #922; a raw
    // file's redefinition stays no binding's)
    let nested = !same_meaning(owned_meaning(n).as_ref(), owners_before[n].as_ref());
    if let Some(after) = tracked_meaning(n)
      && !same_meaning(before[n].as_ref(), Some(&after))
      && (!nested
        || (!same_meaning(owned_meaning(n).as_ref(), Some(&after))
          && matches!(after.get_origin(), DefinitionOrigin::Binding)))
    {
      // (a pool is LaTeXML's own kernel: `kernel`, so a binding that loads it — latexml.sty — does not own its
      // `\author`)
      let owner = if extension == "pool" {
        "kernel".to_string()
      } else {
        s!("{name}.{extension}")
      };
      OWNERS.with(|o| o.borrow_mut()[n] = Some((owner, after)));
    }
  }
  value
}

/// Forget the tracked commands' owners and the class's bindings (a conversion's start).
pub(in crate::base_utilities) fn reset_author_owner() {
  OWNERS.with(|o| *o.borrow_mut() = [const { None }; OWNED]);
  CLASS_BINDINGS.with(|c| *c.borrow_mut() = (Vec::new(), 0, false));
}

/// The binding a block's policy is keyed on (S2a.2 r2, review revfms2a2's answer to question 3): the binding owning
/// the `\author` meaning that makes the `\lx@add@authors*` call ([`author_binding`]); when that is the kernel's, the
/// document class's binding that declared conventions (`jpconf.cls`, `report.cls`, `book.cls`; iopart_support owns its
/// `\author` itself)
/// — never over a package owning `\author` (authblk's stays authblk's).
pub(in crate::base_utilities) fn policy_binding(owner: &str) -> String {
  if owner != "kernel" {
    return owner.to_string();
  }
  CLASS_BINDINGS.with(|c| {
    c.borrow()
      .0
      .iter()
      .find(|name| policy::has_entry(name))
      .cloned()
      .unwrap_or_else(|| owner.to_string())
  })
}

/// The binding owning the `\author` meaning now: the compiled binding [`note_binding_load`] saw define it, while it is
/// still that meaning; else by the meaning's origin — `kernel` (LaTeXML's pools, the format), `document` (the
/// document's own `\renewcommand`), `raw` (a class or package read raw), `fallback` (OmniBus), `binding` (a binding no
/// load noted).
pub(in crate::base_utilities) fn author_binding() -> String { meaning_owner(AUTHOR) }

/// Whether the binding owning the `\author` meaning now (`author_binding`) declares that the class's title page prints
/// what a document appends to `\@author` (`policy::declares_author_tail_printed`: authblk.sty, llncs.cls).
pub fn author_tail_printed() -> bool { policy::declares_author_tail_printed(&author_binding()) }

/// Whether LaTeX's `\author` — the kernel's, a raw file's or the document's own (`author_binding`: `kernel`, `raw`,
/// `document`) — is in force, not a binding's (amsart's, elsarticle's, authblk's, OmniBus's `fallback`): then the
/// title page the title-page check's stand-in prints is taken as the one in force (LaTeX's, article.cls:236-251), and
/// what a document appends to `\@author` as printed. A binding class that keeps the kernel's `\author` answers `kernel`
/// too: its stand-in print stays counted, the safe side (FX27, OXIDIZED_DESIGN_DIVERGENCES #953).
pub fn author_is_latex_own() -> bool {
  matches!(author_binding().as_str(), "kernel" | "raw" | "document")
}

/// The binding owning the tracked command `n`'s meaning now ([`author_binding`]'s reading, for any of
/// [`OWNED_COMMANDS`]); `kernel` when it has none.
fn meaning_owner(n: usize) -> String {
  let Some(meaning) = tracked_meaning(n) else {
    return "kernel".to_string();
  };
  if let Some(name) = OWNERS.with(|o| {
    o.borrow()[n]
      .as_ref()
      .filter(|(_, owned)| Rc::ptr_eq(owned, &meaning))
      .map(|(name, _)| name.clone())
  }) {
    return name;
  }
  match meaning.get_origin() {
    DefinitionOrigin::Document => "document",
    DefinitionOrigin::File => "raw",
    DefinitionOrigin::Fallback | DefinitionOrigin::Stub => "fallback",
    DefinitionOrigin::Binding | DefinitionOrigin::Borrowed => "binding",
    _ => "kernel",
  }
  .to_string()
}

/// Whose the meaning of the e-mail command `cs` (`\email`, `\emailAdd`) is now: the rehearsal's reading of a call to
/// it, the meaning when the note is rehearsed (at the flush, `\maketitle`; `rehearsal::install_event_bindings`) —
/// amsart's, which holds `\thanks` until `\maketitle` (amsart.cls:516), as TeX reads it; aa's `\offprints`
/// (aa.cls:910-911 `\protected@edef`) and llncs's kernel `\thanks` outside `\author` and `\title` expand `\email`
/// at the call, so a paper's redefinition between the call and `\maketitle` reads as no binding's, the note's text,
/// failing safe (review rev92a_r4 B1, OD #913). The class's own, printing the address in place, when the
/// binding owning it ([`meaning_owner`]) declares it ([`policy::declares_email_in_place`]: llncs.cls, also under a
/// raw class that loads it; sv_support.sty, under svjour3, svjour and svmult), or is one of the document class's
/// bindings and one of those declares it (aa.cls, whose aa_support.sty defines it — not kluwer's, which loads
/// aa_support too); another binding's (amsart's ams_support.sty, REVTeX's, jheppub's `\emailAdd`, a package's), whose
/// code routes the address to the current author; else no binding's — the document's own
/// (`\renewcommand{\email}[1]{\texttt{#1}}`), a file read raw, the fallback class's guess for a class with no binding
/// (OmniBus: the real class's print is unknown) —, which prints wherever its definition prints (review rev92a_r3 B1:
/// 1412.0748, 1809.01965). A guess the fallback class locks is its binding's (OmniBus's `\email`, OD #408: 2511.14713).
pub(in crate::base_utilities) fn email_meaning(cs: &str) -> EmailMeaning {
  let Some(n) = OWNED_COMMANDS.iter().position(|c| *c == cs) else {
    return EmailMeaning::Own;
  };
  // (the guess a fallback class makes is marked in the state's own record, whatever the definition carries:
  // `fallback_meaning_origin`)
  let guessed = matches!(
    fallback_meaning_origin(&T_CS!(cs)),
    Some(DefinitionOrigin::Fallback | DefinitionOrigin::Stub)
  );
  // (save the guess the fallback class locks against every later definition, which no document's print replaces: OmniBus's
  // `\email` (`\email:locked`, OD #408), Perl's OmniBus.cls.ltxml:78 `\lx@add@email{#1}`, the binding's call routing the
  // address to the current author — 2511.14713's siamart250211 `\thanks{…, \email{wning@vt.edu}, \email{amiedlar@vt.edu}.}`,
  // read as the document's own, kept in the note's text, which the written note prints without them; main-session ruling
  // 2026-11-08)
  let by_binding = if guessed {
    lookup_bool(&s!("{cs}:locked"))
  } else {
    tracked_meaning(n).is_some_and(|meaning| {
      matches!(
        meaning.get_origin(),
        DefinitionOrigin::Binding | DefinitionOrigin::Borrowed
      )
    })
  };
  if !by_binding {
    return EmailMeaning::Own;
  }
  let owner = meaning_owner(n);
  let in_place = policy::declares_email_in_place(&owner)
    || CLASS_BINDINGS.with(|c| {
      let names = &c.borrow().0;
      names.contains(&owner)
        && names
          .iter()
          .any(|name| policy::declares_email_in_place(name))
    });
  if in_place {
    EmailMeaning::InPlace
  } else {
    EmailMeaning::Binding
  }
}

/// Whether a call now is the document's own: not one the frontmatter is frozen for (a replayed class setter, a
/// speculative run, a rehearsal), nor one a stand-in binding's copy makes (its defaults queue).
fn records_now() -> bool {
  !lookup_bool("lx_replaying_class_setter")
    && !rehearsal::rehearsing()
    && definition::origin::copy_provides() == 0
}

/// The length of the frontmatter queue.
fn queue_length() -> usize {
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue.len(),
    _ => 0,
  })
}

/// The binding's list setter `stuff` calls, if any: a [`FRONTMATTER_SETTERS`] command written in it whose meaning is a
/// binding's that replaces the whole list ([`LIST_CLEAR`]) and numbers its entries by place for the marks to point to —
/// sv_support.sty's, inst_support.sty's, aa.cls's and llncs.cls's `\institute`
/// (`\lx@clear@frontmatter{ltx:contact}[role=affiliation]`, then `labelseq=affiliation` entries), decided from the
/// binding's definition, not a name list. In an author block the rehearsal reads such a call as one affiliation of the
/// block, where the real call queues the numbered entries (cond-mat0605195): the emitter writes the call as it is, its
/// entries the call's, none of the reading's items (`emitter::emit::list_setter_runs`).
fn list_setter_called(stuff: &Tokens) -> Option<&'static str> {
  let toks = stuff.unlist_ref();
  FRONTMATTER_SETTERS.iter().copied().find(|name| {
    let cs = T_CS!(*name);
    toks.contains(&cs)
      && lookup_definition(&cs).ok().flatten().is_some_and(|defn| {
        matches!(
          defn.get_origin(),
          DefinitionOrigin::Binding | DefinitionOrigin::Borrowed
        ) && matches!(defn.get_expansion(), Some(ExpansionBody::Tokens(body))
          if body.unlist_ref().contains(&T_CS!(LIST_CLEAR)))
      })
  })
}

/// The frontmatter API's clear, with which a binding's list setter replaces the list it numbers ([`list_setter_called`]).
const LIST_CLEAR: &str = "\\lx@clear@frontmatter";

/// Whether a block of `len` tokens whose positions start where the field's stamps end (`real_base`) is out of positions
/// ([`Stamp::Overflow`]): the call alone past the author field's 15 bits (`token::AUTHOR_PROV_POS_MASK`).
fn out_of_positions(real_base: usize, len: usize) -> bool {
  real_base + len > usize::from(AUTHOR_PROV_POS_MASK)
}

/// Record an `\author`-family call (`hook`, with `stuff` its argument) — whether it was
/// recorded late ([`AuthorBlock::late`]).
pub(in crate::base_utilities) fn record_author_call(
  hook: &'static str,
  stuff: &Tokens,
  replace: bool,
  sectioned: bool,
  keyvals: Option<&Tokens>,
  cautious: bool,
) -> bool {
  if !records_now() {
    return false;
  }
  let late = lookup_bool("lx_frontmatter_flushing");
  let len = stuff.unlist_ref().len();
  let real_base = usize::try_from(lookup_int("lx_field_base_2")).unwrap_or(0);
  let owner = author_binding();
  let binding = policy_binding(&owner);
  let policy = policy::policy_for(&binding, hook).name;
  // (a policy's own state, read at the call: authblk's `\c@Maxaffil`, which decides whether its notes show)
  let counter = policy::policy_for(&binding, hook)
    .state_counter
    .and_then(|name| lookup_register_quiet(&s!("\\c@{name}")).map(|value| value.value_of()));
  let block = AuthorBlock {
    index: 0,
    hook,
    policy: policy.to_string(),
    binding,
    owner,
    counter,
    replace,
    cleared: false,
    sectioned,
    late,
    cautious,
    keyvals: keyvals.map(|kv| kv.to_string()),
    keyval_tokens: keyvals.cloned(),
    epoch: 0,
    queue_position: queue_length(),
    stamp: Stamp::Projected,
    range: (real_base, real_base + len),
    tokens: stuff.clone(),
    source: stuff.to_string(),
    mark: input_mark(),
    window_open: true,
    outcome: Outcome::Pending,
    list_setter: list_setter_called(stuff),
    queued: Vec::new(),
  };
  with_ledger(|ledger| {
    let mut block = block;
    block.index = ledger.entries().len();
    block.epoch = ledger.flushes;
    if out_of_positions(real_base, len) {
      block.stamp = Stamp::Overflow;
      block.range = (0, 0);
    }
    let entries = ledger.entries_mut();
    for entry in entries.iter_mut() {
      if let Entry::Author(earlier) = entry {
        earlier.window_open = false;
      }
    }
    entries.push(Entry::Author(Box::new(block)));
  });
  late
}

/// `\lx@field@mark` stamped field `field_id`'s value (`len` tokens) from `base` — `None`: out of positions, unstamped.
/// A title records its stamp; the author field's confirms the kernel block it is the stored copy of (the kernel
/// `\author` hands `\@author` to `\lx@add@authors@adaptive` just before it marks the store), whose positions are then
/// the mark's own.
pub(in crate::base_utilities) fn note_field_stamp(field_id: u16, base: Option<usize>, len: usize) {
  if !records_now() {
    return;
  }
  with_ledger(|ledger| {
    if field_id == 1 {
      let index = ledger.entries().len();
      ledger.push(Entry::Title(TitleStamp {
        index,
        length: len,
        range: base.map(|base| (base, base + len)),
      }));
      return;
    }
    if field_id != 2 {
      return;
    }
    let Some(at) = ledger
      .entries()
      .iter()
      .rposition(|entry| matches!(entry, Entry::Author(_)))
    else {
      return;
    };
    // (its entries and its positions change at once: the entries' change counted, `entries_mut`)
    let Some(Entry::Author(block)) = ledger.entries().get(at) else {
      return;
    };
    // (S2b's check, inactive in S2a: a binding call stamped for real takes the positions it was projected at)
    if block.stamp == Stamp::Projected
      && block.tokens.unlist_ref().len() == len
      && let Some(base) = base
    {
      debug_assert!(projected_is_stamped(block, (base, base + len)));
    }
    if block.hook != "\\lx@add@authors@adaptive"
      || block.stamp == Stamp::Kernel
      || block.tokens.unlist_ref().len() != len
    {
      return;
    }
    let Some(Entry::Author(block)) = ledger.entries_mut().get_mut(at) else {
      return;
    };
    // (past the stamps' positions, none: the call's own, projected, as a binding's)
    if let Some(base) = base {
      block.stamp = Stamp::Kernel;
      block.range = (base, base + len);
    }
    // (the kernel block's own calls are all digested once the mark is reached)
    block.window_open = false;
  });
}

/// S2b's assertion, inactive in S2a (notes decision 4): once the bindings' calls are stamped for real
/// (`\lx@field@stamp`), a block's projected positions must be the stamped ones — projected spans are unique only
/// within a block, so an event is keyed by (block index, span) until then.
pub(in crate::base_utilities) const ASSERT_PROJECTED_IS_STAMPED: bool = false;

/// Whether `block`'s positions agree with the positions `stamped` a real stamp gave its tokens (always, while
/// [`ASSERT_PROJECTED_IS_STAMPED`] is off).
pub(in crate::base_utilities) fn projected_is_stamped(
  block: &AuthorBlock,
  stamped: (usize, usize),
) -> bool {
  !ASSERT_PROJECTED_IS_STAMPED || block.stamp != Stamp::Projected || block.range == stamped
}

/// Record a queued creator, contact or note of the frontmatter API (`queue_front_matter`: its tag, its attributes and
/// its command): a binding's single-author call (`\lx@add@creator`, `\lx@add@author`:
/// authblk's unlabelled `\author{Bob}`) is an API call too, unless a block's own expansion made it. Its content (the
/// command's last argument, inside the affiliation wrappers) is kept for the flush to rehearse.
pub(in crate::base_utilities) fn record_api_call(
  tag: &str,
  attr: Option<&KeyVals>,
  command: &Tokens,
) {
  if tag == "ltx:rdf" {
    if records_now() {
      record_unprinted_call(attr, command);
    }
    return;
  }
  // (and the title, whose marks request labelled entries too — aastex's `\title{…\altaffilmark{1}}` and
  // `\altaffiltext{1}{Based on observations …}`, the title's note: 0801.1670; and a publication note, whose e-mails are
  // the authors' — A&A's `\offprints{B. Baker, \email{…}}`, 0711.1152, 0802.0707)
  if !matches!(
    tag,
    "ltx:creator" | "ltx:contact" | "ltx:note" | "ltx:title" | "ltx:pubnote"
  ) || !records_now()
  {
    return;
  }
  let role = attr
    .and_then(|kv| kv.get_value("role"))
    .map(|role| role.to_string())
    .unwrap_or_default();
  // (in source order, each pair as given: a repeated key keeps both)
  let attrs = attr
    .map(|kv| {
      kv.get_pairs()
        .map(|(key, value)| match value {
          // (braced when it holds a comma, which would part it: `\correspondingauthor{Weipeng Lin, Lin Tang}`'s
          // `label={fuzzy:…}`, 2408.02230)
          ArgWrap::Tokens(tokens) if key == "label" => {
            let label = label_text(tokens);
            if label.contains(',') {
              s!("{key}={{{label}}}")
            } else {
              s!("{key}={label}")
            }
          },
          value => s!("{key}={value}"),
        })
        .collect::<Vec<_>>()
        .join(",")
    })
    .unwrap_or_default();
  let content = last_group(command);
  // (content `\lx@affiliation@withsup` wraps — a marked line of `affiliation_calls`, revtex's `\affiliation{$^1$…}` —
  // takes its first superscript as its label, `\lx@sup@setlabel@affiliation`, wherever in the line: recorded for the
  // resolver, `_withsup=true`)
  let attrs = if wraps_withsup(&content) {
    if attrs.is_empty() {
      "_withsup=true".to_string()
    } else {
      s!("{attrs},_withsup=true")
    }
  } else {
    attrs
  };
  let content = queued_affiliation_text(&content);
  let parent = annotated_parent(command);
  let queue_position = queue_length();
  with_ledger(|ledger| {
    let index = ledger.entries().len();
    let mut after_block = None;
    let mut inside = false;
    let last_block = ledger
      .entries()
      .iter()
      .rposition(|entry| matches!(entry, Entry::Author(_)));
    if let Some(at) = last_block
      && let Some(Entry::Author(block)) = ledger.entries().get(at)
    {
      after_block = Some(block.index);
      if block.window_open {
        if input_unread_since(&block.mark) {
          inside = true;
        } else if let Some(Entry::Author(block)) = ledger.entries_mut().get_mut(at) {
          block.window_open = false;
        }
      }
    }
    ledger.push(Entry::Api(ApiCall {
      index,
      tag: tag.to_string(),
      role,
      epoch: ledger.flushes,
      queue_position,
      after_block,
      inside,
      attrs,
      parent,
      content,
      events: Vec::new(),
      rehearsed: false,
      queued: Vec::new(),
      unread: None,
      inside_events: Box::default(),
      cleared: false,
      errors: 0,
      command: Some(command.clone()),
    }));
  });
}

/// What a call no rehearsal read printed, as far as its tokens tell (`ApiCall::unread`, `Ledger::unrecorded`), read at
/// the API boundary that records its content, so the content check reads text alone, parted into words as an ink's is
/// (`resolver::words::printed_words`): its characters; a control symbol or a one-letter control word inside a word (an
/// accent, `Universit\"at`, a dotless i: a letter or digit, or a place read past, before it; a letter, a digit or a
/// group after it, past any group's end and one-letter control word, `F\'{\i}sica`, `Bras\'\i lia`; not past a control
/// symbol, `Smith\,\\Univ` two words), an accent's control symbol or a one-letter control word before a word's first
/// letter (`\'Ecole`, review revH3 2, [`accent_symbol`]; `\v{S}koda`, `\c{C}elik`, review revCK1b finding 4: the
/// accent and its argument one printed letter where the word begins) and a group there (`M{\"u}nchen`) part nothing,
/// the word counted whole, its accents and dotless letters unread (the check takes a word one side reads without them for the one the other prints with them,
/// `Fsica` for `física`, `resolver::placed`); a spacing control symbol (`Dept\\Muller`, `Dept,\\Muller`, review revCK1
/// 2-3; [`spacing_symbol`]; a `\\`'s `*` and `[…]` print nothing, `\\[2mm]`, review revCK1b finding 5,
/// `rehearsal::past_line_break`; a tabular's own call prints nothing, `\begin{tabular}{c}`, review revI2 finding 1,
/// but the text its column spec places, `@{Inst }`, review revI3 finding 1, `rehearsal::past_tabular_call`), another control sequence, its argument's group (`\IEEEauthorrefmark{1}Univ Alpha` →
/// ` 1 Univ Alpha`) or a math shift parts words. Nothing is spelled but a control sequence's length, its letter's
/// catcode and a control symbol's character: a letter command (`Gro{\ss}e`, a control word) parts its word.
pub(super) fn content_text(content: &Tokens) -> ContentText {
  let mut text = String::new();
  // (where it read past a control symbol or a one-letter control word inside a word, or an accent's control symbol or a
  // one-letter control word before its first letter, `\'Ecole Normale` (review revH3 2), `\v{S}koda`: its byte in `text`, and whether a one-letter control
  // word, which may print a letter)
  let mut past: Vec<(usize, bool)> = Vec::new();
  // (for each open group, whether it is a parting command's argument, which parts what is around it)
  let mut groups: Vec<bool> = Vec::new();
  let mut parted_by_command = false;
  let tokens = content.unlist_ref();
  // (past what prints nothing: a `\\`'s `*` and `[…]`, `\\[2mm]`, review revCK1b finding 5; a tabular's own call,
  // `\begin{tabular}{c}`, but for the spans its column spec prints, `{@{Inst }c}`, review revI3 finding 1)
  let mut unprinted_to = 0;
  let mut printed: Vec<std::ops::Range<usize>> = Vec::new();
  for (at, token) in tokens.iter().enumerate() {
    if at < unprinted_to && !printed.iter().any(|span| span.contains(&at)) {
      // (what prints nothing parts what is around it: `@{ and }` read apart from the spec around it)
      if !text.ends_with(' ') {
        text.push(' ');
      }
      parted_by_command = false;
      continue;
    }
    // (inside a word, a letter or digit or a place read past before it, or before its first letter, an accent's
    // control symbol or a one-letter control word there (`\v{S}koda`, review revCK1b finding 4; `\'Ecole`, never
    // `\\`, `\ `, `\,`, `\;`, `\:`, `\>`, the [`spacing_symbol`]s: review revCK1 2); and a letter, a digit or a group
    // after it, past any group's end and one-letter control word: `F\'{\i}sica`, `Bras\'\i lia`; a control
    // symbol before another parts, `Smith\,\\Univ` two words; a spacing one parts, `Dept\\Muller`, review revCK1 3)
    let in_word = || {
      !spacing_symbol(token)
        && (text.chars().next_back().is_some_and(char::is_alphanumeric)
          || past.last().is_some_and(|&(place, _)| place == text.len())
          || accent_symbol(token)
          || letter_cs(token))
        && tokens[at + 1..]
          .iter()
          .find(|next| next.get_catcode() != Catcode::END && !letter_cs(next))
          .is_some_and(starts_word)
    };
    let mut parting_command = false;
    match token.get_catcode() {
      Catcode::LETTER | Catcode::OTHER | Catcode::ACTIVE => text.push_str(&token.to_string()),
      Catcode::BEGIN => groups.push(parted_by_command),
      Catcode::END => {
        if groups.pop().unwrap_or(true) {
          text.push(' ');
        }
      },
      Catcode::CS => {
        if let Some((end, spans)) = past_tabular_call(tokens, at) {
          // (a tabular's own call prints nothing but its spec's printed spans: `\begin{tabular}{c}`, review revI2
          // finding 1; `{c@{ and }c}` prints "and", review revI3 finding 1)
          text.push(' ');
          unprinted_to = end;
          printed = spans;
        } else if let Some(end) = past_line_break(tokens, at) {
          // (a `\\` parts words and takes no group, its `*` and `[…]` print nothing: `\\[2mm]`, review revCK1b
          // finding 5)
          text.push(' ');
          unprinted_to = end;
        } else if short_cs(token) && in_word() {
          past.push((text.len(), letter_cs(token)));
        } else {
          text.push(' ');
          // (a spacing symbol takes no group: `\\{\'E}cole` one word)
          parting_command = !spacing_symbol(token);
        }
      },
      _ => text.push(' '),
    }
    parted_by_command = parting_command;
  }
  // (each word it read past one in, with the places in it, in characters)
  let mut read_past = Vec::new();
  let mut slots = past.into_iter().peekable();
  for word in printed_words(&text) {
    let start = word.as_ptr() as usize - text.as_ptr() as usize;
    let mut places = Vec::new();
    while let Some(&(at, letter)) = slots.peek()
      && at <= start + word.len()
    {
      slots.next();
      if at >= start {
        places.push((text[start..at].chars().count(), letter));
      }
    }
    if !places.is_empty() {
      read_past.push((word.to_lowercase(), places));
    }
  }
  ContentText { text: text.into(), read_past }
}

/// A content's text as the content check reads it ([`content_text`]), and the words it read past a control symbol or a
/// one-letter control word in: an accent (`Universit\"at` read `universitat`), a dotless `\i`/`\j` (`F\'{\i}sica` read
/// `fsica`), a cedilla's `\c` (`Fran\c{c}ois`). Only such a word may read as a word an ink prints with what they printed
/// (`resolver::placed`'s `reads_as`); every other word is matched as spelled.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContentText {
  pub text:      Box<str>,
  /// each word read past one (lowercased), with the places in it (in characters, before the character there) and
  /// whether a one-letter control word was there, which may have printed a letter (a dotless `\i`) — an accent prints
  /// none
  pub read_past: Vec<(String, Vec<(usize, bool)>)>,
}

impl std::ops::Deref for ContentText {
  type Target = str;

  fn deref(&self) -> &str { &self.text }
}

/// A control sequence 2 characters long, the backslash and one more: a control symbol (an accent, `\'`) or a
/// one-letter control word (`\i`), which `content_text` reads inside a word.
fn short_cs(token: &Token) -> bool {
  token.get_catcode() == Catcode::CS && token.with_str(|name| name.chars().count() == 2)
}

/// A one-letter control word (`\i`, `\j`): a short control sequence ([`short_cs`]) whose character has a letter's
/// catcode, no name spelled. A word reads on past it; a control symbol (`\\`, `\,`) it does not.
fn letter_cs(token: &Token) -> bool {
  short_cs(token)
    && token.with_str(|name| name.ends_with(|c| lookup_catcode(c) == Some(Catcode::LETTER)))
}

/// An accent's control symbol (`\'`, `` \` ``, `\^`, `\"`, `\~`, `\=`, `\.`: plain.tex's and latex.ltx's text accents
/// spelled as control symbols), the one control sequence read past before a word's first letter (`\'Ecole`).
fn accent_symbol(token: &Token) -> bool {
  matches!(
    symbol(token),
    Some('\'' | '`' | '^' | '"' | '~' | '=' | '.')
  )
}

/// A control symbol that prints a space or ends a line (`\\`, `\ `, `\,`, `\;`, `\:`, `\>`): it parts a word as the
/// space or the line's end it prints does (`Dept\\Muller` two words, as pdflatex prints them on two lines).
fn spacing_symbol(token: &Token) -> bool {
  matches!(symbol(token), Some('\\' | ' ' | ',' | ';' | ':' | '>'))
}

/// A control symbol's character (`\'` → `'`), the one spelling the content text reads (review revCK1 2-3): none for a
/// control word or a one-letter control word ([`letter_cs`]).
pub(super) fn symbol(token: &Token) -> Option<char> {
  (short_cs(token) && !letter_cs(token)).then(|| token.with_str(|name| name.chars().next_back()))?
}

/// A token that opens or continues a word after a control sequence inside it: a group, a letter or a digit.
fn starts_word(t: &Token) -> bool {
  t.get_catcode() == Catcode::BEGIN || t.with_str(|s| s.starts_with(char::is_alphanumeric))
}

/// The tag a queued call annotates, its first argument (`\lx@annotate@frontmatter@now{ltx:creator}…`), empty for a call
/// that annotates none: the API boundary's one read of a command.
pub(super) fn annotated_parent(command: &Tokens) -> String {
  match command.unlist_ref().as_slice() {
    [first, rest @ ..] if *first == T_CS!("\\lx@annotate@frontmatter@now") => {
      first_group(rest).to_string().trim().to_string()
    },
    _ => String::new(),
  }
}

/// Record a contact that prints nothing ([`UnprintedCall`]): an `ltx:rdf` annotating a creator, not the document's own
/// metadata (`\lx@add@frontmatter{ltx:rdf}[property=dcterms:identifier]`).
fn record_unprinted_call(attr: Option<&KeyVals>, command: &Tokens) {
  if annotated_parent(command) != "ltx:creator" {
    return;
  }
  let value = |key: &str| attr.and_then(|kv| kv.get_value(key)).map(|v| v.to_string());
  let mut event = Event::new(EventKind::Contact);
  event.role = value("property");
  event.unprinted = Some(value("content").unwrap_or_default());
  let attrs = attr
    .map(|kv| {
      kv.get_pairs()
        .map(|(key, value)| s!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(",")
    })
    .unwrap_or_default();
  with_ledger(|ledger| {
    let at = ledger.entries().len();
    let mut after_block = None;
    let mut inside = false;
    if let Some(block) = ledger.entries().iter().rev().find_map(|entry| match entry {
      Entry::Author(block) => Some(block),
      _ => None,
    }) {
      after_block = Some(block.index);
      inside = block.window_open && input_unread_since(&block.mark);
    }
    ledger.unprinted.push(UnprintedCall {
      at,
      after_block,
      inside,
      attrs,
      event,
    });
  });
}

/// Record a clear of the frontmatter queue (`dequeue_front_matter`: `\lx@clear@frontmatter{tag}[role=…]`): the
/// API calls it removes — of its tag and every attribute it names, queued since the last flush,
/// as the dequeue matches the queue's entries — are cleared, and so are, for a creator clear of the authors, the
/// author blocks queued before it, save the one whose own expansion makes the clear (the kernel `\author`'s replacing
/// call, which its `replace` covers). llncs's `\author` given twice — an anonymous or template one, then the real
/// one — clears the first one's creators, which today's drops (2104.04331, 1712.00617).
pub(in crate::base_utilities) fn record_clear(tag: &str, attr: &[(&str, &str)]) {
  if !records_now() {
    return;
  }
  let role = attr
    .iter()
    .find(|(key, _)| *key == "role")
    .map(|(_, role)| *role);
  with_ledger(|ledger| {
    let flushes = ledger.flushes;
    let latest_block = ledger.entries().iter().rev().find_map(|entry| match entry {
      Entry::Author(block) => Some(block.index),
      _ => None,
    });
    let clears = |entry: &Entry| match entry {
      Entry::Api(call) if call.epoch == flushes && call.tag == tag => {
        !call.cleared
          && attr.iter().all(|(key, value)| {
            if *key == "role" {
              call.role == *value
            } else {
              last_attr(&call.attrs, key) == *value
            }
          })
      },
      Entry::Author(block)
        if block.epoch == flushes
          && tag == "ltx:creator"
          && role.is_none_or(|role| role == "author")
          && attr.iter().all(|(key, _)| *key == "role") =>
      {
        let own =
          Some(block.index) == latest_block && block.window_open && input_unread_since(&block.mark);
        !block.cleared && !own
      },
      _ => false,
    };
    ledger.change_each(0..usize::MAX, clears, |entry| match entry {
      Entry::Api(call) => call.cleared = true,
      Entry::Author(block) => block.cleared = true,
      _ => {},
    });
  });
}

/// Clear the calls the last author body made (`queue::begin_author_body`, which removes them from the queue), its own
/// flush's, and open the next body's: the ledger's entries from here.
pub(in crate::base_utilities) fn record_author_body_clear() {
  if !records_now() {
    return;
  }
  with_ledger(|ledger| {
    let flushes = ledger.flushes;
    let (start, end) = std::mem::take(&mut ledger.author_body);
    let clears =
      |entry: &Entry| matches!(entry, Entry::Api(call) if call.epoch == flushes && !call.cleared);
    ledger.change_each(start..end, clears, |entry| {
      if let Entry::Api(call) = entry {
        call.cleared = true;
      }
    });
    ledger.author_body = (ledger.entries().len(), ledger.entries().len());
  });
}

/// The body [`record_author_body_clear`] opened ends: its calls are the ledger's entries so far.
pub(in crate::base_utilities) fn record_author_body_end() {
  if records_now() {
    with_ledger(|ledger| ledger.author_body.1 = ledger.entries().len());
  }
}

/// Record that the frontmatter API's Cut took an uncut copy of a printed list it cut out of the queue (`content`, a
/// contact of `role`: `frontmatter_api::cut::place_uncut_copies`): the first such call queued
/// since the last flush and not cleared is cleared, as [`record_clear`] clears the calls a dequeue removes.
pub(in crate::base_utilities) fn record_cut_copy(role: &str, content: &Tokens) {
  if !records_now() {
    return;
  }
  let content = queued_affiliation_text(content);
  with_ledger(|ledger| {
    let flushes = ledger.flushes;
    let copy = ledger.entries().iter().position(|entry| {
      matches!(entry, Entry::Api(call) if call.epoch == flushes
        && call.tag == "ltx:contact"
        && call.role == role
        && !call.cleared
        && call.content == content)
    });
    if let Some(at) = copy
      && let Some(Entry::Api(call)) = ledger.entries_mut().get_mut(at)
    {
      call.cleared = true;
    }
  });
}

/// The last value of `key` in a call's recorded attributes (`role=affiliation,label=a`), as the queue's attribute hash
/// keeps it (`queue_front_matter`: the last value of a repeated key); empty when it has none. It mirrors the dequeue's
/// match on any attribute, which no caller uses today — every clear passes only `role` (Perl
/// Base_Utility.pool.ltxml:289-311) —, and it splits at every comma, so a braced value holding one (`label={a, b}`,
/// `record_api_call`) would not match: a clear naming another key needs that first (review rev92a_r1 N2).
fn last_attr<'a>(attrs: &'a str, key: &str) -> &'a str {
  attrs
    .split(',')
    .rev()
    .find_map(|kv| {
      let (k, v) = kv.split_once('=')?;
      (k.trim() == key).then(|| v.trim())
    })
    .unwrap_or("")
}

/// Record a footnote's text (`\lx@notetext`'s whatsit, once digested: `\footnotetext[<n>]{…}`),
/// after an author block or the title (whose `\footnotemark` it may answer: `\title{…\footnotemark[1]}`
/// `\footnotetext[1]{Based on observations …}`, 1111.4983): its printed mark, its counter value and its body's events
/// (the body is digested for real, so nothing is rehearsed — only read).
pub fn record_note_text(whatsit: &Whatsit) {
  if !records_now()
    || !with_ledger(|ledger| {
      ledger.entries().iter().any(|e| match e {
        Entry::Author(_) => true,
        Entry::Api(call) => call.tag == "ltx:title",
        _ => false,
      })
    })
  {
    return;
  }
  let printed = |key: &str| match whatsit.get_property(key).as_deref() {
    Some(Stored::Digested(d)) => {
      let mut text = String::new();
      ink_text(d, &mut text);
      Some(text.trim().to_string())
    },
    Some(Stored::String(s)) => Some(to_string(*s).trim().to_string()),
    _ => None,
  };
  if printed("role").as_deref() != Some("footnotetext") {
    return;
  }
  let mark = printed("mark").unwrap_or_default();
  let value = printed("fnvalue");
  let events = whatsit
    .get_arg(4)
    .map(|body| lex_body_with_inks(body).0)
    .unwrap_or_default();
  with_ledger(|ledger| {
    let index = ledger.entries().len();
    ledger.push(Entry::NoteText(NoteText {
      index,
      epoch: ledger.flushes,
      mark,
      value,
      events,
    }));
  });
}

/// A call's label with its parameterless macros written out — `fuzzy:\lx@aas@correspondent` is "fuzzy:Bob Baker", aastex's
/// `\email` after a `\correspondingauthor` (aastex_cls.rs `\lx@aas@email@corresp`; 2506.07978, whose label spelled no
/// one and went to the block): the label today's `\lx@annotate@frontmatter@now` reads once the flush digests its
/// keyvals. A pure lookup of each macro's token body, nothing run through the gullet: the reading changes no state.
/// Bounded by the tokens visited (macros expanding to nothing included, [`LABEL_VISITS`]) and the depth (8).
fn label_text(value: &Tokens) -> String {
  fn written(tokens: &[Token], depth: usize, visited: &mut usize, out: &mut Vec<Token>) {
    for t in tokens {
      if *visited >= LABEL_VISITS {
        return;
      }
      *visited += 1;
      if depth < 8
        && t.get_catcode() == Catcode::CS
        && let Some(Stored::Expandable(defn)) = lookup_meaning(t)
        && defn
          .get_parameters()
          .is_none_or(|p| p.get_parameters().is_empty())
        && let Some(ExpansionBody::Tokens(body)) = defn.get_expansion()
      {
        written(body.unlist_ref(), depth + 1, visited, out);
      } else {
        out.push(*t);
      }
    }
  }
  let mut out = Vec::new();
  written(value.unlist_ref(), 0, &mut 0, &mut out);
  Tokens::new(out).to_string()
}

/// Whether `content` is wrapped in `\lx@affiliation@withsup` (inside the other affiliation wrapper or not), as
/// `queued_affiliation_text` unwraps it.
fn wraps_withsup(content: &Tokens) -> bool {
  let mut v = content.unlist_ref().as_slice();
  loop {
    let wrapped = v.len() >= 3
      && v[1].get_catcode() == Catcode::BEGIN
      && skip_group(v, 1) == v.len()
      && (v[0] == T_CS!("\\lx@affiliation@withinst") || v[0] == T_CS!("\\lx@affiliation@withsup"));
    if !wrapped {
      return false;
    }
    if v[0] == T_CS!("\\lx@affiliation@withsup") {
      return true;
    }
    v = &v[2..v.len() - 1];
  }
}

/// The tokens [`label_text`] visits at most, the label's own and its macros' bodies'.
const LABEL_VISITS: usize = 1024;

/// The content of the first top-level brace group of `tokens` (an annotation command's parent tag).
fn first_group(tokens: &[Token]) -> Tokens {
  let mut depth = 0usize;
  for (i, t) in tokens.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END if depth > 0 => {
        depth -= 1;
        if depth == 0 {
          let open = tokens[..i]
            .iter()
            .position(|t| t.get_catcode() == Catcode::BEGIN)
            .unwrap_or(0);
          return Tokens::new(tokens[open + 1..i].to_vec());
        }
      },
      _ => {},
    }
  }
  Tokens::new(Vec::new())
}

/// `tokens`, each stamped with its position in field `field_id` (`Token::prov`) from `base` on: the positions
/// `\lx@field@mark` gives a store's value, and `\lx@field@stamp` an argument.
pub(in crate::base_utilities) fn field_stamped(
  field_id: u16,
  base: usize,
  tokens: &[Token],
) -> Tokens {
  Tokens::new(
    tokens
      .iter()
      .enumerate()
      .map(|(i, t)| Token {
        prov: field_prov(field_id, base + i),
        ..*t
      })
      .collect(),
  )
}

/// The tokens of `block`, each stamped with its position in the author field, for the rehearsal.
pub(in crate::base_utilities) fn stamped_block_tokens(block: &AuthorBlock) -> Tokens {
  field_stamped(2, block.range.0, block.tokens.unlist_ref())
}
