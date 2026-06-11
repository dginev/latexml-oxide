use std::error::Error;
#[cfg(test)]
use std::rc::Rc;

use once_cell::sync::Lazy;
use rustc_hash::FxHashMap as HashMap;

use crate::{
  semantics::{Operator, XM},
  util::distill_lexeme,
};

/// Different supported pragmatics according to which to enforce consistency.
/// Usually one would use them in increasing generality, iterating over a parse forest
/// until a sufficiently small number of parses is reached.
#[derive(Copy, Clone, Debug)]
pub enum ValidationPragmatics {
  ConsistentLetterBlocks,
  ConsistentCase,
  ConsistentCaseFlat,
  ConsistentCaseFlatUnstyled,
  InconsistentNames, /* TODO: Since name consistency is built into the curry levels, using the
                      * inconsistent names pragma should ensure each LP variable has a globally
                      * unique name */
  FencedAtomsAreNotFunctions,
  FencedLettersAreFunctionArguments,
  AdjacentNumbersDontMultiply,
  AdjacentUnfencedScriptsDontApply,
  AdjacentFunctionsDontUnifyIntoOperator,
  OpfunctionsAreRarelyArguments,
  UnfencedLetterArgumentsRequireVisualCues,
  HigherOrderIDsAreExceptions,
  HigherOrderInvisibleOpsAreExceptions,
  StandaloneDiffopsAreNotNumerators,
  PostfixTermsAreFencedIfSingleArguments,
  RestrictNumeralFractions,
  /// Prefer parses where scripts attach to bases rather than floating standalone.
  MaximizeScriptAttachment,
  /// An expression should never have `absent` on BOTH the left and right sides
  /// of a binary/relational operator.
  NoBilateralAbsent,
  /// Functions prefer wider absorption: `\log 2x^2` means `log(2*x^2)`,
  /// not `log(2)*x^2`. When a function can absorb more factors, prefer that parse.
  /// Implemented by penalizing parses where a function's result is multiplied
  /// by factors that could have been the function's argument.
  FunctionsPreferWiderAbsorption,
  /// Bigops prefer wider absorption: `\int F\times Gdx` means `∫(F×G dx)`,
  /// not `∫(F)×Gdx`. Perl's moreOpArgFactors absorbs MulOp chains.
  /// Rejects trees where mulop(bigop_app(narrow), rhs) when rhs is a simple factor.
  BigopPreferWiderAbsorption,
  /// Prefer binary ADDOP over unary when it follows a complete term.
  /// In `-12x^2 - 4xy + 2y`, the interior `-` and `+` should be binary operators,
  /// not unary prefix applied to the following term. Rejects parses where
  /// prefix_apply(addop, rhs) appears as a non-initial child of an additive chain.
  PreferBinaryAddop,
  /// For invisible-times chains of simple unfenced tokens (e.g., `ppppp`, `xyz`),
  /// enforce left-associative grouping: `((p·p)·p)·p` not `p·(p·(p·p))`.
  /// In mathematical practice, consecutive identifiers without explicit operators
  /// or parentheses form a flat product — associativity doesn't matter semantically,
  /// and enumerating all groupings causes exponential ambiguity.
  FlattenSimpleInvisibleTimesChains,
  /// In `a = b + c + d`, the `=` must be at the outermost level.
  /// An ADDOP/MULOP cannot have an unfenced RELOP child — that would mean
  /// treating a relation as a term in an arithmetic expression.
  /// Exception: fenced relations like `(x=0)` used as modifiers.
  RelopsAreOutermost,
}

impl ValidationPragmatics {
  /// Pragmatic rules that are *always* strictly enforced.
  ///
  /// Architectural note (2026-05-17): these are intended to fire
  /// inside `XM::Apply.specialize(...)` during tree construction,
  /// but in practice `apply_*` actions in `semantics.rs` build their
  /// `XM::Apply` results WITHOUT calling `.specialize()` — so most
  /// expert pragmas that target Apply shapes (e.g.
  /// `FencedLettersAreFunctionArguments`) never get a chance to
  /// fire during action_on. The pragmas listed here that DO run
  /// reliably are those that match `XM::Lexeme` directly (the
  /// translate_node leaf calls `Lexeme.specialize` per line 125
  /// in semantics.rs). Apply-shape pragmas have been moved into
  /// `student_defaults` below where `validate_recursive` actually
  /// invokes them.
  pub fn expert_defaults() -> Vec<Self> {
    use ValidationPragmatics::*;
    vec![
      FencedAtomsAreNotFunctions,
      UnfencedLetterArgumentsRequireVisualCues,
      OpfunctionsAreRarelyArguments,
      AdjacentNumbersDontMultiply,
      StandaloneDiffopsAreNotNumerators,
      PostfixTermsAreFencedIfSingleArguments,
      RestrictNumeralFractions,
      NoBilateralAbsent,
      FlattenSimpleInvisibleTimesChains,
      RelopsAreOutermost,
    ]
  }
  /// Pragmatic rules that are executed at the end of the parse process,
  /// optionally, until there is a single parse left. If a pragmatic rule invalidates
  /// all choices, the rule is skipped.
  pub fn student_defaults() -> Vec<Self> {
    // Order here is crucial - it shows a decreasing level of adoption of each rule in the
    // scientific community, ending with some "crutch"-like rules, as last ditch attempts to discard
    // parses
    use ValidationPragmatics::*;
    vec![
      // First the Apply-shape pragmas that should be expert (always
      // strictly enforced) but in practice need to run here because
      // `apply_*` actions don't call `.specialize()` on their result.
      // Their soft fallback ("skip if all pruned") is harmless: if
      // all surviving trees fail the pragma, the original forest is
      // restored.
      FencedLettersAreFunctionArguments,
      HigherOrderIDsAreExceptions,
      HigherOrderInvisibleOpsAreExceptions,
      AdjacentUnfencedScriptsDontApply,
      AdjacentFunctionsDontUnifyIntoOperator,
      ConsistentLetterBlocks,
      FunctionsPreferWiderAbsorption,
      BigopPreferWiderAbsorption,
      ConsistentCase,
      ConsistentCaseFlat,
      ConsistentCaseFlatUnstyled,
      MaximizeScriptAttachment,
      PreferBinaryAddop,
    ]
  }

  /// Employs the selected pragmatic to augment the validation of the core Meta object
  pub fn validate(&self, tree: &XM) -> Result<(), Box<dyn Error>> {
    use ValidationPragmatics::*;
    match self {
      FencedAtomsAreNotFunctions => pragma_fenced_atoms_are_not_functions(tree),
      UnfencedLetterArgumentsRequireVisualCues => {
        pragma_unfenced_letter_arguments_require_visual_cues(tree)
      },

      OpfunctionsAreRarelyArguments => pragma_opfunctions_are_rarely_arguments(tree),

      FencedLettersAreFunctionArguments => pragma_fenced_letters_are_function_arguments(tree),
      HigherOrderIDsAreExceptions => pragma_higher_order_ids_are_exceptions(tree),
      HigherOrderInvisibleOpsAreExceptions => {
        pragma_higher_order_invisible_ops_are_exceptions(tree)
      },
      AdjacentNumbersDontMultiply => pragma_adjacent_numbers_dont_use_invisible_times(tree),
      AdjacentUnfencedScriptsDontApply => pragma_adjacent_unfenced_scripts_dont_apply(tree),
      StandaloneDiffopsAreNotNumerators => pragma_standalone_diffops_are_not_numerators(tree),
      PostfixTermsAreFencedIfSingleArguments => pragma_postfix_terms_are_fenced_if_single_arg(tree),
      AdjacentFunctionsDontUnifyIntoOperator => pragma_adjacent_functions_dont_unify_into_op(tree),
      RestrictNumeralFractions => pragma_restrict_numeral_fractions(tree),
      MaximizeScriptAttachment => pragma_maximize_script_attachment(tree),
      NoBilateralAbsent => pragma_no_bilateral_absent(tree),
      FunctionsPreferWiderAbsorption => pragma_functions_prefer_wider_absorption(tree),
      BigopPreferWiderAbsorption => pragma_bigop_prefer_wider_absorption(tree),
      PreferBinaryAddop => pragma_prefer_binary_addop(tree),
      FlattenSimpleInvisibleTimesChains => pragma_flatten_simple_invisible_times(tree),
      RelopsAreOutermost => pragma_relops_are_outermost(tree),
      ConsistentLetterBlocks => pragma_consistent_letter_blocks(tree),
      ConsistentCase => pragma_consistent_letter_case(tree),
      ConsistentCaseFlat => pragma_consistent_letter_case_flat(tree),
      ConsistentCaseFlatUnstyled => pragma_consistent_letter_case_flat_unstyled(tree),
      // TODO: implement
      _ => Ok(()),
    }
  }

  /// Recursively checks a tree at each level for applicable pragmatics.
  /// All levels must validate for the full tree to be valid.
  pub fn validate_recursive(&self, tree: &XM) -> Result<(), Box<dyn Error>> {
    self.validate(tree)?; // top-level check
    // Recursive check:
    match tree {
      XM::Choices(subtrees) => {
        for subtree in subtrees.iter() {
          self.validate_recursive(subtree)?
        }
      },
      XM::Apply(Operator(op), args, ..) => {
        self.validate_recursive(op)?;
        for arg_subtree in args.trees() {
          self.validate_recursive(arg_subtree)?;
        }
      },
      XM::Dual(content, pres, ..) => {
        self.validate_recursive(content)?;
        self.validate_recursive(pres)?;
      },
      XM::Wrap(items, ..) => {
        for item in items.iter() {
          self.validate_recursive(item)?;
        }
      },
      _ => {},
    };
    Ok(())
  }
}

/// Letter case, as recognized by the case-consistency pragmas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LetterCase {
  Upper,
  Lower,
}

/// Pragmatic block a single-letter lexeme belongs to. Used by the
/// letter-block consistency pragma to check that peer operands in an
/// invisible-times chain come from the same block.
///
/// `Latin(a, e)` / `Greek(α, γ)` encode the block's endpoints, mirroring the
/// literal block range in the source map. `Standalone(name)` is the fallback
/// for lexemes that don't map to a block (non-block Greek like δ/ϵ/ω, plus
/// multi-char identifiers that aren't recognized Greek names).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LetterBlock {
  Latin(char, char),
  Greek(char, char),
  Standalone(String),
}

