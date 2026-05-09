//! AST normalization (Phase 2 — to be implemented).
//!
//! Port of `RelaxNG.pm` lines 438–525:
//! `simplify`, `simplify_args`, `simplify_override`, `simplifyCombination`,
//! `extractStart`, `eqOp`.
//!
//! Walks the raw AST produced by [`super::scan`], populates
//! [`super::Relaxng`]'s definition / element / "Used by" tables, and
//! returns the simplified AST.

use super::{Pattern, Relaxng};

/// Top-level simplifier. For now a passthrough: phases of the port land
/// here as separate commits with their own golden tests.
pub fn simplify_top(_rng: &mut Relaxng, raw: Vec<Pattern>) -> Vec<Pattern> {
  raw
}
