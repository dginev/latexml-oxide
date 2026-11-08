//! The observer's resolver (Layer C of the design note's §2; OXIDIZED_DESIGN_DIVERGENCES #837): a document's ledger —
//! each author block's events and the frontmatter API calls after them — read into the tuples `ab_audit`'s `front.rs`
//! extracts from the XML (each creator's name, affiliations, contacts and notes, and the author block's own), each
//! association with the rule that decided it. Class-independent; a family's conventions are those its binding declared
//! (`policy.rs`, Layer B). In SHADOW mode only (stage S2a.2): nothing it decides reaches the output.
//!
//! The rules, in order (design §2 points 1-9, the FIDELITY_CHARTER §4 rulings of 2026-10-21):
//! 1. **Geometry**: columns from `Sep(And)` (the `\and` family) and alignment tabs, unless the policy reads `\and` as
//!    inline (llncs); rows from `Break`.
//! 2. **Line type**: a line whose first ink is a mark is a labelled line, unless it is the block's first line and its
//!    entries read as names with their marks answered below (the prefix convention); a column's first line, or one
//!    reading as names, is a names line; a line of contacts only a contact line; any other an unmarked line, the
//!    classify helpers (`name_count`, `reads_as_affiliation`) deciding.
//! 3. **Names**: a names line split at its separators; each mark attached to the name beside it — the one before,
//!    or the one after a comma the mark is not glued to (a prefix), or the one after when no name precedes; an `\inst`
//!    after a comma the one before under the policy (OD #807). A font switch inside a name run keeps it whole (the
//!    lexer's run: gretsi's `first \textsc{last}`, 2508.03934).
//! 4. **Ruling 1**: the marks shown on a name are its requests (`1,2` → 1, 2; a symbol run glued to a label, `*1`, two
//!    requests); the name is its text without them. A labelled entry goes to every name requesting its label.
//! 5. **Ruling 2**: a line led by a footnote-symbol mark labels that line for every name carrying the same character;
//!    a symbol no name carries leaves its line to the block.
//! 6. **Unlabelled lines** go to the names row above them in their column (R12; llncs's `\\` line, OD #792).
//! 7. **Contacts**: own mark → spelled author → (the class's list in author order, `SpelledElseOrder`) → the group's
//!    only author → elimination (the one address no name spells, the one name no address spells: today's, OD #52 under
//!    user ruling R10) → the one author marked as the corresponding author → the block.
//! 8. **Ruling 3**: one contact per print event, nothing deduplicated (a line's addresses one each).
//! 9. **Ruling 4**: a note whose content all became contacts is dropped, and so is a note nothing in which prints (an
//!    empty `\thanks{}`: `Event::inkless`, the user's ruling 2026-10-30; OD #916).
//! 10. **API calls** after a block (not made by its expansion), creators and creator annotations only
//!     ([`creator_call`]): a creator; a labelled contact (its `label=`, its `labelseq=` place, its leading mark, or a
//!     `\label` a `\ref` mark names) to the names requesting it — a creator's own `author:N` exactly, any other label by
//!     Perl's prefix chain ([`label_owners`]); an unlabelled one by its `annotate=` (the last author, all, the run of new
//!     ones), authblk's by its `\AB@thenote` when the marks show.
//!
//! S2a.2 r2 (review revfms2a2) adds: names lines by evidence (`later_line_is_names`), a note's whole text and its
//! legend by its leading Mark, marks mid-line and trailing, today's joined unmarked lines and contact grid, a replacing
//! `\author` call (OXIDIZED_DESIGN_DIVERGENCES #837's r2 paragraph).

use super::{
  events::{self, EmailMeaning, Event, EventKind, SepKind, events_lined_text, events_text},
  policy::{EmailOwner, Policy, policy_for},
  *,
};

/// Where an association came from: the ledger index of its block or API call, and its span (unique only within a
/// block).
pub(super) type Source = (usize, Option<(u16, u16)>);

/// One association the resolver made: a contact, an affiliation or a note, with its text, its label and the rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assoc {
  /// `affiliation`, `email`, `url`, … for a contact; the note's role (`thanks`, `legend`) for a note
  pub role:   String,
  pub text:   String,
  pub label:  Option<String>,
  pub rule:   String,
  /// the event it came from: (ledger index of its block or API call, its span) — spans are unique only within a
  /// block (notes decision 4)
  pub source: Option<(usize, Option<(u16, u16)>)>,
  /// a link's target, where the link points — what the contact holds, its `text` the anchor it prints: an `\href`'s
  /// url (`\href{https://academics.aut.ac.nz/robin.hankin}{work}`, 2210.15991), a `\url`'s text, a `mailto:` link's
  /// address (`\href{mailto:tsamak@clemson.edu}{tsamak}` an e-mail, 2309.10007; `link_target`) — review revfms2a2_8
  /// finding 1, FM S2a.2 r9; none for a contact no link printed
  pub target: Option<String>,
}

/// One resolved creator.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RCreator {
  /// the name without its marks (ruling 1's fuzzy label)
  pub name:         String,
  /// the marks shown on it, its requests
  pub requests:     Vec<String>,
  /// how the name was split out
  pub rule:         String,
  /// the reading's joins of runs a separator parts into this one name, as `rule` tags them ([`NameJoin`]; the separator
  /// check reads them, `net::merged_names`)
  pub joins:        Vec<NameJoin>,
  /// the ledger index of its block (or API call)
  pub source:       usize,
  /// the role of the creator call in its block that printed it, its Name's (`Event::role`: PoS's `\speaker`, `speaker`);
  /// none for a name the block itself printed (an author's)
  pub role:         Option<String>,
  /// that call, its place among its block's (`Event::creator_call`): the keyvals the emitter writes on it
  pub call:         Option<u16>,
  pub affiliations: Vec<Assoc>,
  pub contacts:     Vec<Assoc>,
  pub notes:        Vec<Assoc>,
}

/// A rule of the reading joining runs a separator parts into one name (`lines.rs`, `blocks.rs`), each its tag in the
/// creator's `rule`: a command's own gap (`\EES`), initials or a given name at a gap (`V.G.~ Kagramanova`), runs only a
/// layout parts (`\firstname{…} \surname{…}`), a name written surname first (`Eckart, A.`, 1501.02171), a qualifier
/// (`Robert W. Heath, Jr.`, 1004.2377), a remark under it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameJoin {
  Macro,
  Initials,
  Gap,
  Layout,
  Reversed,
  Qualifier,
  Remark,
}

impl NameJoin {
  const ALL: [NameJoin; 7] = [
    NameJoin::Macro,
    NameJoin::Initials,
    NameJoin::Gap,
    NameJoin::Layout,
    NameJoin::Reversed,
    NameJoin::Qualifier,
    NameJoin::Remark,
  ];

  /// The join a tag names ([`Self::tag`]): a reading read back from its dump (`ab_audit fmshadow`), which writes a creator's
  /// joins as their tags.
  pub fn of_tag(tag: &str) -> Option<NameJoin> {
    Self::ALL.into_iter().find(|join| join.tag() == tag)
  }

  /// Its tag in a creator's `rule`.
  pub fn tag(self) -> &'static str {
    match self {
      NameJoin::Macro => "name:macro-joined",
      NameJoin::Initials => "name:initials-joined",
      NameJoin::Gap => "name:gap-joined",
      NameJoin::Layout => "name:layout-joined",
      NameJoin::Reversed => "name:reversed",
      NameJoin::Qualifier => "name:qualifier",
      NameJoin::Remark => "name:remark",
    }
  }
}

impl RCreator {
  /// Record the reading's `join` of this name: its rule tag, once, and the join.
  pub(super) fn join(&mut self, join: NameJoin) {
    if !self.rule.contains(join.tag()) {
      self.rule.push(' ');
      self.rule.push_str(join.tag());
    }
    if !self.joins.contains(&join) {
      self.joins.push(join);
    }
  }
}

/// A document's resolution.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Resolved {
  pub creators:  Vec<RCreator>,
  /// what no creator owns: the author block's own (a trailing nameless creator in the XML, OD #159)
  pub block:     Vec<Assoc>,
  /// what the rulings dropped: a note whose content all became contacts or nothing in which prints (ruling 4), with
  /// its text — kept for the comparison, which reads today's note there as the ruling's change
  pub dropped:   Vec<Assoc>,
  /// per block (ledger index): the declared conventions it ran under (by name)
  pub policies:  Vec<(usize, &'static str)>,
  /// the lines of each entry that unmarked lines under it continued (`line:continued`), each as printed — what today's
  /// prints as one contact a line (the comparison's `part`, review revfms2a2_7 finding 5); `None` read from a dump made
  /// before FM S2a.2 r8, which holds none (review revfms2a2_8 finding 4: an r8 dump's may be empty, `continue_entry`
  /// skipping an item with no source — no old dump's "any part")
  pub continued: Option<Vec<Vec<String>>>,
  /// the affiliation entries whose printed e-mail addresses the reading took out (`Resolver::take_out_entry_addresses`,
  /// brief 95c), each as it was — for the writer, which gives an entry back where today's cannot cut it the same
  /// (`writer::give_back_addresses`); not in the dump
  pub taken:     Vec<Taken>,
  /// the e-mail calls printing several addresses alone that the reading wrote out address by address
  /// (`Resolver::answer_listed_emails`, brief 95c G3), each as today's holds it — for the writer's name re-part, which
  /// reads them whole (`writer::give_back_addresses`, brief 95e); not in the dump
  pub listed:    Vec<Listed>,
}

/// One affiliation entry the reading took printed addresses out of ([`Resolved::taken`]): its source, its text before
/// and after, and where each holder held it (a creator's affiliation slot; `None` the block's), with the association as
/// it was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Taken {
  pub source:   Option<(usize, Option<(u16, u16)>)>,
  pub original: String,
  pub stripped: String,
  pub holders:  Vec<(Option<usize>, usize, Assoc)>,
  /// the text the runs were read in (`original` with its lines), the bytes of it the take-out cuts (`address_cuts`), and
  /// each run taken out: its bytes, its addresses and the label word it took (`E-mail:`) — what the emitter cuts the
  /// entry's tokens at (`writer::address_cut`, OXIDIZED_DESIGN_DIVERGENCES #980 P1b)
  pub lined:    String,
  pub cuts:     Vec<(usize, usize)>,
  pub runs:     Vec<TakenRun>,
}

/// One address run a take-out took ([`Taken::runs`]): its bytes in [`Taken::lined`], its addresses, its label word.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TakenRun {
  pub bytes:     (usize, usize),
  pub addresses: Vec<String>,
  pub label:     Option<String>,
}

/// One e-mail call printing several addresses alone that the reading wrote out ([`Resolved::listed`]): the contact as
/// the call printed it — its text whole, the rule of the call's own placement (`api:annotate:last`, …) — and the
/// creators that placement gives it, today's holders of the one contact (2109.07363).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
  pub whole:  Assoc,
  pub owners: Vec<usize>,
}