/// Typed consistency key for the letter-blocks pragma.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LetterBlocksKey {
  pub role_prefix: String,
  pub block:       LetterBlock,
}

/// Typed consistency key for the letter-case pragmas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LetterCaseKey {
  pub role_prefix: String,
  pub case:        LetterCase,
}

/// Return true iff `op` is an invisible-times operator head, in either of the
/// two forms the XM tree produces: `XM::Lexeme("…invisible_operator…")` (the
/// Marpa lexeme form) or `XM::Token { role: "MULOP", meaning: "times" }`
/// (the post-`apply_invisible_times` form). Helpers across this file need to
/// match both — historically several sites matched only the Lexeme form and
/// silently never fired on real parses.
fn is_invisible_times_op(op: &XM) -> bool {
  match op {
    XM::Lexeme(oplexeme, _) => oplexeme.contains("invisible_operator"),
    XM::Token(props, _) => {
      props.meaning.as_deref() == Some("times") && props.role.as_deref() == Some("MULOP")
    },
    _ => false,
  }
}

/// Extract the consistency key for the letter-blocks pragma.
///
/// The pragmatics are only used for single-letter values with factor/function
/// role, relying on common conventions in mathematical syntax. The two main
/// alphabets are separated into letter blocks, each block required to
/// coordinate to the same xarith type where each separate style of a letter
/// (italic/bold;Uppercase/lowercase) stands for a separate pragmatic block.
///
/// For example in "Ax^2 + Bx + C" we can obtain a single parse where A, B, C
/// are coefficients (same block) and x is the variable (different block).
fn _pragma_letter_blocks(name: &str) -> LetterBlocksKey {
  let (base, sep, lexeme) = distill_lexeme(name);
  let role_prefix = format!("{base}{sep}");
  let block = if lexeme.chars().count() == 1 {
    let letter = lexeme.chars().next().unwrap();
    _PRAGMATIC_BLOCK_MAP
      .get(&letter)
      .cloned()
      .unwrap_or_else(|| LetterBlock::Standalone(lexeme.to_owned()))
  } else if let Some(greek_letter) = greek_name_to_letter(lexeme) {
    _PRAGMATIC_BLOCK_MAP
      .get(&greek_letter)
      .cloned()
      .unwrap_or_else(|| LetterBlock::Standalone(lexeme.to_owned()))
  } else {
    LetterBlock::Standalone(lexeme.to_owned())
  };
  LetterBlocksKey { role_prefix, block }
}

/// Extract the consistency key for the letter-case pragma. For example in
/// "SUx" the key is Lower.
fn _pragma_letter_case(name: &str) -> LetterCaseKey {
  let (base, sep, lexeme) = distill_lexeme(name);
  let role_prefix = format!("{base}{sep}");
  let case = match lexeme.chars().next() {
    Some(c) if c.is_uppercase() => LetterCase::Upper,
    _ => LetterCase::Lower,
  };
  LetterCaseKey { role_prefix, case }
}

/// Variant of `_pragma_letter_case` that disregards scripted guards
/// (i.e. scripted lexemes must coordinate with unscripted ones).
/// For example in "R S_2" R and S_2 should both decide the same case.
fn _pragma_letter_case_flat(name: &str) -> LetterCaseKey {
  let key = _pragma_letter_case(name);
  LetterCaseKey {
    role_prefix: key.role_prefix.replace("sub__", ""),
    case:        key.case,
  }
}

/// Variant of `_pragma_letter_case_flat` that also drops style annotations
/// before the final separator, so italic/bold/upright letters compare as the
/// same style class.
fn _pragma_letter_case_flat_unstyled(name: &str) -> LetterCaseKey {
  // Normalize "ROLE:style-x" → "ROLE:x" so the style annotation drops.
  let unstyled_name = match name.rfind('-') {
    Some(position) => {
      let (base, trailer) = name.split_at(position);
      let (_sep, lexeme) = trailer.split_at(1);
      match base.rfind(':') {
        Some(position) => {
          let (role, _discard) = base.split_at(position);
          format!("{role}:{lexeme}")
        },
        None => format!("{base}:{lexeme}"),
      }
    },
    None => name.to_owned(),
  };
  _pragma_letter_case_flat(&unstyled_name)
}

/// Gather a canonical "consistency key" for each single-letter operand of an
/// invisible-times `Apply`, run the caller's key-extractor on the lexeme name,
/// and reject if any two peer keys differ. Shared backbone for the four
/// Consistent* pragmas.
///
/// Only fires on `Apply(invisible_operator, …)` — the operator denotes
/// "juxtaposition" where mathematical convention says peer letters play the
/// same role (all coefficients, or all variables). Explicit operators
/// (`a + A`, `a = A`) have their own conventions and are out of scope here.
///
/// A "letter operand" is a fenceless `XM::Lexeme` whose distilled lexeme is
/// either a single char or a Greek name (via `greek_name_to_letter`). Other
/// operand shapes (numbers, sub-applications, fenced groups) are skipped.
fn pragma_consistency_via_key<K>(
  tree: &XM,
  key_of: fn(&str) -> K,
  diagnostic: &str,
) -> Result<(), Box<dyn Error>>
where
  K: PartialEq,
{
  if let XM::Apply(Operator(op), args, ..) = tree {
    if !is_invisible_times_op(op) {
      return Ok(());
    }
    // Only fire when there are ≥3 letter operands. A 2-letter chain like
    // `Ax` is the canonical "coefficient × variable" shape and legitimately
    // crosses letter blocks (A-E × x-z); rejecting it would prune the right
    // parse. 3+ operands is where consistency is much more likely than
    // accidental block-crossing.
    let mut first_key: Option<K> = None;
    let mut letter_count = 0usize;
    let mut keys: Vec<K> = Vec::new();
    for tree_arg in args.trees() {
      if let XM::Lexeme(ref name, ref meta) = *tree_arg {
        if meta.fenced.is_some() {
          continue;
        }
        let (_base, _sep, lexeme) = distill_lexeme(name);
        let is_letter = lexeme.chars().count() == 1 || greek_name_to_letter(lexeme).is_some();
        if !is_letter {
          continue;
        }
        letter_count += 1;
        let key = key_of(name);
        if first_key.is_none() {
          first_key = Some(key);
        } else {
          keys.push(key);
        }
      }
    }
    // ≥3 letters required — see comment above on the 2-letter
    // `Ax`-style coefficient×variable exemption.
    if letter_count < 3 {
      return Ok(());
    }
    if let Some(ref first) = first_key {
      for k in &keys {
        if k != first {
          return Err(diagnostic.into());
        }
      }
    }
  }
  Ok(())
}

fn pragma_consistent_letter_blocks(tree: &XM) -> Result<(), Box<dyn Error>> {
  pragma_consistency_via_key(
    tree,
    _pragma_letter_blocks,
    "pruning parse: invisible-times peers span inconsistent letter blocks",
  )
}

fn pragma_consistent_letter_case(tree: &XM) -> Result<(), Box<dyn Error>> {
  pragma_consistency_via_key(
    tree,
    _pragma_letter_case,
    "pruning parse: invisible-times peers mix letter case",
  )
}

fn pragma_consistent_letter_case_flat(tree: &XM) -> Result<(), Box<dyn Error>> {
  pragma_consistency_via_key(
    tree,
    _pragma_letter_case_flat,
    "pruning parse: invisible-times peers mix letter case (flat)",
  )
}

fn pragma_consistent_letter_case_flat_unstyled(tree: &XM) -> Result<(), Box<dyn Error>> {
  pragma_consistency_via_key(
    tree,
    _pragma_letter_case_flat_unstyled,
    "pruning parse: invisible-times peers mix letter case (flat, unstyled)",
  )
}

fn pragma_fenced_atoms_are_not_functions(tree: &XM) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(Operator(op), ..) = tree
    && let XM::Lexeme(_lexeme, atom_meta) = &**op
    && let Some(ref fences) = atom_meta.fenced
    && fences.as_str() == "parens"
  {
    return Err(
      "pruning non-argument parenthetical atom, used as LHS of function application".into(),
    );
  }
  Ok(())
}

fn pragma_fenced_letters_are_function_arguments(tree: &XM) -> Result<(), Box<dyn Error>> {
  // Mathematical convention: `f(x)` reads as function application, not
  // multiplication, when x is a letter. Marpa produces both parses in the
  // ambiguous forest; this pragma chooses the mathematically-consistent one
  // unconditionally. MATHPARSER_SPECULATE has no role here — the decision is
  // a pragmatic preference, not a grammar switch.
  let XM::Apply(Operator(op), args, ..) = tree else {
    return Ok(());
  };
  if !is_invisible_times_op(op) {
    return Ok(());
  }
  let trees = args.trees();
  let Some(top_lhs) = trees.first() else {
    return Ok(());
  };
  if let Some(ref fences) = top_lhs.get_meta().fenced
    && fences.as_str() == "parens"
    && let XM::Lexeme(lhs_name, _) = top_lhs.get_baseline()
    && !lhs_name.starts_with("NUMBER")
  {
    return Err("pruning non-argument parenthetical atom, used as LHS of invisible times".into());
  }

  // Slightly tricky check -- the top RHS needs to be fenced, but we care about the
  // "baseline" content being a variable - disregarding scripts.
  if let Some(top_rhs) = trees.get(1) {
    if let XM::Lexeme(rhs_name, _) = top_rhs.get_baseline() {
      if let Some(ref fences) = top_rhs.get_meta().fenced
        && fences.as_str() == "parens"
      {
        // if the RHS is a number, prune unless the LHS is fenced (things like cycle
        // notation)
        if !rhs_name.starts_with("NUMBER") {
          return Err(
            "pruning non-argument parenthetical atom, used as RHS of invisible times".into(),
          );
        } else {
          match trees.first() {
            Some(XM::Lexeme(_, lhs_meta)) if lhs_meta.fenced.is_none() => {
              return Err(
                "pruning non-argument parenthetical NUMBER, used as RHS of invisible times".into(),
              );
            },
            Some(XM::Apply(_, _, _, lhs_meta)) if lhs_meta.fenced.is_none() => {
              return Err(
                "pruning non-argument parenthetical NUMBER, used as RHS of invisible times".into(),
              );
            },
            _ => {},
          }
        }
      }
    } else if let Some(prune_reason) = is_dual_fenced_rhs(top_rhs, trees.first().copied()) {
      return Err(prune_reason.into());
    }
  }
  Ok(())
}

