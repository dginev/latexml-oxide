//! The IMS `imsart.sty` support file, as a binding of its own: the frontmatter and keyword markup the imsart.cls binding
//! owns over it ([`crate::imsart_cls::imsart_sty_markup`], `\arxiv`), for a paper that loads imsart.sty without that
//! class. A shipped copy is read after it as the stand-in's copy (OXIDIZED_DESIGN_DIVERGENCES #731), its own extras
//! added: imsart.sty under its own name beside `\documentclass{article}` (1301.1466), and a renamed copy that declares
//! itself `\ProvidesFile{imsart.sty}` (the declaration route, `content.rs` `provided_binding`) — 1905.11425's and
//! 2101.10506's stylefile.sty (loaded by their stylefile.cls, a renamed imsart.cls), math0608476's imsart_mod.sty. Read
//! raw, `\ead[label=e1,mark]{…}` inside `\author` glued "label=e1" into the name (OmniBus's `\ead{}[]`) or lost the
//! e-mail; now each is its author's contact (OXIDIZED_DESIGN_DIVERGENCES #986).
use latexml_package::prelude::*;

use crate::imsart_cls::{imsart_arxiv, imsart_sty_markup};

LoadDefinitions!({
  imsart_sty_markup()?;
  imsart_arxiv()?;
  // `\affiliation[A]{…}` (imsart.sty:1720-1735 of 1905.11425's copy; `\affiliation{…}`, :1028 of 1301.1466's) prints the
  // affiliation under the authors set since the last one (`\let\author@fmt@init\author@fmt@init@def`), or those its label
  // marks. Under the class binding the raw class's `\affiliation` reaches the frontmatter through the kernel's class
  // readers; a stand-in's copy is no raw class read (the binding owns the title page), so it printed in the body: the
  // binding's, for the run of new authors (1301.1466).
  DefMacro!(
    "\\affiliation OptionalSemiverbatim {}",
    "\\lx@add@contact[label={#1},role=affiliation,annotate=run]{#2}",
    locked => true
  );
});