/// The rule tags whose purpose is a change of today's output — an adopted ruling's (FIDELITY_CHARTER §4, 2026-10-21;
/// the S2a.2 brief's one contact per print event and emptied notes): a disagreement a rule holding one of them decided
/// is classified (`ab_audit fmshadow`), and the only difference the writer (FM S2b, `writer/`) writes.
///
/// - ruling 1, the marks shown on a name are its requests: `ruling1:request` (a labelled line's entry, or a note's
///   legend opening with its Mark event), `ruling1:fuzzy-label`, `ruling1:thenote` (authblk's printed number), `ruling1:parted` (a glued mark read as the marks it is: `12`
///   under entries `1` and `2`), `api:leading-mark` (an API entry's leading Mark
///   events: by construction only a real Mark, `resolver/api.rs` `api`), the `\ref`-mark keys (`api:ref-label`, `ref:`);
///   an API entry a symbol leads that reads as a legend, not an affiliation, a legend note of the names showing it
///   (`api:legend`: the role change, its own tag — review revfms2a2_8 finding 2, FM S2a.2 r9);
/// - ruling 2, a footnote-symbol line labels the line for its carriers: `ruling2:symbol-line`, `ruling2:uncarried`;
/// - the corresponding author (charter §4's e-mail owner order): `owner:corresponding` (a creator's own note, or the name a
///   document's note prints before the address, `api:note-name`);
/// - one contact per print event (`ruling3:one-per-print`): counted only where the XML holds the pieces joined into one
///   item (an `overlaps` rule), the change the ruling makes — the owner of each address is its other tag's;
/// - an emptied note dropped, a note's inner contacts its contacts, a note nothing in which prints dropped
///   (`\thanks{}`): `ruling4:`;
/// - a url whose text is one e-mail address is an e-mail (`role:url-as-email`: the S2a.2 r2 brief's item 8, adopted);
/// - a list with one address per author that every spelled address agrees with is in author order (`owner:order`:
///   charter §4's owner order, 2026-10-21, OD #805 — for every class, not only the sequenced lists);
/// - several addresses an entry's e-mail call prints (a braced `{a,b,c}@x` group in an `\institute`/`\affil` entry)
///   are written out, each the author's it spells, an unspelled one the block's (`ruling3:written-out`: the main
///   session's decision 3 of 2026-10-22, user rulings 2026-10-20 #4, #8, #10);
/// - an entry's e-mail call printing one address (an `\email` inside an `\institute` entry) placed by FIDELITY_CHARTER
///   §4's chain, not copied onto every carrier of the entry's label (`owner:inner-email`: the author it spells, the
///   entry's only carrier, the block — the ONE address left the corresponding author's, `place_emails_left` — the llncs
///   and aa gate study's G3, 1606.03774, 2204.03161, 0901.4618; 2408.14427);
/// - an address printed inside an affiliation entry taken out as an e-mail contact by FIDELITY_CHARTER §4's chain — the
///   author it spells, the entry's only carrier, a symbol line's carriers, the author at its place in one run
///   (`owner:entry-address`: brief 95c, OD #938, which names it a change tag; the writer cuts today's entry by the
///   same reader, `writer::plan`'s `CutAddresses`);
/// - a document's own note's address that spells an author (`\offprints{…, \email{…}}`, a title `\thanks`), that
///   author's — the chain's first step, which today's (routing it to the current author) does not take
///   (`api:note-spelled`: `Chain::DocumentNote`, OD #913's chain exactly; 1708.05224, astro-ph0509342, 0802.4021).
///
/// Elimination (`owner:elimination`, the one address left to the one name left) is NOT a change: today's output
/// applies it too (OXIDIZED_DESIGN_DIVERGENCES #52's e-mail lines, `owners_by_elimination`, under user ruling R10
/// 2026-10-20, 77p r10;
/// FIDELITY_CHARTER §4's owner order is silent on it), so its disagreements are the shadow's to explain.
pub const CHANGE_TAGS: &[&str] = &[
  "ruling1:request",
  "ruling1:fuzzy-label",
  "ruling1:thenote",
  "ruling1:parted",
  "role:url-as-email",
  "owner:order",
  "api:leading-mark",
  "api:legend",
  "api:ref-label",
  "ref:",
  "ruling2:symbol-line",
  "ruling2:uncarried",
  "owner:corresponding",
  "ruling3:one-per-print",
  "ruling3:written-out",
  "owner:inner-email",
  "owner:entry-address",
  "api:note-spelled",
  "ruling4:",
];

/// Whether a shadow rule holds one of [`CHANGE_TAGS`] (`ruling3:one-per-print` only on a joined item's `overlaps`
/// rule).
pub fn is_change_rule(rule: &str) -> bool {
  let joined = rule.starts_with("overlaps ");
  rule.split_whitespace().any(|tag| {
    CHANGE_TAGS.iter().any(|c| tag.starts_with(c)) && (joined || tag != "ruling3:one-per-print")
  })
}

/// One e-mail call a document's own note makes (`ltx:pubnote`, `Resolver::api`), placed by `answer_document_notes`.
#[derive(Debug, Clone)]
struct DocumentEmail {
  assoc:    Assoc,
  /// the text the note prints before it (the name it may give: "O. Kochukhov, ")
  printed:  String,
  /// whether the class's binding's `\email` made it, which prints it in the note (`Policy::email_in_place`, the
  /// call's `EmailMeaning::InPlace`), so the name before it designates its owner; else another binding's, which routes
  /// it to the current author
  in_place: bool,
  /// the current author at the note's call (the last creator read before it), whose own call a routed one is
  current:  Option<usize>,
}

/// A resolver input: an author block (its events as the lexer gave them) or an API call made apart from the blocks.
#[derive(Debug, Clone, PartialEq)]
pub enum Input {
  Block {
    index:   usize,
    binding: String,
    hook:    String,
    /// the keyvals the call hands each author (`annotations={a,b}`: requests)
    keyvals: Option<String>,
    /// the policy's state counter at the call (`ledger::AuthorBlock::counter`)
    counter: Option<i64>,
    /// whether the call replaces the authors queued before it (`\lx@clear@frontmatter{ltx:creator}`: the kernel
    /// `\author` given twice), and the flushes before it (the queue it is in)
    replace: bool,
    /// whether a later creator clear in its queue removed its authors (`ledger::AuthorBlock::cleared`: llncs's
    /// `\author` given twice)
    cleared: bool,
    epoch:   usize,
    events:  Vec<Event>,
    /// per creator call of its own expansion (`Event::creator_call`), the labels its `annotations=` give the creator its
    /// Name makes, as Perl's `\lx@add@creator[annotations={ed}]` gives them (`field_notes::FieldCreator`). Not in a dump
    calls:   Vec<Vec<String>>,
  },
  Api {
    index:  usize,
    tag:    String,
    role:   String,
    attrs:  String,
    events: Vec<Event>,
  },
  /// a footnote's text the document made (`\footnotetext[<n>]{…}`, `ledger::NoteText`): its printed mark and its
  /// body's events
  NoteText {
    index:  usize,
    /// the frontmatter flushes before it (`ledger::NoteText::epoch`)
    epoch:  usize,
    mark:   String,
    events: Vec<Event>,
  },
}

mod api;
mod blocks;
mod labels;
mod lines;
mod net;
// (the old reader's owner logic, moved unchanged by the plan's row "Knowledge in, driver out, one pass"; re-exported by `observer/mod.rs`)
pub(in crate::base_utilities) mod owners;
mod placed;
mod post;
mod words;

pub(in crate::base_utilities::frontmatter::observer) use api::split_pieces;
use blocks::Chain;
pub(in crate::base_utilities::frontmatter::observer) use labels::label_matches;
// (and the flush's Kind step for an entry's address line, `insert::kind_address_lines`: OD #966)
pub(in crate::base_utilities::frontmatter) use labels::{AddressRun, address_runs};
// (the address reader of an entry's printed text, the reading's and the writer's cut of today's item alike: brief 95c)
pub(in crate::base_utilities::frontmatter::observer) use labels::{
  address_cuts, listed_addresses, strip_address_runs,
};
// (named, as the prelude has its own of these names: the resolver's shadow them)
pub use labels::{address_take_out, addresses_in, mark_labels, only_addresses};
use labels::{text_tokens, *};
use lines::*;
pub(in crate::base_utilities::frontmatter::observer) use lines::{
  NoPerson, may_name_a_person, name_before_remark, names_no_person, reads_as_names,
};
// (the reading's word check, its block half and what the emitter's call half reads: OD #980)
pub(in crate::base_utilities::frontmatter::observer) use net::{merged_names, uncoordinated_marks};
// (the content invariant on the placed output, under the emitter's `=2`: OD #980)
pub(in crate::base_utilities::frontmatter::observer) use placed::{
  Placed, PlacedItem, PlacedKind, content_check,
};
pub(in crate::base_utilities::frontmatter::observer) use words::{
  BlockCheck, LostInk, Written, check_blocks, printed_email_label, printed_words,
  reads_as_email_label,
};

/// A block's events as the resolver reads them (`Resolver::block`): an ORCID link's name, an IEEE ordinal, a mark
/// printed as text after a name and a glued run of shown symbols each read as the names line reads it.
pub(in crate::base_utilities::frontmatter::observer) fn read_events(
  events: &[Event],
) -> Vec<Event> {
  glue_shown_runs(&split_text_marks(&join_ordinals(&link_names(events))))
}

/// The labels a Mark event requests of the name it is set on, as a names line reads them (`names_line_events`): its
/// labels, a number range spelled out.
pub(in crate::base_utilities::frontmatter::observer) fn mark_requests(
  event: &Event,
) -> Vec<String> {
  spelled_ranges(event_labels(event))
}

/// The entry a line ended with, which an unmarked line under it continues: a labelled entry (its place in
/// `Resolver::pending`).
enum Continued {
  Pending(usize),
  /// an unmarked line's affiliation under a marked names row: each owner's (creator, its place among the affiliations)
  Joined(Vec<(usize, usize)>),
}

/// The resolution of a block's names line: its creators (indices into `creators`) and what it attached.
struct NamesLine {
  creators: Vec<usize>,
}

