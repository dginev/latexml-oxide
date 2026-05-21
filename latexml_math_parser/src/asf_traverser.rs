//! ASF-based math parser traversal.
//!
//! ## What this does
//!
//! Marpa builds an ASF (Abstract Syntax Forest) — a DAG where shared
//! sub-parses collapse to a single **glade**. `MathTraverser` is the
//! per-glade callback: it produces the set of alternative XM trees
//! that can terminate at that glade's input position. Outputs are
//! memoized by the marpa driver, so a glade reached by multiple
//! parents is evaluated once.
//!
//! ## Glade categories
//!
//! Marpa classifies glades for us; this callback only routes:
//!
//! 1. **Token glade** (`glade.is_token()`): a ByteScanner byte. Emits
//!    one `XM::Lexeme("x")` for the byte value.
//! 2. **Lexeme rule** (`builder.is_token(rule_id)`): RELOP / ADDOP /
//!    NUMBER and friends — rule whose body is a byte sequence that
//!    rolls up into a single lexeme name, then is `.specialize`d.
//! 3. **Discard rule** (`builder.is_discard(rule_id)`): whitespace.
//!    Emits one `None`.
//! 4. **No-action rule**: either a grammar passthrough (`is_rule` →
//!    forward the first child's alternatives) or an internal byte-
//!    passthrough scaffolding piece (concatenate child bytes into a
//!    Lexeme).
//! 5. **Rule with action**: cartesian-product child alternatives
//!    across RHS positions, dispatch `Actions::action_on` per combo,
//!    accumulate the surviving results.
//!
//! ## Pruning
//!
//! `Actions::action_on` returns `Ok(Some(_))` (good parse),
//! `Ok(None)` (this position contributes nothing — passed up
//! unchanged), or `Err(_)` (semantic pragma rejection). An `Err`
//! drops that combo from the glade's alternatives. If every alt at
//! a glade is rejected, the glade's Vec becomes empty and the
//! parent's cartesian product yields zero combos — pruning cascades.

use std::cell::RefCell;
use std::rc::Rc;

use libxml::tree::Node;
use marpa::asf::{Glade, Traverser};
use marpa::result::Result as MarpaResult;
use marpa::tree_builder::TreeBuilder;

use latexml_core::document::Document;

use crate::pragmatics::ValidationPragmatics;
use crate::semantics::metadata::Meta;
use crate::semantics::{ActionContext, Actions, XM};

thread_local! {
  static ASCII_LEXEMES: RefCell<Vec<Rc<str>>> = RefCell::new(
    (0u8..=127)
      .map(|b| Rc::<str>::from(std::str::from_utf8(std::slice::from_ref(&b)).unwrap()))
      .collect()
  );
}

#[inline]
fn ascii_lexeme(byte: u8) -> Option<Rc<str>> {
  if byte > 127 {
    return None;
  }
  Some(ASCII_LEXEMES.with(|cache| cache.borrow()[byte as usize].clone()))
}

/// Alternatives at a single glade. Wrapped in `Rc` so the marpa ASF
/// driver's per-glade `cache.insert(_, output.clone())` and `cache
/// .get(&id).clone()` become refcount bumps instead of deep tree
/// copies.
pub type GladeAlts = Rc<Vec<Option<XM>>>;

/// Per-glade traversal callback. Holds the math parser's actions,
/// pragmas, builder, document — the same dependencies the legacy
/// `translate_node` path uses for bottom-up action dispatch.
pub struct MathTraverser<'a> {
  pub actions:      &'a Actions,
  pub pragmas:      &'a [ValidationPragmatics],
  pub builder:      &'a TreeBuilder,
  pub nodes:        &'a [Node],
  pub document:     &'a mut Document,
  /// `action_on(...) -> Err(_)` count. Surfaces in `PARSE_AUDIT`
  /// diagnostics analogous to the legacy `pruned_trees` counter.
  pub pruned_count: usize,
}

impl Traverser for MathTraverser<'_> {
  type ParseTree = GladeAlts;
  type ParseState = ();

  fn traverse_glade(
    &mut self,
    glade: &mut Glade,
    children: &[Option<Self::ParseTree>],
    _state: &mut Self::ParseState,
  ) -> MarpaResult<Self::ParseTree> {
    // Case 1: byte-token glade. ByteScanner symbol_id == byte value.
    if glade.is_token() {
      let sym = glade.symbol_id();
      let alt = if (0..=255).contains(&sym) {
        ascii_lexeme(sym as u8).map(|s| XM::Lexeme(s, Meta::default()))
      } else {
        None
      };
      return Ok(Rc::new(vec![alt]));
    }

    let mut alts: Vec<Option<XM>> = Vec::new();
    // Symch loop — runs once per grammar-level alternative at this
    // position. For unambiguous parses this body runs exactly once.
    loop {
      let rule_id = glade.rule_id();
      let rh_len = glade.rh_length();
      let has_action = self.actions.has_action(rule_id);
      let is_lex_rule = self.builder.is_token(rule_id);

      if self.builder.is_discard(rule_id) {
        // Case 3: whitespace-discard rule.
        alts.push(None);
      } else if is_lex_rule || (!has_action && !self.builder.is_rule(rule_id)) {
        // Cases 2 + 4b: byte-rollup. The lexeme-rule branch (2) also
        // calls `.specialize` on the result; the bare byte-passthrough
        // (4b) doesn't.
        if let Some(lex) = collect_lexeme(glade, rh_len, children) {
          let lexeme = XM::Lexeme(lex, Meta::default());
          if is_lex_rule {
            match lexeme.specialize(Meta::default(), self.pragmas) {
              Ok(x) => alts.push(Some(x)),
              Err(_) => self.pruned_count += 1,
            }
          } else {
            alts.push(Some(lexeme));
          }
        } else {
          alts.push(None);
        }
      } else if !has_action {
        // Case 4a: grammar passthrough — forward the first child's
        // alts unchanged. Mirrors legacy `args.remove(0)`.
        let first_id = glade.rh_glade_id(0).expect("rh 0");
        let first = children
          .get(first_id)
          .and_then(|o| o.as_ref())
          .expect("child precomputed");
        for alt in first.iter() {
          alts.push(alt.clone());
        }
      } else {
        // Case 5: rule with action — cartesian-product children's
        // alts across RHS positions, run `action_on` per combo.
        self.dispatch_action(glade, rule_id, rh_len, children, &mut alts);
      }

      if glade.next().is_none() {
        break;
      }
    }
    Ok(Rc::new(alts))
  }
}

