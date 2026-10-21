//! Base Utilities — Perl: Base_Utility.pool.ltxml
//!
//! Core TeX Implementation for LaTeXML.
//! Also contains shared Rust helper functions (Perl: LaTeXML::Package.pm utilities).

use std::char::{REPLACEMENT_CHARACTER, decode_utf16};

use latexml_core::{
  common::{
    arena::SymHashMap,
    cleaners::clean_label,
    xml::{content_nodes, element_nodes},
  },
  document::tag::{RawFrontmatter, TagAttrs, TagContent, TagData},
};
use libxml::tree::NodeType;
use rustc_hash::FxHashSet as HashSet;
const FRONTMATTER_ELEMENTS: &[&str] = &[
  "ltx:title",
  "ltx:toctitle",
  "ltx:subtitle",
  "ltx:creator",
  "ltx:date",
  "ltx:abstract",
  "ltx:keywords",
  "ltx:classification",
  "ltx:acknowledgements",
];

/// Frontmatter tags that are "replaceable" — only one per document; a later entry
/// replaces the earlier ones instead of stacking. Ported from a later upstream
/// LaTeXML (`Base_Utility.pool.ltxml` `%ReplaceableFrontmatterTags` +
/// `\@add@frontmatter@now`), which the vendored copy predates — the vendored
/// `\lx@add@frontmatter@{now,until}` push unconditionally, so a title/abstract emitted
/// twice keeps BOTH, producing duplicated frontmatter. Surpass over the vendored Perl
/// (a forward-port of the upstream fix). OXIDIZED_DESIGN #154. Witnesses: arXiv
/// 2002.09766 (appendix `\twocolumn[\icmltitle{…}]` re-adds `ltx:title`), 2511.21969
/// (nested `{abstract}` env). Creators/notes are deliberately NOT here — multi-author
/// frontmatter must accumulate.
const REPLACEABLE_FRONTMATTER_TAGS: &[&str] = &[
  "ltx:title",
  "ltx:toctitle",
  "ltx:subtitle",
  "ltx:date",
  "ltx:abstract",
  "ltx:keywords",
];
use crate::prelude::*;

mod author_block;
mod defs;
pub use author_block::*;
mod frontmatter;
pub use frontmatter::*;
mod util_text;
pub use util_text::*;
mod fonts_defs;
pub use fonts_defs::*;
mod boxes_lists;
pub use boxes_lists::*;
mod blocks;
pub use blocks::*;
mod block_insertion;
use block_insertion::*;
mod math_cleanup;
pub use math_cleanup::*;
mod tokens;
pub use tokens::*;
mod misc;
pub use misc::*;

// The `LoadDefinitions!` body is split into the `defs` files, called here in the original order: the dump
// is a definition-ORDER snapshot, so the order is load-bearing.
LoadDefinitions!({
  defs::general::load()?;
  defs::frontmatter_api::load()?;
  defs::frontmatter_fields::load()?;
  defs::author_macros::load()?;
  defs::tags_refnums::load()?;
});

#[cfg(test)]
mod author_split_tests;