/// Detect a parens-fenced `XM::Dual` on the RHS of invisible-times and
/// classify whether it should be pruned in favor of function-application.
///
/// The `fenced` grammar action wraps a parenthesized expression as
/// `Dual(content_ref, Wrap[OPEN, expr, CLOSE])` without setting the
/// `Meta::fenced` field on the Dual itself. The `Lexeme`-based check
/// above misses these. This helper inspects the presentation Wrap
/// directly:
///
/// 1. First and last items of the Wrap are `XM::Token` with role OPEN/CLOSE and content `(` / `)`.
/// 2. Inner content (between the parens) is either a non-NUMBER `Lexeme`, a structured `Apply`
///    (list/vector/formulae), or a `Dual` whose own content is non-numeric.
/// 3. Special-case: if the inner content IS a `NUMBER`, fall through to the legacy "number with
///    non-fenced LHS" exception so the cycle-notation case `(a,b)(c,d)` doesn't double-prune.
///
/// Returns `Some(error_msg)` if the parse should be pruned, `None`
/// otherwise.
/// Recognize an `XM::Dual(_, Wrap[OPEN, ..., CLOSE])` shape on the RHS
/// of invisible-times where the outer delimiters are PARENS, BRACKETS,
/// or VERTBARS — all of which can legitimately indicate function-app
/// when preceded by a letter LHS. Returns `Some(_, error_msg)` if the
/// shape matches.
///
/// `kind` is the fence-pair category for diagnostics:
/// "parens" / "brackets" / "vertbars".
fn is_dual_fenced_rhs(top_rhs: &XM, top_lhs: Option<&XM>) -> Option<&'static str> {
  let XM::Dual(_, presentation, ..) = top_rhs else {
    return None;
  };
  let XM::Wrap(ref items, ..) = **presentation else {
    return None;
  };
  // Recognize PARENS or BRACKETS delimiters. **Vertbars `|...|`
  // are deliberately excluded** — K-12 math convention reads
  // `a|f|b` as `a * |f| * b` (absolute-value multiplication),
  // NOT as `a@(|f|)` (function application). Generalizing the
  // "fence implies function-argument" rule to vertbars regressed
  // 2 legacy tests; the qm_test bra-ket case has its own
  // expected interpretation that this pragma should not enforce.
  enum Fence {
    Parens,
    Brackets,
  }
  // (Marker: vertbar-fenced cases purposely fall through; the qm_test
  // and similar bra-ket tests pass under a different mechanism
  // (the QM context resolves differently in soft_prune).
  let recognize_open = |x: Option<&XM>| -> Option<Fence> {
    match x {
      Some(XM::Token(p, _)) => match (p.role.as_deref(), p.content.as_deref()) {
        (Some("OPEN"), Some("(")) => Some(Fence::Parens),
        (Some("OPEN"), Some("[")) => Some(Fence::Brackets),
        _ => None,
      },
      Some(XM::Lexeme(name, _)) => {
        if name.starts_with("OPEN:(:") {
          Some(Fence::Parens)
        } else if name.starts_with("OPEN:[:") {
          Some(Fence::Brackets)
        } else {
          None
        }
      },
      _ => None,
    }
  };
  let recognize_close = |x: Option<&XM>, fence: &Fence| -> bool {
    match x {
      Some(XM::Token(p, _)) => matches!(
        (p.role.as_deref(), p.content.as_deref(), fence),
        (Some("CLOSE"), Some(")"), Fence::Parens) | (Some("CLOSE"), Some("]"), Fence::Brackets)
      ),
      Some(XM::Lexeme(name, _)) => match fence {
        Fence::Parens => name.starts_with("CLOSE:):"),
        Fence::Brackets => name.starts_with("CLOSE:]:"),
      },
      _ => false,
    }
  };

  let fence = recognize_open(items.first())?;
  if !recognize_close(items.last(), &fence) {
    return None;
  }
  let _ = match fence {
    Fence::Parens => "parenthetical",
    Fence::Brackets => "bracketed",
  };

  // Look at the Dual's content branch — that's the semantic shape.
  let content = match top_rhs {
    XM::Dual(c, ..) => &**c,
    _ => return None,
  };
  // Skip the NUMBER case to defer to the legacy NUMBER-exception code.
  if let XM::Lexeme(inner_name, _) = content.get_baseline()
    && inner_name.starts_with("NUMBER")
  {
    match top_lhs {
      Some(XM::Lexeme(_, lhs_meta)) if lhs_meta.fenced.is_none() => {
        return Some(
          "pruning non-argument fenced NUMBER (Dual-wrapped), used as RHS of \
             invisible times",
        );
      },
      Some(XM::Apply(_, _, _, lhs_meta)) if lhs_meta.fenced.is_none() => {
        return Some(
          "pruning non-argument fenced NUMBER (Dual-wrapped), used as RHS of \
             invisible times",
        );
      },
      _ => return None,
    }
  }
  Some("pruning non-argument fenced Dual atom, used as RHS of invisible times")
}

/// If we have two standalone letters in the same, such as "A x" or "F X", prune parses that
/// interpret them as an application, unless they have a role that clearly indicates the LHS is
/// intended as a functional. This may need to be applied as an optional filter at the end of
/// pruning, as there are known, albeit rare, counter-examples ("f x").
fn pragma_unfenced_letter_arguments_require_visual_cues(tree: &XM) -> Result<(), Box<dyn Error>> {
  match *tree {
    XM::Apply(Operator(ref op), ref args, ..) if args.0.len() == 1 => {
      if let Some(XM::Lexeme(ref arg_name, ref atom_meta)) = args.0[0]
        && atom_meta.fenced.is_none()
      {
        let (arg_base, _sep, lexeme) = distill_lexeme(arg_name);
        let arg_letter: char = if lexeme.len() == 1 {
          lexeme.chars().next().unwrap()
        } else if let Some(greek_letter) = greek_name_to_letter(lexeme) {
          greek_letter
        } else {
          return Ok(()); // rule only applies to single char cases
        };
        if arg_base.starts_with("OPERATOR") {
          // for now single letter OPERATOR arguments just don't make sense without parentheses
          // e.g. OPERATOR:italic-delta is a differential, and wouldn't be used carelessly as an
          // arg
          return Err("single letter argument with role OPERATOR is unusual, prune.".into());
        }
        let op_name = op.base_operator_name();
        let (base, _sep, lexeme) = distill_lexeme(&op_name);
        if base.starts_with("FUNCTION") || base.starts_with("OPERATOR") {
          return Ok(()); // don't doubt dedicated roles
        }

        let op_letter: char = if lexeme.len() == 1 {
          lexeme.chars().next().unwrap()
        } else if let Some(greek_letter) = greek_name_to_letter(lexeme) {
          greek_letter
        } else {
          return Ok(()); // rule only applies to single char cases
        };
        if (op_letter.is_lowercase() && arg_letter.is_lowercase())
          || (op_letter.is_uppercase() && arg_letter.is_uppercase())
        {
          return Err("operator and argument are visually similar, prune.".into());
        }
      }
    },
    _ => {},
  };
  Ok(())
}

/// OPFUNCTION are explicitly marked to be operators in an application, they should not be in an
/// argument role of a single argument Apply. Note that this pragma still allows OPFUNCTIONs to
/// appear as list elements.
fn pragma_opfunctions_are_rarely_arguments(tree: &XM) -> Result<(), Box<dyn Error>> {
  match *tree {
    XM::Apply(_, ref args, ..) if args.0.len() == 1 => {
      if let Some(XM::Lexeme(ref arg_name, ref atom_meta)) = args.0[0]
        && arg_name.starts_with("OPFUNCTION")
        && atom_meta.fenced.is_none()
      {
        return Err("OPFUNCTIONs are rarely arguments, prune.".into());
      }
    },
    _ => {},
  }
  Ok(())
}

/// Higher order "ID ( FUNCTION )" constructs occassionally show up in the
/// current DLMF markup, and should remain valid. But they are exceptions.
/// Meanwhile, a reading of
/// "ID FUNCTION" as an ID "applied to" the FUNCTION should be invalid.
/// For now this looks easiest to achieve via a pragmatic prune,
/// Mostly since we can't enforce IDs to strictly have "curry=1", rather
/// we need them with "curry >= 1", to stay a little more lenient.
fn pragma_higher_order_ids_are_exceptions(tree: &XM) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(op, args, ..) = tree
    && args.0.len() == 1
  {
    match &*op.0 {
      XM::Lexeme(op_name, _) if op_name.starts_with("ID:") => match args.0[0] {
        Some(XM::Lexeme(ref rhs_name, ref rhs_meta)) => {
          if rhs_name.starts_with("FUNCTION") && rhs_meta.fenced.is_none() {
            return Err("ID of a higher order than a FUNCTION is not allowed.".into());
          }
          if rhs_name.starts_with("ID:") && rhs_meta.fenced.is_none() {
            return Err("ID applied to unfenced ID is highly unusual, prune.".into());
          }
        },
        Some(XM::Apply(..)) => {
          return Err("ID of a higher order shouldn't accept any compound trees.".into());
        },
        _ => {},
      },
      _ => {},
    }
  }
  Ok(())
}

/// If we see " h f (x,y) " a parse where
/// "h f" are invisible multiplied, is not our first choice.
/// A right-associative application h(f(x,y)) is instead.
/// Prune it out if possible.
fn pragma_higher_order_invisible_ops_are_exceptions(tree: &XM) -> Result<(), Box<dyn Error>> {
  let XM::Apply(Operator(op), args, ..) = tree else {
    return Ok(());
  };
  if !is_invisible_times_op(op) {
    return Ok(());
  }
  let trees = args.trees();
  if trees.len() == 2 {
    let lhs = trees[0];
    let rhs = trees[1];
    if let XM::Lexeme(lhs_name, _) = lhs.get_baseline()
      && let XM::Lexeme(rhs_name, _) = rhs.get_baseline()
      && name_is_functional_or_id(lhs_name)
      && name_is_functional(rhs_name)
    {
      return Err(
        "Pruning higher order 'FUNCTION x FUNCTION' parse to give precedence to \
             right-associative readings"
          .into(),
      );
    }
  }
  Ok(())
}

