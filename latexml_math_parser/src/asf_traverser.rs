//! ASF-based math parser traversal (Stage-2 ASF migration scaffolding).
//!
//! See [`docs/MATH_PARSER_AND_ASF.md`](../../docs/MATH_PARSER_AND_ASF.md)
//! for the full migration plan. This module is **not yet wired into
//! `parse_marpa`** — it is the entry point a follow-up session can
//! flip on behind `LATEXML_MARPA_ASF=1` once the per-glade output
//! semantics are validated against the existing tree-iteration path.
//!
//! ## Design summary
//!
//! Each glade produces `Vec<Option<XM>>` — the set of alternative
//! XM trees that can terminate at that parse position. The marpa
//! ASF driver memoizes one Vec per glade, then composes parents by
//! Cartesian-multiplying children at each (symch, factoring) pair.
//!
//! Three kinds of glades are handled:
//!
//! 1. **Byte-token glade** (`glade.is_token()` true) — emits a
//!    single-character `XM::Lexeme`. ByteScanner symbol IDs are the
//!    byte values themselves.
//! 2. **Lexeme-rule glade** (rule_id matches `TreeBuilder.is_token`,
//!    e.g. RELOP, ADDOP, NUMBER) — aggregates child byte alternatives
//!    into a full lexeme string, mirrors `TreeBuilder::rollup_token`.
//! 3. **Standard rule glade** — Cartesian-products child alternatives,
//!    calls `Actions::action_on` per combination, collects
//!    `Vec<Option<XM>>` of distinct results.
//!
//! ## Pruning
//!
//! `Actions::action_on` returns `Ok(Some(xm))`, `Ok(None)` (no XM at
//! this position — pushed unchanged for the parent to handle), or
//! `Err(_)` (semantic pragma rejection — counted as a prune,
//! dropped from this glade's alternatives). When ALL alternatives at
//! a child glade get pruned, the child's Vec becomes empty and the
//! parent's Cartesian product yields zero combos — pruning cascades
//! up naturally.

use std::collections::HashMap;

use libxml::tree::Node;
use marpa::asf::{Glade, Traverser};
use marpa::result::Result as MarpaResult;
use marpa::tree_builder::TreeBuilder;

use latexml_core::document::Document;

use crate::pragmatics::ValidationPragmatics;
use crate::semantics::metadata::Meta;
use crate::semantics::{ActionContext, Actions, XM};

/// Per-glade traversal callback for the math parser's ASF migration.
/// Borrows the math parser's actions, pragmas, builder, document so
/// it can drive the same semantic-action dispatch as the legacy
/// `translate_node` path.
///
/// **Scaffolding** — not yet wired into `parse_marpa`. The follow-up
/// session will (a) add an env-gated alternative code path that
/// drives this traverser via `engine.parse_and_traverse_forest(...)`,
/// (b) run it side-by-side with the legacy path for parity
/// validation, (c) switch the default once validated.
#[allow(dead_code)]
pub struct MathTraverser<'a> {
  pub actions:      &'a Actions,
  pub pragmas:      &'a [ValidationPragmatics],
  pub builder:      &'a TreeBuilder,
  pub nodes:        &'a [Node],
  pub document:     &'a mut Document,
  /// Count of `action_on` calls that returned `Err(_)`. Surfaces in
  /// `PARSE_AUDIT` diagnostics analogous to the legacy `pruned_trees`
  /// counter. Reset between glade traversals is up to the caller.
  pub pruned_count: usize,
}

impl Traverser for MathTraverser<'_> {
  /// Each glade contributes the **set of alternative XM trees** that
  /// can terminate at this parse position. `Some(xm)` is a successful
  /// translation; `None` is a no-op position (e.g. discard rule);
  /// `Err` inside `action_on` is recorded as a prune and dropped.
  type ParseTree = Vec<Option<XM>>;
  type ParseState = ();

  fn traverse_glade(
    &mut self,
    glade: &mut Glade,
    children: &HashMap<usize, Self::ParseTree>,
    _state: &mut Self::ParseState,
  ) -> MarpaResult<Self::ParseTree> {
    // 1. Byte-token glade. The ByteScanner-fed symbol_id IS the byte
    // value, so we materialize a one-character Lexeme. Pragmas at the
    // byte level are typically no-ops; bigger semantic checks happen
    // at the lexeme-rule level (case 2).
    if glade.is_token() {
      let sym = glade.symbol_id();
      if (0..=255).contains(&sym) {
        let byte = sym as u8;
        let s = String::from_utf8(vec![byte]).unwrap_or_default();
        return Ok(vec![Some(XM::Lexeme(s, Meta::default()))]);
      }
      // Out-of-range symbol id (shouldn't happen for ByteScanner).
      return Ok(vec![]);
    }

    let mut all_alts: Vec<Option<XM>> = Vec::new();
    loop {
      let rule_id = glade.rule_id();
      let rh_len = glade.rh_length();

      // 2. Lexeme-rule glade (e.g. RELOP, ADDOP, NUMBER). Aggregate
      // child byte-strings into a single Lexeme; matches Perl-side
      // `TreeBuilder::rollup_token` semantics.
      if self.builder.is_token(rule_id) {
        let mut bytes: Vec<u8> = Vec::with_capacity(rh_len);
        for ix in 0..rh_len {
          let child_id = glade.rh_glade_id(ix).expect("rh position has a child glade");
          let child_alts = children.get(&child_id).expect("child precomputed in post-order");
          // Pick the first (= only) alternative; lexeme rules don't
          // produce branching alternatives at their byte children.
          if let Some(Some(XM::Lexeme(s, _))) = child_alts.first() {
            bytes.extend_from_slice(s.as_bytes());
          }
        }
        let lexeme = String::from_utf8(bytes).unwrap_or_default();
        all_alts.push(Some(XM::Lexeme(lexeme, Meta::default())));
      // 3. Discarded rule (whitespace, comments). Emit a single
      // None — parents handle that gracefully via `Vec<Option<XM>>`.
      } else if self.builder.is_discard(rule_id) {
        all_alts.push(None);
      } else {
        // 4. Standard rule. Cartesian-product child alternatives
        // across RHS positions, apply `Actions::action_on` per combo.
        let mut per_pos: Vec<&Self::ParseTree> = Vec::with_capacity(rh_len);
        for ix in 0..rh_len {
          let child_id = glade.rh_glade_id(ix).expect("rh position has a child glade");
          per_pos.push(children.get(&child_id).expect("child precomputed in post-order"));
        }
        let mut combos: Vec<Vec<Option<XM>>> = vec![Vec::new()];
        for child_alts in &per_pos {
          let mut next_combos = Vec::with_capacity(combos.len() * child_alts.len());
          for prefix in &combos {
            for alt in *child_alts {
              let mut new_combo = prefix.clone();
              new_combo.push(alt.clone());
              next_combos.push(new_combo);
            }
          }
          combos = next_combos;
        }
        // Apply action_on per combo.
        for combo in combos {
          let ctxt = ActionContext {
            nodes:    self.nodes,
            document: &mut *self.document,
          };
          match self.actions.action_on(rule_id, combo, self.pragmas, ctxt) {
            Ok(opt_xm) => all_alts.push(opt_xm),
            Err(_) => self.pruned_count += 1,
          }
        }
      }

      if glade.next().is_none() {
        break;
      }
    }
    Ok(all_alts)
  }
}
