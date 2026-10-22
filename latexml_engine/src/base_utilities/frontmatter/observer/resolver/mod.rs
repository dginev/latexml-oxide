//! The observer's resolver (Layer C of the design note's §2; OXIDIZED_DESIGN_DIVERGENCES #837): a document's ledger —
//! each author block's events and the frontmatter API calls after them — read into the tuples `ab_audit`'s `front.rs`
//! extracts from the XML (each creator's name, affiliations, contacts and notes, and the author block's own), each
//! association with the rule that decided it. Class-independent; a family's conventions come from its policy entry
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
//! 9. **Ruling 4**: a note whose content all became contacts is dropped; an empty `\thanks{}` is kept.
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
  events::{Event, EventKind, SepKind},
  policy::{EmailOwner, Policy, policy_for},
  *,
};

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
  pub creators: Vec<RCreator>,
  /// what no creator owns: the author block's own (a trailing nameless creator in the XML, OD #159)
  pub block:    Vec<Assoc>,
  /// what the rulings dropped: a note whose content all became contacts (ruling 4), with its text — kept for the
  /// comparison, which reads today's note there as the ruling's change
  pub dropped:  Vec<Assoc>,
  /// per block (ledger index): the policy entry it ran under
  pub policies: Vec<(usize, &'static str)>,
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
}

mod api;
mod blocks;
mod labels;
mod lines;
mod post;

// (named, as the prelude has its own of these names: the resolver's shadow them)
pub use labels::{addresses_in, mark_labels};
use labels::{text_tokens, *};
use lines::{reads_as_names, *};

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
  /// per creator of a block: that block's queue (its `epoch`); the creators a later block replaced
  epoch_of:         Vec<Option<usize>>,
  replaced:         Vec<usize>,
  /// a note's e-mails (the creator the note is on, the contact), placed once every name is read
  note_emails:      Vec<(Option<usize>, Assoc)>,
  /// the requests a glued mark was parted into (creator, request), `part_glued_labels`
  parted:           Vec<(usize, String)>,
  /// while an API call's marked entries are resolved one by one (`mark_led_pieces`)
  splitting:        bool,
  /// the pending entries ruling 1 labels by a leading mark that today's attaches where its `annotate=` puts it, with
  /// those authors (`distribute_upfront` reads today's pile with them)
  piled:            Vec<(usize, Vec<usize>)>,
  /// the empty entries today's attaches (their ledger index, role and authors), in its pile for `distribute_upfront`
  empty_piled:      Vec<(usize, String, Vec<usize>)>,
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
  fn new_creator(
    &mut self,
    name: String,
    rule: &str,
    source: usize,
    requests: Vec<String>,
    group: usize,
  ) -> usize {
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

  /// The one candidate an address spells (`spelled_owners`), if one.
  fn spelled(&self, address: &str, candidates: &[usize]) -> Option<usize> {
    let keys: Vec<NameKeys> = candidates
      .iter()
      .map(|&c| NameKeys::of(&text_tokens(&self.out.creators[c].name)))
      .collect();
    match spelled_owners(&keys, address).as_slice() {
      [one] => Some(candidates[*one]),
      _ => None,
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

/// Resolve a document's ledger (its inputs in call order).
pub fn resolve(inputs: &[Input]) -> Resolved {
  let mut resolver = Resolver {
    out:              Resolved::default(),
    roles:            Vec::new(),
    sequence:         Vec::new(),
    pending:          Vec::new(),
    group:            Vec::new(),
    sequenced:        Vec::new(),
    sequence_calls:   0,
    last_run:         None,
    order:            Vec::new(),
    thenote:          TheNote {
      newaffil: true,
      ..TheNote::default()
    },
    thenote_requests: Vec::new(),
    epoch_of:         Vec::new(),
    replaced:         Vec::new(),
    note_emails:      Vec::new(),
    parted:           Vec::new(),
    splitting:        false,
    piled:            Vec::new(),
    empty_piled:      Vec::new(),
  };
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
        epoch,
        events,
      } => {
        policy = policy_for(binding, hook);
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
    }
  }
  resolver.thenote_finish();
  // (today's grid, `distribute_upfront_contacts`, runs before `relocate_annotations` answers the labels: the contacts
  // a label places are no part of the pile — 2311.04097's two `\email`s under `\author{A$^1$ and B$^2$}` with their
  // affiliations marked; the amsart upfront grids 1302.1274, 1610.05835, 2204.13910, 2205.05923, 2507.07513)
  resolver.distribute_upfront();
  resolver.answer_note_emails();
  resolver.answer_pending();
  resolver.answer_sequence();
  resolver.held_before_creators();
  resolver.sole_author_takes_the_block();
  let mut out = resolver.out;
  if !resolver.replaced.is_empty() {
    let mut at = 0usize;
    out.creators.retain(|_| {
      at += 1;
      !resolver.replaced.contains(&(at - 1))
    });
  }
  out
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
            epoch:   block.epoch,
            events:  events.clone(),
          });
        }
      },
      ledger::Entry::Api(call) if !call.inside && creator_call(&call.tag, &call.parent) => inputs
        .push(Input::Api {
          index:  call.index,
          tag:    call.tag.clone(),
          role:   call.role.clone(),
          attrs:  call.attrs.clone(),
          events: call.events.clone(),
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
    assert_eq!(label_owners(&creators, "author:2"), [1]);
    assert_eq!(label_owners(&creators, "author:9"), Vec::<usize>::new());
    assert_eq!(label_owners(&creators, "1"), [0]);
    assert_eq!(label_owners(&creators, "label1"), [1]);
    assert_eq!(label_owners(&creators, "affiliation:2"), [2]);
    assert_eq!(label_owners(&creators, "institute:1"), [0]);
  }

  /// Item 2: a `fuzzy:` label and a `\ref` key stay whole; the name's letters skip its marks.
  #[test]
  fn fuzzy_labels_and_their_letters() {
    assert_eq!(mark_labels("fuzzy:Luca Passalacqua (\\Envelope) "), [
      "fuzzy:Luca Passalacqua (\\Envelope)"
    ]);
    assert_eq!(mark_labels("*1, a"), ["*", "1", "a"]);
    assert_eq!(
      fold_name(&tex_letters("Luca Passalacqua (\\Envelope)")),
      "lucapassalacqua"
    );
    assert_eq!(
      fold_name(&tex_letters("Jean-Ren\\'e Chazottes\\inst{2}")),
      "jeanrenechazottes"
    );
  }

  /// FM S2a.2 r3 item 1c: a glued label as the known labels written together, the fewest pieces; a symbol run on its
  /// own only where symbols may stand alone; none when no piece is known.
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
  }

  #[test]
  fn address_lines() {
    assert!(only_addresses("E-mail: {ann,bob}@x.org, cat@y.org"));
    assert!(!only_addresses("Barcelona, SPAIN. ann@x.org"));
    assert!(is_one_address("mailto:ann@x.org"));
    assert!(!is_one_address("https://x.org"));
  }
}
