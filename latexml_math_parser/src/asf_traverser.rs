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
//! 1. **Token glade** (`glade.is_token()`): a ByteScanner byte. Emits one `XM::Lexeme("x")` for the
//!    byte value.
//! 2. **Lexeme rule** (`builder.is_token(rule_id)`): RELOP / ADDOP / NUMBER and friends — rule
//!    whose body is a byte sequence that rolls up into a single lexeme name, then is
//!    `.specialize`d.
//! 3. **Discard rule** (`builder.is_discard(rule_id)`): whitespace. Emits one `None`.
//! 4. **No-action rule**: either a grammar passthrough (`is_rule` → forward the first child's
//!    alternatives) or an internal byte- passthrough scaffolding piece (concatenate child bytes
//!    into a Lexeme).
//! 5. **Rule with action**: cartesian-product child alternatives across RHS positions, dispatch
//!    `Actions::action_on` per combo, accumulate the surviving results.
//!
//! ## Pruning
//!
//! `Actions::action_on` returns `Ok(Some(_))` (good parse),
//! `Ok(None)` (this position contributes nothing — passed up
//! unchanged), or `Err(_)` (semantic pragma rejection). An `Err`
//! drops that combo from the glade's alternatives. If every alt at
//! a glade is rejected, the glade's Vec becomes empty and the
//! parent's cartesian product yields zero combos — pruning cascades.

use std::{cell::RefCell, rc::Rc};

use latexml_core::document::Document;
use libxml::tree::Node;
use marpa::{
  asf::{Glade, Traverser},
  result::Result as MarpaResult,
  tree_builder::TreeBuilder,
};

use crate::{
  pragmatics::ValidationPragmatics,
  semantics::{ActionContext, Actions, XM, metadata::Meta},
};

thread_local! {
  /// Every byte as the char of the same number (U+0000-U+00FF): `byte_lexeme`'s cache.
  static BYTE_LEXEMES: RefCell<Vec<Rc<str>>> = RefCell::new(
    (0u8..=255).map(|b| Rc::<str>::from(char::from(b).to_string())).collect()
  );
}

/// A byte-token glade's lexeme. Below a lexeme rule the rollup carries each input byte as the char
/// of the same number, so a multi-byte character survives it whole, as the tree builder's
/// `rollup_token_rec` keeps the token's raw bytes; `token_text` decodes them once the lexeme rule
/// completes. (Bytes above 127 were dropped: ASF named the tree route's `ATOM:Γ:33` `ATOM::33`,
/// and the two routes' readings of one formula never compared equal.)
#[inline]
fn byte_lexeme(byte: u8) -> Rc<str> {
  BYTE_LEXEMES.with(|cache| cache.borrow()[byte as usize].clone())
}

/// A lexeme rule's text: its raw bytes (`byte_lexeme`) decoded as UTF-8, as the tree route decodes a
/// token's bytes (`Actions::translate_node`, semantics.rs).
fn token_text(raw: Rc<str>) -> Rc<str> {
  if raw.is_ascii() {
    return raw;
  }
  let bytes: Vec<u8> = raw
    .chars()
    .map(|c| u8::try_from(u32::from(c)).unwrap_or(b'?'))
    .collect();
  String::from_utf8(bytes).map_or_else(|_| Rc::from("malformed-utf8"), Rc::from)
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
  pub actions:              &'a Actions,
  pub pragmas:              &'a [ValidationPragmatics],
  pub builder:              &'a TreeBuilder,
  pub nodes:                &'a [Node],
  pub document:             &'a mut Document,
  /// `action_on(...) -> Err(_)` count. Surfaces in `PARSE_AUDIT`
  /// diagnostics analogous to the legacy `pruned_trees` counter.
  pub pruned_count:         usize,
  /// The work bound of the tree iterator's second chance (`MathParser::parse_marpa`); `None` on
  /// the ordinary ASF route, whose bocages are small.
  pub budget:               Option<AsfBudget>,
  /// Does the formula hold an expectation lexeme that the grammar also reads as a letter
  /// (`EXPECTATION:𝔼.letter`, the letter retry's, `expectation_letter`, M3)? Each glade then keeps its
  /// alternatives with the fewest letter readings (`keep_fewest_letter_readings`).
  pub letter_readings:      bool,
  /// Does the formula hold an integral's letter `d` (`XDIFFUNK`/`XDIFFID`)? Each glade then keeps its alternatives
  /// with the fewest letter `d`s before a variable their differential takes (`keep_fewest_letter_differentials`).
  pub differential_letters: bool,
}

