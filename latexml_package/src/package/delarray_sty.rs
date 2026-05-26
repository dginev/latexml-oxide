//! delarray.sty — array delimiter package.
//!
//! delarray.sty (D. Carlisle, 1991-1994) provides the
//! `\begin{array}{l}\left(...\right)` delimiter-augmented form. It
//! `\RequirePackage{array}` then `\def\@@array[#1]{...}` to wrap the
//! kernel `\@array` with delimiter detection.
//!
//! Perl LaTeXML has no `delarray.sty.ltxml`. With its default
//! `INCLUDE_STYLES=false`, the raw `delarray.sty` is NOT loaded —
//! Perl emits a "missing binding" warning and continues with the
//! binding-aware `\@@array` from `latex_constructs.rs`. The user's
//! `\begin{array}{l}\left(\right)` then digests cleanly because our
//! `\@@array` binding plus inline `\left\right` already cover this
//! pattern.
//!
//! In Rust we default to `INCLUDE_STYLES=true` (ar5iv preload sets
//! it). Without this stub the raw `delarray.sty` IS loaded — its
//! `\def\@@array[#1]{...}` overwrites our binding-aware `\@@array`
//! and the next `\begin{array}` falls through to the LaTeX kernel
//! `\@array`, which needs `\@classz`/`\@acol`/... (not defined by our
//! `array.sty.ltxml` stub). Result: cascade of "undefined" errors and
//! a fatal TooManyErrors abort.
//!
//! Stubbing this binding suppresses the raw-load (Perl-parity) so
//! `\@@array` keeps pointing at our binding. The delimiter detection
//! in the user's source still works through the standard array+`\left
//! \right` path.
//!
//! Witnesses: canvas-3 0809.4328, 0810.2088, 0810.2091, 0811.2514,
//! 0811.4484, 0812.1967, 0901.2107, 0901.3167.
use crate::prelude::*;

LoadDefinitions!({
  RequirePackage!("array");
});