/// If two numbers are left next to each other, as in "10(5)" it is rarely (never?) the intention
/// that they are to be multiplied Prune such parses. We have special rules for some notations, such
/// as "dlmf_range".
fn pragma_adjacent_numbers_dont_use_invisible_times(tree: &XM) -> Result<(), Box<dyn Error>> {
  let XM::Apply(Operator(op), args, ..) = tree else {
    return Ok(());
  };
  if !is_invisible_times_op(op) {
    return Ok(());
  }
  let arg_trees = args.trees();
  if arg_trees.len() == 2
    && let Some(lhs) = arg_trees.first()
    && lhs.base_operator_name().starts_with("NUMBER")
    && let Some(rhs) = arg_trees.get(1)
    && rhs.base_operator_name().starts_with("NUMBER")
  {
    return Err("pruning two adjacent NUMBERs that used an invisible operator".into());
  }
  Ok(())
}

/// Sometimes differentials can be written in numerators, but only if followed by their variable.
/// The only case (that I currently know) of a standalone "d" is in the Lebnitz derivative notation.
/// For which we have a special rule. So the generic fraction parse should be pruned.
fn pragma_standalone_diffops_are_not_numerators(tree: &XM) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(Operator(op), args, ..) = tree {
    match **op {
      XM::Lexeme(ref oplexeme, _) if oplexeme.starts_with("MULOP") => {
        if let Some(XM::Lexeme(numlexeme, _)) = args.trees().first()
          && numlexeme.starts_with("DIFFOP")
        {
          return Err("pruning standalone diffops are not numerators".into());
        }
      },
      _ => {},
    }
  }
  Ok(())
}

/// Constructs such as " e_i z_0 " are rarely applying e(z). This is even a common expectation of
/// function symbols say " f_i g_j ". This pragma prunes trees that applies two adjacent scripted
/// constructs.
fn pragma_adjacent_unfenced_scripts_dont_apply(tree: &XM) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(Operator(op), args, ..) = tree
    && args.trees().len() == 1
    && let XM::Apply(Operator(ref op_op), _, _, ref op_meta) = **op
  {
    let op_base_name = op_op.base_operator_name();
    if op_meta.fenced.is_none() && (op_base_name == "unknown.subscript") {
      // only subscripts on the outer one to avoid pruning e.g. \nabla^2 u_{0,0}
      if let Some(XM::Apply(Operator(arg_op), _, _, arg_meta)) = args.trees().first() {
        let arg_op_name = arg_op.base_operator_name();
        if arg_meta.fenced.is_none()
          && (arg_op_name == "unknown.subscript" || arg_op_name == "unknown.superscript")
        {
          return Err("Prune: adjacent unfenced scripts do not form an application.".into());
        }
      }
    }
  }
  Ok(())
}

/// Adjacent functions don't unify into a single operator
/// as they are meant to apply right-to-left, one-by-one
fn pragma_adjacent_functions_dont_unify_into_op(tree: &XM) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(Operator(op), ..) = tree
    && let XM::Apply(Operator(inner_op), inner_args, _, inner_meta) = &**op
    && inner_meta.fenced.is_none()
    && let XM::Lexeme(name, _) = inner_op.get_baseline()
    && name_is_functional_or_id(name)
  {
    let inner_trees = inner_args.trees();
    if inner_trees.len() == 1
      && let XM::Lexeme(rhs_name, _) = inner_args.trees().first().unwrap().get_baseline()
      && name_is_functional(rhs_name)
    {
      return Err("Two applied FUNCTIONS as operator violates right-associative behavior.".into());
    }
  }
  Ok(())
}
/// Postfix
fn pragma_postfix_terms_are_fenced_if_single_arg(tree: &XM) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(Operator(op), args, ..) = tree {
    let arg_trees = args.trees();
    if arg_trees.len() == 1
      && let Some(XM::Apply(Operator(arg_op), _, _, arg_meta)) = arg_trees.first()
      && arg_op.base_operator_name().starts_with("POSTFIX")
      && arg_meta.fenced.is_none()
    {
      let op_name = op.base_operator_name();
      if op_name.starts_with("ID")
        || op_name.starts_with("UNKNOWN")
        || op_name.starts_with("FUNCTION")
        || op_name.starts_with("OPERATOR")
        || op_name.starts_with("DIFFOP")
      {
        return Err("pruning postfix term used as single argument without fences".into());
      }
    }
  }
  Ok(())
}

/// Scripts are inherently relational — they modify a base. A standalone floating
/// script (_b by itself, with base=absent) indicates a pre-script that should
/// attach to a subsequent base. Prefer parses where scripts are attached to
/// bases over parses where scripts float independently.
///
/// This is a universal mathematical convention: in `_b^a A^c_d`, the `_b^a`
/// are pre-scripts of A, not standalone subscript/superscript expressions.
fn pragma_maximize_script_attachment(tree: &XM) -> Result<(), Box<dyn Error>> {
  // Detect any script-op Apply where the first arg (base) is "absent".
  // This pattern means a standalone floating script that should be a pre-script.
  if let XM::Apply(Operator(op), args, ..) = tree
    && let XM::Token(ref props, _) = **op
    && props
      .role
      .as_deref()
      .is_some_and(|r| r == "SUBSCRIPTOP" || r == "SUPERSCRIPTOP")
  {
    // Check if base (first arg) is "absent" — standalone script
    if let Some(Some(XM::Token(base_props, _))) = args.0.first()
      && base_props.meaning.as_deref() == Some("absent")
    {
      return Err(
        "Prune: standalone floating script (base=absent) should attach as pre-script.".into(),
      );
    }
  }
  Ok(())
}

/// An expression with `absent` on BOTH sides of a relational/binary operator is
/// never valid mathematical notation. `absent < a` (prefix relop) or `a > absent`
/// (postfix relop) can occur, but `absent < a > absent` is two missing operands —
/// too tortured. This prunes `< a >` from being parsed as a bilateral relation chain,
/// allowing QM expectation `<a>` to win instead.
fn pragma_no_bilateral_absent(tree: &XM) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(_, args, ..) = tree {
    let trees = args.trees();
    if trees.len() >= 2 {
      let first_absent = matches!(trees.first(),
        Some(XM::Token(p, _)) if p.meaning.as_deref() == Some("absent"));
      let last_absent = matches!(trees.last(),
        Some(XM::Token(p, _)) if p.meaning.as_deref() == Some("absent"));
      if first_absent && last_absent {
        return Err(
          "Prune: bilateral absent (absent on both sides) is never valid notation.".into(),
        );
      }
    }
  }
  Ok(())
}

/// Functions prefer wider argument absorption: `\log 2x^2` means `log(2*x^2)`.
/// Prune parses where a function application is immediately followed by invisible
/// multiplication with unfenced factors. The competing parse would have the function
/// absorb those factors as arguments.
fn pragma_functions_prefer_wider_absorption(tree: &XM) -> Result<(), Box<dyn Error>> {
  // Reject: invisible_times(function_app(f, narrow_arg), simple_rhs)
  // This forces the wider parse: function_app(f, narrow_arg * rhs)
  // Only fires when RHS is a simple factor (Lexeme/Token/Wrap), NOT another
  // function application — chained functions like sin(πx)*cos(2πy) should stay separate.
  if let XM::Apply(Operator(op), args, ..) = tree
    && is_invisible_times_op(op)
  {
    let trees = args.trees();
    if trees.len() == 2 {
      // LHS is a function application (Apply(function, arg))
      if let XM::Apply(Operator(func_op), func_args, _, func_meta) = trees[0] {
        // Check if the function op is FUNCTION/OPFUNCTION/TRIGFUNCTION
        let func_name = func_op.base_operator_name();
        if (func_name.starts_with("OPFUNCTION") || func_name.starts_with("TRIGFUNCTION"))
          && func_meta.fenced.is_none()
          && func_args.trees().len() == 1
        {
          // RHS must be a simple factor — not another function application
          // or compound expression. Scripted factors (x^2) are simple.
          let rhs = trees[1];
          let rhs_is_simple = match rhs {
            XM::Lexeme(..) | XM::Token(..) | XM::Wrap(..) => true,
            XM::Apply(Operator(rhs_op), ..) => {
              // Scripted factors (SUPERSCRIPTOP/SUBSCRIPTOP) are simple
              let rhs_role = match &**rhs_op {
                XM::Token(props, _) => props.role.as_deref().unwrap_or(""),
                XM::Lexeme(lex, _) => lex.split(':').next().unwrap_or(""),
                _ => "",
              };
              rhs_role == "SUPERSCRIPTOP" || rhs_role == "SUBSCRIPTOP"
            },
            _ => false,
          };
          let rhs_meta = rhs.get_meta();
          if rhs_is_simple && rhs_meta.fenced.is_none() {
            return Err(
              "Prune: function application followed by simple unfenced factor — \
                 prefer wider absorption."
                .into(),
            );
          }
        }
      }
    }
  }
  // Also check N-ary invisible_times chains for bare OPFUNCTION in non-terminal
  // positions. E.g. Apply(×, [f@(x), d, x]) where d is bare OPFUNCTION at index 1
  // (not the last). The competing parse Apply(×, [f@(x), d@(x)]) is preferred.
  // This covers the pattern: ∫ f(x) \diffd x → f@(x) * diffd@(x), not f@(x)*d*x.
  if let XM::Apply(Operator(op), args, ..) = tree
    && is_invisible_times_op(op)
  {
    let trees = args.trees();
    // Check non-terminal positions for bare OPFUNCTION/TRIGFUNCTION tokens.
    // Both types absorb bare arguments via prefix_apply. If they appear as
    // standalone factors in an invisible_times chain (not the last element),
    // the competing parse with absorption is preferred.
    // FUNCTION is excluded — it only absorbs fenced (parenthesized) args,
    // so `f * x` (invisible_times) is correct for bare FUNCTION.
    if trees.len() >= 3 {
      for tree in trees.iter().take(trees.len() - 1) {
        let is_bare_absorbing_func = match tree {
          XM::Token(props, _) => matches!(
            props.role.as_deref(),
            Some("OPFUNCTION") | Some("TRIGFUNCTION")
          ),
          XM::Lexeme(lex, _) => lex.starts_with("OPFUNCTION:") || lex.starts_with("TRIGFUNCTION:"),
          _ => false,
        };
        if is_bare_absorbing_func {
          return Err(
            "Prune: bare OPFUNCTION/TRIGFUNCTION in N-ary invisible_times chain \
               (non-terminal) — prefer absorption via prefix_apply."
              .into(),
          );
        }
      }
    }
  }
  Ok(())
}

