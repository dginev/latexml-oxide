//! The observer's ledger: one block per `\author`-family call, the S1 stamp of each `\title` call, and the
//! contact/affiliation API calls, in call order (OXIDIZED_DESIGN_DIVERGENCES #837). Recorded only under the shadow
//! switch ([`super::shadow::fm_shadow_enabled`]) and never while the frontmatter is frozen (a replayed class setter, a
//! speculative title-page run, the observer's own rehearsal: those calls are not the document's).

use std::{cell::RefCell, rc::Rc};

use latexml_core::{
  definition::{Definition, origin::DefinitionOrigin},
  gullet::{InputMark, input_mark, input_unread_since},
  token::{FIELD_PROV_POS_MASK, field_prov},
};

use super::{
  events::{Event, lex_events},
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
  /// Rehearsed and lexed: its events, and the errors and warnings the rehearsal held.
  Events {
    events:   Vec<Event>,
    errors:   usize,
    warnings: usize,
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
  /// the policy entry the call runs under (`policy::policy_for` of `binding` and `hook`: `kernel` for every family
  /// without one)
  pub policy:         String,
  /// the binding the policy is keyed on ([`policy_binding`]): the `\author` owner, or the document class's binding
  /// when that is the kernel's — with `hook`, the key of the policy table (`policy::policy_for`; S2a.2, notes decision 2)
  pub binding:        String,
  /// the binding owning the `\author` meaning at the call ([`author_binding`]): a compiled binding's file name
  /// (`revtex4_support.sty`), else `kernel`, `document`, `raw` or `fallback` by its definition origin
  pub owner:          String,
  /// the value of the policy's state counter at the call (`policy::Policy::state_counter`), when it has one defined
  pub counter:        Option<i64>,
  /// whether the call replaces the authors before it (`\lx@clear@frontmatter{ltx:creator}`)
  pub replace:        bool,
  /// whether the IJCAI sectioned split applies
  pub sectioned:      bool,
  /// a per-author binding's cautious name split
  pub cautious:       bool,
  /// the keyvals the call hands each author (revtex `annotations=`)
  pub keyvals:        Option<String>,
  /// the frontmatter flushes (`digest_front_matter`) before the call: the queue the position is in (each flush
  /// empties it)
  pub epoch:          usize,
  /// the frontmatter queue's length at the call, and the authors queued then
  pub queue_position: usize,
  pub authors_queued: usize,
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

thread_local! {
  /// The compiled binding that defined the current `\author` meaning, with that meaning itself ([`note_binding_load`];
  /// held, so its identity is `Rc::ptr_eq`'s and no later definition can take its address), under the shadow switch
  /// only.
  static AUTHOR_OWNER: RefCell<Option<(String, Rc<dyn Definition>)>> = const { RefCell::new(None) };
  /// The document class's compiled bindings: the first class binding loaded outside any other binding's load, and every
  /// binding its load loaded (`report.cls`; iopart.cls's `iopart_support.sty`), in load order; and how deep the loads
  /// nest now, with whether the outermost is that class's.
  static CLASS_BINDINGS: RefCell<ClassBindings> = const { RefCell::new((Vec::new(), 0, false)) };
}

/// The current `\author` meaning, if any (what [`note_binding_load`] compares, by `Rc::ptr_eq`).
fn author_meaning() -> Option<Rc<dyn Definition>> {
  lookup_definition(&T_CS!("\\author")).ok().flatten()
}

/// Whether two `\author` meanings are the same definition (or both absent).
fn same_meaning(a: Option<&Rc<dyn Definition>>, b: Option<&Rc<dyn Definition>>) -> bool {
  match (a, b) {
    (Some(a), Some(b)) => Rc::ptr_eq(a, b),
    (None, None) => true,
    _ => false,
  }
}

/// Run a compiled binding's loader (`name`.`extension`, the binding's file: `revtex4_support.sty`), noting it as the owner of the
/// `\author` meaning when its load changed it and no binding it loaded did (revtex4-2.cls's binding loads
/// revtex4_support.sty, whose `\author` it is): the binding half of the policy key (`author_binding`). Under the
/// shadow switch only; otherwise just the load.
pub fn note_binding_load<R>(name: &str, extension: &str, load: impl FnOnce() -> R) -> R {
  if !fm_shadow_enabled() {
    return load();
  }
  let before = author_meaning();
  let owner_before = AUTHOR_OWNER.with(|o| o.borrow().as_ref().map(|(_, defn)| Rc::clone(defn)));
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
  let after = author_meaning();
  let owner_after = AUTHOR_OWNER.with(|o| o.borrow().as_ref().map(|(_, defn)| Rc::clone(defn)));
  if let Some(after) = after
    && !same_meaning(before.as_ref(), Some(&after))
    && same_meaning(owner_after.as_ref(), owner_before.as_ref())
  {
    // (a pool is LaTeXML's own kernel: `kernel`, so a binding that loads it — latexml.sty — does not own its `\author`)
    let owner = if extension == "pool" {
      "kernel".to_string()
    } else {
      s!("{name}.{extension}")
    };
    AUTHOR_OWNER.with(|o| *o.borrow_mut() = Some((owner, after)));
  }
  value
}

/// Forget the `\author` owner and the class's bindings (a conversion's start).
pub(in crate::base_utilities) fn reset_author_owner() {
  AUTHOR_OWNER.with(|o| *o.borrow_mut() = None);
  CLASS_BINDINGS.with(|c| *c.borrow_mut() = (Vec::new(), 0, false));
}

/// The binding a block's policy is keyed on (S2a.2 r2, review revfms2a2's answer to question 3): the binding owning
/// the `\author` meaning that makes the `\lx@add@authors*` call ([`author_binding`]); when that is the kernel's, the
/// document class's binding with a policy entry (iopart.cls's `iopart_support.sty`, `jpconf.cls`, `report.cls`) — never
/// over a package owning `\author` (authblk's stays authblk's).
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
pub(in crate::base_utilities) fn author_binding() -> String {
  let Some(meaning) = author_meaning() else {
    return "kernel".to_string();
  };
  if let Some(name) = AUTHOR_OWNER.with(|o| {
    o.borrow()
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
  let block = AuthorBlock {
    index: 0,
    hook,
    policy: policy.to_string(),
    binding,
    owner,
    counter,
    replace,
    sectioned,
    cautious,
    keyvals: keyvals.map(|kv| kv.to_string()),
    epoch: 0,
    queue_position: queue_length(),
    authors_queued: queued_author_count(),
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
  // (and the title, whose marks request labelled entries too — aastex's `\title{…\altaffilmark{1}}` and
  // `\altaffiltext{1}{Based on observations …}`, the title's note: 0801.1670)
  if !matches!(
    tag,
    "ltx:creator" | "ltx:contact" | "ltx:note" | "ltx:title"
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
          ArgWrap::Tokens(tokens) if key == "label" => s!("{key}={}", label_text(tokens)),
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
  let parent = match command.unlist_ref().as_slice() {
    [first, rest @ ..] if *first == T_CS!("\\lx@annotate@frontmatter@now") => {
      first_group(rest).to_string().trim().to_string()
    },
    _ => String::new(),
  };
  let queue_position = queue_length();
  with_ledger(|ledger| {
    let index = ledger.entries.len();
    let mut after_block = None;
    let mut inside = false;
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
      if block.window_open {
        if input_unread_since(&block.mark) {
          inside = true;
        } else {
          block.window_open = false;
        }
      }
    }
    ledger.entries.push(Entry::Api(ApiCall {
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
    }));
  });
}

/// Record a footnote's text (`\lx@notetext`'s whatsit, once digested: `\footnotetext[<n>]{…}`), under the shadow switch
/// only, after an author block: its printed mark, its counter value and its body's events (the body is digested for
/// real, so nothing is rehearsed — only read).
pub fn record_note_text(whatsit: &Whatsit) {
  if !records_now()
    || !with_ledger(|ledger| ledger.entries.iter().any(|e| matches!(e, Entry::Author(_))))
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
