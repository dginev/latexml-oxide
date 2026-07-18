//! blkarray — block-array matrices.
//!
//! The raw `blkarray.sty` builds each matrix cell as inline math inside a raw
//! `\halign`/`\ialign`, and its `block` delimiter machinery, digested inside
//! display math, drives BOTH LaTeXML engines into the `\halign`-in-math runaway
//! (Rust OOMs at the 4.5 GB cap in ~12 s; same-host Perl hangs ~90 s → rc=124;
//! `pdflatex` renders fine). Full analysis + 4-line reproducer:
//! `docs/known_crashes/blkarray_halign_math/`. This binding SHADOWS the raw
//! `.sty` (so it is never raw-loaded, even under `--includestyles`) and routes
//! `blockarray`/`block` through the engine's well-behaved `array` alignment
//! machinery instead. Surpass-Perl: upstream LaTeXML has no `blkarray.sty.ltxml`.
//! Witnesses: arXiv:1811.10792 (ar5iv #594), arXiv:2310.17416 (ar5iv #473).
//!
//! **Faithfulness note (documented simplification).** In `blkarray`, a `block`'s
//! column-spec delimiters (`(`/`[` in e.g. `block{c(cccccc)}`) wrap a SUB-REGION
//! of the shared matrix — a construct LaTeXML's `array` cannot express (its
//! `left=`/`right=` wrap the whole matrix). We render each `block` transparently:
//! its rows flow into the single `blockarray` alignment (correct structure +
//! content, and the label row/column are preserved), but the block's delimiter
//! parentheses are DROPPED. This is chosen deliberately over the raw `.sty`'s OOM
//! — a matrix without its outer parens beats losing the whole document section.
use crate::prelude::*;

LoadDefinitions!({
  // `\begin{blockarray}[pos]{spec}` is defined as a MAGIC control sequence
  // (`\begin{...}` takes the defined-CS fast path in latex_constructs.rs:3064,
  // which does NOT inject a `\begingroup`). That is essential: a transparent
  // `block` nested inside must not open a group that crosses the alignment's
  // `\\` row boundaries. Route to the same machinery as `\array`
  // (latex_constructs.rs `\@array@bindings`/`\@@array`/`\lx@begin@alignment`).
  // blkarray's own blockarray column specs are plain (`c cccccc`, `cccc`) — the
  // delimiters live on the block specs, which we gobble — so the spec passes to
  // the AlignmentTemplate parser unchanged.
  DefMacro!(
    T_CS!("\\begin{blockarray}"),
    "[]{}",
    "\\@array@bindings[#1]{#2}\\@@array[#1]{#2}\\lx@begin@alignment"
  );
  DefMacro!(
    T_CS!("\\end{blockarray}"),
    None,
    "\\lx@end@alignment\\@end@array"
  );

  // `block` / `block*` are transparent: gobble the column spec (which may carry
  // `(`, `[`, `|` delimiters we cannot render sub-region-wise) and contribute
  // their rows directly to the enclosing blockarray alignment. Magic CSes → no
  // `\begingroup`, so the block boundary does not disturb the alignment grouping.
  DefMacro!(T_CS!("\\begin{block}"), "{}", "");
  DefMacro!(T_CS!("\\end{block}"), None, "");
  DefMacro!(T_CS!("\\begin{block*}"), "{}", "");
  DefMacro!(T_CS!("\\end{block*}"), None, "");
});