/// The state of a document's resolution.
struct Resolver {
  out:                Resolved,
  /// per creator: its block (input index) and the roles its contacts hold
  roles:              Vec<Vec<String>>,
  /// the labels of `labelseq=<role>` entries so far, per role
  sequence:           Vec<(String, usize)>,
  /// labelled entries of the blocks not yet answered (label, entry), resolved at the end against every request
  pending:            Vec<(Vec<String>, bool, Assoc)>,
  /// per creator: its group (the block's input index, or the policy's call group)
  group:              Vec<usize>,
  /// per group, its creators, in order: what a call's group reads ([`Resolver::last_group`])
  group_members:      HashMap<usize, Vec<usize>>,
  /// per creator: the block (input index) and `\and` column of the names line it was read from, if a block's
  column:             Vec<Option<(usize, usize)>>,
  /// the block line being read (input index, column): the column a creator made now is read from
  line_column:        Option<(usize, usize)>,
  /// the column (input index, column) of each block line's events, keyed by (input index, an event's span start):
  /// where a labelled entry stands (`answer_pending`'s per-column labels)
  entry_column:       HashMap<(usize, u16), usize>,
  /// the addresses every line of the block being read prints (`place_line_contacts`' author-order list: one address
  /// per author, every spelled address of the block agreeing)
  block_addresses:    Vec<String>,
  /// the addresses of a class's list printed apart from the names (`EmailOwner::SpelledElseOrder`), each with its
  /// group's creators
  /// (each with its call's place in the list: `_sequence=<n>`, else the calls' count)
  sequenced:          Vec<(String, Assoc, Vec<usize>, usize)>,
  /// the calls of the list so far
  sequence_calls:     usize,
  /// the last `annotate=run` contact's role, the creators then, the run it reached, and the run's extent (the run and
  /// the authors its walk passed over as answered, which a legend continuing it reaches: `record_run`)
  last_run:           Option<(String, usize, Vec<usize>, Vec<usize>)>,
  /// per role, the creators made when the last contact of it that bounds a run was read — a block's, labelled or not;
  /// a call's a printed mark labels: the names before it are no run's of a later unlabelled one (`record_run`,
  /// `new_run`)
  run_from:           HashMap<String, usize>,
  /// per creator: its contacts in the order they were attached (an affiliation, by its place among the
  /// affiliations; else a contact's), for [`Resolver::distribute_upfront`]
  order:              Vec<Vec<(bool, usize)>>,
  /// authblk's notes (`Policy::thenote`): the state its `\author` and `\affil` keep
  thenote:            TheNote,
  /// the requests authblk's printed `\AB@thenote` made (creator, label): what ruling 1 reads off the PDF's marks
  thenote_requests:   Vec<(usize, String)>,
  /// the labels the title's marks request (aastex's `\title{…\altaffilmark{1}}`): an entry they alone ask for is the
  /// title's note
  title_requests:     Vec<String>,
  /// the footnote marks the title shows (`\title{…\footnotemark[1]}`): a footnote text only they show is the title's
  /// note (`answer_note_texts`)
  title_fn_marks:     Vec<String>,
  /// per creator of a block: that block's queue (its `epoch`); the creators a later block replaced
  epoch_of:           Vec<Option<usize>>,
  replaced:           Vec<usize>,
  /// the creators each queued author entry made (`start..end`), in queue order: a block's or a creator call's, under
  /// `LATEXML_FM_EMIT=2` (`whole_entries`), where each call queues one entry, a block whole (`author_entries`)
  entries:            Vec<(usize, usize)>,
  whole_entries:      bool,
  /// the block being read's creator calls' labels (`Input::Block::calls`)
  call_labels:        Vec<Vec<String>>,
  /// the emitter's switch is on (`LATEXML_FM_EMIT=2`, as `whole_entries`): the reading's rules only its output may move yet,
  /// the switch-off output as it was until the flip (brief LO1: a line's other text by its row, a line's addresses by the
  /// address reader; brief LO2: a row line's address its holder's, an entry's trailing note its holders')
  emitting:           bool,
  /// the flush's one contact declaring a run and sharing (`_share=authors,annotate=run`: cas's unlabelled
  /// `\affiliation`), every author's as `shared_row::share_row_contacts` shares it — with two or more, each its run's
  /// (2201.00009, 2506.17280, 2106.02306; 2202.06096). Under `emitting` only
  sole_shared_run:    bool,
  /// a note's links, and the notes they may empty, placed once every name is read (`answer_note_links`)
  note_links:         Vec<NoteLink>,
  /// the e-mail calls a document's own note makes (`ltx:pubnote`: A&A's `\offprints{B. Baker, \email{…}}`), placed
  /// once every name is read (`answer_document_notes`)
  document_emails:    Vec<DocumentEmail>,
  /// the e-mail calls printing several addresses alone (brief 95c G3): each call's addresses, its contact, its own
  /// authors and its rule — placed once every name is read (`answer_listed_emails`)
  listed_emails:      Vec<(Vec<String>, Assoc, Vec<usize>, String)>,
  /// the e-mails inside an unlabelled affiliation-kind entry an API call made, with the entry's carriers — placed with
  /// the labelled entries' (`answer_inner_emails`), the entries printing the same addresses together (OD #956)
  entry_emails:       Vec<(Vec<usize>, Assoc, bool)>,
  /// the marks a name's notes show on it (creator, label: `Ann\footnote[1]{…}` shows `∗`), what an entry led by the
  /// same `\footnotemark` answers (`answer_pending`; 1607.07280)
  note_marks:         Vec<(usize, String)>,
  /// whether the block being read prints its notes' marks (`Policy::unmarked_notes`)
  notes_marked:       bool,
  /// the legends of the contact commands' notes (`Event::note_mark`, a call's `_mark` key), in call order: each mark,
  /// whether it is the footnote counter's number, and whether its text names the corresponding author — what the
  /// authors writing that mark on their names carry (`Resolver::corresponding`; with the emitter's switch on alone)
  note_legends:       Vec<(String, bool, bool)>,
  /// the requests a glued mark was parted into (creator, request), `part_glued_labels`
  parted:             Vec<(usize, String)>,
  /// the pending entries (their place in `pending`) whose label was a compound of distinct footnote symbols parted
  /// (`$^{\dagger\ddagger}$ Project lead`): the names carrying all of them (`part_glued_labels`, `answer_pending`)
  compound:           Vec<usize>,
  /// while an API call's marked entries are resolved one by one (`mark_led_pieces`)
  splitting:          bool,
  /// the last API call when it was a store's line a mark leads (its ledger index, its `_store=`, role and labels): the
  /// store's next line, unmarked, continues it (`Resolver::api`, `line:continued`)
  led_line:           Option<(usize, String, String, Vec<String>)>,
  /// the pending entries ruling 1 labels by a leading mark that today's attaches where its `annotate=` puts it, with
  /// those authors (`distribute_upfront` reads today's pile with them)
  piled:              Vec<(usize, Vec<usize>)>,
  /// the empty entries today's attaches (their ledger index, role and authors), in its pile for `distribute_upfront`
  empty_piled:        Vec<(usize, String, Vec<usize>)>,
  /// the pending entries labelled by a leading mark (their place in `pending`) with the authors their `annotate=`
  /// reaches, where today's attaches them when no name shows a mark of their kind (`answer_pending`)
  annotated:          Vec<(usize, Vec<usize>)>,
  /// the pending entries whose marked line the binding attached by its group rule (`_grouped=true` at the call)
  grouped:            Vec<usize>,
  /// the requests a `\footnotemark` made (creator, label): a footnote's mark, answered by a note, never by a labelled
  /// affiliation (`Kui Wu\inst{3\footnotemark[1]}` under `\institute{A \and B …}`, 2404.09857)
  fn_requests:        Vec<(usize, String)>,
  /// the requests the class's own request command made (creator, label): a Mark of llncs's and aa's `\inst` (role
  /// `inst`), a binding's `\lx@request@frontmatter@annotation` (`request`), an `annotations=` list — not a mark printed
  /// by hand (`$^{1}$`, `\textsuperscript{1}`, a paper's mark macro), which [`Resolver::by_hand`] reads off the rest
  /// (`answer_pending`: a `_share=authors` entry only the class's `\inst{n}` picks)
  class_requests:     Vec<(usize, String)>,
  /// the pending entries (their place in `pending`) a `_share=authors` call made (llncs's one `\institute`, no `\and`
  /// in it: every author's unless the class's `\inst{n}` picks it), whether the entry holds a mark of its own, and
  /// whether it is a piece its institute's marks split it into (`_markpiece=true`: the numbers shown by hand label the
  /// other pieces, not it — [`Resolver::share_unpicked`])
  shared:             Vec<(usize, bool, bool)>,
  /// the pending entries (their place in `pending`) a call labelled by a key its class never prints
  /// (`Policy::keyed_labels`, elsarticle's `\address[*]{…}`): no footnote's mark answers them (`answer_pending`)
  keyed:              Vec<usize>,
  /// the pending entries of a numbered call (`labelseq=`) whose content holds marks of its own, which no split read
  /// (`answer_pending`: its number is no reading of those marks, so an owner found by a hand mark is not ruling 1's)
  marked_entries:     Vec<usize>,
  /// the footnote texts the document made (ledger index, flushes before it, printed mark, text), each read where a
  /// name shows its mark (`answer_note_texts`)
  note_texts:         Vec<(usize, usize, String, String)>,
  /// the entries unmarked lines continued (their source) with their lines (`continue_entry`, `Resolved::continued`)
  continued:          Vec<(Source, Vec<String>)>,
  /// the affiliation texts of API calls that print line breaks (their ledger index, the text, the text with its lines:
  /// [`events_lined_text`]), where `take_out_entry_addresses` reads a line's end (brief 95c r2 B2)
  lined:              Vec<(usize, String, String)>,
  /// the collaborations an unframed line names that an address line under it may still make an institution's (the
  /// creator, its contact: [`Resolver::attach`], `collaboration_line_before`)
  unframed:           Vec<(usize, usize)>,
  /// the first creator no collaboration covers yet, of the document (a call of its own naming one covers from here:
  /// [`Resolver::names_line_events`])
  collaboration_from: usize,
}

/// A note's link placed once every name of the document is read (`Resolver::answer_note_links`; main-session decisions
/// of int66, review revfms2a2_6 findings 2, 3, FM S2a.2 r7): an e-mail or a url of a note, with the creator the note is
/// on and whether that creator takes it when no name spells it (the note's only link, its mark shown on the name —
/// decision 2); or a note whose links are all its text holds besides punctuation, kept or dropped (ruling 4) once they
/// are placed — at the note, a name after it in the block is not read yet (`\author{Ann\thanks{ann@x.org, bob@x.org}
/// \and Bob}`: the note was kept beside both addresses as contacts).
enum NoteLink {
  Link {
    owner:    Option<usize>,
    /// (its target, `Assoc::target`: a `\url`'s text, an `\href`'s first argument, a `mailto:` link's address)
    assoc:    Assoc,
    fallback: bool,
  },
  Note {
    index: usize,
    note:  Box<Event>,
    owner: Option<usize>,
    rule:  String,
    /// (each with whether it is its owner's when it spells no one: the note's only link, an e-mail call routing it)
    links: Vec<(Assoc, bool)>,
  },
}

/// authblk's `\AB@thenote` state (authblk.sty:114-118, 148-151), read in call order: the `affil` counter, stepped at
/// the first `\author` after an `\affil` (`\ifnewaffil`); whether any label was given (which sets `Maxaffil` to 0,
/// so the marks show); `\c@Maxaffil` at the last block's call; each unlabelled author's number; each unlabelled
/// `\affil`'s number, with the rule tag its run placement carries; and whether any `\affil` was given, without which
/// authblk prints `\AB@author`, the names with no note (authblk.sty:168 `\ifx\AB@affillist\AB@empty\AB@author`).
#[derive(Default)]
struct TheNote {
  counter:  usize,
  newaffil: bool,
  affil:    bool,
  labelled: bool,
  maxaffil: Option<i64>,
  run_of:   Vec<(usize, usize)>,
  affils:   Vec<(usize, String)>,
}

impl Resolver {
  fn new() -> Self {
    Resolver {
      out:                Resolved::default(),
      roles:              Vec::new(),
      sequence:           Vec::new(),
      pending:            Vec::new(),
      group:              Vec::new(),
      group_members:      HashMap::default(),
      column:             Vec::new(),
      line_column:        None,
      entry_column:       HashMap::default(),
      block_addresses:    Vec::new(),
      sequenced:          Vec::new(),
      sequence_calls:     0,
      last_run:           None,
      run_from:           HashMap::default(),
      order:              Vec::new(),
      thenote:            TheNote {
        newaffil: true,
        ..TheNote::default()
      },
      thenote_requests:   Vec::new(),
      title_requests:     Vec::new(),
      title_fn_marks:     Vec::new(),
      epoch_of:           Vec::new(),
      replaced:           Vec::new(),
      entries:            Vec::new(),
      whole_entries:      fm_emit_mode() == 2,
      call_labels:        Vec::new(),
      emitting:           fm_emit_mode() == 2,
      sole_shared_run:    false,
      note_links:         Vec::new(),
      document_emails:    Vec::new(),
      listed_emails:      Vec::new(),
      entry_emails:       Vec::new(),
      note_marks:         Vec::new(),
      notes_marked:       true,
      note_legends:       Vec::new(),
      parted:             Vec::new(),
      compound:           Vec::new(),
      splitting:          false,
      led_line:           None,
      piled:              Vec::new(),
      empty_piled:        Vec::new(),
      annotated:          Vec::new(),
      grouped:            Vec::new(),
      fn_requests:        Vec::new(),
      class_requests:     Vec::new(),
      shared:             Vec::new(),
      keyed:              Vec::new(),
      marked_entries:     Vec::new(),
      note_texts:         Vec::new(),
      continued:          Vec::new(),
      lined:              Vec::new(),
      unframed:           Vec::new(),
      collaboration_from: 0,
    }
  }