/// A bound on one traversal's work: the actions it attempts (pruned ones too), the alternatives its
/// glades produce, and a deadline, a safety net only. What a traversal costs follows the readings
/// that survive at each glade, not the bocage's size — bar pairs multiply them (`|a||b|…|k|`) — so
/// the counts are what bound it. A traversal cut short anywhere is `exhausted`: its readings are an
/// arbitrary part of the whole, so the caller discards them — the formula has no parse, or keeps
/// the tree iterator's sample if that was cut short at `max_unique`.
pub struct AsfBudget {
  pub attempts:     usize,
  pub alternatives: usize,
  pub deadline:     std::time::Instant,
  pub exhausted:    bool,
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
    if !self.within_budget() {
      return Ok(Rc::new(Vec::new()));
    }
    // Case 1: byte-token glade. ByteScanner symbol_id == byte value.
    if glade.is_token() {
      let sym = glade.symbol_id();
      let alt = u8::try_from(sym)
        .ok()
        .map(|byte| XM::Lexeme(byte_lexeme(byte), Meta::default()));
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
        if let Some(raw) = collect_lexeme(glade, rh_len, children) {
          if is_lex_rule {
            let lexeme = XM::Lexeme(token_text(raw), Meta::default());
            match lexeme.specialize(Meta::default(), self.pragmas) {
              Ok(x) => alts.push(Some(x)),
              Err(_) => self.pruned_count += 1,
            }
          } else {
            alts.push(Some(XM::Lexeme(raw, Meta::default())));
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
          if !self.within_budget() {
            break;
          }
          self.spend();
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
    if self.letter_readings && alts.len() > 1 {
      keep_fewest_letter_readings(&mut alts);
    }
    if self.differential_letters && alts.len() > 1 {
      keep_fewest_letter_differentials(&mut alts);
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
      if !self.within_budget() {
        break;
      }
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
    if !self.within_budget() {
      return;
    }
    self.spend_attempt();
    let ctxt = ActionContext {
      nodes:    self.nodes,
      document: &mut *self.document,
    };
    match self.actions.action_on(rule_id, combo, self.pragmas, ctxt) {
      Ok(opt_xm) => {
        self.spend();
        out.push(opt_xm)
      },
      Err(_) => self.pruned_count += 1,
    }
  }

  /// Is there work left in the budget (always, without one)? The first time there is not, the
  /// traversal is cut short: `exhausted`.
  fn within_budget(&mut self) -> bool {
    let Some(budget) = self.budget.as_mut() else {
      return true;
    };
    if !budget.exhausted {
      budget.exhausted = budget.attempts == 0
        || budget.alternatives == 0
        || std::time::Instant::now() >= budget.deadline;
    }
    !budget.exhausted
  }

  /// Was the traversal cut short, its readings incomplete?
  pub fn budget_spent(&self) -> bool { self.budget.as_ref().is_some_and(|budget| budget.exhausted) }

  /// Count one attempted action, pruned or not, against the budget.
  fn spend_attempt(&mut self) {
    if let Some(budget) = self.budget.as_mut() {
      budget.attempts = budget.attempts.saturating_sub(1);
    }
  }

  /// Count one produced alternative against the budget.
  fn spend(&mut self) {
    if let Some(budget) = self.budget.as_mut() {
      budget.alternatives = budget.alternatives.saturating_sub(1);
    }
  }
}

/// An expectation's letter reading (`expectation_letter`, M3) yields to an operator reading of the same
/// symbol over the same span: a glade keeps the alternatives with the fewest letter readings, so the
/// letter twin of each expectation does not multiply through the Cartesian products above it (the
/// root's `ExpectationLettersAreFallbacks` ranks whole trees the same way, for the tree iterator's). A hard
/// drop, unlike the root's: it assumes no parent refuses the typed alternative yet accepts the letter one
/// (none of the M3.1 review's ~150 probes did).
fn keep_fewest_letter_readings(alts: &mut Vec<Option<XM>>) {
  // (a `None` alternative, an action's empty result, is no reading: kept, and not ranked; M3 review)
  let counts: Vec<Option<usize>> = alts
    .iter()
    .map(|alt| alt.as_ref().map(crate::semantics::expectation_letter_count))
    .collect();
  let Some(fewest) = counts.iter().flatten().copied().min() else {
    return;
  };
  if counts.iter().flatten().all(|&count| count == fewest) {
    return;
  }
  let mut counts = counts.into_iter();
  alts.retain(|_| counts.next().flatten().is_none_or(|count| count == fewest));
}

/// An integral's letter `d` before a variable yields to the differential of that variable over the same span
/// (`diffop_apply`, `differential_power_apply`; Perl's IntFactor tries `diffd ATOM_OR_ID addScripts` first,
/// MathGrammar:640-647): a glade keeps the alternatives with the fewest such letters, so each differential's letter
/// twin does not multiply through the Cartesian products above it (the root's
/// `LetterDsBeforeVariablesAreDifferentials` ranks whole trees the same way, for the tree iterator's; 57cj.20; 2605.28900,
/// 2605.08899, 2605.29990). A hard drop, as `keep_fewest_letter_readings`.
fn keep_fewest_letter_differentials(alts: &mut Vec<Option<XM>>) {
  let pragma = ValidationPragmatics::LetterDsBeforeVariablesAreDifferentials;
  let counts: Vec<Option<usize>> = alts
    .iter()
    .map(|alt| alt.as_ref().map(|alt| pragma.rank_violations(alt)))
    .collect();
  let Some(fewest) = counts.iter().flatten().copied().min() else {
    return;
  };
  if counts.iter().flatten().all(|&count| count == fewest) {
    return;
  }
  let mut counts = counts.into_iter();
  alts.retain(|_| counts.next().flatten().is_none_or(|count| count == fewest));
}

/// Build a lexeme's raw bytes (`byte_lexeme`) from the first alternative of each child glade,
/// as the tree builder's `rollup_token_rec` concatenates a token's bytes: byte-passthrough
/// intermediate rules pre-roll their subtree into one Lexeme, so the outer rule just chains. The
/// one-child case is common for byte-passthrough scaffolding, so return the child's existing
/// `Rc<str>` rather than allocating a second one.
fn collect_lexeme(glade: &Glade, rh_len: usize, children: &[Option<GladeAlts>]) -> Option<Rc<str>> {
  let child_lexeme = |ix: usize| -> Option<&Rc<str>> {
    let cid = glade.rh_glade_id(ix).expect("rh position has child glade");
    let alts = children
      .get(cid)
      .and_then(|o| o.as_ref())
      .expect("child precomputed");
    match alts.first() {
      Some(Some(XM::Lexeme(s, _))) => Some(s),
      _ => None,
    }
  };
  if rh_len == 1 {
    return child_lexeme(0).cloned();
  }
  let mut raw = String::with_capacity(rh_len * 2);
  for ix in 0..rh_len {
    if let Some(s) = child_lexeme(ix) {
      raw.push_str(s);
    }
  }
  (!raw.is_empty()).then(|| Rc::from(raw))
}

#[cfg(test)]
mod tests {
  use super::*;

  /// A lexeme's bytes survive the rollup: each byte a char of its own number, decoded once as
  /// UTF-8 at the lexeme rule — the tree route's `ATOM:Γ:33`, not `ATOM::33` (57at).
  #[test]
  fn a_lexeme_keeps_its_non_ascii_bytes() {
    let raw: String = "ATOM:Γ¯z:33"
      .bytes()
      .map(|byte| byte_lexeme(byte).to_string())
      .collect();
    assert_eq!(&*token_text(Rc::from(raw)), "ATOM:Γ¯z:33");
    // ASCII passes through untouched.
    assert_eq!(&*token_text(Rc::from("UNKNOWN:x:1")), "UNKNOWN:x:1");
  }
}