/// Bigop prefer wider absorption: reject mulop(bigop_app(narrow), rhs)
/// when rhs is a simple factor that could have been part of the bigop's argument.
/// Perl's moreOpArgFactors absorbs MulOp chains into bigop arguments.
fn pragma_bigop_prefer_wider_absorption(tree: &XM) -> Result<(), Box<dyn Error>> {
  // Pattern: mulop(bigop_app, simple_rhs) or invisible_times(bigop_app, simple_rhs)
  if let XM::Apply(Operator(op), args, ..) = tree {
    let is_mulop = match **op {
      XM::Token(ref props, _) => props.role.as_deref() == Some("MULOP"),
      XM::Lexeme(ref lex, _) => lex.starts_with("MULOP") || lex.contains("invisible_operator"),
      _ => false,
    };
    if is_mulop {
      let trees = args.trees();
      if trees.len() == 2 {
        // LHS is a bigop application (Apply with BIGOP/SUMOP/INTOP/LIMITOP/DIFFOP op)
        if let XM::Apply(Operator(bigop_op), ..) = trees[0] {
          let bigop_name = bigop_op.base_operator_name();
          let is_bigop = bigop_name.starts_with("BIGOP")
            || bigop_name.starts_with("SUMOP")
            || bigop_name.starts_with("INTOP")
            || bigop_name.starts_with("LIMITOP")
            || bigop_name.starts_with("DIFFOP");
          if is_bigop {
            // RHS should be a simple factor, not another bigop or function
            let rhs = trees[1];
            let rhs_is_simple = match rhs {
              XM::Lexeme(..) | XM::Token(..) | XM::Wrap(..) => true,
              XM::Apply(Operator(rhs_op), ..) => {
                let rhs_role = match &**rhs_op {
                  XM::Token(props, _) => props.role.as_deref().unwrap_or(""),
                  XM::Lexeme(lex, _) => lex.split(':').next().unwrap_or(""),
                  _ => "",
                };
                // Scripted factors and invisible_times products are simple
                rhs_role == "SUPERSCRIPTOP"
                  || rhs_role == "SUBSCRIPTOP"
                  || rhs_role == "MULOP"
                  || rhs_role == "DIFFOP"
              },
              _ => false,
            };
            if rhs_is_simple {
              return Err(
                "Prune: bigop application followed by mulop factor — \
                 prefer wider bigop absorption."
                  .into(),
              );
            }
          }
        }
      }
    }
  }
  Ok(())
}

/// Prefer binary ADDOP interpretation over unary prefix when the ADDOP
/// is between terms in an additive chain. Rejects parses where a unary prefix
/// (like `-x`) appears as a non-first argument to an infix ADDOP application.
/// For `a - b + c`, this rejects `a + (prefix(-,b+c))` in favor of `(a-b)+c`.
fn pragma_prefer_binary_addop(tree: &XM) -> Result<(), Box<dyn Error>> {
  // Check: infix ADDOP application where one argument is itself a prefix ADDOP
  if let XM::Apply(Operator(op), args, ..) = tree {
    let is_addop = match **op {
      XM::Token(ref props, _) => props.role.as_deref() == Some("ADDOP"),
      XM::Lexeme(ref lex, _) => lex.starts_with("ADDOP"),
      _ => false,
    };
    if is_addop {
      // Check arguments: if any non-first argument is a unary prefix ADDOP application,
      // this parse used unary where binary was more appropriate.
      let trees = args.trees();
      for (i, arg) in trees.iter().enumerate() {
        if i == 0 {
          continue;
        } // first arg can legitimately start with unary
        if is_unary_addop_prefix(arg) {
          return Err("prefer_binary_addop: non-initial argument is unary ADDOP prefix".into());
        }
      }
    }
  }
  Ok(())
}

/// Check if a tree is a unary ADDOP prefix application (like -x or +x).
fn is_unary_addop_prefix(tree: &XM) -> bool {
  if let XM::Apply(Operator(op), args, ..) = tree {
    let is_addop = match **op {
      XM::Token(ref props, _) => props.role.as_deref() == Some("ADDOP"),
      XM::Lexeme(ref lex, _) => lex.starts_with("ADDOP"),
      _ => false,
    };
    if is_addop && args.trees().len() == 1 {
      return true; // unary prefix: addop(x) with single argument
    }
  }
  false
}

fn pragma_restrict_numeral_fractions(tree: &XM) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(Operator(op), args, ..) = tree {
    match **op {
      XM::Lexeme(ref oplexeme, _) if &**oplexeme == "arith1.divide" => {
        let arg_trees = args.trees();
        if arg_trees.len() == 2
          && let XM::Lexeme(arg1_name, arg1_meta) = arg_trees[0]
          && let XM::Lexeme(arg2_name, arg2_meta) = arg_trees[1]
          && (arg1_name.starts_with("NUMBER")
            && arg2_name.starts_with("NUMBER")
            && !arg1_meta.syntax_trace.is_empty()
            || !arg2_meta.syntax_trace.is_empty())
        {
          return Err(
            "only tokens are allowed in numeric fractions, derived rules are pruned to \
                   avoid redundancy."
              .into(),
          );
        }
      },
      _ => {},
    }
  }
  Ok(())
}

/// Auxiliary function transforming the spelled-out name of a greek letter to its unicode character
pub fn greek_name_to_letter(name: &str) -> Option<char> {
  match name {
    "alpha" => Some('α'),
    "beta" => Some('β'),
    "gamma" => Some('γ'),
    "delta" => Some('δ'),
    "epsilon" => Some('ϵ'),
    "zeta" => Some('ζ'),
    "eta" => Some('η'),
    "theta" => Some('θ'),
    "iota" => Some('ι'),
    "kappa" => Some('κ'),
    "lambda" => Some('λ'),
    "mu" => Some('μ'),
    "nu" => Some('ν'),
    "xi" => Some('ξ'),
    "omicron" => Some('ο'),
    "pi" => Some('π'),
    "rho" => Some('ρ'),
    "sigma" => Some('σ'),
    "tau" => Some('τ'),
    "upsilon" => Some('υ'),
    "phi" => Some('ϕ'),
    "chi" => Some('χ'),
    "psi" => Some('ψ'),
    "omega" => Some('ω'),
    "Alpha" => Some('Α'),
    "Beta" => Some('Β'),
    "Gamma" => Some('Γ'),
    "Delta" => Some('Δ'),
    "Epsilon" => Some('Ε'),
    "Zeta" => Some('Ζ'),
    "Eta" => Some('Η'),
    "Theta" => Some('Θ'),
    "Iota" => Some('Ι'),
    "Kappa" => Some('Κ'),
    "Lambda" => Some('Λ'),
    "Mu" => Some('Μ'),
    "Nu" => Some('Ν'),
    "Xi" => Some('Ξ'),
    "Omicron" => Some('Ο'),
    "Pi" => Some('Π'),
    "Rho" => Some('Ρ'),
    "Sigma" => Some('Σ'),
    "Tau" => Some('Τ'),
    "Upsilon" => Some('Υ'),
    "Phi" => Some('Φ'),
    "Chi" => Some('Χ'),
    "Psi" => Some('Ψ'),
    "Omega" => Some('Ω'),
    _ => None,
  }
}

pub fn name_is_functional(name: &str) -> bool {
  name.starts_with("FUNCTION")
    || name.starts_with("OPFUNCTION")
    || name.starts_with("TRIGFUNCTION")
    || name.starts_with("UNKNOWN")
}

pub fn name_is_functional_or_id(name: &str) -> bool {
  name.starts_with("ID") || name_is_functional(name)
}

static _PRAGMATIC_BLOCK_MAP: Lazy<HashMap<char, LetterBlock>> = Lazy::new(|| {
  // Generally, we can observe that the latin alphabet shares "intent"
  // in blocks of 3 letters in mathematics, as a fast-and-loose rule of thumb.
  // a-e is an exception as a rather stable 5-letter block with shared utility.
  let mut map = HashMap::default();
  // |a b c d e | f g h |i j k| |l m n| |o p q| |r s t| |u v w| |x y z|
  let latin_blocks = [
    ('a', 'e'),
    ('f', 'h'),
    ('i', 'k'),
    ('l', 'n'),
    ('o', 'q'),
    ('r', 't'),
    ('u', 'w'),
    ('x', 'z'),
  ];
  // |α β γ| δ | ϵ | ζ η θ | ι κ | λ μ ν ξ | ο π ρ | σ τ υ | ϕ χ ψ | ω
  let greek_blocks = [
    ('α', 'γ'),
    ('ζ', 'θ'),
    ('ι', 'κ'),
    ('λ', 'ξ'),
    ('ο', 'ρ'),
    ('σ', 'υ'),
    ('ϕ', 'ψ'),
  ];
  // | Α Β Γ | Δ | Ε | Ζ Η Θ | Ι Κ | Λ Μ Ν Ξ | Ο Π Ρ | Σ Τ Υ | Φ Χ Ψ | Ω
  let up_greek_blocks = [
    ('Α', 'Γ'),
    ('Ζ', 'Θ'),
    ('Ι', 'Κ'),
    ('Λ', 'Ξ'),
    ('Ο', 'Ρ'),
    ('Σ', 'Υ'),
    ('Φ', 'Ψ'),
  ];
  for (start, end) in latin_blocks.iter() {
    let lower = LetterBlock::Latin(*start, *end);
    for c_u8 in (*start as u8)..=(*end as u8) {
      map.insert(c_u8.into(), lower.clone());
    }
    let up_start = start.to_ascii_uppercase();
    let up_end = end.to_ascii_uppercase();
    let upper = LetterBlock::Latin(up_start, up_end);
    for c_u8 in (up_start as u8)..=(up_end as u8) {
      map.insert(c_u8.into(), upper.clone());
    }
  }
  for (start, end) in greek_blocks.iter().chain(up_greek_blocks.iter()) {
    let block = LetterBlock::Greek(*start, *end);
    for c_u32 in (*start as u32)..=(*end as u32) {
      map.insert(std::char::from_u32(c_u32).unwrap(), block.clone());
    }
  }
  map
});

/// For invisible-times chains of simple unfenced tokens, enforce left-associative grouping.
///
/// Mathematical practice: `pppppp` is always read as a flat product `p·p·p·p·p·p`.
/// The grammar produces exponentially many associativity groupings (Catalan number growth),
/// but they are all semantically identical for simple identifier chains.
///
/// Rule: if `Apply(invisible_times, [lhs, rhs])` where `rhs` is itself
/// `Apply(invisible_times, ...)`, AND all leaf operands are simple unfenced identifiers
/// (no fences, no operators, no numbers), then prune — only left-associative grouping
/// `Apply(invisible_times, [Apply(invisible_times, [...]), single])` survives.
fn pragma_flatten_simple_invisible_times(tree: &XM) -> Result<(), Box<dyn Error>> {
  check_invisible_times_recursive(tree)
}

