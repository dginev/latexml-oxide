//! The frontmatter observer (frontmatter model M3, stage S2a; KERNEL_CAPABILITIES K23): what the document's
//! `\title`/`\author`-family calls hold, read as TeX prints them, kept beside the conversion — in SHADOW mode only
//! (OXIDIZED_DESIGN_DIVERGENCES #837).
//!
//! - [`ledger`]: a block per `\author`-family call (the kernel `\author`, the bindings' `\lx@add@authors*` calls) with
//!   its stamped position range, its options and its place in the frontmatter queue; the S1 stamp of each `\title`;
//!   the contact/affiliation API calls (`\lx@annotate@frontmatter`'s queue entries); the footnote texts made after a
//!   block (`\footnotetext`, read for the `\footnotemark` a name shows).
//! - [`rehearsal`]: a block's stamped tokens digested in a group of Layer-B event bindings (`\and` a separator, `\\` a
//!   break, `\thanks` a note, the contact commands contacts), through the speculative-run engine: assignments rolled
//!   back, diagnostics held, output muted, the frontmatter frozen; the digested list handed to the lexer, then
//!   discarded.
//! - [`events`]: the digested list read into events (names, marks, separators, breaks, contacts, notes, layout), each
//!   with its span of the field's positions (`fieldProv`/`fieldSerial`, core `gullet::field_cover_of`) and its ink.
//! - [`marks`]: whether a formula in a name or an affiliation is a mark, read from its digested XMath (as digested and
//!   as parsed), the one reading of the writer's view, `ab_audit fmshadow`, `relocate::shown_marks` and
//!   `name_text::is_name_mark` (frontmatter refactor stage 1).
//! - [`resolver`] (S2a.2): the ledger's blocks and API calls read into the frontmatter tuples (creators with their
//!   names, affiliations, contacts and notes; the block's own), each association with the rule that decided it, under
//!   the family's conventions from [`policy`] (each binding declares its own at load; read keyed on the binding owning
//!   `\author` and the hook); written into the dump, compared with the XML by `ab_audit fmshadow`.
//! - [`shadow`]: the switch, the runtime environment variable `LATEXML_FM_SHADOW=1`, under which the flush
//!   (`digest_front_matter`) rehearses and lexes each block and the CLI writes `<dest>.fmshadow.json`. Without it no
//!   part of the observer runs, and with it the conversion's output is the same: the observer decides nothing yet
//!   (the resolver is stage S2a.2) — but for a family whose declared conventions are `written` (FM S2b):
//! - [`writer`] (S2b): once today's frontmatter is in the document (`insert_frontmatter`), the paper of a written
//!   family is compared with the resolver's reading, and where every difference is one of the writer's operations —
//!   an entry given to the authors the reading gives it, one call's marked entries cut apart, a legend made a note,
//!   a url printing an address made an e-mail — those are written, from the tokens the document queued; any other
//!   difference leaves the paper as today's.

use super::*;

pub(in crate::base_utilities) mod events;
pub(in crate::base_utilities) mod ledger;
pub(in crate::base_utilities) mod marks;
pub(in crate::base_utilities) mod policy;
pub(in crate::base_utilities) mod rehearsal;
pub(in crate::base_utilities) mod resolver;
pub(in crate::base_utilities) mod shadow;
pub(in crate::base_utilities) mod writer;

// (for `ab_audit fmshadow`, which re-resolves a dump's events offline; prefixed, as the prelude re-exports these)
pub use events::{Event as FmEvent, EventKind as FmEventKind, SepKind as FmSepKind};
pub(in crate::base_utilities) use ledger::{
  field_stamped, note_field_stamp, record_api_call, record_author_call,
};
pub use ledger::{note_binding_load, record_note_text as fm_record_note_text};
// (the one reading of a formula's mark-ness, for `ab_audit fmshadow`'s view of the final XML)
pub use marks::{
  FmMathNode, FormulaMarks as FmFormulaMarks, glued_in_a_word as fm_glued_in_a_word,
  printed_children as fm_printed_children, read_formula as fm_read_formula,
};
// (for the bindings, which declare their conventions at load, and `ab_audit fmshadow --re-resolve`)
pub use policy::{
  EmailOwner as FmEmailOwner, KERNEL as FM_KERNEL_CONVENTIONS, Policy as FmConventions,
  TitleMarks as FmTitleMarks, conventions_among as fm_conventions_among,
  declare_frontmatter_conventions,
};
pub use resolver::{
  Assoc as FmAssoc, CHANGE_TAGS as FM_CHANGE_TAGS, Input as FmInput, RCreator as FmCreator,
  Resolved as FmResolved, addresses_in as fm_addresses_in, creator_call as fm_creator_call,
  is_change_rule as fm_is_change_rule, resolve as fm_resolve, resolve_with as fm_resolve_with,
};
pub(in crate::base_utilities) use shadow::shadow_flush;
pub use shadow::{
  fm_shadow_enabled, fm_shadow_json, reset_fm_shadow, set_fm_shadow_override, write_fm_shadow,
};
pub(in crate::base_utilities) use writer::write_observed_frontmatter;
