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
//! Each glade produces `Rc<Vec<Option<XM>>>` — the set of alternative
//! XM trees that can terminate at that parse position, wrapped in a
//! reference-counted pointer so the marpa ASF driver's cache.clone()
//! at every glade is a refcount bump instead of a deep tree copy.
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
use std::rc::Rc;

use libxml::tree::Node;
use marpa::asf::{Glade, Traverser};
use marpa::result::Result as MarpaResult;
use marpa::tree_builder::TreeBuilder;

use latexml_core::document::Document;

use crate::pragmatics::ValidationPragmatics;
use crate::semantics::metadata::Meta;
use crate::semantics::{ActionContext, Actions, XM};

// Thread-local cache of `Rc<str>` for each ASCII byte. The byte-glade
// hot path emits a single-character `XM::Lexeme` per byte; with this
// cache, repeated bytes share the same `Rc<str>` instance and the
// emit path is a refcount bump instead of a `String` allocation.
// `Rc<str>` is single-threaded; the math parser runs single-threaded
// per document, matching the existing `with_arena_mut` model in
// `latexml_core::common::arena`.
thread_local! {
  static ASCII_BYTE_RC: [std::cell::OnceCell<Rc<str>>; 128] = const {
    [const { std::cell::OnceCell::new() }; 128]
  };
}

#[inline]
fn byte_lexeme_rc(byte: u8) -> Rc<str> {
  if byte < 128 {
    ASCII_BYTE_RC.with(|cache| {
      cache[byte as usize]
        .get_or_init(|| {
          // SAFETY: byte < 128 → valid 1-byte UTF-8.
          let s = unsafe { std::str::from_utf8_unchecked(std::slice::from_ref(&byte)) };
          Rc::from(s)
        })
        .clone()
    })
  } else {
    // Non-ASCII byte: ByteScanner doesn't emit these in practice, but
    // be defensive — produce an empty Rc<str> instead of panicking.
    Rc::from("")
  }
}

/// Alternatives at a single glade. Wrapped in `Rc` so that the marpa
/// ASF driver's per-glade `cache.insert(_, output.clone())` and
/// `cache.get(&id).clone()` paths are refcount bumps instead of deep
/// tree copies — on math-heavy papers this eliminates the dominant
/// per-glade allocation cost.
pub type GladeAlts = Rc<Vec<Option<XM>>>;

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
  type ParseTree = GladeAlts;
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
        return Ok(Rc::new(vec![Some(XM::Lexeme(byte_lexeme_rc(byte), Meta::default()))]));
      }
      // Out-of-range symbol id (shouldn't happen for ByteScanner).
      return Ok(Rc::new(vec![]));
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
          // SAFETY: bytes were already valid UTF-8 in the child Lexemes
          // (`Rc<str>` is utf-8 by construction). `extend_from_slice`
          // appends valid UTF-8 segments, preserving validity overall.
          let lexeme_str: Rc<str> =
            unsafe { Rc::from(std::str::from_utf8_unchecked(&bytes)) };
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
          for alt in first_child_alts.iter() {
            all_alts.push(alt.clone());
          }
        } else {
          // (4b) Byte passthrough.
          let bytes = collect_lexeme_bytes(glade, rh_len, children);
          if bytes.is_empty() {
            all_alts.push(None);
          } else {
            // SAFETY: bytes were already valid UTF-8 in the child Lexemes
            // (`Rc<str>` is utf-8 by construction).
            let lexeme: Rc<str> =
              unsafe { Rc::from(std::str::from_utf8_unchecked(&bytes)) };
            all_alts.push(Some(XM::Lexeme(lexeme, Meta::default())));
          }
        }
      } else {
        // 5. Standard rule with semantic action. Cartesian-product
        // child alternatives across RHS positions, apply
        // `Actions::action_on` per combo.
        let mut per_pos: Vec<&Vec<Option<XM>>> = Vec::with_capacity(rh_len);
        for ix in 0..rh_len {
          let child_id = glade.rh_glade_id(ix).expect("rh position has a child glade");
          per_pos.push(children.get(&child_id).expect("child precomputed in post-order").as_ref());
        }
        // Fast path: when every RHS position has exactly one
        // alternative (the common case for unambiguous math), the
        // cartesian product is a single combo. Skip the
        // `Vec<Vec<Option<XM>>>` allocation chain and build one
        // combo directly. This eliminates N+1 Vec allocations per
        // rule reduction for the typical case (where ~95%+ of
        // ASF traversal time was previously spent in glade
        // cartesian expansion).
        let total: usize = per_pos.iter().map(|p| p.len()).product();
        if total == 1 {
          let combo: Vec<Option<XM>> =
            per_pos.iter().map(|p| p.first().cloned().flatten().map(Some).unwrap_or(None)).collect();
          let ctxt = ActionContext {
            nodes:    self.nodes,
            document: &mut *self.document,
          };
          match self.actions.action_on(rule_id, combo, self.pragmas, ctxt) {
            Ok(opt_xm) => all_alts.push(opt_xm),
            Err(_) => self.pruned_count += 1,
          }
        } else if total == 0 {
          // At least one child has zero alternatives — the whole
          // rule reduction has no products. Skip.
        } else {
          // General path: cartesian product.
          let mut combos: Vec<Vec<Option<XM>>> = Vec::with_capacity(total);
          combos.push(Vec::with_capacity(rh_len));
          for child_alts in &per_pos {
            let mut next_combos: Vec<Vec<Option<XM>>> =
              Vec::with_capacity(combos.len() * child_alts.len());
            for prefix in &combos {
              for alt in child_alts.iter() {
                let mut new_combo = Vec::with_capacity(prefix.len() + 1);
                new_combo.extend(prefix.iter().cloned());
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
      }

      if glade.next().is_none() {
        break;
      }
    }
    Ok(Rc::new(all_alts))
  }
}

/// Collect bytes from every child glade's first Lexeme alternative.
/// Mirrors legacy `TreeBuilder::rollup_token_rec`: intermediate non-
/// action rules are byte-passthrough, so the child's first
/// `XM::Lexeme(s, _)` already contains the full concatenation for
/// its subtree, and we just chain them at the outer level.
fn collect_lexeme_bytes(glade: &Glade, rh_len: usize, children: &HashMap<usize, GladeAlts>) -> Vec<u8> {
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
