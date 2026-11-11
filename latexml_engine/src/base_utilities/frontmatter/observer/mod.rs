//! The frontmatter observer (frontmatter model M3; KERNEL_CAPABILITIES K23): what the document's `\title`/`\author`-family
//! calls hold, read as TeX prints them, which the emitter writes Perl's own API calls from (OXIDIZED_DESIGN_DIVERGENCES
//! #837, #980).
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
//!   as parsed), the one reading of the content check's view of the placed output, `relocate::shown_marks` and
//!   `name_text::is_name_mark` (frontmatter refactor stage 1).
//! - [`resolver`] (S2a.2): the ledger's blocks and API calls read into the frontmatter tuples (creators with their
//!   names, affiliations, contacts and notes; the block's own), each association with the rule that decided it, under
//!   the family's conventions from [`policy`] (each binding declares its own at load; read keyed on the binding owning
//!   `\author` and the hook).
//! - [`reading`]: the flush's part — every real flush (`digest_front_matter`) rehearses and lexes each block not
//!   rehearsed yet — and the ledger's JSON dump the guards read the reading by, in process.
//! - [`emitter`]: the emitter (`emitter::emit`, OD #980), which replaces the flush's queued creators and their annotations
//!   by Perl's calls built from the reading, its vocabulary and cuts, and the content check on the placed output. (The S2b
//!   writer that rewrote placed frontmatter after `insert_frontmatter`, and the census switch `LATEXML_FM_SHADOW`, were
//!   deleted in DEL4.)
//! - [`separators`]: a binding's query on an argument's `\and`-family separators by meaning (`argument_separators`,
//!   `\lx@fm@separators`; OD #995).

use super::*;

pub(in crate::base_utilities) mod emitter;
pub(in crate::base_utilities) mod events;
pub(in crate::base_utilities) mod field_notes;
pub(in crate::base_utilities) mod ledger;
pub(in crate::base_utilities) mod marks;
pub(in crate::base_utilities) mod policy;
pub(in crate::base_utilities) mod reading;
pub(in crate::base_utilities) mod rehearsal;
pub(in crate::base_utilities) mod resolver;
pub(in crate::base_utilities) mod separators;

// (for the guards, which read a dump's events and the resolver's tuples, and the types of a public event's fields;
// prefixed, as the prelude re-exports these)
// (the emitter's vocabulary, moved unchanged, for its callers in `frontmatter/` and `defs/`: the plan's row "Knowledge in, driver out, one pass")
pub(in crate::base_utilities) use emitter::{
  EMITTING, SETTERS_RECOVERED, block_creator_written, calls::*, check_placed_content,
  emit_commands, emitted, setters_recovered, take_digested,
};
pub use events::{
  BreakKind as FmBreakKind, EmailMeaning as FmEmailMeaning, Event as FmEvent,
  EventKind as FmEventKind, MarkBase as FmMarkBase, MarkFormula as FmMarkFormula,
  PrintedLabel as FmPrintedLabel, SepKind as FmSepKind,
};
pub use ledger::{
  author_tail_declared as fm_author_tail_declared, note_binding_load,
  record_note_text as fm_record_note_text,
};
pub(in crate::base_utilities) use ledger::{
  field_stamped, note_field_stamp, record_api_call, record_author_body_clear,
  record_author_body_end, record_author_call, record_clear, record_cut_copy,
};
// (the one reading of a formula's mark-ness, for `relocate` and `classify`)
pub(in crate::base_utilities) use marks::{
  FormulaMarks as FmFormulaMarks, read_formula as fm_read_formula,
};
// (for the bindings, which declare their conventions at load)
pub use policy::{
  AuthorPage as FmAuthorPage, EmailOwner as FmEmailOwner, KERNEL as FM_KERNEL_CONVENTIONS,
  Policy as FmConventions, TitleMarks as FmTitleMarks, TitlePageDecl as FmTitlePageDecl,
  declare_frontmatter_conventions,
};
pub(in crate::base_utilities) use reading::read_at_flush;
pub use reading::{fm_reading_json, reset_fm_reading};
// (the old reader's owner logic, moved unchanged, for its callers in `frontmatter/` and `defs/`: the plan's row "Knowledge in, driver out, one pass")
pub(in crate::base_utilities) use resolver::owners::*;
// (for the guards, which resolve a block's input again)
pub use resolver::{Input as FmInput, Resolved as FmResolved, resolve as fm_resolve};
// (for the bindings, which ask what an argument's separators mean: Option C P2, OXIDIZED_DESIGN_DIVERGENCES #995)
pub use separators::{
  ArgSeparator as FmArgSeparator, SepMeaning as FmSepMeaning, argument_separators,
  kernel_separator_runs, split_at_kernel_separators,
};