/// Recursively check all subtrees for right-associative invisible-times chains.
fn check_invisible_times_recursive(tree: &XM) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(Operator(op), args, ..) = tree {
    if is_invisible_times_op(op) {
      let trees = args.trees();
      if trees.len() == 2 {
        let rhs = trees[1];
        // Check if the RHS is itself an invisible-times application
        if is_invisible_times_apply(rhs) && all_simple_identifiers(tree) {
          return Err(
            "Pruning right-associative invisible-times of simple identifier chain. \
             Flat left-associative product is the only reasonable reading."
              .into(),
          );
        }
      }
    }
    // Recurse into all subtrees
    for subtree in args.trees() {
      check_invisible_times_recursive(subtree)?;
    }
  }
  Ok(())
}

/// Check if a tree node is an invisible-times application, in either operator form.
fn is_invisible_times_apply(tree: &XM) -> bool {
  if let XM::Apply(Operator(op), ..) = tree {
    return is_invisible_times_op(op);
  }
  false
}

/// Check if ALL leaf tokens in a tree are simple unfenced atoms
/// (identifiers or numbers, no fences, no operators)
fn all_simple_identifiers(tree: &XM) -> bool {
  match tree {
    XM::Lexeme(name, meta) => {
      // Simple atom: UNKNOWN, ID, or NUMBER role, no fences
      meta.fenced.is_none()
        && (name.starts_with("UNKNOWN") || name.starts_with("ID") || name.starts_with("NUMBER"))
    },
    XM::Apply(Operator(op), args, ..) => {
      // For invisible-times applications (either operator shape),
      // check operator and all args
      if is_invisible_times_op(op) {
        return args.trees().iter().all(|a| all_simple_identifiers(a));
      }
      if let XM::Lexeme(ref oplexeme, _) = **op {
        // Scripted atoms (subscript/superscript of simple identifiers) are also simple
        if oplexeme.starts_with("SUBSCRIPTOP")
          || oplexeme.starts_with("SUPERSCRIPTOP")
          || oplexeme.starts_with("POSTSUPERSCRIPT")
          || oplexeme.starts_with("POSTSUBSCRIPT")
        {
          return args.trees().iter().all(|a| all_simple_identifiers(a));
        }
      }
      false
    },
    XM::Token(props, _) => {
      // Token with UNKNOWN/ID/NUMBER role
      props
        .role
        .as_deref()
        .is_some_and(|r| r == "UNKNOWN" || r == "ID" || r == "NUMBER")
        && props.meaning.as_deref() != Some("absent")
    },
    _ => false,
  }
}

/// In `a = b + c + d`, RELOP must be at the outermost level.
/// Rejects parses where an ADDOP or MULOP contains an unfenced RELOP child.
/// Exception: fenced relations like `(x=0)` are allowed as terms.
fn pragma_relops_are_outermost(tree: &XM) -> Result<(), Box<dyn Error>> {
  check_relops_recursive(tree, false)
}

fn check_relops_recursive(tree: &XM, inside_addop_or_mulop: bool) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(Operator(op), args, ..) = tree
    && let XM::Lexeme(ref name, _) = **op
  {
    let is_addop = name.starts_with("ADDOP");
    let is_mulop = name.starts_with("MULOP") || &**name == "x.invisible_operator";
    let is_relop = name.starts_with("RELOP");

    // If we're inside an addop/mulop and this node is a relop, reject
    if inside_addop_or_mulop && is_relop {
      return Err(
        "Pruning: RELOP found inside ADDOP/MULOP — relations must be at the outermost level".into(),
      );
    }

    let child_context = inside_addop_or_mulop || is_addop || is_mulop;
    for subtree in args.trees() {
      // Don't propagate into fenced subtrees — (x=0) as a term is fine
      if !is_fenced(subtree) {
        check_relops_recursive(subtree, child_context)?;
      }
    }
  }
  Ok(())
}

