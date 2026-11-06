//! The observer's ledger: one block per `\author`-family call, the S1 stamp of each `\title` call, and the
//! contact/affiliation API calls, in call order (OXIDIZED_DESIGN_DIVERGENCES #837). Recorded only under the shadow
//! switch ([`super::shadow::fm_shadow_enabled`]) and never while the frontmatter is frozen (a replayed class setter, a
//! speculative title-page run, the observer's own rehearsal: those calls are not the document's).

use std::{cell::RefCell, rc::Rc};

use latexml_core::{
  definition::{
    Definition,
    origin::{DefinitionOrigin, frontmatter_watch, watch_frontmatter_setter_calls},
  },
  gullet::{InputMark, input_mark, input_unread_since},
  state::{fallback_meaning_origin, frontmatter_setter_calls, open_frontmatter_setter},
  token::{FIELD_PROV_POS_MASK, field_prov},
};

use super::{
  events::{EmailMeaning, Event, EventKind, HeldError, lex_events},
  resolver::Resolved,
  shadow::{FmShadow, fm_shadow_mode},
  *,
};

/// How a block's tokens got their positions in the author field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::base_utilities) enum Stamp {
  /// The kernel `\author`'s `\lx@field@mark` stamped the stored value these tokens are a copy of, at these positions
  /// (S1, OXIDIZED_DESIGN #715).
  Kernel,
  /// No mark stamps them (a binding's own `\lx@add@authors*` call): the positions an argument stamper
  /// (`\lx@field@stamp`) would give them, continuing the field's positions after every block before.
  Projected,
  /// Out of positions (14 bits per field, cumulative per document): the value went unstamped.
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
    events:       Vec<Event>,
    errors:       usize,
    warnings:     usize,
    declarations: Vec<Event>,
    breaks:       Vec<Event>,
    held:         Vec<HeldError>,
  },
  /// Not rehearsed, and why.
  Skipped(SkipReason),
}

