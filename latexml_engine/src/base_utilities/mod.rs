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

/// Frontmatter tags that are "replaceable" — a later entry of the same name and language replaces the earlier ones
/// instead of stacking — and, with `frontmatter_seal_key`, the tags an entry after the document body began is a
/// supplement's of. Ported from upstream LaTeXML's `%ReplaceableFrontmatterTags` (bb048fc1, #2573), which upstream
/// later removed (Bruce Miller's frontmatter refactor 23f3acfa, #2767: only `\lx@clear@frontmatter` removes duplicated
/// entries, from the queue of the ones not yet digested); the vendored copy is that HEAD, whose
/// `\lx@add@frontmatter@{now,until}` push unconditionally. Kept for the titles and dates: a re-emission after the
/// flush (arXiv 2002.09766's appendix `\twocolumn[\icmltitle{…}]` re-adds `ltx:title`) is one element; and for the
/// keywords, which bindings re-add as their list grows. Not for the abstracts ([`PRINTED_WHERE_THEY_STAND`]): a second
/// one digested in place is printed where it stands, both are kept, as the vendored Perl keeps them (1006.5288,
/// 2406.04383). OXIDIZED_DESIGN #154. Witness 2511.21969
/// (nested `{abstract}` env). Creators/notes are deliberately NOT here — multi-author frontmatter must accumulate.
const REPLACEABLE_FRONTMATTER_TAGS: &[&str] = &[
  "ltx:title",
  "ltx:toctitle",
  "ltx:subtitle",
  "ltx:date",
  "ltx:abstract",
  "ltx:keywords",
];

/// The replaceable tags a later entry does not replace ([`REPLACEABLE_FRONTMATTER_TAGS`]): an abstract digested after
/// the one before it is printed where it stands (amsart.cls:854's `{abstract}` after `\maketitle`, 1006.5288; article,
/// IEEEtran after `\maketitle`), so both are kept, as the vendored Perl keeps them (`Base_Utility.pool.ltxml:391-445`);
/// only a repeat with the same text is dropped (`frontmatter_drop_repeats`: a block a class or binding adds twice, the later copy kept). An
/// abstract command queued for the flush is still replaced, by the dequeue of the entries not yet digested
/// (`\lx@clear@frontmatter`), as in Perl; an environment a class stores for `\maketitle` and overwrites (amsart.cls:843,
/// revtex, before `\maketitle`: pdflatex prints the last) is digested in place, and both are kept, as in Perl. Keywords
/// stay replaceable: bindings re-add the whole list as it grows and rely on the replacement (OmniBus's `\kwd`,
/// tex-label's `\labels`, a class's keywords store set again after `\maketitle`), and a repeated keywords block lost no
/// words in the corpus (IEEEtran's `{IEEEkeywords}` accumulate). OXIDIZED_DESIGN #154.
const PRINTED_WHERE_THEY_STAND: &[&str] = &["ltx:abstract"];
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
  // (the frontmatter observer's event constructors, used only by its shadow rehearsal: OXIDIZED_DESIGN_DIVERGENCES #837)
  rehearsal::load()?;
});

#[cfg(test)]
mod author_split_tests;
