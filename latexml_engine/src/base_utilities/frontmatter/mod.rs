//! The front matter queue, its digestion, and its placement into the document.

use super::*;

// (the old reader's text rules and token primitives, moved: MV1)
mod classify;
pub use classify::*;
mod queue;
pub use queue::*;
mod queue_digest;
pub use queue_digest::*;
mod queue_reading;
pub use queue_reading::*;
mod place;
pub use place::*;
mod label_refs;
pub use label_refs::*;
mod field_ink;
pub use field_ink::*;
mod titlepage_layout;
pub use titlepage_layout::*;
mod insert;
pub use insert::*;
mod address_lists;
use address_lists::*;
mod title_page_check;
pub use title_page_check::*;
mod cover_order;
pub use cover_order::*;
mod relocate;
pub use relocate::*;
mod shared_row;
pub(in crate::base_utilities) use shared_row::*;
pub(in crate::base_utilities) mod observer;
pub use observer::*;
