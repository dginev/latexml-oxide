//! The observer's ledger: one block per `\author`-family call, the S1 stamp of each `\title` call, and the
//! contact/affiliation API calls, in call order (OXIDIZED_DESIGN_DIVERGENCES #837). Recorded only under the shadow
//! switch ([`super::shadow::fm_shadow_enabled`]) and never while the frontmatter is frozen (a replayed class setter, a
//! speculative title-page run, the observer's own rehearsal: those calls are not the document's).

use std::cell::RefCell;

use latexml_core::{
  gullet::{InputMark, input_mark, input_unread_since},
  token::{FIELD_PROV_POS_MASK, field_prov},
};

use super::{events::Event, *};

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
  /// the binding policy the call runs under: `kernel` for the kernel `\author`, else the hook's own family
  pub policy:         String,
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

/// An entry of the ledger.
#[derive(Debug, Clone)]
pub(in crate::base_utilities) enum Entry {
  Author(Box<AuthorBlock>),
  Title(TitleStamp),
  Api(ApiCall),
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
  let policy = if hook == "\\lx@add@authors@adaptive" {
    "kernel"
  } else {
    hook.trim_start_matches("\\lx@")
  };
  let block = AuthorBlock {
    index: 0,
    hook,
    policy: policy.to_string(),
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

/// Record a queued creator, contact or note of the frontmatter API (`queue_front_matter`), under the shadow switch
/// only: a binding's single-author call (`\lx@add@creator`, `\lx@add@author`: authblk's unlabelled `\author{Bob}`) is
/// an API call too, unless a block's own expansion made it.
pub(in crate::base_utilities) fn record_api_call(tag: &str, role: &str) {
  if !matches!(tag, "ltx:creator" | "ltx:contact" | "ltx:note") || !records_now() {
    return;
  }
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
      role: role.to_string(),
      epoch: ledger.flushes,
      queue_position,
      after_block,
      inside,
    }));
  });
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
