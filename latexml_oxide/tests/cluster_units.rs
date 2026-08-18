//! `cluster_units` — small standalone unit tests folded into one link unit to
//! cut CI/build link steps (the same lever as the 2026-08 `cluster_*`
//! consolidation; see CLAUDE.md "Build & Test"). Each former top-level
//! `tests/<n>.rs` file is a module under `tests/cluster_units/`, so the four
//! link units collapse to one binary; test fn names are preserved (now under a
//! `<module>::` path).
//!
//! Folded 2026-08-18: `701_unit_footnote` → `footnote`, `127_latexmlversion` →
//! `latexmlversion`, `005_latexmlmath_single_structure` →
//! `latexmlmath_single_structure`, `700_unit_parse` → `parse`.
//!
//! These do only a handful of conversions total (two are subprocess-driven), so
//! they add negligible in-process libxml2 residue. Do NOT fold the `tex_tests!`
//! fixture sweeps or the `114_streaming_*` / memory-scale binaries in here — each
//! is a separate process on purpose (libxml2 RSS fuse, `streaming_sweep/mod.rs`).

#[path = "cluster_units/footnote.rs"]
mod footnote;
#[path = "cluster_units/latexmlmath_single_structure.rs"]
mod latexmlmath_single_structure;
#[path = "cluster_units/latexmlversion.rs"]
mod latexmlversion;
#[path = "cluster_units/parse.rs"]
mod parse;
