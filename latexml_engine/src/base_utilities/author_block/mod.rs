//! Author-block parsing: the `\author` line split into names, marks, affiliations, e-mails, links and notes.

use super::*;

mod affiliations;
pub use affiliations::*;
mod email_lines;
pub(super) use email_lines::*;
mod preprocess;
pub(super) use preprocess::*;
mod names_split;
pub use names_split::*;
mod space_parted;
pub(super) use space_parted::*;
mod separators;
pub use separators::*;
mod marks;
pub use marks::*;
mod rows;
pub use rows::*;
mod row_emails;
pub use row_emails::*;
mod symbols;
pub(super) use symbols::*;
mod notes;
pub use notes::*;
mod name_text;
pub use name_text::*;
mod email_owners;
pub(super) use email_owners::*;
mod links;
pub(super) use links::*;
mod email_block_calls;
pub use email_block_calls::*;
mod line_kinds;
pub(super) use line_kinds::*;
mod dispatch;
pub(super) use dispatch::*;
