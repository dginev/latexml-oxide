//! The old reader's text rules on printed ink and its token primitives, moved unchanged from `author_block/` into the
//! one engine's own library (user ruling 2026-11-08 "knowledge in, driver out, one pass";
//! docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md, the row of that name). One submodule per source file, so that two same-named private
//! rules stay apart (`links::printed_words`, `notes::printed_words`); each re-exported as `author_block/mod.rs` did.

use super::*;

mod collaborations;
pub(in crate::base_utilities) use collaborations::*;
mod email_block_calls;
pub(in crate::base_utilities) use email_block_calls::*;
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
// (and the IEEEtran binding's names continued on a line below them: ieeetran_cls.rs; Z18m, from `author_block/name_text.rs`)
pub(in crate::base_utilities) use name_text::*;
pub use name_text::{grade_words, names_continue};
mod names_split;
pub(in crate::base_utilities) use names_split::*;
mod notes;
pub use notes::*;
mod preprocess;
pub(in crate::base_utilities) use preprocess::*;
mod rows;
// (authblk's `\author` gates read an author line by them: authblk_sty.rs `has_separated_names`, `lines_of_names_before_affil`;
// Z18m, from `author_block/rows.rs`)
pub(in crate::base_utilities) use rows::*;
pub use rows::{
  holds_a_macro_printing_a_link, line_breaks_as_name_separators, lines_under_names_read_as_names,
};
mod separators;
// (authblk's `\author` gate too; Z18m, from `author_block/separators.rs`)
pub use separators::has_author_block_separator;
pub(in crate::base_utilities) use separators::*;
// (names parted by horizontal space alone, whose one caller is `rows::lines_under_names_read_as_names`; Z18m, from
// `author_block/space_parted.rs`)
mod space_parted;
mod symbols;
pub(in crate::base_utilities) use symbols::*;
// (a class's title code read where it reads `\@author`, ERR2's; Z18m, from `defs/frontmatter_fields/names.rs`)
pub(in crate::base_utilities) mod title_code;
