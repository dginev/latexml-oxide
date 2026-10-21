//! The `LoadDefinitions!` body, split verbatim into parts that `super::load_definitions` runs in the original order.

use super::*;

pub(super) mod author_macros;
pub(super) mod frontmatter_api;
pub(super) mod frontmatter_fields;
pub(super) mod general;
pub(super) mod tags_refnums;
