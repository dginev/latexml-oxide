//! The frontmatter observer (frontmatter model M3, stage S2a; KERNEL_CAPABILITIES K23): what the document's
//! `\title`/`\author`-family calls hold, read as TeX prints them, kept beside the conversion — in SHADOW mode only
//! (OXIDIZED_DESIGN_DIVERGENCES #837).
//!
//! - [`ledger`]: a block per `\author`-family call (the kernel `\author`, the bindings' `\lx@add@authors*` calls) with
//!   its stamped position range, its options and its place in the frontmatter queue; the S1 stamp of each `\title`;
//!   the contact/affiliation API calls (`\lx@annotate@frontmatter`'s queue entries).
//! - [`rehearsal`]: a block's stamped tokens digested in a group of Layer-B event bindings (`\and` a separator, `\\` a
//!   break, `\thanks` a note, the contact commands contacts), through the speculative-run engine: assignments rolled
//!   back, diagnostics held, output muted, the frontmatter frozen; the digested list handed to the lexer, then
//!   discarded.
//! - [`events`]: the digested list read into events (names, marks, separators, breaks, contacts, notes, layout), each
//!   with its span of the field's positions (`fieldProv`/`fieldSerial`, core `gullet::field_cover_of`) and its ink.
//! - [`resolver`] (S2a.2): the ledger's blocks and API calls read into the frontmatter tuples (creators with their
//!   names, affiliations, contacts and notes; the block's own), each association with the rule that decided it, under
//!   the family's conventions from [`policy`] (keyed on the binding owning `\author` and the hook); written into the
//!   dump, compared with the XML by `ab_audit fmshadow`.
//! - [`shadow`]: the switch, the runtime environment variable `LATEXML_FM_SHADOW=1`, under which the flush
//!   (`digest_front_matter`) rehearses and lexes each block and the CLI writes `<dest>.fmshadow.json`. Without it no
//!   part of the observer runs, and with it the conversion's output is the same: the observer decides nothing yet
//!   (the resolver is stage S2a.2).

use super::*;

pub(in crate::base_utilities) mod events;
pub(in crate::base_utilities) mod ledger;
pub(in crate::base_utilities) mod policy;
pub(in crate::base_utilities) mod rehearsal;
pub(in crate::base_utilities) mod resolver;
pub(in crate::base_utilities) mod shadow;

// (for `ab_audit fmshadow`, which re-resolves a dump's events offline; prefixed, as the prelude re-exports these)
pub use events::{Event as FmEvent, EventKind as FmEventKind, SepKind as FmSepKind};
pub use ledger::note_binding_load;
pub(in crate::base_utilities) use ledger::{
  field_stamped, note_field_stamp, record_api_call, record_author_call,
};
pub use resolver::{
  Assoc as FmAssoc, Input as FmInput, RCreator as FmCreator, Resolved as FmResolved,
  creator_call as fm_creator_call, resolve as fm_resolve,
};
pub(in crate::base_utilities) use shadow::shadow_flush;
pub use shadow::{
  fm_shadow_enabled, fm_shadow_json, reset_fm_shadow, set_fm_shadow_override, write_fm_shadow,
};