  fn new_creator(
    &mut self,
    name: String,
    rule: &str,
    source: usize,
    requests: Vec<String>,
    group: usize,
  ) -> usize {
    // (a resolved name is its printed text, its spaces collapsed: a visible glyph printing no letter or digit stays — a
    // printed U+2010, the `·` between the parts of a name —, as the join paths keep it and today's XML prints it,
    // 1706.00234's "TIÊ ´N-SO . N PHẠM"; a zero-width character is no ink already, the lexer's `is_zero_width` — main-
    // session decision on review revfms2a2_5 finding 2, superseding r5's piece filter of revfms2a2_4 finding 6)
    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    self.out.creators.push(RCreator {
      name,
      requests,
      rule: rule.to_string(),
      source,
      ..RCreator::default()
    });
    self.roles.push(Vec::new());
    self.order.push(Vec::new());
    self.group.push(group);
    self
      .group_members
      .entry(group)
      .or_default()
      .push(self.out.creators.len() - 1);
    self.column.push(self.line_column);
    self.out.creators.len() - 1
  }

  fn attach(&mut self, owner: Option<usize>, mut assoc: Assoc, note: bool) {
    // (an unlabelled line of the block that names a collaboration is its owners' collaboration, as today's reader makes
    // it — `(XYZ Collaboration)` under the names, 2410.11607; `collaboration_line_framing`, OXIDIZED_DESIGN_DIVERGENCES
    // #946 —, not a class's affiliation command holding one: revtex's `\affiliation{(WA98 Collaboration)}`, 0708.2630;
    // a team's only framed by more than "the", `The Robotics Group` an institution's)
    let line_affiliation = !note
      && owner.is_some()
      && assoc.role == "affiliation"
      && assoc.label.is_none()
      && assoc.rule.starts_with("line:");
    let framing = line_affiliation
      .then(|| collaboration_line_framing(&assoc.text))
      .flatten();
    if framing.is_some() {
      assoc.role = "collaboration".into();
    }
    match owner {
      Some(at) => {
        // (an address line under an unframed one naming a collaboration makes that one an institution's, its address
        // going on: `\textit{WIDE Consortium}\\ \.Zabbar\\ Malta`, 2501.13770 — as today's reader,
        // `collaboration_line_before`)
        if line_affiliation && framing.is_none() {
          self.institution_after_all(at);
        }
        let creator = &mut self.out.creators[at];
        if note {
          creator.notes.push(assoc);
        } else {
          self.roles[at].push(assoc.role.clone());
          if assoc.role == "affiliation" || assoc.role == "address" {
            self.order[at].push((true, creator.affiliations.len()));
            creator.affiliations.push(assoc);
          } else {
            if framing == Some(Framing::Unframed) {
              self.unframed.push((at, creator.contacts.len()));
            }
            self.order[at].push((false, creator.contacts.len()));
            creator.contacts.push(assoc);
          }
        }
      },
      None => self.out.block.push(assoc),
    }
  }

  /// The unframed collaboration just attached to creator `at`, an institution's after all: its line an affiliation
  /// again, the address line under it going on (`Resolver::attach`).
  fn institution_after_all(&mut self, at: usize) {
    let Some(&(false, contact)) = self.order[at].last() else {
      return;
    };
    let Some(slot) = self.unframed.iter().position(|u| *u == (at, contact)) else {
      return;
    };
    self.unframed.remove(slot);
    let creator = &mut self.out.creators[at];
    if contact + 1 != creator.contacts.len() {
      return;
    }
    let mut assoc = creator.contacts.remove(contact);
    assoc.role = "affiliation".into();
    if let Some(order) = self.order[at].last_mut() {
      *order = (true, creator.affiliations.len());
    }
    creator.affiliations.push(assoc);
    if let Some(role) = self.roles[at]
      .iter_mut()
      .rev()
      .find(|r| *r == "collaboration")
    {
      *role = "affiliation".into();
    }
  }

  /// The creators today's output holds: those of no replaced `\author` call (`resolve` deletes them at the end; the
  /// post-passes that count the authors, or pick one, read these — review revfms2a2_5 finding 1; 2510.26354, 1611.08655).
  /// The creators of the last creator's group, in order (none before the first creator).
  fn last_group(&self) -> Vec<usize> {
    self
      .group
      .last()
      .and_then(|last| self.group_members.get(last))
      .cloned()
      .unwrap_or_default()
  }

  fn live(&self) -> Vec<usize> {
    (0..self.out.creators.len())
      .filter(|c| !self.replaced.contains(c))
      .collect()
  }

  /// The queue's author entries, each the live creators it made, in queue order, as an `author:N` label names the N-th
  /// (`queued_author_marks`' positions at the call, the N-th digested author's label): under `LATEXML_FM_EMIT=2` one a call, a
  /// block's whole entry its names (`whole_block_call`), an entry a later call dequeued none; else one a creator, as the old
  /// reader queued them (the live ones, `label_owners`).
  fn author_entries(&self) -> Vec<Vec<usize>> {
    if !self.whole_entries {
      return self.live().into_iter().map(|c| vec![c]).collect();
    }
    self
      .entries
      .iter()
      .filter(|&&(start, end)| start == end || (start..end).any(|c| !self.replaced.contains(&c)))
      .map(|&(start, end)| {
        (start..end)
          .filter(|c| !self.replaced.contains(c))
          .collect()
      })
      .collect()
  }

  /// The creators an `author:N` label names (`author_entries`): the N-th entry's; an address among several, the one of them it
  /// spells, else none (the block's), as the call's own reading of the queue placed it by the names it spelled
  /// (`email_block_calls`, `placed_contact_owner`: IEEEtran's e-mail line under one `\IEEEauthorblockN` of four names,
  /// 2010.07442, 2003.03692, 2510.03288, 2607.02294).
  fn entry_owners(&self, n: &str, assoc: &Assoc) -> Vec<usize> {
    let Some(entry) = n
      .trim()
      .parse::<usize>()
      .ok()
      .and_then(|n| n.checked_sub(1))
      .and_then(|at| self.author_entries().into_iter().nth(at))
    else {
      return Vec::new();
    };
    match entry.as_slice() {
      [_, _, ..] if assoc.role == "email" => {
        self.spelled(&assoc.text, &entry).into_iter().collect()
      },
      _ => entry,
    }
  }

  /// The one candidate an address spells (`spelled_owners`), if one — none for a list of local parts the address
  /// reader could not part (`(npalmer|pwg)@dcs.warwick.ac.uk`, cs0607047: several authors' address, the block's by user
  /// ruling R10, as today's `spelled_owner` reads it — `lists_local_parts`). A name is read as an address spells it
  /// (`NameKeys::of`): by its base letters, the digested name's accents dropped (`Ștefan Ciobâcă` "stefan", "ciobaca",
  /// 2001.09649's `stefan.ciobaca@`), and by its transliteration (`Schäffler` "schaeffler", html_feedback #7233), as the
  /// old reader's source reading reads the accent commands.
  fn spelled(&self, address: &str, candidates: &[usize]) -> Option<usize> {
    let names: Vec<&str> = candidates
      .iter()
      .map(|&c| self.out.creators[c].name.as_str())
      .collect();
    spelled_name(&names, address).map(|one| candidates[one])
  }

  /// The one candidate of a class's address list printed apart from the names (`address`, the one left over) marked as the
  /// corresponding author, else the block: the switch-off step's reading (`place::marked_corresponding_author`, OD #809,
  /// user ruling 2026-10-21) on the reading's facts. A candidate is marked by a note of theirs naming them so — a
  /// `\thanks`, or a contact command's note, jheppub's `\note{Corresponding author.}` in their `\author` (1901.08773's
  /// A. Kopylov, 2510.11030's J.-H. Ha) — or by a mark they request whose legend says so: a labelled affiliation's
  /// (`\affiliation[*]{Corresponding author}`, 1304.1216), a contact command's note's own mark (`\note[*]{…}`) or the
  /// footnote counter's number it prints under, unless an affiliation is labelled with that number too, which the number
  /// then requests (`note_legends`; 2005.05695). Exactly one marked, else the block (1802.03677's "Corresponding author."
  /// note, whose number 1 two authors carry; 2606.26596's `\note[*]{Corresponding authors.}` under two `*`s); none marked,
  /// the one candidate carrying a `*` no legend explains where the address says it is the corresponding author's
  /// (2308.06532's `\emailAdd{zyxu@gzu.edu.cn(Corresponding author)}`).
  fn listed_corresponding(
    &self,
    candidates: &[usize],
    address: &str,
  ) -> (Option<usize>, &'static str) {
    let creators = &self.out.creators;
    let says = |a: &Assoc| names_correspondence(&a.text);
    let mut legends: Vec<(String, bool)> = Vec::new();
    let affiliations = creators.iter().flat_map(|c| &c.affiliations).chain(
      self
        .out
        .block
        .iter()
        .filter(|a| matches!(a.role.as_str(), "affiliation" | "address")),
    );
    for a in affiliations {
      for label in a.label.as_deref().map(mark_labels).unwrap_or_default() {
        legends.push((label, says(a)));
      }
    }
    let affiliation_legends = legends.len();
    for (mark, numbered, says) in &self.note_legends {
      if *numbered
        && legends[..affiliation_legends]
          .iter()
          .any(|(label, _)| label == mark)
      {
        continue;
      }
      legends.extend(mark_labels(mark).into_iter().map(|label| (label, *says)));
    }
    let marked: Vec<usize> = candidates
      .iter()
      .copied()
      .filter(|&c| {
        let mut own = creators[c]
          .notes
          .iter()
          .chain(creators[c].contacts.iter().filter(|a| a.role == "note"));
        own.any(says)
          || creators[c].requests.iter().any(|request| {
            legends
              .iter()
              .any(|(label, says)| *says && label == request)
          })
      })
      .collect();
    match marked.as_slice() {
      [one] if candidates.len() > 1 => return (Some(*one), "owner:corresponding"),
      [] if candidates.len() > 1 && names_correspondence(address) => {},
      _ => return (None, "owner:block"),
    }
    let star = mark_labels("*");
    let starred: Vec<usize> = candidates
      .iter()
      .copied()
      .filter(|&c| {
        creators[c].requests.iter().any(|request| {
          star.contains(request) && !legends.iter().any(|(label, _)| label == request)
        })
      })
      .collect();
    match starred.as_slice() {
      [one] => (Some(*one), "owner:corresponding"),
      _ => (None, "owner:block"),
    }
  }