/// Why a block was not rehearsed (or its rehearsal not kept).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::base_utilities) enum SkipReason {
  /// its value went unstamped: out of positions (14 bits per field, cumulative per document)
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
  /// the binding owning the `\author` meaning at the call ([`author_binding`]): a compiled binding's file name
  /// (`revtex4_support.sty`), else `kernel`, `document`, `raw` or `fallback` by its definition origin
  pub owner:          String,
  /// the frontmatter setter calls made before it (`state::frontmatter_setter_calls`): a setter whose expansion is open
  /// at an API call its own expansion makes was called inside it when its number is greater ([`record_api_call`])
  pub setters_before: usize,
  /// the value of the policy's state counter at the call (`policy::Policy::state_counter`), when it has one defined
  pub counter:        Option<i64>,
  /// whether the call replaces the authors before it (`\lx@clear@frontmatter{ltx:creator}`)
  pub replace:        bool,
  /// whether a later creator clear in its queue removed its authors ([`record_clear`]: llncs's `\author` given twice,
  /// its `\lx@clear@creators[role=author]` — a call of its own, not the block's `replace`)
  pub cleared:        bool,
  /// whether the IJCAI sectioned split applies
  pub sectioned:      bool,
  /// a per-author binding's cautious name split
  pub cautious:       bool,
  /// the keyvals the call hands each author (revtex `annotations=`)
  pub keyvals:        Option<String>,
  /// the frontmatter flushes (`digest_front_matter`) before the call: the queue the position is in (each flush
  /// empties it)
  pub epoch:          usize,
  /// the frontmatter queue's length at the call, and the authors queued then — counted under the census only
  /// (`FmShadow::On`), the dump ([`fm_shadow_json`](super::fm_shadow_json)) its only reader: a count of the queue per
  /// author call is quadratic in the authors
  pub queue_position: usize,
  pub authors_queued: Option<usize>,
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
  /// the binding of the frontmatter setter whose expansion made it ([`FRONTMATTER_SETTERS`], [`setter_binding`]:
  /// svjour3's `\institute`, sv_support.sty's; authblk's `\affil`; a raw class's setter rerouted in place, the class's
  /// family), `unobserved` for a call made before the watch began ([`watch_unwritten_setters`]) — `None` for a call no
  /// observed setter's expansion made (a block's own, the document's `\lx@add@…`, a binding command of its own:
  /// `\thanks`, `\IEEEauthorblockA`; an entry a raw file's store hands on at `\maketitle`): what the writer writes only
  /// when that binding's family is written (`policy::family_written`, `writer::unwritten_calls`)
  pub binding:        Option<String>,
  /// the entry's attributes as the call gave them (`role=affiliation,label=a,annotate=new`)
  pub attrs:          String,
  /// the element an annotation goes into (`\lx@annotate@frontmatter{ltx:creator}…`, `ltx:title` for a title's note),
  /// empty for an entry of its own (`\lx@add@frontmatter`)
  pub parent:         String,
  /// the entry's content (its command's last argument, inside the affiliation wrappers), and its events once the
  /// flush rehearsed it (`shadow::shadow_flush`; empty before)
  pub content:        Tokens,
  pub events:         Vec<Event>,
  /// whether a flush rehearsed its content
  pub rehearsed:      bool,
  /// whether a later clear removed it from the queue before any flush digested it ([`record_clear`])
  pub cleared:        bool,
  /// the errors its content's rehearsal held (no positions: an API call's content has none)
  pub errors:         usize,
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
  pub entries:      Vec<Entry>,
  /// the positions the projected blocks took beyond the field's own stamps
  pub shadow_extra: usize,
  /// the frontmatter flushes so far (`shadow::shadow_flush`)
  pub flushes:      usize,
  /// the writer's outcome for a written family's paper (`writer::write_observed_frontmatter`): `same` (today's reads
  /// as the reading), `written` (including a paper written with its e-mail calls each kept one contact, where their
  /// write-out declined, `writer::give_back_listed`), `written-different: …` (what was written still differs, the
  /// writer's own defect) or `declined: …` (a difference no operation explains, today's kept — also where keeping the
  /// e-mail calls whole leaves today's as it was: the write-out the reading wanted, declined; and where an unwritten
  /// family's items kept as today's leave it so, `writer::keep_as_today`); none where the writer did not run
  pub writer:       Option<String>,
  /// the contacts recorded that print nothing, in call order ([`UnprintedCall`])
  pub unprinted:    Vec<UnprintedCall>,
  /// the census's (E0): why the writer's plan or re-part refused, its first refusal or the writer's own decline — what
  /// `writer: declined …` counts under in `ab_audit fmshadow`'s closed list (`writer::note_refusal`)
  pub refusal:      Option<String>,
  /// the census's (E0): the reading the writer last compared today's with, its give-backs applied: what the read-back
  /// compares the final document with where the writer got that far (`writer::read_back_frontmatter`)
  pub compared:     Option<Resolved>,
  /// the census's (E0): the read-back at the end of `insert_frontmatter`, none where it never got there
  pub read_back:    Option<ReadBack>,
  /// the entries the last binding's `\author` body made, a later `\author` clears them ([`record_author_body_clear`])
  author_body:      (usize, usize),
}

