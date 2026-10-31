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
  events::{self, EmailMeaning, Event, EventKind, SepKind, events_text},
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
  /// the ledger index of its block (or API call)
  pub source:       usize,
  pub affiliations: Vec<Assoc>,
  pub contacts:     Vec<Assoc>,
  pub notes:        Vec<Assoc>,
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
mod post;

pub(in crate::base_utilities::frontmatter::observer) use api::split_pieces;
use blocks::Chain;
pub(in crate::base_utilities::frontmatter::observer) use labels::label_matches;
// (named, as the prelude has its own of these names: the resolver's shadow them)
pub use labels::{addresses_in, mark_labels};
use labels::{text_tokens, *};
use lines::*;
pub(in crate::base_utilities::frontmatter::observer) use lines::{
  may_name_a_person, name_before_remark, reads_as_names,
};

/// A block's events as the resolver reads them (`Resolver::block`): an ORCID link's name, an IEEE ordinal, a mark
/// printed as text after a name and a glued run of shown symbols each read as the names line reads it.
pub(in crate::base_utilities::frontmatter::observer) fn read_events(
  events: &[Event],
) -> Vec<Event> {
  glue_shown_runs(&split_text_marks(&join_ordinals(&orcid_link_names(events))))
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
  out:              Resolved,
  /// per creator: its block (input index) and the roles its contacts hold
  roles:            Vec<Vec<String>>,
  /// the labels of `labelseq=<role>` entries so far, per role
  sequence:         Vec<(String, usize)>,
  /// labelled entries of the blocks not yet answered (label, entry), resolved at the end against every request
  pending:          Vec<(Vec<String>, bool, Assoc)>,
  /// per creator: its group (the block's input index, or the policy's call group)
  group:            Vec<usize>,
  /// per creator: the block (input index) and `\and` column of the names line it was read from, if a block's
  column:           Vec<Option<(usize, usize)>>,
  /// the block line being read (input index, column): the column a creator made now is read from
  line_column:      Option<(usize, usize)>,
  /// the column (input index, column) of each block line's events, keyed by (input index, an event's span start):
  /// where a labelled entry stands (`answer_pending`'s per-column labels)
  entry_column:     HashMap<(usize, u16), usize>,
  /// the addresses every line of the block being read prints (`place_line_contacts`' author-order list: one address
  /// per author, every spelled address of the block agreeing)
  block_addresses:  Vec<String>,
  /// the addresses of a class's list printed apart from the names (`EmailOwner::SpelledElseOrder`), each with its
  /// group's creators
  /// (each with its call's place in the list: `_sequence=<n>`, else the calls' count)
  sequenced:        Vec<(String, Assoc, Vec<usize>, usize)>,
  /// the calls of the list so far
  sequence_calls:   usize,
  /// the last `annotate=run` contact's role, the creators then, and the run it reached
  last_run:         Option<(String, usize, Vec<usize>)>,
  /// per creator: its contacts in the order they were attached (an affiliation, by its place among the
  /// affiliations; else a contact's), for [`Resolver::distribute_upfront`]
  order:            Vec<Vec<(bool, usize)>>,
  /// authblk's notes (`Policy::thenote`): the state its `\author` and `\affil` keep
  thenote:          TheNote,
  /// the requests authblk's printed `\AB@thenote` made (creator, label): what ruling 1 reads off the PDF's marks
  thenote_requests: Vec<(usize, String)>,
  /// the labels the title's marks request (aastex's `\title{…\altaffilmark{1}}`): an entry they alone ask for is the
  /// title's note
  title_requests:   Vec<String>,
  /// the footnote marks the title shows (`\title{…\footnotemark[1]}`): a footnote text only they show is the title's
  /// note (`answer_note_texts`)
  title_fn_marks:   Vec<String>,
  /// per creator of a block: that block's queue (its `epoch`); the creators a later block replaced
  epoch_of:         Vec<Option<usize>>,
  replaced:         Vec<usize>,
  /// a note's links, and the notes they may empty, placed once every name is read (`answer_note_links`)
  note_links:       Vec<NoteLink>,
  /// the e-mail calls a document's own note makes (`ltx:pubnote`: A&A's `\offprints{B. Baker, \email{…}}`), placed
  /// once every name is read (`answer_document_notes`)
  document_emails:  Vec<DocumentEmail>,
  /// the marks a name's notes show on it (creator, label: `Ann\footnote[1]{…}` shows `∗`), what an entry led by the
  /// same `\footnotemark` answers (`answer_pending`; 1607.07280)
  note_marks:       Vec<(usize, String)>,
  /// whether the block being read prints its notes' marks (`Policy::unmarked_notes`)
  notes_marked:     bool,
  /// the requests a glued mark was parted into (creator, request), `part_glued_labels`
  parted:           Vec<(usize, String)>,
  /// the pending entries (their place in `pending`) whose label was a compound of distinct footnote symbols parted
  /// (`$^{\dagger\ddagger}$ Project lead`): the names carrying all of them (`part_glued_labels`, `answer_pending`)
  compound:         Vec<usize>,
  /// while an API call's marked entries are resolved one by one (`mark_led_pieces`)
  splitting:        bool,
  /// the last API call when it was a store's line a mark leads (its ledger index, its `_store=`, role and labels): the
  /// store's next line, unmarked, continues it (`Resolver::api`, `line:continued`)
  led_line:         Option<(usize, String, String, Vec<String>)>,
  /// the pending entries ruling 1 labels by a leading mark that today's attaches where its `annotate=` puts it, with
  /// those authors (`distribute_upfront` reads today's pile with them)
  piled:            Vec<(usize, Vec<usize>)>,
  /// the empty entries today's attaches (their ledger index, role and authors), in its pile for `distribute_upfront`
  empty_piled:      Vec<(usize, String, Vec<usize>)>,
  /// the pending entries labelled by a leading mark (their place in `pending`) with the authors their `annotate=`
  /// reaches, where today's attaches them when no name shows a mark of their kind (`answer_pending`)
  annotated:        Vec<(usize, Vec<usize>)>,
  /// the pending entries whose marked line the binding attached by its group rule (`_grouped=true` at the call)
  grouped:          Vec<usize>,
  /// the requests a `\footnotemark` made (creator, label): a footnote's mark, answered by a note, never by a labelled
  /// affiliation (`Kui Wu\inst{3\footnotemark[1]}` under `\institute{A \and B …}`, 2404.09857)
  fn_requests:      Vec<(usize, String)>,
  /// the requests the class's own request command made (creator, label): a Mark of llncs's and aa's `\inst` (role
  /// `inst`), a binding's `\lx@request@frontmatter@annotation` (`request`), an `annotations=` list — not a mark printed
  /// by hand (`$^{1}$`, `\textsuperscript{1}`, a paper's mark macro), which [`Resolver::by_hand`] reads off the rest
  /// (`answer_pending`: a `_share=authors` entry only the class's `\inst{n}` picks)
  class_requests:   Vec<(usize, String)>,
  /// the pending entries (their place in `pending`) a `_share=authors` call made (llncs's one `\institute`, no `\and`
  /// in it: every author's unless the class's `\inst{n}` picks it), and whether the entry holds a mark of its own
  shared:           Vec<(usize, bool)>,
  /// the pending entries of a numbered call (`labelseq=`) whose content holds marks of its own, which no split read
  /// (`answer_pending`: its number is no reading of those marks, so an owner found by a hand mark is not ruling 1's)
  marked_entries:   Vec<usize>,
  /// the footnote texts the document made (ledger index, flushes before it, printed mark, text), each read where a
  /// name shows its mark (`answer_note_texts`)
  note_texts:       Vec<(usize, usize, String, String)>,
  /// the entries unmarked lines continued (their source) with their lines (`continue_entry`, `Resolved::continued`)
  continued:        Vec<(Source, Vec<String>)>,
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
    index:    usize,
    note:     Event,
    owner:    Option<usize>,
    rule:     String,
    links:    Vec<Assoc>,
    fallback: bool,
  },
}