  /// The legends of the contact commands' notes an input holds (`note_legends`): an author block's notes, each its own
  /// mark or else the footnote counter's next number (jheppub.sty:84-89, the counter stepped by an unmarked `\note`
  /// alone), and a note given apart from the names, its own mark only — as the switch-off step reads the names' notes
  /// and the queue's (`email_block_calls::author_notes`, `place::marked_corresponding_author`).
  fn read_note_legends(&mut self, events: &[Event], apart: Option<&str>) {
    if let Some(mark) = apart {
      if !mark.is_empty() {
        let says = names_correspondence(&events_text(&events.iter().collect::<Vec<_>>()));
        self.note_legends.push((mark.to_string(), false, says));
      }
      return;
    }
    let notes = events
      .iter()
      .filter(|e| e.kind == EventKind::Contact && e.role.as_deref() == Some("note"));
    for e in notes {
      let (mark, numbered) = match e.note_mark.as_deref() {
        Some("") => {
          let numbered = self
            .note_legends
            .iter()
            .filter(|(_, numbered, _)| *numbered);
          ((numbered.count() + 1).to_string(), true)
        },
        Some(mark) => (mark.to_string(), false),
        None => continue,
      };
      let says = names_correspondence(&e.text);
      self.note_legends.push((mark, numbered, says));
    }
  }

  /// The one candidate whose note names them the corresponding author, else the block.
  fn corresponding(&self, candidates: &[usize]) -> (Option<usize>, &'static str) {
    let corresponding: Vec<usize> = candidates
      .iter()
      .copied()
      .filter(|&c| {
        self.out.creators[c]
          .notes
          .iter()
          .any(|n| names_correspondence(&n.text))
      })
      .collect();
    match corresponding.as_slice() {
      [one] if candidates.len() > 1 => (Some(*one), "owner:corresponding"),
      _ => (None, "owner:block"),
    }
  }
}

/// The rule tag of an entry's e-mail whose two-letter local part is a word the entry prints, a mailbox spelling no one
/// (`email_owners::is_entry_word_mailbox`; review rev100b S3).
const ENTRY_MAILBOX: &str = " api:entry-mailbox";

/// The one of `names` an address spells (`spelled_owners`), if one — none for a list of local parts the address reader
/// could not part (`lists_local_parts`); each name read as an address spells it, by its base letters ([`base_letters`])
/// and its transliteration ([`transliterated`], both `NameKeys::of`; [`Resolver::spelled`]; the writer's check of an address today's holds on the creator it spells, `writer::plan`).
pub(in crate::base_utilities::frontmatter::observer) fn spelled_name(
  names: &[&str],
  address: &str,
) -> Option<usize> {
  if lists_local_parts(address) {
    return None;
  }
  let keys: Vec<NameKeys> = names
    .iter()
    .map(|name| NameKeys::of(&text_tokens(name)))
    .collect();
  match spelled_owners(&keys, address).as_slice() {
    [one] => Some(*one),
    _ => None,
  }
}

/// `text` with its letters' accents dropped (canonical decomposition, the combining marks aside), and the letters
/// that decompose to none their base letter (`ł` l, `ø` o, `đ` d, `ı` i; `æ` a, `œ` o, `ß` s, as the author block's
/// reading of `\ae`, `\oe`, `{\ss}` gives them, `author_block::letter_command`) — as an address spells a name (the
/// reading's and `insert::respell_emails`'), one fold of it; the transliteration another ([`transliterated`]).
pub(in crate::base_utilities) fn base_letters(text: &str) -> String {
  use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};
  text
    .nfd()
    .filter(|c| !is_combining_mark(*c))
    .map(|c| match c {
      'ł' => 'l',
      'Ł' => 'L',
      'ø' => 'o',
      'Ø' => 'O',
      'đ' => 'd',
      'Đ' => 'D',
      'ı' => 'i',
      'æ' => 'a',
      'Æ' => 'A',
      'œ' => 'o',
      'Œ' => 'O',
      'ß' => 's',
      c => c,
    })
    .collect()
}

/// The conventional transliteration of a letter, where it has one: the German umlauts and sharp s (`ä` ae, `ö` oe, `ü`
/// ue, `ß` ss) and the Nordic letters (`å` aa, `ø` oe, `æ` ae), `œ` oe; upper case likewise. The one letter table every
/// spelling of a name by an address reads ([`transliterated`], `NameKeys::of`; `mo.schaeffler@` for Moritz Schäffler,
/// html_feedback #7233; OXIDIZED_DESIGN_DIVERGENCES #970).
pub(in crate::base_utilities) fn transliteration(c: char) -> Option<&'static str> {
  Some(match c {
    'ä' | 'æ' => "ae",
    'ö' | 'ø' | 'œ' => "oe",
    'ü' => "ue",
    'ß' => "ss",
    'å' => "aa",
    'Ä' | 'Æ' => "Ae",
    'Ö' | 'Ø' | 'Œ' => "Oe",
    'Ü' => "Ue",
    'ẞ' => "SS",
    'Å' => "Aa",
    _ => return None,
  })
}

/// `text` with each letter that has a conventional transliteration written so ([`transliteration`]), its other
/// characters as they are (composed first: `a` and a combining diaeresis are `ä`).
pub(in crate::base_utilities) fn transliterated(text: &str) -> String {
  use unicode_normalization::UnicodeNormalization;
  let mut out = String::with_capacity(text.len());
  for c in text.nfc() {
    match transliteration(c) {
      Some(spelled) => out.push_str(spelled),
      None => out.push(c),
    }
  }
  out
}

/// Resolve a document's ledger (its inputs in call order), each block under the conventions its binding declared
/// (`policy::policy_for`).
pub fn resolve(inputs: &[Input]) -> Resolved { resolve_with(inputs, &policy_for) }

/// Resolve inputs, each block under the conventions `conventions` gives for its binding and hook (`ab_audit fmshadow
/// --re-resolve`, offline: the declarations it holds, as `policy::conventions_among` reads them).
pub fn resolve_with(
  inputs: &[Input],
  conventions: &dyn Fn(&str, &str) -> &'static Policy,
) -> Resolved {
  let mut resolver = Resolver::new();
  let mut policy: &'static Policy = &policy::KERNEL;
  // (counted across the whole queue, as `share_row_contacts` counts them)
  resolver.sole_shared_run = resolver.emitting
    && inputs
      .iter()
      .filter(|input| {
        matches!(input, Input::Api { tag, attrs, .. } if tag == "ltx:contact" && shares_a_run(attrs))
      })
      .count()
      == 1;
  for input in inputs {
    match input {
      Input::Block {
        index,
        binding,
        hook,
        keyvals,
        counter,
        replace,
        cleared,
        epoch,
        events,
        calls,
      } => {
        policy = conventions(binding, hook);
        if resolver.emitting {
          resolver.read_note_legends(events, None);
        }
        resolver.call_labels.clone_from(calls);
        resolver.line_column = None;
        resolver.block_addresses.clear();
        // (a call replacing the authors before it in its queue: `\author{A, B}` then `\author{A\thanks{…}, B}`,
        // 2502.11323, 2509.11381 — the later one's are the document's)
        if *replace {
          for (c, e) in resolver.epoch_of.iter().enumerate() {
            if *e == Some(*epoch) {
              resolver.replaced.push(c);
            }
          }
        }
        let start = resolver.out.creators.len();
        // (a per-author class binding's call, which splits its names only where they read as names: the
        // `\lx@add@authors@append` hook, `split_author_line_cautious`)
        let cautious = hook == "\\lx@add@authors@append";
        resolver.block(*index, policy, cautious, keyvals.as_deref(), events);
        resolver.entries.push((start, resolver.out.creators.len()));
        // (a block whose authors a later clear removed, as a replacing call's: `ledger::record_clear`)
        if *cleared {
          resolver.replaced.extend(start..resolver.out.creators.len());
        }
        resolver.epoch_of.resize(start, None);
        resolver
          .epoch_of
          .resize(resolver.out.creators.len(), Some(*epoch));
        if policy.thenote {
          if counter.is_some() {
            resolver.thenote.maxaffil = *counter;
          }
          let labelled = keyvals
            .as_deref()
            .and_then(|kv| attr_value(kv, "annotations"))
            .is_some_and(|l| !l.is_empty());
          resolver.thenote_author(start, labelled);
        }
      },
      Input::Api {
        index,
        tag,
        role,
        attrs,
        events,
      } => {
        let start = resolver.out.creators.len();
        if resolver.emitting && tag == "ltx:contact" && role == "note" {
          resolver.read_note_legends(events, Some(attr_value(attrs, "_mark").unwrap_or("")));
        }
        resolver.api(*index, tag, role, attrs, events, policy);
        if tag == "ltx:creator" && role == "author" {
          resolver.entries.push((start, resolver.out.creators.len()));
        }
      },
      Input::NoteText { index, epoch, mark, events } => {
        resolver
          .note_texts
          .push((*index, *epoch, mark.clone(), events::printed_text(events)))
      },
    }
  }
  resolver.thenote_finish();
  // (today's grid, `distribute_upfront_contacts`, runs before `relocate_annotations` answers the labels: the contacts
  // a label places are no part of the pile — 2311.04097's two `\email`s under `\author{A$^1$ and B$^2$}` with their
  // affiliations marked; the amsart upfront grids 1302.1274, 1610.05835, 2204.13910, 2205.05923, 2507.07513)
  resolver.distribute_upfront();
  resolver.answer_note_links();
  resolver.answer_pending();
  resolver.answer_note_texts();
  resolver.answer_sequence();
  resolver.answer_document_notes();
  resolver.answer_listed_emails();
  resolver.respell_emails();
  resolver.held_before_creators();
  resolver.sole_author_takes_the_block();
  resolver.take_out_entry_addresses(inputs);
  let mut out = resolver.out;
  out.continued = Some(
    resolver
      .continued
      .into_iter()
      .map(|(_, lines)| lines)
      .collect(),
  );
  if !resolver.replaced.is_empty() {
    // (and the block's own items of a replaced call: today's `\lx@clear@frontmatter` keeps nothing of it — 2510.26354's
    // first `\author`, whose `firstname.lastname@h-its.org` the second call's addresses write out)
    let replaced_blocks: Vec<usize> = resolver
      .replaced
      .iter()
      .map(|&c| out.creators[c].source)
      .collect();
    let kept_block: Vec<bool> = out
      .block
      .iter()
      .map(|a| {
        a.source
          .is_none_or(|(index, _)| !replaced_blocks.contains(&index))
      })
      .collect();
    let mut at = 0usize;
    out.block.retain(|_| {
      at += 1;
      kept_block[at - 1]
    });
    let mut at = 0usize;
    out.creators.retain(|_| {
      at += 1;
      !resolver.replaced.contains(&(at - 1))
    });
    // (the take-outs' holders held where the deletions moved them: a creator's index past the replaced ones before it,
    // a block slot past the deleted block items before it — 2602.12633's first `\author{Author Names Omitted…}`, whose
    // deletion left the second call's last holder past the end, a panic in the writer's give-back; 2011.06172's apa.cls
    // `\author{Author}` before `\threeauthors`, the bundle int124a A/B's, guard `a_replaced_author_call_before_an_entry_…`)
    let shift =
      |kept: &dyn Fn(usize) -> bool, i: usize| kept(i).then(|| (0..i).filter(|&j| kept(j)).count());
    let live_creator = |c: usize| !resolver.replaced.contains(&c);
    let live_block = |i: usize| kept_block.get(i).copied().unwrap_or(false);
    for taken in &mut out.taken {
      taken.holders = std::mem::take(&mut taken.holders)
        .into_iter()
        .filter_map(|(holder, slot, assoc)| match holder {
          Some(c) => shift(&live_creator, c).map(|c| (Some(c), slot, assoc)),
          None => shift(&live_block, slot).map(|slot| (None, slot, assoc)),
        })
        .collect();
    }
    out.taken.retain(|taken| !taken.holders.is_empty());
    for listed in &mut out.listed {
      listed.owners = std::mem::take(&mut listed.owners)
        .into_iter()
        .filter_map(|c| shift(&live_creator, c))
        .collect();
    }
    out.listed.retain(|listed| !listed.owners.is_empty());
  }
  // (each creator's notes and contacts, and the block's own, in source order — the order the PDF prints them: a note
  // and a link placed once every name is read (`answer_note_links`) came after the ones the source gives later,
  // `\author{Ann Able\thanks{\url{ann@x.org}, \url{zed@y.org}}\thanks{Second note.} \and Bob Baker}` read `[Second
  // note., ann@x.org, zed@y.org]` — review revfms2a2_7 finding 1, FM S2a.2 r8)
  for creator in &mut out.creators {
    in_source_order(&mut creator.notes);
    in_source_order(&mut creator.contacts);
  }
  in_source_order(&mut out.block);
  out
}

