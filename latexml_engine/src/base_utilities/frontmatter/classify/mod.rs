//! The old reader's text rules on printed ink and its token primitives, moved unchanged from `author_block/` into the
//! one engine's own library (user ruling 2026-11-08 "knowledge in, driver out, one pass";
//! docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md, the row of that name). One submodule per source file, so that two same-named private
//! rules stay apart (`links::printed_words`, `notes::printed_words`); each re-exported as `author_block/mod.rs` did.

use super::*;

mod collaborations;
pub(in crate::base_utilities) use collaborations::*;
mod email_block_calls;
pub(in crate::base_utilities) use email_block_calls::*;
// (the cut of an entry's class e-mail calls the emitter writes, and the text rules it reads a call by: moved from
// `shared_row/` with the old row split's deletion, P27b)
mod entry_starts;
pub(in crate::base_utilities) use entry_starts::*;
mod entry_emails;
pub(in crate::base_utilities) use entry_emails::{
  entry_email_cut, is_call_separator, is_whole_address, shared_call_item,
};
// (`classify::email_lines`, the text rules on an e-mail's printed ink; not `frontmatter_api::email_lines`, the API's line
// readers)
mod email_lines;
pub(in crate::base_utilities) use email_lines::*;
mod email_owners;
// (the IEEEtran binding reads a bare run between its blocks by it: ieeetran_cls.rs `wrap_bare_author_block_text`)
pub use email_owners::prints_nothing;
pub(in crate::base_utilities) use email_owners::*;
mod links;
pub(in crate::base_utilities) use links::*;
mod marks;
pub use marks::*;
mod name_text;
// (the IEEEtran binding reads an italic grade before a stop by it: ieeetran_cls.rs `grade_text`)
pub use name_text::grade_words;
pub(in crate::base_utilities) use name_text::*;
mod names_split;
pub(in crate::base_utilities) use names_split::*;
mod notes;
pub use notes::*;
// (the old reader's last items, moved unchanged from `author_block/` by Z18m into files of their own, so the e0 fence
// tells them from this library: `old_reader/mod.rs`)
mod old_reader;
pub use old_reader::*;
mod preprocess;
pub(in crate::base_utilities) use preprocess::*;
mod rows;
pub(in crate::base_utilities) use rows::*;
mod separators;
pub(in crate::base_utilities) use separators::*;
mod symbols;
pub(in crate::base_utilities) use symbols::*;
// (a class's title code read where it reads `\@author`, ERR2's; Z18m, from `defs/frontmatter_fields/names.rs`)
pub(in crate::base_utilities) mod title_code;
