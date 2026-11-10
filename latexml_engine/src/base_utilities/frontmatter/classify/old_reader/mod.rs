//! The old reader's last items, moved unchanged from `author_block/` by Z18m, which deleted it: the mark readers
//! authblk's `\author` reads a name by, authblk's gates on an author line's rows, separators and space-parted names, the
//! IEEEtran binding's names continued and lines of notes, and what only they call. They sit in files of their own,
//! apart from the one engine's library (`classify/`'s other files), so that the e0 fence counts them as the old
//! reader's (`perfect_kernel_batche0.rs` `MOVED_OLD_READER_FILES`, review revZ18m B1); each is deleted or rerouted by
//! Option C (P2-P4). One submodule per deleted file, each re-exported as `author_block/mod.rs` did.

use super::*;

mod collaborations;
pub(in crate::base_utilities) use collaborations::*;
mod marks;
// (authblk's `\author` reads a name by them: authblk_sty.rs)
pub use marks::*;
mod name_text;
// (the IEEEtran binding's names continued on a line below them: ieeetran_cls.rs)
pub use name_text::*;
mod names_split;
pub(in crate::base_utilities) use names_split::*;
mod notes;
// (the IEEEtran binding's lines of notes: ieeetran_cls.rs)
pub use notes::*;
mod rows;
// (authblk's `\author` gates read an author line by them: authblk_sty.rs `has_separated_names`, `lines_of_names_before_affil`)
pub use rows::*;
mod separators;
// (authblk's `\author` gate too)
pub use separators::*;
// (names parted by horizontal space alone, whose one caller is `rows::lines_under_names_read_as_names`)
mod space_parted;