/// The final document read against the reading (`writer::read_back_frontmatter`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::base_utilities) struct ReadBack {
  /// where they first differ (`plan::difference`), none where the document reads as the reading
  pub difference: Option<String>,
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
/// is, [`email_meaning`]), and the other [`FRONTMATTER_SETTERS`] (whose an API call their expansion makes is,
/// [`setter_binding`]).
const OWNED_COMMANDS: [&str; 12] = [
  "\\author",
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
const AUTHOR: usize = 0;

/// The frontmatter setters whose expansions the observer watches (`state::open_frontmatter_setter`): an API call one
/// of them makes is the binding's owning it ([`ApiCall::binding`]) — the e-mail commands, the brief's `\institute`,
/// `\address` and `\affil` (95b r3), and the contact setters of a raw class's stores, which the store reroute makes
/// API calls of (`latexml_core::binding::store_setters::STORE_SETTERS`: `\affiliation`, `\ead`, …) — not `\inst`, a
/// binding's mark request (inst_support.sty's, which aa loads: no API call, and it would watch every aa paper).
/// `\author` is a block's (its own expansion's calls are the block's, `ApiCall::inside`).
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

/// The compiled bindings loaded, in load order, and each one's first nested load declaring conventions (`LOADED`).
type LoadFamilies = (Vec<String>, rustc_hash::FxHashMap<String, String>);

/// A tracked command's owning binding and the meaning it made.
type Owner = Option<(String, Rc<dyn Definition>)>;
const OWNED: usize = OWNED_COMMANDS.len();

thread_local! {
  /// For each of [`OWNED_COMMANDS`], the compiled binding that defined its current meaning, with that meaning itself
  /// ([`note_binding_load`]; held, so its identity is `Rc::ptr_eq`'s and no later definition can take its address),
  /// under the shadow switch only.
  static OWNERS: RefCell<[Owner; OWNED]> = const { RefCell::new([const { None }; OWNED]) };
  /// The document class's compiled bindings: the first class binding loaded outside any other binding's load, and every
  /// binding its load loaded (`report.cls`; iopart.cls's `iopart_support.sty`), in load order; and how deep the loads
  /// nest now, with whether the outermost is that class's.
  static CLASS_BINDINGS: RefCell<ClassBindings> = const { RefCell::new((Vec::new(), 0, false)) };
  /// The binding owning `\author` when the document class's load ended, when one of the class's bindings: the class's
  /// own `\author` reading (elsarticle's elsart_support_core.sty), whose family is the class's ([`class_family`]).
  static CLASS_AUTHOR: RefCell<Option<String>> = const { RefCell::new(None) };
  /// The compiled bindings loaded so far, in load order, and for each binding whose load loaded one declaring
  /// conventions, the first such ([`note_binding_load`]): its family, when it declares none (`setter_binding`).
  static LOADED: RefCell<LoadFamilies> = RefCell::new(Default::default());
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
/// (`email_meaning`). Under the shadow switch only; otherwise just the load.
pub fn note_binding_load<R>(name: &str, extension: &str, load: impl FnOnce() -> R) -> R {
  if !fm_shadow_enabled() {
    return load();
  }
  let before: [Option<Rc<dyn Definition>>; OWNED] = std::array::from_fn(tracked_meaning);
  let owners_before: [Option<Rc<dyn Definition>>; OWNED] = std::array::from_fn(owned_meaning);
  let loads_before = LOADED.with(|l| {
    let mut l = l.borrow_mut();
    l.0.push(s!("{name}.{extension}"));
    l.0.len()
  });
  let class_load = CLASS_BINDINGS.with(|c| {
    let (names, depth, in_class) = &mut *c.borrow_mut();
    let class_load = *depth == 0 && extension == "cls" && names.is_empty();
    if class_load {
      *in_class = true;
    }
    if *in_class && extension != "pool" {
      names.push(s!("{name}.{extension}"));
    }
    *depth += 1;
    class_load
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
  if class_load {
    let owner = meaning_owner(AUTHOR);
    let own = CLASS_BINDINGS.with(|c| c.borrow().0.contains(&owner));
    CLASS_AUTHOR.with(|a| *a.borrow_mut() = own.then_some(owner));
  }
  // (the first binding its load loaded that declared conventions: whose family a binding declaring none is,
  // `setter_binding` — revtex3_support.sty's revtex4_support.sty)
  LOADED.with(|l| {
    let mut l = l.borrow_mut();
    if let Some(family) = l.0[loads_before..]
      .iter()
      .find(|n| policy::has_entry(n))
      .cloned()
    {
      l.1.entry(s!("{name}.{extension}")).or_insert(family);
    }
  });
  value
}

/// Forget the tracked commands' owners and the class's bindings (a conversion's start).
pub(in crate::base_utilities) fn reset_author_owner() {
  OWNERS.with(|o| *o.borrow_mut() = [const { None }; OWNED]);
  CLASS_BINDINGS.with(|c| *c.borrow_mut() = (Vec::new(), 0, false));
  CLASS_AUTHOR.with(|a| *a.borrow_mut() = None);
  LOADED.with(|l| *l.borrow_mut() = Default::default());
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

/// The binding an API call made in the expansion of the frontmatter setter `cs` ([`FRONTMATTER_SETTERS`]) is keyed on:
/// the binding owning the setter's meaning now ([`meaning_owner`]: by the binding's own redefinition, OD #922; else
/// `raw`, `document`, `fallback` by its origin), when it declared conventions of its own (authblk's `\affil`, also
/// under a raw class that loads it — pnas-new.cls, 1706.06282; sv_support.sty's `\institute` under svjour3; aa.cls's
/// own `\institute`); the family of the class ([`class_family`]) when the owner is the kernel, a file read raw (a raw
/// class's or style's setter rerouted to the API in place, OD #936 — read as the kernel's under a class whose bindings
/// declare none, the kernel's gate, OD #925) or one of the class's bindings declaring none (revtex4-2.cls's setters,
/// revtex4_support.sty's family); for another binding declaring none, the first binding its load loaded that declared
/// conventions (`\documentstyle{revtex}`'s revtex3_support.sty, loaded by revtex.sty after the article class, whose
/// `\address` is REVTeX's under the revtex4_support.sty it loads: cond-mat0505458, cond-mat9803141, hep-ph9909393 —
/// 95b r3's scorer run); else the owner, as a block's is keyed ([`policy_binding`]: a package binding declaring none
/// and loading none that does, the document's own definition — `UNDECLARED`, written since 97e).
fn setter_binding(cs: &Token) -> Option<String> {
  // (a robust setter's body is its `\cs␣`'s: `state::watch_frontmatter_setters`)
  let n = cs.with_cs_name(|name| {
    OWNED_COMMANDS
      .iter()
      .position(|c| *c == name.trim_end_matches(' '))
  })?;
  let owner = meaning_owner(n);
  if policy::has_entry(&owner) {
    return Some(owner);
  }
  if matches!(owner.as_str(), "kernel" | "raw")
    || CLASS_BINDINGS.with(|c| c.borrow().0.contains(&owner))
  {
    return Some(class_family());
  }
  Some(
    LOADED
      .with(|l| l.borrow().1.get(&owner).cloned())
      .unwrap_or_else(|| policy_binding(&owner)),
  )
}

/// The binding an API call is given when its setter's expansion went unobserved ([`watch_unwritten_setters`]): no
/// family's — not written (`policy::family_written`).
const UNOBSERVED: &str = policy::UNOBSERVED_BINDING;

/// At a written family's block: observe the frontmatter setters' expansions from now on ([`ApiCall::binding`]) when one
/// of them is defined and an unwritten family's ([`setter_binding`]: svjour3's `\institute` beside authblk's
/// `\author`, 2609.05466) — the only paper whose API calls the writer must tell apart; every other converts without the
/// watch, which costs every macro call (`Expandable::invoke`). The API calls made before it, by no block's expansion,
/// are given [`UNOBSERVED`]: whose they were is not known, so they are not written (a setter called before `\author`).
fn watch_unwritten_setters() {
  if frontmatter_watch() {
    return;
  }
  let unwritten = FRONTMATTER_SETTERS.iter().any(|name| {
    let cs = T_CS!(name);
    lookup_definition(&cs).ok().flatten().is_some()
      && setter_binding(&cs).is_some_and(|binding| !policy::family_written(&binding))
  });
  if !unwritten {
    return;
  }
  watch_frontmatter_setter_calls(Some(&FRONTMATTER_SETTERS[..]));
  with_ledger(|ledger| {
    for entry in &mut ledger.entries {
      if let Entry::Api(call) = entry
        && !call.inside
        && call.binding.is_none()
        && !matches!(call.tag.as_str(), "ltx:title" | "ltx:creator")
      {
        call.binding = Some(UNOBSERVED.to_string());
      }
    }
  });
}

/// The family of the document class: the first of its bindings that declared conventions, in load order (aa.cls
/// before the aa_support.sty and inst_support.sty it loads; sv_support.sty under svjour3.cls), else the class's own
/// `\author` binding (a class whose bindings declare none, its `\author` an undeclared support file's), else the
/// kernel's (a class whose bindings read no `\author` of their own: OmniBus, the kernel's gate, OD #925).
fn class_family() -> String {
  CLASS_BINDINGS
    .with(|c| {
      c.borrow()
        .0
        .iter()
        .find(|name| policy::has_entry(name))
        .cloned()
    })
    .unwrap_or_else(|| {
      CLASS_AUTHOR
        .with(|a| a.borrow().clone())
        .unwrap_or_else(|| "kernel".to_string())
    })
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
/// 1412.0748, 1809.01965).
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
  let by_binding = !guessed
    && tracked_meaning(n).is_some_and(|meaning| {
      matches!(
        meaning.get_origin(),
        DefinitionOrigin::Binding | DefinitionOrigin::Borrowed
      )
    });
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
  fm_shadow_enabled()
    && !lookup_bool("lx_replaying_class_setter")
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

/// Record an `\author`-family call (`hook`, with `stuff` its argument), under the shadow switch only.
pub(in crate::base_utilities) fn record_author_call(
  hook: &'static str,
  stuff: &Tokens,
  replace: bool,
  sectioned: bool,
  keyvals: Option<&Tokens>,
  cautious: bool,
) {
  if !records_now() {
    return;
  }
  let len = stuff.unlist_ref().len();
  let real_base = usize::try_from(lookup_int("lx_field_base_2")).unwrap_or(0);
  let owner = author_binding();
  let binding = policy_binding(&owner);
  let policy = policy::policy_for(&binding, hook).name;
  // (a policy's own state, read at the call: authblk's `\c@Maxaffil`, which decides whether its notes show)
  let counter = policy::policy_for(&binding, hook)
    .state_counter
    .and_then(|name| lookup_register_quiet(&s!("\\c@{name}")).map(|value| value.value_of()));
  if policy::policy_for(&binding, hook).written {
    watch_unwritten_setters();
  }
  let block = AuthorBlock {
    index: 0,
    hook,
    policy: policy.to_string(),
    binding,
    owner,
    setters_before: frontmatter_setter_calls(),
    counter,
    replace,
    cleared: false,
    sectioned,
    cautious,
    keyvals: keyvals.map(|kv| kv.to_string()),
    epoch: 0,
    queue_position: queue_length(),
    authors_queued: (fm_shadow_mode() == FmShadow::On).then(queued_author_count),
    stamp: Stamp::Projected,
    range: (0, 0),
    tokens: stuff.clone(),
    source: stuff.to_string(),
    mark: input_mark(),
    window_open: true,
    outcome: Outcome::Pending,
  };
  with_ledger(|ledger| {
    let base = real_base + ledger.shadow_extra;
    let mut block = block;
    block.index = ledger.entries.len();
    block.epoch = ledger.flushes;
    if base + len > usize::from(FIELD_PROV_POS_MASK) {
      block.stamp = Stamp::Overflow;
    } else {
      block.range = (base, base + len);
      ledger.shadow_extra += len;
    }
    for entry in &mut ledger.entries {
      if let Entry::Author(earlier) = entry {
        earlier.window_open = false;
      }
    }
    ledger.entries.push(Entry::Author(Box::new(block)));
  });
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
      let index = ledger.entries.len();
      ledger.entries.push(Entry::Title(TitleStamp {
        index,
        length: len,
        range: base.map(|base| (base, base + len)),
      }));
      return;
    }
    if field_id != 2 {
      return;
    }
    let Some(block) = ledger
      .entries
      .iter_mut()
      .rev()
      .find_map(|entry| match entry {
        Entry::Author(block) => Some(block),
        _ => None,
      })
    else {
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
    if block.stamp == Stamp::Projected {
      ledger.shadow_extra -= len;
    }
    match base {
      Some(base) => {
        block.stamp = Stamp::Kernel;
        block.range = (base, base + len);
      },
      None => {
        block.stamp = Stamp::Overflow;
        block.range = (0, 0);
      },
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
/// its command), under the shadow switch only: a binding's single-author call (`\lx@add@creator`, `\lx@add@author`:
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
  let setter = open_frontmatter_setter();
  with_ledger(|ledger| {
    let index = ledger.entries.len();
    let mut after_block = None;
    let mut inside = false;
    let mut setters_before = 0;
    if let Some(block) = ledger
      .entries
      .iter_mut()
      .rev()
      .find_map(|entry| match entry {
        Entry::Author(block) => Some(block),
        _ => None,
      })
    {
      after_block = Some(block.index);
      setters_before = block.setters_before;
      if block.window_open {
        if input_unread_since(&block.mark) {
          inside = true;
        } else {
          block.window_open = false;
        }
      }
    }
    // (the innermost of the block's expansion and a setter's: a setter called inside the block's own expansion makes
    // its calls, a block called inside a setter's its)
    let binding = setter
      .filter(|(called, _)| !inside || *called > setters_before)
      .and_then(|(_, cs)| setter_binding(&cs));
    ledger.entries.push(Entry::Api(ApiCall {
      index,
      tag: tag.to_string(),
      role,
      epoch: ledger.flushes,
      queue_position,
      after_block,
      inside,
      binding,
      attrs,
      parent,
      content,
      events: Vec::new(),
      rehearsed: false,
      cleared: false,
      errors: 0,
    }));
  });
}

/// The tag a queued call annotates, its first argument (`\lx@annotate@frontmatter@now{ltx:creator}…`), empty for a call
/// that annotates none: the API boundary's one read of a command.
fn annotated_parent(command: &Tokens) -> String {
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
    let at = ledger.entries.len();
    let mut after_block = None;
    let mut inside = false;
    if let Some(block) = ledger.entries.iter().rev().find_map(|entry| match entry {
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

/// Record a clear of the frontmatter queue (`dequeue_front_matter`: `\lx@clear@frontmatter{tag}[role=…]`), under the
/// shadow switch only: the API calls it removes — of its tag and every attribute it names, queued since the last flush,
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
    let latest_block = ledger.entries.iter().rev().find_map(|entry| match entry {
      Entry::Author(block) => Some(block.index),
      _ => None,
    });
    for entry in &mut ledger.entries {
      match entry {
        Entry::Api(call) if call.epoch == flushes && call.tag == tag => {
          let matches = attr.iter().all(|(key, value)| {
            if *key == "role" {
              call.role == *value
            } else {
              last_attr(&call.attrs, key) == *value
            }
          });
          if matches {
            call.cleared = true;
          }
        },
        Entry::Author(block)
          if block.epoch == flushes
            && tag == "ltx:creator"
            && role.is_none_or(|role| role == "author")
            && attr.iter().all(|(key, _)| *key == "role") =>
        {
          let own = Some(block.index) == latest_block
            && block.window_open
            && input_unread_since(&block.mark);
          if !own {
            block.cleared = true;
          }
        },
        _ => {},
      }
    }
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
    for entry in ledger.entries.iter_mut().take(end).skip(start) {
      if let Entry::Api(call) = entry
        && call.epoch == flushes
      {
        call.cleared = true;
      }
    }
    ledger.author_body = (ledger.entries.len(), ledger.entries.len());
  });
}

/// The body [`record_author_body_clear`] opened ends: its calls are the ledger's entries so far.
pub(in crate::base_utilities) fn record_author_body_end() {
  if records_now() {
    with_ledger(|ledger| ledger.author_body.1 = ledger.entries.len());
  }
}

/// Record that the frontmatter API's Cut took an uncut copy of a printed list it cut out of the queue (`content`, a
/// contact of `role`: `affiliations::place_uncut_copies`), under the shadow switch only: the first such call queued
/// since the last flush and not cleared is cleared, as [`record_clear`] clears the calls a dequeue removes.
pub(in crate::base_utilities) fn record_cut_copy(role: &str, content: &Tokens) {
  if !records_now() {
    return;
  }
  let content = queued_affiliation_text(content);
  with_ledger(|ledger| {
    let flushes = ledger.flushes;
    if let Some(call) = ledger.entries.iter_mut().find_map(|entry| match entry {
      Entry::Api(call)
        if call.epoch == flushes
          && call.tag == "ltx:contact"
          && call.role == role
          && !call.cleared
          && call.content == content =>
      {
        Some(call)
      },
      _ => None,
    }) {
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

/// Record a footnote's text (`\lx@notetext`'s whatsit, once digested: `\footnotetext[<n>]{…}`), under the shadow switch
/// only, after an author block or the title (whose `\footnotemark` it may answer: `\title{…\footnotemark[1]}`
/// `\footnotetext[1]{Based on observations …}`, 1111.4983): its printed mark, its counter value and its body's events
/// (the body is digested for real, so nothing is rehearsed — only read).
pub fn record_note_text(whatsit: &Whatsit) {
  if !records_now()
    || !with_ledger(|ledger| {
      ledger.entries.iter().any(|e| match e {
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
  let events = whatsit.get_arg(4).map(lex_events).unwrap_or_default();
  with_ledger(|ledger| {
    let index = ledger.entries.len();
    ledger.entries.push(Entry::NoteText(NoteText {
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
/// keyvals. A pure lookup of each macro's token body, nothing run through the gullet: the shadow changes no state.
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
