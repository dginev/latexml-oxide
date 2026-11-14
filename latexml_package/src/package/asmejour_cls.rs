use crate::{
  prelude::*,
  shipped_class::{RawRead, read_raw},
};

// asmejour.cls (TeX Live; papers also ship it) loads raw as written. Its `\keywords{…}` (asmejour.cls:1367-1371)
// stores `\__asmejour_keywords`, which the abstract's coffin sets after the abstract text as "Keywords: …"
// (:1373-1382); the locked `{abstract}` never builds that coffin, so the keywords were lost with 0 errors
// (2510.10694). They are kept as the document's keywords under the class's label (`\keywordname`, :1178), set where
// the class's setter runs (its own store is read only by that coffin). The class's `\date` needs nothing: its
// unnumbered first-column footnote (`\DateFootnote`, :1474-1488) is set by the class's `\maketitle` and keeps the
// date. Silent-loss inventory #11; repro sectioning-frontmatter/class_notes/asmejour_keywords_kept; guard
// `perfect_kernel_batch74b::asmejour_keywords_are_kept`.
const ASMEJOUR: RawRead = RawRead {
  setters: &[("\\keywords", r"\lx@add@keywords[name={\keywordname:~}]{#1}")],
  ..RawRead::new("asmejour", "cls")
};

LoadDefinitions!({
  read_raw(&ASMEJOUR)?;
});
