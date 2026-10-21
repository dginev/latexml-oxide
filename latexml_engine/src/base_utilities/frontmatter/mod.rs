//! The front matter queue, its digestion, and its placement into the document.

use super::*;

mod queue;
pub use queue::*;
mod queue_digest;
pub use queue_digest::*;
mod place;
pub use place::*;
mod field_ink;
pub use field_ink::*;
mod titlepage_layout;
pub use titlepage_layout::*;
mod insert;
pub use insert::*;
mod relocate;
pub use relocate::*;
