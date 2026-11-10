//! Author-block parsing: the `\author` line split into names, marks, affiliations, e-mails, links and notes.

use super::*;

mod email_lines;
// (authblk's `\affil` reads a line with its empty marks gone: authblk_sty.rs `holds_a_mark_authors_carry`)
pub use email_lines::drop_empty_marks;
mod preprocess;
pub(super) use preprocess::*;
mod names_split;
pub(super) use names_split::*;
mod collaborations;
pub(super) use collaborations::*;
mod space_parted;
pub(super) use space_parted::*;
mod separators;
pub use separators::*;
mod marks;
pub use marks::*;
mod rows;
pub use rows::*;
mod notes;
pub use notes::*;
mod name_text;
pub use name_text::*;