impl MathTraverser<'_> {
  /// Cartesian-product children, dispatch `action_on` per combo,
  /// push surviving results into `out`. Pull this out of
  /// `traverse_glade` so the loop body in case 5 doesn't compete
  /// with the simpler cases for visual real estate.
  fn dispatch_action(
    &mut self,
    glade: &Glade,
    rule_id: i32,
    rh_len: usize,
    children: &[Option<GladeAlts>],
    out: &mut Vec<Option<XM>>,
  ) {
    // Resolve children once.
    let mut per_pos: Vec<&Vec<Option<XM>>> = Vec::with_capacity(rh_len);
    for ix in 0..rh_len {
      let cid = glade.rh_glade_id(ix).expect("rh position has child glade");
      let child = children
        .get(cid)
        .and_then(|o| o.as_ref())
        .expect("child precomputed");
      per_pos.push(child.as_ref());
    }
    let total: usize = per_pos.iter().map(|p| p.len()).product();
    if total == 0 {
      return;
    }
    if total == 1 {
      // Common case: every RHS position has one alternative. Build
      // one combo without the odometer machinery.
      let combo: Vec<Option<XM>> = per_pos.iter().map(|p| p[0].clone()).collect();
      self.run_action(rule_id, combo, out);
      return;
    }
    // General case: stream combinations via an odometer over per-
    // position indices instead of materialising the full
    // `Vec<Vec<Option<XM>>>` accumulator. The accumulator approach
    // re-cloned every prefix as it grew (O(total × rh_len) extra
    // clones on top of the inevitable per-combo clones); the
    // odometer pays only the unavoidable `total × rh_len` clones
    // and a single fixed-size `indices` buffer. Memory drops from
    // O(total × rh_len) to O(rh_len).
    let mut indices = vec![0usize; rh_len];
    'odometer: loop {
      let combo: Vec<Option<XM>> = indices
        .iter()
        .enumerate()
        .map(|(pos, &ix)| per_pos[pos][ix].clone())
        .collect();
      self.run_action(rule_id, combo, out);
      // Advance: increment the rightmost position; on wrap, carry
      // left. If the leftmost wraps too, we've enumerated every
      // combination.
      for pos in (0..rh_len).rev() {
        indices[pos] += 1;
        if indices[pos] < per_pos[pos].len() {
          continue 'odometer;
        }
        indices[pos] = 0;
      }
      break;
    }
  }

  #[inline]
  fn run_action(&mut self, rule_id: i32, combo: Vec<Option<XM>>, out: &mut Vec<Option<XM>>) {
    let ctxt = ActionContext {
      nodes:    self.nodes,
      document: &mut *self.document,
    };
    match self.actions.action_on(rule_id, combo, self.pragmas, ctxt) {
      Ok(opt_xm) => out.push(opt_xm),
      Err(_) => self.pruned_count += 1,
    }
  }
}

/// Build a lexeme from the first alternative of each child glade.
/// The one-child case is common for byte-passthrough scaffolding, so
/// return the child's existing `Rc<str>` rather than allocating a
/// temporary byte buffer and a second `Rc<str>`.
fn collect_lexeme(glade: &Glade, rh_len: usize, children: &[Option<GladeAlts>]) -> Option<Rc<str>> {
  if rh_len == 1 {
    let cid = glade.rh_glade_id(0).expect("rh position has child glade");
    let alts = children
      .get(cid)
      .and_then(|o| o.as_ref())
      .expect("child precomputed");
    if let Some(Some(XM::Lexeme(s, _))) = alts.first() {
      return Some(s.clone());
    }
    return None;
  }

  let bytes = collect_lexeme_bytes(glade, rh_len, children);
  if bytes.is_empty() {
    return None;
  }
  std::str::from_utf8(&bytes).ok().map(Rc::<str>::from)
}

/// Concatenate the `s.as_bytes()` of each child glade's first
/// `Some(XM::Lexeme(s, _))` alternative. Mirrors `TreeBuilder::
/// rollup_token_rec`: byte-passthrough intermediate rules pre-roll
/// their subtree into one Lexeme, so the outer rule just chains.
fn collect_lexeme_bytes(glade: &Glade, rh_len: usize, children: &[Option<GladeAlts>]) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::with_capacity(rh_len * 2);
  for ix in 0..rh_len {
    let cid = glade.rh_glade_id(ix).expect("rh position has child glade");
    let alts = children
      .get(cid)
      .and_then(|o| o.as_ref())
      .expect("child precomputed");
    if let Some(Some(XM::Lexeme(s, _))) = alts.first() {
      bytes.extend_from_slice(s.as_bytes());
    }
  }
  bytes
}