/// Check if a tree represents a fenced (parenthesized) expression.
fn is_fenced(tree: &XM) -> bool {
  match tree {
    XM::Lexeme(_, meta) => meta.fenced.is_some(),
    XM::Apply(_, _, _, meta) => meta.fenced.is_some(),
    _ => false,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn greek_name_to_letter_lowercase() {
    assert_eq!(greek_name_to_letter("alpha"), Some('α'));
    assert_eq!(greek_name_to_letter("beta"), Some('β'));
    assert_eq!(greek_name_to_letter("omega"), Some('ω'));
    assert_eq!(greek_name_to_letter("phi"), Some('ϕ'));
    assert_eq!(greek_name_to_letter("epsilon"), Some('ϵ'));
  }

  #[test]
  fn greek_name_to_letter_uppercase() {
    assert_eq!(greek_name_to_letter("Alpha"), Some('Α'));
    assert_eq!(greek_name_to_letter("Omega"), Some('Ω'));
    assert_eq!(greek_name_to_letter("Delta"), Some('Δ'));
    assert_eq!(greek_name_to_letter("Sigma"), Some('Σ'));
  }

  #[test]
  fn greek_name_to_letter_unknown_returns_none() {
    assert_eq!(greek_name_to_letter("notgreek"), None);
    assert_eq!(greek_name_to_letter(""), None);
    assert_eq!(
      greek_name_to_letter("ALPHA"),
      None,
      "uppercase ALL-CAPS is not a recognized spelling"
    );
  }

  #[test]
  fn greek_name_to_letter_case_sensitive() {
    // "Alpha" (title case) is uppercase Α; "alpha" (lowercase) is α;
    // "ALPHA" is not recognized at all.
    assert_ne!(greek_name_to_letter("alpha"), greek_name_to_letter("Alpha"));
  }

  #[test]
  fn name_is_functional_prefixes() {
    assert!(name_is_functional("FUNCTION"));
    assert!(name_is_functional("FUNCTION:sin"));
    assert!(name_is_functional("OPFUNCTION"));
    assert!(name_is_functional("OPFUNCTION:ln"));
    assert!(name_is_functional("TRIGFUNCTION"));
    assert!(name_is_functional("TRIGFUNCTION:cos"));
    assert!(name_is_functional("UNKNOWN"));
    assert!(name_is_functional("UNKNOWN:foo"));
  }

  #[test]
  fn name_is_functional_rejects_others() {
    assert!(!name_is_functional("RELOP"));
    assert!(!name_is_functional("NUMBER"));
    assert!(!name_is_functional("ID:x"));
    assert!(!name_is_functional("function")); // lowercase doesn't match
    assert!(!name_is_functional(""));
  }

  #[test]
  fn name_is_functional_or_id_includes_id() {
    assert!(name_is_functional_or_id("ID"));
    assert!(name_is_functional_or_id("ID:x"));
    // And still accepts what name_is_functional accepts.
    assert!(name_is_functional_or_id("FUNCTION"));
    assert!(name_is_functional_or_id("UNKNOWN"));
    // But not non-ID, non-functional.
    assert!(!name_is_functional_or_id("RELOP"));
    assert!(!name_is_functional_or_id("NUMBER"));
  }

  // ----- _pragma_letter_case: Lower/Upper on first char -----

  #[test]
  fn pragma_letter_case_lowercase() {
    let k = _pragma_letter_case("UNKNOWN:italic-x");
    assert_eq!(k.role_prefix, "UNKNOWN:italic-");
    assert_eq!(k.case, LetterCase::Lower);
    assert_eq!(_pragma_letter_case("FOO-y").case, LetterCase::Lower);
  }

  #[test]
  fn pragma_letter_case_uppercase() {
    let k = _pragma_letter_case("UNKNOWN:italic-X");
    assert_eq!(k.role_prefix, "UNKNOWN:italic-");
    assert_eq!(k.case, LetterCase::Upper);
    assert_eq!(_pragma_letter_case("FOO-Y").case, LetterCase::Upper);
  }

  #[test]
  fn pragma_letter_case_colon_separator_path() {
    // No dash, but a colon → distill splits at the rightmost colon.
    let lo = _pragma_letter_case("UNKNOWN:x");
    assert_eq!(lo.role_prefix, "UNKNOWN:");
    assert_eq!(lo.case, LetterCase::Lower);
    assert_eq!(_pragma_letter_case("UNKNOWN:X").case, LetterCase::Upper);
  }

  #[test]
  fn pragma_letter_case_bare_lexeme_has_empty_prefix() {
    // No separator → distill returns ("", "", name).
    let k = _pragma_letter_case("x");
    assert_eq!(k.role_prefix, "");
    assert_eq!(k.case, LetterCase::Lower);
    assert_eq!(_pragma_letter_case("X").case, LetterCase::Upper);
  }

  // ----- _pragma_letter_case_flat: strips sub__ stacking marker -----

  #[test]
  fn pragma_letter_case_flat_strips_sub_marker() {
    // Prefix has sub__ removed so scripted and unscripted peers compare equal.
    let k = _pragma_letter_case_flat("FOOsub__-l");
    assert_eq!(k.role_prefix, "FOO-");
    assert_eq!(k.case, LetterCase::Lower);
  }

  #[test]
  fn pragma_letter_case_flat_no_sub_marker_is_identity() {
    let k = _pragma_letter_case_flat("FOO-l");
    assert_eq!(k.role_prefix, "FOO-");
    assert_eq!(k.case, LetterCase::Lower);
  }

  // ----- _pragma_letter_blocks: typed LetterBlock variant per lexeme -----

  #[test]
  fn pragma_letter_blocks_latin_block_a_to_e() {
    for ch in ['a', 'b', 'c', 'd', 'e'] {
      let k = _pragma_letter_blocks(&format!("FOO-{ch}"));
      assert_eq!(k.role_prefix, "FOO-");
      assert_eq!(k.block, LetterBlock::Latin('a', 'e'));
    }
  }

  #[test]
  fn pragma_letter_blocks_latin_block_x_to_z() {
    for ch in ['x', 'y', 'z'] {
      let k = _pragma_letter_blocks(&format!("FOO-{ch}"));
      assert_eq!(k.block, LetterBlock::Latin('x', 'z'));
    }
  }

  #[test]
  fn pragma_letter_blocks_uppercase_latin_blocks() {
    assert_eq!(
      _pragma_letter_blocks("FOO-A").block,
      LetterBlock::Latin('A', 'E'),
    );
    assert_eq!(
      _pragma_letter_blocks("FOO-F").block,
      LetterBlock::Latin('F', 'H'),
    );
  }

  #[test]
  fn pragma_letter_blocks_multichar_greek_name_maps_to_block() {
    // "alpha" → U+03B1, lands in the α-γ block.
    let k = _pragma_letter_blocks("FOO-alpha");
    assert_eq!(k.block, LetterBlock::Greek('α', 'γ'));
  }

  #[test]
  fn pragma_letter_blocks_standalone_greek_falls_back_to_name() {
    // δ/delta is outside any Greek block → Standalone carries the spelt name.
    let k = _pragma_letter_blocks("FOO-delta");
    assert_eq!(k.block, LetterBlock::Standalone("delta".into()));
  }

  #[test]
  fn pragma_letter_blocks_unmatched_multichar_is_standalone() {
    let k = _pragma_letter_blocks("FOO-identifier");
    assert_eq!(k.block, LetterBlock::Standalone("identifier".into()));
  }

  // ----- end-to-end Consistent* prune behavior via validate() -----
  //
  // Smoke-test the wiring: the pragmas return Ok on an operator-only tree and
  // never panic on shapes they don't recognize. Tree-shape round-trips for
  // the prune-on-mismatch case are covered by the full math-parser test
  // suite; here we just lock the fall-through path.

  #[test]
  fn consistent_pragmas_accept_empty_tree() {
    use crate::semantics::metadata::Meta;
    let leaf = XM::Lexeme(Rc::from("UNKNOWN:italic-x"), Meta::default());
    assert!(
      ValidationPragmatics::ConsistentLetterBlocks
        .validate(&leaf)
        .is_ok()
    );
    assert!(ValidationPragmatics::ConsistentCase.validate(&leaf).is_ok());
    assert!(
      ValidationPragmatics::ConsistentCaseFlat
        .validate(&leaf)
        .is_ok()
    );
    assert!(
      ValidationPragmatics::ConsistentCaseFlatUnstyled
        .validate(&leaf)
        .is_ok()
    );
  }

  // ----- end-to-end pruning behavior on real XM::Token operator shape -----

  fn inv_times_chain(letter_names: &[&str]) -> XM {
    use std::borrow::Cow;

    use crate::semantics::{
      metadata::Meta,
      tree::{Args, Operator, XProps},
    };

    let op_props = XProps {
      role: Some(Cow::Borrowed("MULOP")),
      meaning: Some(Cow::Borrowed("times")),
      content: Some(Cow::Borrowed("\u{2062}")),
      ..XProps::default()
    };
    let op = Operator(Box::new(XM::Token(op_props, Meta::default())));
    let args = Args(
      letter_names
        .iter()
        .map(|n| Some(XM::Lexeme(Rc::from(*n), Meta::default())))
        .collect(),
    );
    XM::Apply(op, args, XProps::default(), Meta::default())
  }

  #[test]
  fn consistent_blocks_accepts_two_letter_coefficient_variable() {
    // `Ax` legitimately mixes blocks (A-E × x-z). The ≥3-peer gate exempts
    // it — the 2-peer shape is ambiguous and must not be pruned.
    let tree = inv_times_chain(&["UNKNOWN:italic-A", "UNKNOWN:italic-x"]);
    assert!(
      ValidationPragmatics::ConsistentLetterBlocks
        .validate(&tree)
        .is_ok(),
      "2-letter invisible-times chain should be exempt from block-consistency"
    );
  }

  #[test]
  fn consistent_blocks_accepts_three_same_block_letters() {
    // `abc` — all a-e block. Accept.
    let tree = inv_times_chain(&["UNKNOWN:italic-a", "UNKNOWN:italic-b", "UNKNOWN:italic-c"]);
    assert!(
      ValidationPragmatics::ConsistentLetterBlocks
        .validate(&tree)
        .is_ok(),
      "3 same-block letters should pass"
    );
  }

  #[test]
  fn consistent_blocks_rejects_three_mixed_block_letters() {
    // `abx` — a-e, a-e, x-z. The third letter breaks consistency → prune.
    let tree = inv_times_chain(&["UNKNOWN:italic-a", "UNKNOWN:italic-b", "UNKNOWN:italic-x"]);
    assert!(
      ValidationPragmatics::ConsistentLetterBlocks
        .validate(&tree)
        .is_err(),
      "3 mixed-block letters should be pruned"
    );
  }

  #[test]
  fn consistent_case_rejects_three_mixed_case_letters() {
    let tree = inv_times_chain(&["UNKNOWN:italic-a", "UNKNOWN:italic-b", "UNKNOWN:italic-C"]);
    assert!(
      ValidationPragmatics::ConsistentCase
        .validate(&tree)
        .is_err(),
      "mixed case in 3-peer chain should be pruned"
    );
  }

  #[test]
  fn consistent_case_accepts_three_same_case_letters() {
    let tree = inv_times_chain(&["UNKNOWN:italic-a", "UNKNOWN:italic-b", "UNKNOWN:italic-c"]);
    assert!(
      ValidationPragmatics::ConsistentCase.validate(&tree).is_ok(),
      "all-lower 3-peer chain should pass case consistency"
    );
  }

  // ----- pragma_fenced_letters_are_function_arguments, Token operator shape -----
  //
  // These exercise the session-128+ audit fix: the pragma now recognises the
  // invisible-times operator in both its Marpa Lexeme form and the
  // `apply_invisible_times`-produced Token form. Prior to the fix the pragma
  // silently never fired — these tests lock in the active pruning behaviour.

  fn inv_times_pair(lhs: Option<XM>, rhs: Option<XM>) -> XM {
    use std::borrow::Cow;

    use crate::semantics::{
      metadata::Meta,
      tree::{Args, Operator, XProps},
    };

    let op_props = XProps {
      role: Some(Cow::Borrowed("MULOP")),
      meaning: Some(Cow::Borrowed("times")),
      content: Some(Cow::Borrowed("\u{2062}")),
      ..XProps::default()
    };
    let op = Operator(Box::new(XM::Token(op_props, Meta::default())));
    let args = Args(vec![lhs, rhs]);
    XM::Apply(op, args, XProps::default(), Meta::default())
  }

  fn letter_lexeme(name: &str, fenced: Option<&str>) -> XM {
    use crate::semantics::metadata::Meta;
    let mut meta = Meta::default();
    meta.fenced = fenced.map(|s| s.to_string());
    XM::Lexeme(Rc::from(name), meta)
  }

  #[test]
  fn fenced_letters_pragma_prunes_fenced_lhs_letter_on_token_op() {
    // `(f)(x)` with invisible-times Token operator:
    // LHS is fenced letter → prune (LHS is not an argument).
    let tree = inv_times_pair(
      Some(letter_lexeme("UNKNOWN:italic-f", Some("parens"))),
      Some(letter_lexeme("UNKNOWN:italic-x", None)),
    );
    assert!(
      ValidationPragmatics::FencedLettersAreFunctionArguments
        .validate(&tree)
        .is_err(),
      "fenced LHS letter under invisible times (Token shape) must be pruned"
    );
  }

  #[test]
  fn fenced_letters_pragma_prunes_fenced_rhs_letter_on_token_op() {
    // `f(x)` with invisible-times Token operator, RHS fenced letter → prune
    // (the function-application parse wins over multiplication by `(x)`).
    let tree = inv_times_pair(
      Some(letter_lexeme("UNKNOWN:italic-f", None)),
      Some(letter_lexeme("UNKNOWN:italic-x", Some("parens"))),
    );
    assert!(
      ValidationPragmatics::FencedLettersAreFunctionArguments
        .validate(&tree)
        .is_err(),
      "fenced RHS letter under invisible times (Token shape) must be pruned"
    );
  }

  #[test]
  fn fenced_letters_pragma_accepts_fenced_number_rhs_with_fenced_number_lhs() {
    // The "cycle-notation preserve" branch inside the RHS check only fires
    // when the LHS did NOT already trip the earlier fenced-letter-LHS prune.
    // That means the LHS must either be unfenced or itself NUMBER. This case
    // exercises the fenced-NUMBER LHS path — unusual but valid given the
    // original Perl contract. The branch matches `Some(XM::Lexeme(_, m))
    // if m.fenced.is_none()` / `Some(XM::Apply(_, _, _, m)) if m.fenced.is_none()`
    // and returns Err; otherwise falls through to Ok.
    let tree = inv_times_pair(
      Some(letter_lexeme("NUMBER:italic-5", Some("parens"))),
      Some(letter_lexeme("NUMBER:italic-2", Some("parens"))),
    );
    assert!(
      ValidationPragmatics::FencedLettersAreFunctionArguments
        .validate(&tree)
        .is_ok(),
      "fenced NUMBER RHS with fenced NUMBER LHS must be preserved"
    );
  }

  #[test]
  fn fenced_letters_pragma_prunes_fenced_number_rhs_with_unfenced_lhs() {
    // `x(2)` — unfenced variable LHS, fenced NUMBER RHS → prune.
    // The parse is likely function-application rather than `x × 2`.
    let tree = inv_times_pair(
      Some(letter_lexeme("UNKNOWN:italic-x", None)),
      Some(letter_lexeme("NUMBER:italic-2", Some("parens"))),
    );
    assert!(
      ValidationPragmatics::FencedLettersAreFunctionArguments
        .validate(&tree)
        .is_err(),
      "fenced NUMBER RHS with unfenced LHS must be pruned"
    );
  }

  #[test]
  fn fenced_letters_pragma_accepts_unfenced_pair() {
    // `fx` — neither operand fenced → pragma is a no-op (returns Ok).
    let tree = inv_times_pair(
      Some(letter_lexeme("UNKNOWN:italic-f", None)),
      Some(letter_lexeme("UNKNOWN:italic-x", None)),
    );
    assert!(
      ValidationPragmatics::FencedLettersAreFunctionArguments
        .validate(&tree)
        .is_ok(),
      "neither-operand-fenced chain must pass untouched"
    );
  }

  #[test]
  fn fenced_letters_pragma_accepts_fenced_lhs_number() {
    // LHS is a NUMBER in parens, e.g. `(2)x` — the NUMBER branch in the
    // LHS check is explicitly exempted (the guard is `!starts_with("NUMBER")`).
    let tree = inv_times_pair(
      Some(letter_lexeme("NUMBER:italic-2", Some("parens"))),
      Some(letter_lexeme("UNKNOWN:italic-x", None)),
    );
    assert!(
      ValidationPragmatics::FencedLettersAreFunctionArguments
        .validate(&tree)
        .is_ok(),
      "fenced NUMBER LHS must be accepted (number × variable)"
    );
  }

  #[test]
  fn fenced_letters_pragma_noop_when_operator_not_invisible_times() {
    // Apply with a random Token operator — the pragma must not fire.
    use std::borrow::Cow;

    use crate::semantics::{
      metadata::Meta,
      tree::{Args, Operator, XProps},
    };

    let op_props = XProps {
      role: Some(Cow::Borrowed("ADDOP")),
      meaning: Some(Cow::Borrowed("plus")),
      content: Some(Cow::Borrowed("+")),
      ..XProps::default()
    };
    let op = Operator(Box::new(XM::Token(op_props, Meta::default())));
    let args = Args(vec![
      Some(letter_lexeme("UNKNOWN:italic-f", Some("parens"))),
      Some(letter_lexeme("UNKNOWN:italic-x", Some("parens"))),
    ]);
    let tree = XM::Apply(op, args, XProps::default(), Meta::default());
    assert!(
      ValidationPragmatics::FencedLettersAreFunctionArguments
        .validate(&tree)
        .is_ok(),
      "non-invisible-times operator must leave the pragma a no-op"
    );
  }

  // ----- pragma_higher_order_invisible_ops_are_exceptions, Token op shape -----

  fn lexeme(name: &str) -> XM {
    use crate::semantics::metadata::Meta;
    XM::Lexeme(Rc::from(name), Meta::default())
  }

  #[test]
  fn higher_order_pragma_prunes_functional_x_function_on_token_op() {
    // `h f` where both are OPFUNCTION/TRIGFUNCTION → right-associative
    // application is preferred, so prune the invisible-times reading.
    let tree = inv_times_pair(
      Some(lexeme("OPFUNCTION:italic-h")),
      Some(lexeme("OPFUNCTION:italic-f")),
    );
    assert!(
      ValidationPragmatics::HigherOrderInvisibleOpsAreExceptions
        .validate(&tree)
        .is_err(),
      "FUNCTION × FUNCTION under invisible times (Token shape) must be pruned"
    );
  }

  #[test]
  fn higher_order_pragma_prunes_unknown_x_unknown() {
    // `xy` — both UNKNOWN lexemes pass `name_is_functional` (it recognises
    // UNKNOWN:* as a potentially-functional letter head). Per the pragma's
    // semantics, two adjacent functional/ID letters prefer the
    // right-associative application parse, so the invisible-times one is
    // pruned.
    let tree = inv_times_pair(
      Some(lexeme("UNKNOWN:italic-x")),
      Some(lexeme("UNKNOWN:italic-y")),
    );
    assert!(
      ValidationPragmatics::HigherOrderInvisibleOpsAreExceptions
        .validate(&tree)
        .is_err(),
      "UNKNOWN × UNKNOWN under invisible times must be pruned"
    );
  }

  #[test]
  fn higher_order_pragma_prunes_function_x_unknown() {
    // `f x` — OPFUNCTION LHS, UNKNOWN RHS. Both satisfy `name_is_functional`,
    // so the pragma prunes the invisible-times reading.
    let tree = inv_times_pair(
      Some(lexeme("OPFUNCTION:italic-f")),
      Some(lexeme("UNKNOWN:italic-x")),
    );
    assert!(
      ValidationPragmatics::HigherOrderInvisibleOpsAreExceptions
        .validate(&tree)
        .is_err(),
      "OPFUNCTION × UNKNOWN under invisible times must be pruned"
    );
  }

  #[test]
  fn higher_order_pragma_accepts_number_times_unknown() {
    // `5 x` — NUMBER LHS breaks `name_is_functional_or_id`. Pragma no-op.
    let tree = inv_times_pair(
      Some(lexeme("NUMBER:italic-5")),
      Some(lexeme("UNKNOWN:italic-x")),
    );
    assert!(
      ValidationPragmatics::HigherOrderInvisibleOpsAreExceptions
        .validate(&tree)
        .is_ok(),
      "NUMBER × letter must be preserved (NUMBER is not functional)"
    );
  }

  #[test]
  fn higher_order_pragma_accepts_unknown_times_number() {
    // `x 5` — NUMBER RHS fails `name_is_functional`. Pragma no-op.
    let tree = inv_times_pair(
      Some(lexeme("UNKNOWN:italic-x")),
      Some(lexeme("NUMBER:italic-5")),
    );
    assert!(
      ValidationPragmatics::HigherOrderInvisibleOpsAreExceptions
        .validate(&tree)
        .is_ok(),
      "letter × NUMBER must be preserved (NUMBER is not functional)"
    );
  }

  // ----- pragma_adjacent_numbers_dont_use_invisible_times, Token op shape -----

  #[test]
  fn adjacent_numbers_pragma_prunes_number_pair_on_token_op() {
    // `10 5` under invisible times → implausible, prune.
    let tree = inv_times_pair(
      Some(lexeme("NUMBER:italic-10")),
      Some(lexeme("NUMBER:italic-5")),
    );
    assert!(
      ValidationPragmatics::AdjacentNumbersDontMultiply
        .validate(&tree)
        .is_err(),
      "adjacent NUMBER × NUMBER under invisible times (Token shape) must be pruned"
    );
  }

  #[test]
  fn adjacent_numbers_pragma_accepts_number_times_letter() {
    // `5 x` — legitimate coefficient × variable.
    let tree = inv_times_pair(
      Some(lexeme("NUMBER:italic-5")),
      Some(lexeme("UNKNOWN:italic-x")),
    );
    assert!(
      ValidationPragmatics::AdjacentNumbersDontMultiply
        .validate(&tree)
        .is_ok(),
      "NUMBER × letter must be preserved"
    );
  }

  #[test]
  fn adjacent_numbers_pragma_accepts_letter_pair() {
    // `x y` — not numbers at all.
    let tree = inv_times_pair(
      Some(lexeme("UNKNOWN:italic-x")),
      Some(lexeme("UNKNOWN:italic-y")),
    );
    assert!(
      ValidationPragmatics::AdjacentNumbersDontMultiply
        .validate(&tree)
        .is_ok(),
      "letter × letter must be preserved"
    );
  }

  // ----- is_invisible_times_op helper -----

  #[test]
  fn is_invisible_times_op_accepts_lexeme_form() {
    use crate::semantics::metadata::Meta;
    let op = XM::Lexeme(Rc::from("x.invisible_operator"), Meta::default());
    assert!(is_invisible_times_op(&op));
  }

  #[test]
  fn is_invisible_times_op_accepts_token_form() {
    use std::borrow::Cow;

    use crate::semantics::{metadata::Meta, tree::XProps};
    let props = XProps {
      role: Some(Cow::Borrowed("MULOP")),
      meaning: Some(Cow::Borrowed("times")),
      ..XProps::default()
    };
    let op = XM::Token(props, Meta::default());
    assert!(is_invisible_times_op(&op));
  }

  #[test]
  fn is_invisible_times_op_rejects_plus_token() {
    use std::borrow::Cow;

    use crate::semantics::{metadata::Meta, tree::XProps};
    let props = XProps {
      role: Some(Cow::Borrowed("ADDOP")),
      meaning: Some(Cow::Borrowed("plus")),
      ..XProps::default()
    };
    let op = XM::Token(props, Meta::default());
    assert!(!is_invisible_times_op(&op));
  }

  #[test]
  fn is_invisible_times_op_rejects_unrelated_lexeme() {
    use crate::semantics::metadata::Meta;
    let op = XM::Lexeme(Rc::from("MULOP:plus"), Meta::default());
    assert!(!is_invisible_times_op(&op));
  }

  // ----- pragma_flatten_simple_invisible_times, Token operator shape -----
  //
  // These exercise the shared helper fix: before the audit the three inner
  // predicates (check_invisible_times_recursive, is_invisible_times_apply,
  // all_simple_identifiers) matched only the Lexeme form, so the flatten
  // rule silently never fired on post-apply_invisible_times trees.

  fn nested_inv_times(letter_names: &[&str]) -> XM {
    // Build a right-associative chain `a × (b × (c × d))` under the
    // XM::Token{MULOP, times} operator shape.
    use std::borrow::Cow;

    use crate::semantics::{
      metadata::Meta,
      tree::{Args, Operator, XProps},
    };

    assert!(letter_names.len() >= 2, "need at least 2 operands");
    let mut acc = XM::Lexeme(Rc::from(*letter_names.last().unwrap()), Meta::default());
    for name in letter_names.iter().rev().skip(1) {
      let op_props = XProps {
        role: Some(Cow::Borrowed("MULOP")),
        meaning: Some(Cow::Borrowed("times")),
        content: Some(Cow::Borrowed("\u{2062}")),
        ..XProps::default()
      };
      let op = Operator(Box::new(XM::Token(op_props, Meta::default())));
      let args = Args(vec![
        Some(XM::Lexeme(Rc::from(*name), Meta::default())),
        Some(acc),
      ]);
      acc = XM::Apply(op, args, XProps::default(), Meta::default());
    }
    acc
  }

  #[test]
  fn flatten_simple_inv_times_prunes_right_assoc_on_token_op() {
    // Right-associative chain `a × (b × c)` of three simple identifiers.
    // The flatten pragma prefers the left-associative grouping and prunes
    // this shape.
    let tree = nested_inv_times(&["UNKNOWN:italic-a", "UNKNOWN:italic-b", "UNKNOWN:italic-c"]);
    assert!(
      ValidationPragmatics::FlattenSimpleInvisibleTimesChains
        .validate(&tree)
        .is_err(),
      "right-assoc simple-identifier chain (Token op) must be pruned"
    );
  }

  #[test]
  fn flatten_simple_inv_times_accepts_two_operand_chain() {
    // Only 2 operands — no nested invisible-times RHS, nothing to flatten.
    let tree = inv_times_pair(
      Some(lexeme("UNKNOWN:italic-a")),
      Some(lexeme("UNKNOWN:italic-b")),
    );
    assert!(
      ValidationPragmatics::FlattenSimpleInvisibleTimesChains
        .validate(&tree)
        .is_ok(),
      "2-operand chain must pass untouched"
    );
  }
}
