//! mdwtab.sty — alternative tabular environment by Mark Wooding (1996).
//!
//! mdwtab.sty completely reimplements `\tabular` with its own
//! `\tab@*` preamble-parsing machinery (`\tab@right`,
//! `\tab@restorehlstate`, `\tab@bgroup`, `\tab@multicol`, etc.).
//! Raw-loading it would redefine our locked `\tabular` binding —
//! the redefinition is silently ignored, but then mdwtab's own
//! `\tab@*` helpers (referenced by every column type and `\@arstrut`
//! handler) end up undefined when the user actually enters a
//! tabular, cascading through `\omit`/`\@startsection` mode errors
//! to a TooManyErrors fatal.
//!
//! Perl LaTeXML has no `mdwtab.sty.ltxml`; with its default
//! `INCLUDE_STYLES=false` the raw `mdwtab.sty` is NOT loaded —
//! Perl emits a "missing binding" warning and continues with the
//! binding-aware `\tabular`. The user's tables then render
//! through standard array/tabular machinery (no mdwtab-specific
//! ornaments, but cleanly).
//!
//! In Rust we default to `INCLUDE_STYLES=true` (ar5iv preload sets
//! it). This stub suppresses the raw-load so `\tabular` keeps
//! pointing at our binding-aware constructor. Same pattern as
//! `delarray_sty.rs` / `trace_sty.rs`.
//!
//! Witness: canvas-3 stage-26 0910.3293 (uses `\usepackage{mathenv}`
//! which `\RequirePackage{mdwtab}`).
use crate::prelude::*;

LoadDefinitions!({});