/// authblk's `\AB@thenote` state (authblk.sty:114-118, 148-151), read in call order: the `affil` counter, stepped at
/// the first `\author` after an `\affil` (`\ifnewaffil`); whether any label was given (which sets `Maxaffil` to 0,
/// so the marks show); `\c@Maxaffil` at the last block's call; each unlabelled author's number; each unlabelled
/// `\affil`'s number, with the rule tag its run placement carries.
#[derive(Default)]
struct TheNote {
  counter:  usize,
  newaffil: bool,
  labelled: bool,
  maxaffil: Option<i64>,
  run_of:   Vec<(usize, usize)>,
  affils:   Vec<(usize, String)>,
}

impl Resolver {
  fn new() -> Self {
    Resolver {
      out:              Resolved::default(),
      roles:            Vec::new(),
      sequence:         Vec::new(),
      pending:          Vec::new(),
      group:            Vec::new(),
      column:           Vec::new(),
      line_column:      None,
      entry_column:     HashMap::default(),
      block_addresses:  Vec::new(),
      sequenced:        Vec::new(),
      sequence_calls:   0,
      last_run:         None,
      order:            Vec::new(),
      thenote:          TheNote {
        newaffil: true,
        ..TheNote::default()
      },
      thenote_requests: Vec::new(),
      title_requests:   Vec::new(),
      title_fn_marks:   Vec::new(),
      epoch_of:         Vec::new(),
      replaced:         Vec::new(),
      note_links:       Vec::new(),
      document_emails:  Vec::new(),
      note_marks:       Vec::new(),
      notes_marked:     true,
      parted:           Vec::new(),
      compound:         Vec::new(),
      splitting:        false,
      led_line:         None,
      piled:            Vec::new(),
      empty_piled:      Vec::new(),
      annotated:        Vec::new(),
      grouped:          Vec::new(),
      fn_requests:      Vec::new(),
      class_requests:   Vec::new(),
      shared:           Vec::new(),
      marked_entries:   Vec::new(),
      note_texts:       Vec::new(),
      continued:        Vec::new(),
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
    self.column.push(self.line_column);
    self.out.creators.len() - 1
  }

  fn attach(&mut self, owner: Option<usize>, assoc: Assoc, note: bool) {
    match owner {
      Some(at) => {
        let creator = &mut self.out.creators[at];
        if note {
          creator.notes.push(assoc);
        } else {
          self.roles[at].push(assoc.role.clone());
          if assoc.role == "affiliation" || assoc.role == "address" {
            self.order[at].push((true, creator.affiliations.len()));
            creator.affiliations.push(assoc);
          } else {
            self.order[at].push((false, creator.contacts.len()));
            creator.contacts.push(assoc);
          }
        }
      },
      None => self.out.block.push(assoc),
    }
  }

  /// The creators today's output holds: those of no replaced `\author` call (`resolve` deletes them at the end; the
  /// post-passes that count the authors, or pick one, read these — review revfms2a2_5 finding 1; 2510.26354, 1611.08655).
  fn live(&self) -> Vec<usize> {
    (0..self.out.creators.len())
      .filter(|c| !self.replaced.contains(c))
      .collect()
  }

  /// The one candidate an address spells (`spelled_owners`), if one — none for a list of local parts the address
  /// reader could not part (`(npalmer|pwg)@dcs.warwick.ac.uk`, cs0607047: several authors' address, the block's by user
  /// ruling R10, as today's `spelled_owner` reads it — `lists_local_parts`). A name is read by its base letters, as
  /// an address spells it: the digested name's accents dropped (`Ștefan Ciobâcă` "stefan", "ciobaca", 2001.09649's
  /// `stefan.ciobaca@`), as the old reader's source reading drops the accent commands.
  fn spelled(&self, address: &str, candidates: &[usize]) -> Option<usize> {
    let names: Vec<&str> = candidates
      .iter()
      .map(|&c| self.out.creators[c].name.as_str())
      .collect();
    spelled_name(&names, address).map(|one| candidates[one])
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

/// The one of `names` an address spells (`spelled_owners`), if one — none for a list of local parts the address reader
/// could not part (`lists_local_parts`); each name read by its base letters ([`base_letters`]), as an address spells it
/// ([`Resolver::spelled`]; the writer's check of an address today's holds on the creator it spells, `writer::plan`).
pub(in crate::base_utilities::frontmatter::observer) fn spelled_name(
  names: &[&str],
  address: &str,
) -> Option<usize> {
  if lists_local_parts(address) {
    return None;
  }
  let keys: Vec<NameKeys> = names
    .iter()
    .map(|name| NameKeys::of(&text_tokens(&base_letters(name))))
    .collect();
  match spelled_owners(&keys, address).as_slice() {
    [one] => Some(*one),
    _ => None,
  }
}

/// `text` with its letters' accents dropped (canonical decomposition, the combining marks aside), and the letters
/// that decompose to none their base letter (`ł` l, `ø` o, `đ` d, `ı` i).
fn base_letters(text: &str) -> String {
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
      c => c,
    })
    .collect()
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
      } => {
        policy = conventions(binding, hook);
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
      } => resolver.api(*index, tag, role, attrs, events, policy),
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
  resolver.respell_emails();
  resolver.held_before_creators();
  resolver.sole_author_takes_the_block();
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
    out.block.retain(|a| {
      a.source
        .is_none_or(|(index, _)| !replaced_blocks.contains(&index))
    });
    let mut at = 0usize;
    out.creators.retain(|_| {
      at += 1;
      !resolver.replaced.contains(&(at - 1))
    });
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

/// `items` in source order: those with a source sorted by it (its block or API call's ledger index, then its span's
/// start — spans are positions within one block; one with no span after its block's spanned ones, review revfms2a2_8
/// finding 6), stably, in the places they held; one without a source stays where it is.
fn in_source_order(items: &mut [Assoc]) {
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
  for entry in &ledger.entries {
    match entry {
      ledger::Entry::Author(block) => {
        if let ledger::Outcome::Events { events, .. } = &block.outcome {
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
    assert!(!only_addresses("Barcelona, SPAIN. ann@x.org"));
    assert!(is_one_address("mailto:ann@x.org"));
    assert!(!is_one_address("https://x.org"));
  }
}
