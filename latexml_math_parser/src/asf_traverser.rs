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
/// Wired into `parse_marpa` behind the `LATEXML_MARPA_ASF=1` env
/// flag (off by default). Runs side-by-side with the legacy path
/// for parity validation. The cap-deletion + pragmatics audit
/// follow once parity is confirmed.
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

      // 2. Outer lexeme/token rule (e.g. RELOP:less-than:3 ).
      // Aggregate child byte-strings into a single Lexeme AND call
      // `.specialize(...)` — mirrors the legacy `Node::Token` arm in
      // `translate_node`, which calls `XM::Lexeme(s, ...).specialize(
      // Meta::default(), pragmas)?` after `TreeBuilder::rollup_token`
      // already concatenated the bytes.
      if self.builder.is_token(rule_id) {
        let bytes = collect_lexeme_bytes(glade, rh_len, children);
        if bytes.is_empty() {
          all_alts.push(None);
        } else {
          let lexeme_str = String::from_utf8(bytes).unwrap_or_default();
          let lexeme = XM::Lexeme(lexeme_str, Meta::default());
          match lexeme.specialize(Meta::default(), self.pragmas) {
            Ok(specialized) => all_alts.push(Some(specialized)),
            Err(_) => self.pruned_count += 1,
          }
        }
      // 3. Discarded rule (whitespace separators). Emit a single
      // None — parents handle gracefully via `Vec<Option<XM>>`.
      } else if self.builder.is_discard(rule_id) {
        all_alts.push(None);
      // 4. Rule with no registered semantic action.
      //
      // Two flavors:
      //
      // (4a) `builder.is_rule(rule_id) == true` — a grammar-level
      // rule (e.g. `factor = factor_base | function | …`) without
      // a custom action. Legacy fallback in `Actions::action_on`
      // returns `args[0]` unchanged for 1-arg cases and `args[0]`
      // with a stderr warning for n>1. In ASF we mirror this by
      // passing through the first child's alternatives.
      //
      // (4b) Not a builder rule — a literal_string wrapper or an
      // internal Aycock-Horspool scaffolding piece (subrule of
      // `grammar!().rule(None, ...)` that wasn't passed through
      // `builder!().rule(...)`). These are byte-passthroughs:
      // aggregate child Lexemes, matching legacy
      // `rollup_token_rec` semantics.
      } else if !self.actions.has_action(rule_id) {
        if rh_len == 0 {
          all_alts.push(None);
        } else if self.builder.is_rule(rule_id) {
          // (4a) Passthrough grammar rule. Mirror Legacy:
          // `Ok(args.remove(0))` — return the first RHS position's
          // alternatives unchanged.
          let first_child_id = glade.rh_glade_id(0).expect("rh 0");
          let first_child_alts = children.get(&first_child_id).expect("child precomputed");
          for alt in first_child_alts {
            all_alts.push(alt.clone());
          }
        } else {
          // (4b) Byte passthrough.
          let bytes = collect_lexeme_bytes(glade, rh_len, children);
          if bytes.is_empty() {
            all_alts.push(None);
          } else {
            let lexeme = String::from_utf8(bytes).unwrap_or_default();
            all_alts.push(Some(XM::Lexeme(lexeme, Meta::default())));
          }
        }
      } else {
        // 5. Standard rule with semantic action. Cartesian-product
        // child alternatives across RHS positions, apply
        // `Actions::action_on` per combo.
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

/// Collect bytes from every child glade's first Lexeme alternative.
/// Mirrors legacy `TreeBuilder::rollup_token_rec`: intermediate non-
/// action rules are byte-passthrough, so the child's first
/// `XM::Lexeme(s, _)` already contains the full concatenation for
/// its subtree, and we just chain them at the outer level.
fn collect_lexeme_bytes(glade: &Glade, rh_len: usize, children: &HashMap<usize, Vec<Option<XM>>>) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::with_capacity(rh_len * 2);
  for ix in 0..rh_len {
    let child_id = glade.rh_glade_id(ix).expect("rh position has a child glade");
    let child_alts = children.get(&child_id).expect("child precomputed in post-order");
    if let Some(Some(XM::Lexeme(s, _))) = child_alts.first() {
      bytes.extend_from_slice(s.as_bytes());
    }
  }
  bytes
}