/// Whether a contact call declares a run and every author's share (`_share=authors,annotate=run`: cas's unlabelled
/// `\affiliation`, `cas_dc_cls.rs` `cas_labelled_entry`).
pub(super) fn shares_a_run(attrs: &str) -> bool {
  attr_value(attrs, "_share") == Some("authors") && attr_value(attrs, "annotate") == Some("run")
}

/// `items` in source order: those with a source sorted by it (its block or API call's ledger index, then its span's
/// start — spans are positions within one block; one with no span after its block's spanned ones, review revfms2a2_8
/// finding 6), stably, in the places they held; one without a source stays where it is.
pub(in crate::base_utilities::frontmatter::observer) fn in_source_order(items: &mut [Assoc]) {
  let slots: Vec<usize> = (0..items.len())
    .filter(|&i| items[i].source.is_some())
    .collect();
  let mut sourced: Vec<Assoc> = slots.iter().map(|&i| items[i].clone()).collect();
  sourced.sort_by_key(|a| {
    a.source
      .map(|(index, span)| (index, span.map_or((1, 0), |(start, _)| (0, start))))
  });
  for (slot, assoc) in slots.into_iter().zip(sourced) {
    items[slot] = assoc;
  }
}

/// The resolver's inputs from the ledger: each block rehearsed (its events), and each API call made apart from a
/// block's expansion that is a creator or a creator's annotation ([`creator_call`]).
pub(in crate::base_utilities) fn ledger_inputs(ledger: &ledger::Ledger) -> Vec<Input> {
  let mut inputs = Vec::new();
  for entry in ledger.entries() {
    match entry {
      ledger::Entry::Author(block) => {
        if let ledger::Outcome::Events { events, creators, .. } = &block.outcome {
          inputs.push(Input::Block {
            index:   block.index,
            binding: block.binding.clone(),
            hook:    block.hook.to_string(),
            keyvals: block.keyvals.clone(),
            counter: block.counter,
            replace: block.replace,
            cleared: block.cleared,
            epoch:   block.epoch,
            events:  events.clone(),
            calls:   creators
              .iter()
              .map(field_notes::FieldCreator::labels)
              .collect(),
          });
        }
      },
      // (and the title, for the labels its marks request: `Resolver::api`'s `ltx:title`)
      ledger::Entry::Api(call)
        if !call.inside && !call.cleared && resolver_call(&call.tag, &call.parent) =>
      {
        inputs.push(Input::Api {
          index:  call.index,
          tag:    call.tag.clone(),
          role:   call.role.clone(),
          attrs:  call.attrs.clone(),
          events: call.events.clone(),
        })
      },
      ledger::Entry::NoteText(text) => inputs.push(Input::NoteText {
        index:  text.index,
        epoch:  text.epoch,
        mark:   text.mark.clone(),
        events: text.events.clone(),
      }),
      _ => {},
    }
  }
  inputs
}

/// Whether an API call is the resolver's: a creator of its own (`\lx@add@creator`, tag `ltx:creator`), or an
/// annotation of the creators (`\lx@annotate@frontmatter{ltx:creator}…`) — not a document's note of its own (MDPI's
/// `reftitle`, an author-contributions note: no creator holds them in the XML, review revfms2a2 finding 7, 2608.12223),
/// nor a title's.
pub fn creator_call(tag: &str, parent: &str) -> bool {
  (tag == "ltx:creator" && parent.is_empty()) || parent == "ltx:creator"
}

/// Whether an API call made apart from the blocks is the resolver's input (`ledger_inputs`, and `ab_audit fmshadow
/// --re-resolve` reading a dump): a creator call (`creator_call`), the title (for the labels its marks request), or a
/// publication note (`ltx:pubnote`, for the e-mails it prints: A&A's `\offprints{B. Baker, \email{…}}`, 0711.1152,
/// 0802.0707).
pub fn resolver_call(tag: &str, parent: &str) -> bool {
  creator_call(tag, parent) || tag == "ltx:title" || tag == "ltx:pubnote"
}

#[cfg(test)]
mod tests {
  use super::*;

  fn creator(name: &str, requests: &[&str]) -> RCreator {
    RCreator {
      name: name.into(),
      requests: requests.iter().map(|r| r.to_string()).collect(),
      ..RCreator::default()
    }
  }

  /// Item 7 (review revfms2a2 finding 7): only creators and creator annotations are the resolver's — not a document's
  /// note of its own (MDPI's `reftitle`, 2608.12223), nor a title's.
  #[test]
  fn intake_takes_creator_calls_only() {
    assert!(creator_call("ltx:creator", ""));
    assert!(creator_call("ltx:contact", "ltx:creator"));
    assert!(creator_call("ltx:note", "ltx:creator"));
    assert!(!creator_call("ltx:note", ""));
    assert!(!creator_call("ltx:note", "ltx:title"));
    assert!(!creator_call("ltx:contact", ""));
    assert!(resolver_call("ltx:pubnote", "") && resolver_call("ltx:title", ""));
    assert!(!resolver_call("ltx:note", ""));
  }

  /// Items 3 and 4: `author:N` names the N-th creator exactly; any other label falls back past the prefixes in Perl's
  /// order, the first tier that finds one deciding.
  #[test]
  fn label_owners_exact_then_unprefixed() {
    let creators = [
      creator("Ann", &["affiliation:1"]),
      creator("Bob", &["fn:label1"]),
      creator("Cat", &["2"]),
    ];
    let live = [0, 1, 2];
    assert_eq!(label_owners(&creators, &live, "author:2"), [1]);
    assert_eq!(
      label_owners(&creators, &live, "author:9"),
      Vec::<usize>::new()
    );
    assert_eq!(label_owners(&creators, &live, "1"), [0]);
    assert_eq!(label_owners(&creators, &live, "label1"), [1]);
    assert_eq!(label_owners(&creators, &live, "affiliation:2"), [2]);
    assert_eq!(label_owners(&creators, &live, "institute:1"), [0]);
    // (FM S2a.2 r7, review revfms2a2_6 finding 4: the N-th live creator — a replaced first one shifts no label)
    assert_eq!(label_owners(&creators, &[1, 2], "author:1"), [1]);
    assert_eq!(
      label_owners(&creators, &[1, 2], "author:3"),
      Vec::<usize>::new()
    );
  }

  /// FM S2b r3: a name's number range is every number in it; a falling or long range stays as written (2012.15817).
  #[test]
  fn number_ranges_are_spelled_out() {
    let spelled = |labels: &[&str]| spelled_ranges(labels.iter().map(|l| l.to_string()).collect());
    assert_eq!(spelled(&["1-4", "a"]), ["1", "2", "3", "4", "a"]);
    assert_eq!(spelled(&["2\u{2013}3"]), ["2", "3"]);
    assert_eq!(spelled(&["1--3"]), ["1", "2", "3"]);
    // (a binding's prefix stays on each piece of a mark: aastex's `altaffil:`, 0911.4695)
    assert_eq!(mark_labels("altaffil:2,altaffil:**"), [
      "altaffil:2",
      "altaffil:**"
    ]);
    assert_eq!(mark_labels("altaffil:*1"), ["altaffil:*", "altaffil:1"]);
    assert_eq!(spelled(&["4-1", "1-99", "a-c"]), ["4-1", "1-99", "a-c"]);
  }

  /// Item 2: a `fuzzy:` label and a `\ref` key stay whole; the name's letters skip its marks.
  #[test]
  fn fuzzy_labels_and_their_letters() {
    assert_eq!(mark_labels("fuzzy:Luca Passalacqua (\\Envelope) "), [
      "fuzzy:Luca Passalacqua (\\Envelope)"
    ]);
    assert_eq!(mark_labels("*1, a"), ["*", "1", "a"]);
    // (a mark holding several `\ref`s is a list, each key whole: `\inst{\ref{ia},\ref{ib}}`, 1707.06238, 1302.1442)
    assert_eq!(mark_labels("ref:sec one (a)"), ["ref:sec one (a)"]);
    assert_eq!(mark_labels("ref:ia, ref:ib"), ["ref:ia", "ref:ib"]);
    assert_eq!(mark_labels("1,ref:ib"), ["1", "ref:ib"]);
    // (what prints between two references is no label: `\ref{a}--\ref{c}`, `\ref{a};\ref{b}`, as
    // `events::script_ink` writes them; review rev92a_r1 B3)
    assert_eq!(mark_labels("ref:a,\u{2013},ref:c"), ["ref:a", "ref:c"]);
    assert_eq!(mark_labels("ref:a,;,ref:b"), ["ref:a", "ref:b"]);
    assert_eq!(mark_labels("ref:a,*,ref:b"), ["ref:a", "*", "ref:b"]);
    assert_eq!(
      fold_name(&tex_letters("Luca Passalacqua (\\Envelope)")),
      "lucapassalacqua"
    );
    assert_eq!(
      fold_name(&tex_letters("Jean-Ren\\'e Chazottes\\inst{2}")),
      "jeanrenechazottes"
    );
    // (FM S2a.2 r5: the kernel's accent commands written out, their names glued to the next letters, 2205.11550)
    assert_eq!(
      fold_name(&tex_letters("Gizem \\lxaccentc{S}eng\\lxaccentumlautor")),
      "gizemsengor"
    );
    assert_eq!(
      fold_name(&tex_letters("Luca Passalacqua (\\@chooseSymbol{'014})")),
      "lucapassalacqua"
    );
  }

  /// FM S2a.2 r3 item 1c: a glued label as the known labels written together, the fewest pieces; a symbol run on its
  /// own only where symbols may stand alone; none when no piece is known.
  /// A document note's name (review rev92a_r2 B2): its pieces part at `,`, `;`, `:`, `/`, `&`, "and" and "or" (a
  /// label a piece of its own, which names no one: review rev92a_r3 S1); a piece is a creator's name only when every
  /// word agrees with it.
  #[test]
  fn a_note_piece_names_a_creator_only_when_it_is_their_name() {
    assert_eq!(note_pieces("O. Kochukhov, "), ["O. Kochukhov"]);
    assert_eq!(note_pieces("Corresponding: A. Able, "), [
      "Corresponding",
      "A. Able"
    ]);
    assert_eq!(note_pieces("Ann Able: corresponding author, "), [
      "Ann Able",
      "corresponding author"
    ]);
    assert_eq!(note_pieces("A. Able: "), ["A. Able"]);
    assert_eq!(
      note_pieces("A. Able or B. Baker; C. Cole/D. Dunn & E. Eve and F. Fox"),
      [
        "A. Able", "B. Baker", "C. Cole", "D. Dunn", "E. Eve", "F. Fox"
      ]
    );
    let agree = |piece: &str, name: &str| names_agree(&name_words(piece), &name_words(name));
    assert!(agree("O. Kochukhov", "Oleg Kochukhov"));
    assert!(agree("B. Bitsch", "Bertram Bitsch"));
    assert!(agree("Bertram Bitsch", "B. Bitsch"));
    assert!(agree("Kochukhov", "Oleg Kochukhov"));
    assert!(agree("J.-P. Dupont", "Jean-Pierre Dupont"));
    assert!(agree("A.B. Smith", "Adam Bruce Smith"));
    assert!(agree("A. Smith", "Adam Bruce Smith"));
    assert!(agree("M. Iłkiewicz", "Mateusz Iłkiewicz"));
    assert!(!agree("J. Smith", "Adam Smith"));
    assert!(!agree("B. A. Smith", "Adam Bruce Smith"));
    assert!(!agree("the editor of Bob Baker", "Bob Baker"));
    assert!(!agree("Dr. Baker", "Bob Baker"));
    assert!(!agree("Baker B.", "Bob Baker"));
    assert!(!agree("Bob", "Bob Baker"));
    assert!(!agree("Corresponding author", "Ann Able"));
    assert!(!agree("Correspondence to", "Ann Able"));
  }

