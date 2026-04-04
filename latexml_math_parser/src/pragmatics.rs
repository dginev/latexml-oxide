use once_cell::sync::Lazy;
use rustc_hash::FxHashMap as HashMap;
use std::error::Error;

use crate::semantics::{Operator, XM};
use crate::util::distill_lexeme;

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
  /// Pragmatic rules that are *always* strictly enforced
  pub fn expert_defaults() -> Vec<Self> {
    use ValidationPragmatics::*;
    vec![
      FencedAtomsAreNotFunctions,
      FencedLettersAreFunctionArguments,
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
      XM::Dual(ref content, ref pres, _, _) => {
        self.validate_recursive(content)?;
        self.validate_recursive(pres)?;
      },
      XM::Wrap(ref items, _, _) => {
        for item in items.iter() {
          self.validate_recursive(item)?;
        }
      },
      _ => {},
    };
    Ok(())
  }
}

/// Validate a pragmatic class, as indicated by a string name,
///   against a context dictionary of known objects updating the dictionary when needed
///
/// The pragmatics are only used for single-letter values with factor/function role,
///   relying on common conventions in mathematical syntax:
///
/// The two main alphabets are separated into letter blocks, each block
///   required to coordinate to the same xarith type where each separate style of a letter
///   (italic/bold;Uppercase/lowercase) stands for a separate pragmatic block.
///
/// For example in "Ax^2 + Bx + C" we can obtain a single parse where A,B,C are coefficients
fn _pragma_letter_blocks(name: &str) -> String {
  let (base, sep, lexeme) = distill_lexeme(name);
  let lexeme_pragmatic = if lexeme.len() == 1 {
    let letter = lexeme.chars().next().unwrap();
    match _PRAGMATIC_BLOCK_MAP.get(&letter) {
      Some(block) => block,
      None => lexeme,
    }
  } else if let Some(greek_letter) = greek_name_to_letter(lexeme) {
    match _PRAGMATIC_BLOCK_MAP.get(&greek_letter) {
      Some(block_name) => block_name,
      None => lexeme,
    }
  } else {
    lexeme
  };
  base.to_owned() + sep + lexeme_pragmatic
}

/// Validate a pragmatic class, indicated by the case of a named lexeme
///
/// For example in SUx
fn _pragma_letter_case(name: &str) -> String {
  let (base, sep, lexeme) = distill_lexeme(name);
  let lexeme_pragmatic = if lexeme.chars().next().unwrap().is_uppercase() {
    "U"
  } else {
    "l"
  };
  base.to_owned() + sep + lexeme_pragmatic
}

/// Validate a pragmatic class, indicated by the case of a named lexeme
/// ALSO disregarding any scripted guards
/// (i.e. scripted lexemes must coordinate with unscripted ones)
/// For example in "R S_2"
fn _pragma_letter_case_flat(name: &str) -> String { _pragma_letter_case(name).replace("sub__", "") }

fn _pragma_letter_case_flat_unstyled(name: &str) -> String {
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

fn pragma_fenced_atoms_are_not_functions(tree: &XM) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(Operator(op), ..) = tree {
    match &**op {
      XM::Lexeme(ref _lexeme, ref atom_meta) => {
        if let Some(ref fences) = atom_meta.fenced {
          if fences.as_str() == "parens" {
            return Err(
              "pruning non-argument parenthetical atom, used as LHS of function application"
                .into(),
            );
          }
        }
      }
      _ => {}
    }
  }
  Ok(())
}



