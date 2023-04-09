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
      ConsistentCase,
      ConsistentCaseFlat,
      ConsistentCaseFlatUnstyled,
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
      XM::Apply(Operator(op), args, _, _) => {
        self.validate_recursive(op)?;
        for arg_subtree in args.trees() {
          self.validate_recursive(arg_subtree)?;
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
  if let XM::Apply(Operator(op), _, _, _) = tree {
    if let XM::Lexeme(ref _lexeme, ref atom_meta) = **op {
      if let Some(ref fences) = atom_meta.fenced {
        if fences.as_str() == "parens" {
          return Err(
            "pruning non-argument parenthetical atom, used as LHS of function application".into(),
          );
        }
      }
    }
  }
  Ok(())
}

fn pragma_fenced_letters_are_function_arguments(tree: &XM) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(Operator(op), ref args, _, _) = tree {
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
    XM::Apply(Operator(ref op), ref args, _, _) if args.0.len() == 1 => {
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
    XM::Apply(_, ref args, _, _) if args.0.len() == 1 => {
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
  if let XM::Apply(ref op, ref args, _, _) = tree {
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
          Some(XM::Apply(_, _, _, _)) => {
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
  if let XM::Apply(Operator(op), ref args, _, _) = tree {
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
  if let XM::Apply(Operator(op), ref args, _, _) = tree {
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
  if let XM::Apply(Operator(op), ref args, _, _) = tree {
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
  if let XM::Apply(Operator(op), ref args, _, _) = tree {
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
  if let XM::Apply(Operator(op), _, _, _) = tree {
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
  if let XM::Apply(Operator(op), ref args, _, _) = tree {
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

fn pragma_restrict_numeral_fractions(tree: &XM) -> Result<(), Box<dyn Error>> {
  if let XM::Apply(Operator(op), ref args, _, _) = tree {
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