  #[test]
  fn glued_labels_part_into_known_ones() {
    let known = |ls: &[&str]| ls.iter().map(|l| l.to_string()).collect::<Vec<_>>();
    assert_eq!(
      parted_label("12", &known(&["1", "2"]), true),
      Some(vec!["1".into(), "2".into()])
    );
    assert_eq!(
      parted_label("1⋆", &known(&["1", "2"]), true),
      Some(vec!["1".into(), "⋆".into()])
    );
    assert_eq!(parted_label("1⋆", &known(&["1", "2"]), false), None);
    assert_eq!(
      parted_label("**", &known(&["*"]), false),
      Some(vec!["*".into(), "*".into()])
    );
    assert_eq!(parted_label("ab", &known(&["1"]), true), None);
    assert_eq!(parted_label("7", &known(&["7"]), true), None);
    // a run of symbols is parted into the known labels it spells, before it stands alone as one symbol piece
    // (`$^{\dagger\natural\sharp}$` under entries `†`, `♮`, `♯`: 1004.1402)
    assert_eq!(
      parted_label("†♮♯", &known(&["†", "♮", "♯"]), true),
      Some(vec!["†".into(), "♮".into(), "♯".into()])
    );
    // a known number with a single letter glued to it, the letter a note's mark (`$^{1{\rm a}}$` under entries `1`,
    // `2`: 1904.02582); letters alone part nothing
    assert_eq!(
      parted_label("1a", &known(&["1", "2"]), true),
      Some(vec!["1".into(), "a".into()])
    );
    assert_eq!(parted_label("1a", &known(&["1", "2"]), false), None);
    assert_eq!(parted_label("ab", &known(&["1", "2"]), true), None);
    // (a label as its source spells it is what it spells before its first command: authblk's `\author[3\authfn{2}]`,
    // 2405.16051; `\author[1\thanks{…}]`, 2202.05736)
    assert_eq!(
      parted_label("3\\authfn{2}", &known(&["1", "2", "3"]), true),
      Some(vec!["3".into()])
    );
    assert_eq!(
      parted_label("1\\thanks{\\tt{a@b.de}}", &known(&["1", "2"]), true),
      Some(vec!["1".into()])
    );
    assert_eq!(
      parted_label("12\\fn{3}", &known(&["1", "2", "3"]), true),
      Some(vec!["1".into(), "2".into()])
    );
    assert_eq!(parted_label("\\dag", &known(&["1"]), true), None);
    // a run of digits is one number under multi-digit numbering: BESIII's sub-lettered `23A` is `23` and `A`, never
    // `2`, `3` and `A` (2007.02542), and `48A` with no entry 48 parts into nothing (1507.02068; review rev98e B1)
    assert_eq!(
      parted_label("23A", &known(&["1", "2", "3", "23", "a"]), true),
      Some(vec!["23".into(), "A".into()])
    );
    assert_eq!(parted_label("48A", &known(&["4", "8", "12"]), true), None);
    assert_eq!(
      parted_label("12A", &known(&["1", "2"]), true),
      Some(vec!["1".into(), "2".into(), "A".into()])
    );
  }

  /// FM S2a.2 r5 (review revfms2a2_4 finding 5): an annotation made before the authors that is a note (`api:note`) is
  /// a sole author's note, not a contact holding a role; a name is its printed text, a visible U+2010 kept, its spaces
  /// collapsed (r6: review revfms2a2_5 finding 2 — a zero-width character never reaches it, the lexer's `is_zero_width`).
  #[test]
  fn a_sole_authors_note_made_before_it_is_its_note() {
    let mut resolver = Resolver::new();
    resolver.new_creator("Ann  Able ‐".into(), "line:names", 1, Vec::new(), 1);
    resolver.out.block.push(Assoc {
      role:   "thanks".into(),
      text:   "Supported by X.".into(),
      label:  None,
      rule:   "api:no-creator owner:block api:note".into(),
      source: None,
      target: None,
    });
    resolver.held_before_creators();
    let ann = &resolver.out.creators[0];
    assert_eq!(ann.name, "Ann Able ‐");
    assert_eq!(
      ann
        .notes
        .iter()
        .map(|n| n.text.as_str())
        .collect::<Vec<_>>(),
      ["Supported by X."]
    );
    assert!(ann.contacts.is_empty() && resolver.roles[0].is_empty());
    assert!(resolver.out.block.is_empty());
  }

  /// FM S2a.2 r8 (main-session decision 1 on r7, review revfms2a2_7 finding 3): a link's page is its last path segment, an
  /// `index.*` or `*.html` page after it aside, else its host.
  #[test]
  fn a_links_page_is_its_last_segment_or_its_host() {
    assert_eq!(link_page("http://www.pengxu.net/"), "pengxu.net");
    assert_eq!(
      link_page("https://sites.google.com/view/kschewior"),
      "kschewior"
    );
    assert_eq!(
      link_page("http://insti.physics.sunysb.edu/~siegel/plan.html"),
      "~siegel"
    );
    assert_eq!(
      link_page("https://example.org/~cole/index.php?x=1#top"),
      "~cole"
    );
    assert_eq!(
      link_page("https://github.com/KuangenZhang/StructuredRL"),
      "StructuredRL"
    );
    assert_eq!(
      link_page("https://mengyuest.github.io/gnn_vae_coord/"),
      "gnn_vae_coord"
    );
    assert_eq!(
      link_page("https://staff.aist.go.jp/kamimura.a/accvideo.mp4"),
      "accvideo.mp4"
    );
    assert_eq!(link_page("index.html"), "index.html");
    // (FM S2a.2 r9, review revfms2a2_8 finding 5: a page right under the host spells its owner; a `@user` path is a page)
    assert_eq!(link_page("http://www.cs.x.edu/jsmith.html"), "jsmith");
    // (FM S2b r1, review revfms2a2_9 finding 4: a `@user` page is the user's name, which an owner can spell)
    assert_eq!(link_page("https://medium.com/@jsmith"), "jsmith");
    assert!(addresses_in("medium.com/@jsmith").is_empty());
    assert!(!addresses_in("brunnerj, edemaine, dylanhen, wellman@mit.edu").is_empty());
  }

  /// FM S2a.2 r8 (review revfms2a2_7 finding 1): items in source order, by ledger index then span; one without a source
  /// keeps its place.
  #[test]
  fn items_in_source_order() {
    let item = |text: &str, source: Option<(usize, Option<(u16, u16)>)>| Assoc {
      role: "thanks".into(),
      text: text.into(),
      label: None,
      rule: String::new(),
      source,
      target: None,
    };
    let mut items = vec![
      item("b", Some((3, Some((40, 50))))),
      item("x", None),
      item("c", Some((7, None))),
      item("a", Some((3, Some((10, 20))))),
    ];
    in_source_order(&mut items);
    assert_eq!(items.iter().map(|a| a.text.as_str()).collect::<Vec<_>>(), [
      "a", "x", "b", "c"
    ]);
    // (FM S2a.2 r9, review revfms2a2_8 finding 6: a span-less item after its block's spanned ones)
    let mut items = vec![
      item("n", Some((3, None))),
      item("b", Some((3, Some((40, 50))))),
      item("a", Some((3, Some((10, 20))))),
      item("p", Some((2, None))),
    ];
    in_source_order(&mut items);
    assert_eq!(items.iter().map(|a| a.text.as_str()).collect::<Vec<_>>(), [
      "p", "a", "b", "n"
    ]);
  }

  #[test]
  fn address_lines() {
    assert!(only_addresses("E-mail: {ann,bob}@x.org, cat@y.org"));
    // (FM S2a.2 r5: an address before a braced group is the list's too, 1808.08866)
    assert_eq!(addresses_in("dan@z.org; {ann,bob}@x.org"), [
      "dan@z.org",
      "ann@x.org",
      "bob@x.org"
    ]);
    // (batch 103a: a list of local parts written without spaces is each part at the domain, as `address_runs` and the
    // author block's `email_addresses` read it — `dzyang` was dropped, 2112.00969; a capitalised word or a digit run is
    // no part of it, and a spaced list is not written out)
    assert_eq!(addresses_in("ann@a.edu; dzyang,ashishbshah@fb.com"), [
      "ann@a.edu",
      "dzyang@fb.com",
      "ashishbshah@fb.com"
    ]);
    assert_eq!(addresses_in("Berlin,Germany,foo@bar.de"), ["foo@bar.de"]);
    assert_eq!(addresses_in("Tel:5551234,info@x.org"), ["info@x.org"]);
    // (review rev103a B1, S1: a word naming a contact, an initial, is no part of it)
    assert_eq!(addresses_in("email,ann@x.org, bob@x.org"), [
      "ann@x.org",
      "bob@x.org"
    ]);
    assert_eq!(addresses_in("ann@a.edu; d,ashishbshah@fb.com"), [
      "ann@a.edu",
      "ashishbshah@fb.com"
    ]);
    assert_eq!(addresses_in("smith,j@x.org, doe@x.org"), [
      "j@x.org",
      "doe@x.org"
    ]);
    assert_eq!(addresses_in("ann@a.edu; dzyang, ashishbshah@fb.com"), [
      "ann@a.edu",
      "ashishbshah@fb.com"
    ]);
    assert!(!only_addresses("Barcelona, SPAIN. ann@x.org"));
    assert!(is_one_address("mailto:ann@x.org"));
    assert!(!is_one_address("https://x.org"));
  }

  /// 107c: a list's other printed forms read as its braced one — local parts in square brackets before the domain
  /// (2405.17925), a group spaced round its `@` (2104.11106, 1707.04775), pieces ending in a dot or an `@` before the
  /// domain after the brace (2108.00873, 2605.01350), local parts sharing the tail after the brace (1908.06816) — and a
  /// group whose joined pieces are no addresses read as none.
  #[test]
  fn address_runs_read_bracketed_spaced_and_joined_groups() {
    let none = |_: &str| false;
    let read = |text: &str| -> Vec<String> {
      address_runs(text, &none)
        .into_iter()
        .flat_map(|r| r.addresses)
        .collect()
    };
    assert_eq!(read("Email: [i.iudice, d.pascarella]@cira.it"), [
      "i.iudice@cira.it",
      "d.pascarella@cira.it"
    ]);
    assert_eq!(read("{aremonda, skrebs} @ know-center.at"), [
      "aremonda@know-center.at",
      "skrebs@know-center.at"
    ]);
    assert_eq!(
      read("{junwei@link., lizhen@}cuhk.edu.cn, s.kevin.zhou@gmail.com"),
      [
        "junwei@link.cuhk.edu.cn",
        "lizhen@cuhk.edu.cn",
        "s.kevin.zhou@gmail.com"
      ]
    );
    assert_eq!(read("{fikadu.t.dagefu, brian.m.sadler6}.civ@mail.mil"), [
      "fikadu.t.dagefu.civ@mail.mil",
      "brian.m.sadler6.civ@mail.mil"
    ]);
    assert_eq!(read("{xxh22@mails, huichen@mail}.tsinghua.edu.cn"), [
      "xxh22@mails.tsinghua.edu.cn",
      "huichen@mail.tsinghua.edu.cn"
    ]);
    assert!(read("{Dept A, Univ B}.civ@x.mil").is_empty());
    assert!(read("[1] Univ A, Town").is_empty());
  }