fn pragma_fenced_letters_are_function_arguments(tree: &XM) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(Operator(op), ref args, ..) = tree {
    match **op {
      XM::Lexeme(ref oplexeme, _) if oplexeme == "x.invisible_operator" => {
        if let Some(top_lhs) = args.trees().first() {
          if let Some(ref fences) = top_lhs.get_meta().fenced {
            if fences.as_str() == "parens" {
              if let XM::Lexeme(lhs_name, _) = top_lhs.get_baseline() {
                if !lhs_name.starts_with("NUMBER") {
                  return Err(
                    "pruning non-argument parenthetical atom, used as LHS of invisible times"
                      .into(),
                  );
                }
              }
            }
          }

          // Slightly tricky check -- the top RHS needs to be fenced, but we care about the
          // "baseline" content being a variable - disregarding scripts.
          if let Some(top_rhs) = args.trees().get(1) {
            if let XM::Lexeme(rhs_name, _) = top_rhs.get_baseline() {
              if let Some(ref fences) = top_rhs.get_meta().fenced {
                if fences.as_str() == "parens" {
                  // if the RHS is a number, prune unless the LHS is fenced (things like cycle
                  // notation)
                  if !rhs_name.starts_with("NUMBER") {
                    return Err(
                      "pruning non-argument parenthetical atom, used as RHS of invisible times"
                        .into(),
                    );
                  } else {
                    match args.trees().first() {
                      Some(XM::Lexeme(_, lhs_meta)) if lhs_meta.fenced.is_none() => {
                        return Err(
                          "pruning non-argument parenthetical NUMBER, used as RHS of invisible \
                           times"
                            .into(),
                        );
                      },
                      Some(XM::Apply(_, _, _, lhs_meta)) if lhs_meta.fenced.is_none() => {
                        return Err(
                          "pruning non-argument parenthetical NUMBER, used as RHS of invisible \
                           times"
                            .into(),
                        );
                      },
                      _ => {},
                    }
                  }
                }
              }
            }
          }
        }
      },
      _ => {},
    }
  }
  Ok(())
}