  /// Brief 95c: the address reader of an entry's printed text — each run's addresses written out, its range with the
  /// label word and parentheses, the separators a cut strands taken with it.
  #[test]
  fn address_runs_read_groups_lists_and_labels() {
    let spells = |local: &str| ["ann.able", "bob.baker", "aable", "bbaker"].contains(&local);
    let none = |_: &str| false;
    let read = |text: &str, spells: &dyn Fn(&str) -> bool| -> (String, Vec<String>) {
      let runs = address_runs(text, spells);
      (
        strip_address_runs(text, &runs),
        runs.into_iter().flat_map(|r| r.addresses).collect(),
      )
    };
    let owned = |v: &[&str]| v.iter().map(ToString::to_string).collect::<Vec<_>>();
    // (a list written without spaces: the main session's ruling, 0810.2791, 1110.2127)
    assert_eq!(
      read(
        "Dept X, Trieste, Italy (saro,borgani,tornator@oats.inaf.it)",
        &none
      ),
      (
        "Dept X, Trieste, Italy".into(),
        owned(&[
          "saro@oats.inaf.it",
          "borgani@oats.inaf.it",
          "tornator@oats.inaf.it"
        ])
      )
    );
    // (with spaces: each part spelling an author, on a line of its own, 2308.10621; or opening the text, 0812.3572)
    assert_eq!(
      read("3Dwe.ai\nann.able, bob.baker@x.org", &spells),
      (
        "3Dwe.ai".into(),
        owned(&["ann.able@x.org", "bob.baker@x.org"])
      )
    );
    assert_eq!(
      read("cid, dee@x.org", &none).1,
      owned(&["cid@x.org", "dee@x.org"])
    );
    // (a word naming a contact or an initial is no part of a list, nor does an initial end one: review rev103a B1, S1)
    assert_eq!(
      read("email,ann@x.org, bob@x.org", &none).1,
      owned(&["ann@x.org", "bob@x.org"])
    );
    assert_eq!(
      read("ann@a.edu; d,ashishbshah@fb.com", &none).1,
      owned(&["ann@a.edu", "ashishbshah@fb.com"])
    );
    assert_eq!(
      read("smith,j@x.org, doe@x.org", &none).1,
      owned(&["j@x.org", "doe@x.org"])
    );
    // (a spaced list takes the spaced-list rule, as on base: the unspaced list's checks are that list's alone — review
    // rev103a2 B1)
    assert_eq!(
      read("E-mail: ann, b@x.org", &none).1,
      owned(&["ann@x.org", "b@x.org"])
    );
    assert_eq!(
      read("E-mail: a, bob@x.org", &none).1,
      owned(&["a@x.org", "bob@x.org"])
    );
    // (prose before one address: no list — a capital, or a word spelling no one mid-text, 0705.3421)
    assert_eq!(
      read(
        "Univ Tübingen, Germany, wilhelm.kley@uni-tuebingen.de",
        &none
      ),
      (
        "Univ Tübingen, Germany".into(),
        owned(&["wilhelm.kley@uni-tuebingen.de"])
      )
    );
    assert_eq!(
      read("Univ X, uk, ann@x.org", &none).1,
      owned(&["ann@x.org"])
    );
    // (a braced group, its spaces before the `@` the group's: 2009.00299, 2311.14729)
    assert_eq!(
      read("CVSSP, Univ X, UK, {ann.able, bob.baker}@x.org,", &none),
      (
        "CVSSP, Univ X, UK".into(),
        owned(&["ann.able@x.org", "bob.baker@x.org"])
      )
    );
    assert_eq!(
      read("Univ X\n{ aable, bbaker } @x.org", &none).1,
      owned(&["aable@x.org", "bbaker@x.org"])
    );
    // (the label word goes with the address; a cut between two separators takes the one after: cond-mat0607219,
    // 1909.06145)
    assert_eq!(
      read("Dept X, FAX: 1, email: aable@y.edu, Jan. 4", &none),
      ("Dept X, FAX: 1, Jan. 4".into(), owned(&["aable@y.edu"]))
    );
    assert_eq!(
      read("Besançon, France - e-mail: julien@obs.fr", &none).0,
      "Besançon, France"
    );
    assert_eq!(
      read("Univ X, E-mail addresses: a@x.org, b@y.org", &none).0,
      "Univ X"
    );
    // (two addresses, each a list's end of none: `a@x.org, b@y.org` is no `x.org@…`)
    assert_eq!(
      read("a@x.org, b@y.org", &none).1,
      owned(&["a@x.org", "b@y.org"])
    );
    // (only at a list boundary — prose, a parenthesised tag after it, keep it: brief 95c r2 B2)
    for prose in [
      "Univ X, write to info@x.org or data@x.org for data",
      "Univ Y, City, Emails: ann@y.org (Ann), bob@y.org (Bob)",
      "Univ X, write ann@x.org now",
      "Univ X, see https://medium.com/@ann.able, mail ann@x.org now",
      // (one boundary side beside a lowercase word without a colon: prose, brief 95c r3 F5)
      "Univ X, see also data@x.org",
      "Univ Y. Write to info@y.org",
      "Dept Z, Univ Z, write to a@z.org.",
      "Univ W, see sales@w.org, City W",
      "Univ X, E-mail: ann@x.org now",
      // (`mail` is no e-mail label word, `author_block::is_email_label`'s, r3 N1)
      "Univ X, please mail ann@x.org",
    ] {
      assert_eq!(
        read(prose, &spells),
        (prose.to_string(), Vec::new()),
        "{prose}"
      );
    }
    // (a list's parts letters: a digit run is no local part, B3)
    assert_eq!(
      read("Univ X, Tel:5551234,info@x.org", &none),
      ("Univ X, Tel:5551234".into(), owned(&["info@x.org"]))
    );
    assert_eq!(
      read("Univ Y, ORCID 0000-0002-1825-0097,zq@y.org", &none),
      (
        "Univ Y, ORCID 0000-0002-1825-0097".into(),
        owned(&["zq@y.org"])
      )
    );
    // (a label word without a colon only opening its line, S4)
    // (one side a boundary is enough: 2601.07312, 2509.12185, 2112.12175)
    assert_eq!(
      read("Univ X Correspondence: {ann, bob}@x.org", &none),
      (
        "Univ X Correspondence".into(),
        owned(&["ann@x.org", "bob@x.org"])
      )
    );
    assert_eq!(read("KTH, Sweden ann,bob@kth.se", &none).0, "KTH, Sweden");
    assert_eq!(
      read("Univ X\nEmail ann@x.org", &none),
      ("Univ X".into(), owned(&["ann@x.org"]))
    );
    assert_eq!(
      read("Dept X, Univ Y, SPAIN. ann@x.org", &none),
      ("Dept X, Univ Y, SPAIN.".into(), owned(&["ann@x.org"]))
    );
    // (a bare label word after a separator or a period, its run ending the text: the label, r3 F4)
    assert_eq!(
      read("Univ X, E-mail ann@x.org", &none),
      ("Univ X".into(), owned(&["ann@x.org"]))
    );
    assert_eq!(
      read("Univ Y. Email bob@y.org", &none),
      ("Univ Y.".into(), owned(&["bob@y.org"]))
    );
    // (a cut keeps one space of two, N1; a line's end ends a run's separators, its break kept, N2 — cond-mat0607219)
    let cut = |text: &str| {
      let runs = address_runs(text, &none);
      let mut out = String::new();
      let mut at = 0;
      for (start, end) in address_cuts(text, &runs) {
        out.push_str(&text[at..start]);
        at = end;
      }
      out + &text[at..]
    };
    assert_eq!(
      cut("Dept X, Univ Y, ann@x.org, Phone 123, office@x.org"),
      "Dept X, Univ Y, Phone 123"
    );
    assert_eq!(
      cut("Univ X\nPh: 1; FAX: 2; email: rlake@ee.ucr.edu\nJan. 4, 2005"),
      "Univ X\nPh: 1; FAX: 2\nJan. 4, 2005"
    );
    assert_eq!(cut("Univ X\nann@x.org\nCity"), "Univ X\nCity");
    assert_eq!(cut("Univ X (ann@x.org), City"), "Univ X, City");
    // (no address: an `@` with no domain, 2202.02533's `Wireless@VT`)
    assert!(address_runs("Wireless@VT, Dept X", &none).is_empty());
    // (a several-address call printing addresses alone; a label word beside them keeps it whole)
    assert_eq!(
      listed_addresses("julioleite,tmariz@fis.ufal.br", &none),
      Some(owned(&["julioleite@fis.ufal.br", "tmariz@fis.ufal.br"]))
    );
    assert_eq!(listed_addresses("Emails: {a,b}@x.org", &none), None);
    assert_eq!(listed_addresses("ann@x.org", &none), None);
  }

  /// P1c (review revP1br2 S2): the label the Kind step for an address line reads before a run the take-out takes goes with
  /// it in the reading (`take_labels`, RT1 rows 64-67, 72), the entry's text the reading keeps and the e-mails' name its own
  /// — a label line whole with the break before it (2008.08137, 1807.09187), none after words on its line, a legend's
  /// words with no e-mail word left for the Kind step for a legend (row 84: 1812.03816), an unmarked one's taken
  /// (2305.02090, 2404.05566).
  #[test]
  fn take_labels_takes_the_label_before_a_run() {
    let none = |_: &str| false;
    let take = |lined: &str, legend: bool| -> (String, Vec<Option<String>>) {
      let runs = address_runs(lined, &none);
      let mut cuts = address_cuts(lined, &runs);
      let mut taken: Vec<TakenRun> = runs
        .iter()
        .map(|run| TakenRun {
          bytes:     (run.start, run.end),
          addresses: run.addresses.clone(),
          label:     run.label.clone(),
        })
        .collect();
      take_labels(lined, &mut taken, &mut cuts, &none, legend);
      (
        strip_cuts(lined, &cuts),
        taken.into_iter().map(|run| run.label).collect(),
      )
    };
    assert_eq!(
      take("Univ Z\nCorrespondence: cy@z.org", false),
      ("Univ Z".into(), vec![Some("Correspondence:".into())])
    );
    // (words before the label on its line: the Kind step declines the line, as the switch off — "Contact" stays)
    assert_eq!(
      take("Univ Z, Contact e-mail: cy@z.org", false),
      ("Univ Z, Contact".into(), vec![Some("e-mail:".into())])
    );
    assert_eq!(
      take("Corresponding author: ann@x.org", true),
      ("Corresponding author".into(), vec![None])
    );
    assert_eq!(
      take("Corresponding author: ann@x.org", false),
      ("".into(), vec![Some("Corresponding author:".into())])
    );
  }
}