/// If we have two standalone letters in the same, such as "A x" or "F X", prune parses that
/// interpret them as an application, unless they have a role that clearly indicates the LHS is
/// intended as a functional. This may need to be applied as an optional filter at the end of
/// pruning, as there are known, albeit rare, counter-examples ("f x").
fn pragma_unfenced_letter_arguments_require_visual_cues(tree: &XM) -> Result<(), Box<dyn Error>> {
  match *tree {
    XM::Apply(Operator(ref op), ref args, ..) if args.0.len() == 1 => {
      if let Some(XM::Lexeme(ref arg_name, ref atom_meta)) = args.0[0] {
        if atom_meta.fenced.is_none() {
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
      if let Some(XM::Lexeme(ref arg_name, ref atom_meta)) = args.0[0] {
        if arg_name.starts_with("OPFUNCTION") && atom_meta.fenced.is_none() {
          return Err("OPFUNCTIONs are rarely arguments, prune.".into());
        }
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
  if let XM::Apply(ref op, ref args, ..) = tree {
    if args.0.len() == 1 {
      match &*op.0 {
        XM::Lexeme(ref op_name, _) if op_name.starts_with("ID:") => match args.0[0] {
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
  }
  Ok(())
}

/// If we see " h f (x,y) " a parse where
/// "h f" are invisible multiplied, is not our first choice.
/// A right-associative application h(f(x,y)) is instead.
/// Prune it out if possible.
fn pragma_higher_order_invisible_ops_are_exceptions(tree: &XM) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(Operator(op), ref args, ..) = tree {
    match **op {
      XM::Lexeme(ref oplexeme, _) if oplexeme == "x.invisible_operator" => {
        let trees = args.trees();
        if trees.len() == 2 {
          let lhs = trees[0];
          let rhs = trees[1];
          if let XM::Lexeme(ref lhs_name, _) = lhs.get_baseline() {
            if let XM::Lexeme(ref rhs_name, _) = rhs.get_baseline() {
              if name_is_functional_or_id(lhs_name) && name_is_functional(rhs_name) {
                return Err(
                  "Pruning higher order 'FUNCTION x FUNCTION' parse to give precedence to \
                   right-associative readings"
                    .into(),
                );
              }
            }
          }
        }
      },
      _ => {},
    }
  }

  Ok(())
}

/// If two numbers are left next to each other, as in "10(5)" it is rarely (never?) the intention
/// that they are to be multiplied Prune such parses. We have special rules for some notations, such
/// as "dlmf_range".
fn pragma_adjacent_numbers_dont_use_invisible_times(tree: &XM) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(Operator(op), ref args, ..) = tree {
    match **op {
      XM::Lexeme(ref oplexeme, _) if oplexeme == "x.invisible_operator" => {
        let arg_trees = args.trees();
        if arg_trees.len() == 2 {
          if let Some(lhs) = arg_trees.first() {
            if lhs.base_operator_name().starts_with("NUMBER") {
              if let Some(rhs) = arg_trees.get(1) {
                if rhs.base_operator_name().starts_with("NUMBER") {
                  return Err(
                    "pruning two adjacent NUMBERs that used an invisible operator".into(),
                  );
                }
              }
            }
          }
        }
      },
      _ => {},
    }
  }
  Ok(())
}

/// Sometimes differentials can be written in numerators, but only if followed by their variable.
/// The only case (that I currently know) of a standalone "d" is in the Lebnitz derivative notation.
/// For which we have a special rule. So the generic fraction parse should be pruned.
fn pragma_standalone_diffops_are_not_numerators(tree: &XM) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(Operator(op), ref args, ..) = tree {
    match **op {
      XM::Lexeme(ref oplexeme, _) if oplexeme.starts_with("MULOP") => {
        if let Some(XM::Lexeme(ref numlexeme, _)) = args.trees().first() {
          if numlexeme.starts_with("DIFFOP") {
            return Err("pruning standalone diffops are not numerators".into());
          }
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
  if let XM::Apply(Operator(op), ref args, ..) = tree {
    if args.trees().len() == 1 {
      if let XM::Apply(Operator(ref op_op), _, _, ref op_meta) = **op {
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
    }
  }
  Ok(())
}

/// Adjacent functions don't unify into a single operator
/// as they are meant to apply right-to-left, one-by-one
fn pragma_adjacent_functions_dont_unify_into_op(tree: &XM) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(Operator(op), ..) = tree {
    if let XM::Apply(Operator(inner_op), inner_args, _, inner_meta) = &**op {
      if inner_meta.fenced.is_none() {
        if let XM::Lexeme(name, _) = inner_op.get_baseline() {
          if name_is_functional_or_id(name) {
            let inner_trees = inner_args.trees();
            if inner_trees.len() == 1 {
              if let XM::Lexeme(rhs_name, _) = inner_args.trees().first().unwrap().get_baseline() {
                if name_is_functional(rhs_name) {
                  return Err(
                    "Two applied FUNCTIONS as operator violates right-associative behavior.".into(),
                  );
                }
              }
            }
          }
        }
      }
    }
  }
  Ok(())
}
/// Postfix
fn pragma_postfix_terms_are_fenced_if_single_arg(tree: &XM) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(Operator(op), ref args, ..) = tree {
    let arg_trees = args.trees();
    if arg_trees.len() == 1 {
      if let Some(XM::Apply(Operator(arg_op), _, _, arg_meta)) = arg_trees.first() {
        if arg_op.base_operator_name().starts_with("POSTFIX") && arg_meta.fenced.is_none() {
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
  if let XM::Apply(Operator(ref op), ref args, _, _) = tree {
    if let XM::Token(ref props, _) = **op {
      if props.role.as_deref().is_some_and(|r|
        r == "SUBSCRIPTOP" || r == "SUPERSCRIPTOP")
      {
        // Check if base (first arg) is "absent" — standalone script
        if let Some(Some(XM::Token(ref base_props, _))) = args.0.first() {
          if base_props.meaning.as_deref() == Some("absent") {
            return Err(
              "Prune: standalone floating script (base=absent) should attach as pre-script.".into()
            );
          }
        }
      }
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
  if let XM::Apply(_, ref args, ..) = tree {
    let trees = args.trees();
    if trees.len() >= 2 {
      let first_absent = matches!(trees.first(),
        Some(XM::Token(ref p, _)) if p.meaning.as_deref() == Some("absent"));
      let last_absent = matches!(trees.last(),
        Some(XM::Token(ref p, _)) if p.meaning.as_deref() == Some("absent"));
      if first_absent && last_absent {
        return Err(
          "Prune: bilateral absent (absent on both sides) is never valid notation.".into()
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
  if let XM::Apply(Operator(op), ref args, ..) = tree {
    let is_invisible_times = match **op {
      XM::Lexeme(ref oplexeme, _) => oplexeme.contains("invisible_operator"),
      XM::Token(ref props, _) => {
        props.meaning.as_deref() == Some("times")
          && props.role.as_deref() == Some("MULOP")
      },
      _ => false,
    };
    if is_invisible_times {
      let trees = args.trees();
      if trees.len() == 2 {
        // LHS is a function application (Apply(function, arg))
        if let XM::Apply(Operator(ref func_op), ref func_args, _, ref func_meta) = trees[0] {
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
              XM::Apply(Operator(ref rhs_op), _, _, _) => {
                // Scripted factors (SUPERSCRIPTOP/SUBSCRIPTOP) are simple
                let rhs_role = match &**rhs_op {
                  XM::Token(ref props, _) => props.role.as_deref().unwrap_or(""),
                  XM::Lexeme(ref lex, _) => lex.split(':').next().unwrap_or(""),
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
                 prefer wider absorption.".into()
              );
            }
          }
        }
      }
    }
  }
  // Also check N-ary invisible_times chains for bare OPFUNCTION in non-terminal
  // positions. E.g. Apply(×, [f@(x), d, x]) where d is bare OPFUNCTION at index 1
  // (not the last). The competing parse Apply(×, [f@(x), d@(x)]) is preferred.
  // This covers the pattern: ∫ f(x) \diffd x → f@(x) * diffd@(x), not f@(x)*d*x.
  if let XM::Apply(Operator(op), ref args, ..) = tree {
    let is_invisible_times = match **op {
      XM::Lexeme(ref oplexeme, _) => oplexeme.contains("invisible_operator"),
      XM::Token(ref props, _) => {
        props.meaning.as_deref() == Some("times")
          && props.role.as_deref() == Some("MULOP")
      },
      _ => false,
    };
    if is_invisible_times {
      let trees = args.trees();
      // Check non-terminal positions for bare OPFUNCTION tokens
      if trees.len() >= 3 {
        for i in 0..trees.len() - 1 {
          let is_bare_opfunction = match trees[i] {
            XM::Token(ref props, _) => props.role.as_deref() == Some("OPFUNCTION"),
            XM::Lexeme(ref lex, _) => lex.starts_with("OPFUNCTION:"),
            _ => false,
          };
          if is_bare_opfunction {
            return Err(
              "Prune: bare OPFUNCTION in N-ary invisible_times chain (non-terminal) — \
               prefer absorption (opfunction@(next_arg))."
                .into(),
            );
          }
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
  if let XM::Apply(Operator(op), ref args, ..) = tree {
    let is_mulop = match **op {
      XM::Token(ref props, _) => {
        props.role.as_deref() == Some("MULOP")
      },
      XM::Lexeme(ref lex, _) => {
        lex.starts_with("MULOP") || lex.contains("invisible_operator")
      },
      _ => false,
    };
    if is_mulop {
      let trees = args.trees();
      if trees.len() == 2 {
        // LHS is a bigop application (Apply with BIGOP/SUMOP/INTOP/LIMITOP/DIFFOP op)
        if let XM::Apply(Operator(ref bigop_op), _, _, _) = trees[0] {
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
              XM::Apply(Operator(ref rhs_op), _, _, _) => {
                let rhs_role = match &**rhs_op {
                  XM::Token(ref props, _) => props.role.as_deref().unwrap_or(""),
                  XM::Lexeme(ref lex, _) => lex.split(':').next().unwrap_or(""),
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
  if let XM::Apply(Operator(op), ref args, ..) = tree {
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
        if i == 0 { continue; } // first arg can legitimately start with unary
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
  if let XM::Apply(Operator(op), ref args, ..) = tree {
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
  if let XM::Apply(Operator(op), ref args, ..) = tree {
    match **op {
      XM::Lexeme(ref oplexeme, _) if oplexeme == "arith1.divide" => {
        let arg_trees = args.trees();
        if arg_trees.len() == 2 {
          if let XM::Lexeme(arg1_name, arg1_meta) = arg_trees[0] {
            if let XM::Lexeme(arg2_name, arg2_meta) = arg_trees[1] {
              if arg1_name.starts_with("NUMBER")
                && arg2_name.starts_with("NUMBER")
                && !arg1_meta.syntax_trace.is_empty()
                || !arg2_meta.syntax_trace.is_empty()
              {
                return Err(
                  "only tokens are allowed in numeric fractions, derived rules are pruned to \
                   avoid redundancy."
                    .into(),
                );
              }
            }
          }
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

static _PRAGMATIC_BLOCK_MAP: Lazy<HashMap<char, String>> = Lazy::new(|| {
  // generally, we can observe that the latin alphabet shares "intent"
  // in blocks of 3 letter in mathematics,
  // as a fast-and-loose rule of thumb. a-e is an exception as
  // a rather stable 5 letter block with shared utility.
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
    let mark = format!("{start}{end}");
    let up_start = start.to_ascii_uppercase();
    let up_end = end.to_ascii_uppercase();
    for c_u8 in (*start as u8)..=(*end as u8) {
      map.insert(c_u8.into(), mark.clone());
    }
    let up_mark = format!("{up_start}{up_end}");
    for c_u8 in (up_start as u8)..=(up_end as u8) {
      map.insert(c_u8.into(), up_mark.clone());
    }
  }
  for (start, end) in greek_blocks.iter().chain(up_greek_blocks.iter()) {
    let mark = format!("{start}{end}");
    for c_u32 in (*start as u32)..=(*end as u32) {
      map.insert(std::char::from_u32(c_u32).unwrap(), mark.clone());
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
  if let XM::Apply(Operator(op), ref args, ..) = tree {
    if let XM::Lexeme(ref oplexeme, _) = **op {
      if oplexeme == "x.invisible_operator" {
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
    }
    // Recurse into all subtrees
    for subtree in args.trees() {
      check_invisible_times_recursive(subtree)?;
    }
  }
  Ok(())
}

/// Check if a tree node is an invisible-times application
fn is_invisible_times_apply(tree: &XM) -> bool {
  if let XM::Apply(Operator(op), ..) = tree {
    if let XM::Lexeme(ref oplexeme, _) = **op {
      return oplexeme == "x.invisible_operator";
    }
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
    XM::Apply(Operator(op), ref args, ..) => {
      if let XM::Lexeme(ref oplexeme, _) = **op {
        // For invisible-times applications, check operator and all args
        if oplexeme == "x.invisible_operator" {
          return args.trees().iter().all(|a| all_simple_identifiers(a));
        }
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
    XM::Token(ref props, _) => {
      // Token with UNKNOWN/ID/NUMBER role
      props.role.as_deref().is_some_and(|r| {
        r == "UNKNOWN" || r == "ID" || r == "NUMBER"
      }) && props.meaning.as_deref() != Some("absent")
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
  if let XM::Apply(Operator(op), ref args, ..) = tree {
    if let XM::Lexeme(ref name, _) = **op {
      let is_addop = name.starts_with("ADDOP");
      let is_mulop = name.starts_with("MULOP") || name == "x.invisible_operator";
      let is_relop = name.starts_with("RELOP");

      // If we're inside an addop/mulop and this node is a relop, reject
      if inside_addop_or_mulop && is_relop {
        return Err(
          "Pruning: RELOP found inside ADDOP/MULOP — relations must be at the outermost level"
            .into(),
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
  }
  Ok(())
}

/// Check if a tree represents a fenced (parenthesized) expression.
fn is_fenced(tree: &XM) -> bool {
  match tree {
    XM::Lexeme(_, ref meta) => meta.fenced.is_some(),
    XM::Apply(_, _, _, ref meta) => meta.fenced.is_some(),
    _ => false,
  }
}
