use rustc_hash::FxHashMap as HashMap;
use std::borrow::Cow;
use std::error::Error;
use std::rc::Rc;

use libxml::tree::Node as XMLNode;
use marpa::lexer::token::Token;
use marpa::stack::*;
use marpa::thin::Value;
use marpa::tree_builder::*;

use latexml_core::common::font::{self, Font};
use latexml_core::common::xml::element_nodes;
use latexml_core::document::Document;
use latexml_core::raw_map;

use self::tree::lookup_lex_node;
pub use self::tree::{Args, Operator, XM, XProps};
use latexml_core::binding::def::dialect::get_xmarg_id;

use crate::pragmatics::ValidationPragmatics;
use crate::util::create_xmrefs;

mod curry;
mod from;
pub mod metadata;
pub mod tree;

use metadata::Meta;

/// A runtime context for a semantic math parser action
/// Ideally, these are all immutable borrows of various `Core` data.
pub struct ActionContext<'a> {
  /// The original XML nodes involved in this parse request
  pub nodes:    &'a [XMLNode],
  /// The owner document of the parsed nodes
  pub document: &'a mut Document,
}
pub type ActionClosure = Rc<
  dyn Fn(
    i32,
    Vec<Option<XM>>,
    &[ValidationPragmatics],
    ActionContext,
  ) -> Result<Option<XM>, Box<dyn Error>>,
>;

#[derive(Default)]
pub struct Actions {
  dispatch: HashMap<i32, ActionClosure>,
}

impl Actions {
  pub fn register(&mut self, id: i32, closure: ActionClosure) { self.dispatch.insert(id, closure); }
  /// Whether a rule has a registered semantic action. Used by the
  /// ASF traverser to discriminate "structural/literal" rules (treat
  /// as transparent byte-passthrough — match legacy `rollup_token_rec`)
  /// from "semantic" rules (call `action_on`).
  pub fn has_action(&self, id: i32) -> bool { self.dispatch.contains_key(&id) }
  pub fn action_on(
    &self,
    id: i32,
    mut args: Vec<Option<XM>>,
    pragmas: &[ValidationPragmatics],
    ctxt: ActionContext,
  ) -> Result<Option<XM>, Box<dyn Error>> {
    if let Some(action) = self.dispatch.get(&id) {
      action(id, args, pragmas, ctxt)
    } else {
      match args.len() {
        0 => Ok(None),
        1 => Ok(args.remove(0)),
        more => {
          eprintln!(
            "Only returning first of {more:?} elements at rule id {id:?} content: {args:?}"
          );
          Ok(args.remove(0))
        },
      }
    }
  }

  pub fn get_tree(
    &self,
    b: TreeBuilder,
    v: Value,
    pragmas: &[ValidationPragmatics],
    ctxt: ActionContext,
  ) -> Result<Option<XM>, Box<dyn Error>> {
    let handle = proc_value(b, v);
    self.translate_node(&handle, pragmas, ctxt)
  }

  pub fn translate_node<T: Token>(
    &self,
    n: &Handle<T>,
    pragmas: &[ValidationPragmatics],
    ctxt: ActionContext,
  ) -> Result<Option<XM>, Box<dyn Error>> {
    match *n.borrow() {
      Node::Tree(ref rule, ref children) => {
        let mut translated_children = Vec::with_capacity(children.len());
        for child in children.iter() {
          let translated = self.translate_node(child, pragmas, ActionContext {
            nodes:    ctxt.nodes,
            document: ctxt.document,
          })?;
          translated_children.push(translated);
        }
        self.action_on(*rule, translated_children, pragmas, ctxt)
      },
      Node::Rule(ref rule, ref children) => {
        let mut translated_children = Vec::with_capacity(children.len());
        for child in children.iter() {
          translated_children.push(self.translate_node(child, pragmas, ActionContext {
            nodes:    ctxt.nodes,
            document: ctxt.document,
          })?);
        }
        self.action_on(*rule, translated_children, pragmas, ctxt)
      },
      Node::Token(_ty, ref val) => {
        let token_str = ::std::str::from_utf8(val).unwrap_or("malformed-utf8");
        Ok(Some(
          XM::Lexeme(Rc::from(token_str), Meta::default()).specialize(Meta::default(), pragmas)?,
        ))
      },
      Node::Leaf(ref tok) => Ok(Some(XM::Lexeme(Rc::from(tok.to_string().as_str()), Meta::default()))),
      Node::Null(_) => {
        // e.g.* argument failed nothing, just skip.
        Ok(None)
        // XM::Lexeme("null".into())
      },
    }
  }
}

/// standard infix application of an operator
pub fn infix_apply(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => arg1, infixop, arg2);
  // Composition (meaning="compose") requires function-level operands.
  // Prune parses where an operand is an applied function (ground term).
  // f∘sin x → prefer (f∘sin)(x), not f∘(sin(x))
  if let Some(XM::Lexeme(ref lex, _)) = infixop {
    if lex.contains(":compose:") {
      // Check that operands are function-level (not applied/ground)
      if is_applied_function(&arg1) || is_applied_function(&arg2) {
        return Err(
          "infix_apply: compose requires function-level operands, not applied functions".into(),
        );
      }
      // Compose left-associativity: reject right-nested form `f ∘ (g ∘ h)`.
      // The grammar admits both `(f ∘ g) ∘ h` and `f ∘ (g ∘ h)` for chains
      // like `f * g * h`. Math convention is to left-associate composition,
      // so canonicalize by dropping the right-nested form here; Marpa's
      // alternative parse with the left-nested form survives.
      if let Some(XM::Apply(Operator(ref rhs_op), ..)) = arg2 {
        let rhs_is_compose = match &**rhs_op {
          XM::Lexeme(rl, _) => rl.contains(":compose:"),
          XM::Token(p, _) => p.meaning.as_deref() == Some("compose"),
          _ => false,
        };
        if rhs_is_compose {
          return Err(
            "infix_apply: compose is left-associative — reject right-nested f ∘ (g ∘ h)".into(),
          );
        }
      }
    }
  }
  let apply_tree = XM::Apply(
    infixop.into(),
    Args(vec![arg1, arg2]),
    XProps::default(),
    Meta::default(),
  );
  Ok(Some(apply_tree))
}

/// Check if an XM node is an applied function (curry level 1 / ground term).
/// Applied functions are Apply(function, args...) — the function has been applied to arguments.
fn is_applied_function(xm: &Option<XM>) -> bool {
  if let Some(XM::Apply(ref op, ref args, ..)) = xm {
    // An Apply with a function/trigfunction/opfunction operator that has arguments
    // is a ground-level application, not a function value.
    if !args.0.is_empty() {
      if let XM::Lexeme(ref lex, _) = *op.0 {
        return lex.starts_with("TRIGFUNCTION:")
          || lex.starts_with("OPFUNCTION:")
          || lex.starts_with("FUNCTION:");
      }
    }
  }
  false
}

/// Perl MathGrammar: Anything : Statement PUNCT <leftop: Statement PUNCT Statement>
/// Creates a list@(...) or formulae@(...) XMDual: content arm is Apply(meaning=list/formulae,
/// refs...), presentation arm is Wrap(items with separators).
/// Left-recursive: first call creates a 2-item list, subsequent calls extend it.
/// Perl distinction: comma-separated relational formulas at top level → "formulae",
/// comma-separated plain expressions → "list".
pub fn list_apply(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  // Grammar is left-recursive: `statements punct statement => list_apply`
  // so `left` is the accumulated structure and `right` is the new item.
  unp!(args => left, sep, right);
  let mut left = left;
  let mut right = right.unwrap();
  let sep = sep.unwrap();

  // list_apply always produces "list". Reject certain cases to force
  // Marpa to use the competing formula_list rule (formulae_apply) instead.
  //
  // Rule 1: Both items relational → should be formulae, not list.
  let left_rel = left.as_ref().is_some_and(is_relational_item);
  let right_rel = is_relational_item(&right);
  if left_rel && right_rel {
    return Err("list_apply: both items relational, use formulae_apply instead".into());
  }
  // Rule 2: If EITHER item is relational, reject. This prevents commas from
  // creating lists within relational formulas. For `a=b, c=d`, the inner
  // `list_apply(b, comma, c)` is rejected because the formula_list rule
  // should handle the comma as a formula boundary instead.
  // Exception: left can be relational when it's being extended (left is already
  // a list/formulae Dual from a previous list_apply — handled by extension below).
  if left_rel || right_rel {
    // Allow extension of existing list/formulae Duals (flat accumulation)
    let left_is_list_dual = left.as_ref().is_some_and(|l| {
      if let XM::Dual(ref content, ..) = l {
        if let XM::Apply(ref op, ..) = **content {
          if let XM::Token(ref props, _) = *op.0 {
            return props.meaning.as_deref() == Some("list")
              || props.meaning.as_deref() == Some("formulae");
          }
        }
      }
      false
    });
    if !left_is_list_dual {
      return Err("list_apply: item is relational, use formulae_apply instead".into());
    }
  }
  // Rule 3: Reject when either item has `absent` as a relop operand.
  // `absent` means equation fragment — fragments should be single formulas,
  // not list items. If `absent` appears, the expression should be parsed
  // as a single formula, not broken into a list.
  if left.as_ref().is_some_and(has_absent_relop_operand) || has_absent_relop_operand(&right) {
    return Err("list_apply: absent relop operand (should be single formula, not list)".into());
  }
  // Rule 4: Reject when an item is a BARE `conditional@(...)` Apply
  // — `|` (conditional / MODIFIEROP) binds LOOSER than `,` (list
  // separator) when unfenced, so the conditional should wrap the
  // list. For `x|y, z, t`, prefer `conditional@(x, list@(y, z, t))`.
  //
  // **Exception**: when the conditional IS inside a parens-fenced
  // group (e.g. `(a|b), (a|b)`), each `(a|b)` is a complete
  // `Dual(conditional(a,b), Wrap[(, a, |, b, )])` unit, which CAN
  // legitimately be a list item. Detect this by checking whether
  // the Dual's presentation Wrap starts with OPEN paren.
  let bare_conditional = |item: &XM| -> bool {
    match item {
      // Naked Apply(conditional, ...): always bare.
      XM::Apply(Operator(op), ..) => {
        let meaning = match &**op {
          XM::Token(p, _) => p.meaning.as_deref(),
          XM::Lexeme(name, _) => Some(&**name),
          _ => None,
        };
        meaning == Some("conditional")
      },
      // Dual wrapping conditional: bare only if presentation is NOT
      // parens-fenced.
      XM::Dual(content, pres, _, _) => {
        let inner_is_conditional = if let XM::Apply(Operator(ref op), ..) = **content {
          let meaning = match &**op {
            XM::Token(p, _) => p.meaning.as_deref(),
            XM::Lexeme(name, _) => Some(&**name),
            _ => None,
          };
          meaning == Some("conditional")
        } else {
          false
        };
        if !inner_is_conditional {
          return false;
        }
        // Check presentation for parens fence.
        let presentation_is_parens_fenced = if let XM::Wrap(ref items, ..) = **pres {
          let first_is_open_paren = matches!(items.first(),
            Some(XM::Token(p, _))
              if p.role.as_deref() == Some("OPEN")
                && p.content.as_deref() == Some("("))
            || matches!(items.first(),
              Some(XM::Lexeme(name, _)) if name.starts_with("OPEN:(:"));
          first_is_open_paren
        } else {
          false
        };
        !presentation_is_parens_fenced
      },
      _ => false,
    }
  };
  if left.as_ref().is_some_and(bare_conditional) || bare_conditional(&right) {
    return Err(
      "list_apply: child is BARE `conditional@` at root — conditional/MODIFIEROP binds \
       looser than comma, so the bare conditional should wrap the list (the parens-fenced \
       case is allowed)."
        .into(),
    );
  }
  let meaning = "list";

  // If left is already a list/formulae Dual, extend it (flat accumulation).
  // For formulae with \quad separators, post-processing in restructure_formulae_right
  // converts flat to right-recursive nesting (matching Perl's moreRHS/maybeColRHS).
  if let Some(XM::Dual(ref mut content, ref mut pres, ..)) = left {
    if let XM::Apply(ref op, ref mut op_args, ..) = **content {
      if let XM::Token(ref props, _) = *op.0 {
        if props.meaning.as_deref() == Some("list") || props.meaning.as_deref() == Some("formulae")
        {
          // Extend: add ref for new item to content args
          let new_ref = create_xmrefs(&mut [&mut right], ctxt)?;
          op_args.0.extend(new_ref.into_iter().map(Option::Some));
          // Add separator and new item to presentation wrap
          if let XM::Wrap(ref mut items, ..) = **pres {
            items.push(sep);
            items.push(right);
          }
          return Ok(left);
        }
      }
    }
  }

  list_or_formulae_create(left.unwrap(), sep, right, meaning, ctxt)
}

/// Perl: within a Formula, comma-separated expressions after a relop form a list RHS.
/// Like list_apply but rejects items that contain relations (those should go to statement level).
/// This prevents `1<x<10,2<y<20` from being parsed as `1 < x < list(10,2) < y < ...`.
pub fn formula_list_apply(
  rule_id: i32,
  args: Vec<Option<XM>>,
  p: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  // Check if ANY of the items contain relations — if so, reject this parse
  // so Marpa falls back to the statement-level comma separation.
  let has_relational = args
    .iter()
    .any(|a| a.as_ref().is_some_and(is_relational_item));
  if has_relational {
    return Err("formula_list_apply: items contain relations, use statement-level list".into());
  }
  // Also check if the left side is already a list Dual containing relational items
  if let Some(Some(XM::Dual(ref content, ..))) = args.first() {
    if let XM::Apply(ref op, ref op_args, ..) = **content {
      if let XM::Token(ref props, _) = *op.0 {
        if props.meaning.as_deref() == Some("list") {
          // Check if any list item is relational
          for arg in &op_args.0 {
            if arg.as_ref().is_some_and(is_relational_item) {
              return Err("formula_list_apply: list contains relational items".into());
            }
          }
        }
      }
    }
  }
  list_apply(rule_id, args, p, ctxt)
}

/// Check if an XM tree contains `absent` as a direct operand of a relop.
/// Absent operands are valid at the top level (equation fragments like `= f(x)`)
/// but should be pruned when inside inner rules (lists, fenced expressions, function args).
fn has_absent_relop_operand(xm: &XM) -> bool {
  if let XM::Apply(ref op, ref args, ..) = xm {
    let is_rel = match &*op.0 {
      XM::Token(ref props, _) => {
        props.meaning.as_deref() == Some("multirelation")
          || props
            .role
            .as_deref()
            .is_some_and(|r| r.contains("RELOP") || r.contains("ARROW"))
      },
      XM::Lexeme(ref lex, _) => lex
        .split(':')
        .next()
        .is_some_and(|r| r.contains("RELOP") || r.contains("ARROW")),
      _ => false,
    };
    if is_rel {
      for arg in &args.0 {
        if let Some(XM::Token(ref props, _)) = arg {
          if props.meaning.as_deref() == Some("absent") {
            return true;
          }
        }
      }
    }
  }
  false
}

/// Check if an XM tree is a relational formula (contains RELOP or multirelation).
/// Used to distinguish Perl's "formulae" (comma-separated relations at top level)
/// from "list" (comma-separated plain expressions).
fn is_relational_item(xm: &XM) -> bool {
  match xm {
    XM::Apply(ref op, ..) => match &*op.0 {
      XM::Token(ref props, _) => {
        props.meaning.as_deref() == Some("multirelation")
          || props
            .role
            .as_deref()
            .is_some_and(|r| r.contains("RELOP") || r.contains("ARROW"))
      },
      XM::Lexeme(ref lex, _) => lex
        .split(':')
        .next()
        .is_some_and(|r| r.contains("RELOP") || r.contains("ARROW")),
      _ => false,
    },
    // A formulae XMDual is inherently relational (it wraps relational items)
    XM::Dual(ref content, ..) => {
      if let XM::Apply(ref op, ..) = **content {
        if let XM::Token(ref props, _) = *op.0 {
          return props.meaning.as_deref() == Some("formulae");
        }
      }
      false
    },
    _ => false,
  }
}

/// Perl: NewFormulae — punct-separated formulas at top level → meaning="formulae".
/// This action is used by `formula_list` (the top-level rule that competes with
/// `statements` via `list_apply`). It ALWAYS produces meaning="formulae", but
/// REJECTS the parse (returns Err) if no items are relational — causing Marpa
/// to fall back to the `statements` parse which produces "list".
pub fn formulae_apply(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => left, sep, right);
  let mut left = left;
  let mut right = right.unwrap();
  let sep = sep.unwrap();

  // Check if ANY item is relational — if not, reject this parse.
  // This forces Marpa to use the `statements` rule (list_apply) instead.
  let left_rel = left.as_ref().is_some_and(is_relational_item);
  let right_rel = is_relational_item(&right);

  // Reject when BOTH items are fragments (have `absent` as a relop operand).
  // A fragment + a complete formula is a common arXiv pattern: align'd
  // equation continuations followed by a side-condition, e.g.
  //   `\lesssim T(r,f) + \log r, \quad r \notin E_3`
  // where the LEFT (`\lesssim ...`) has an absent LHS because the
  // original LHS is on a previous line of an align block, and the RIGHT
  // (`r \notin E_3`) is the trailing condition annotation. The earlier
  // strict rule (reject if EITHER is a fragment) was correct only for
  // inner contexts (fenced lists, function args); at the top level
  // these pairings are well-formed and need to survive pragmatic prune.
  // True inner-context fragment cases are still pruned because BOTH
  // sides are typically fragments (or one is a single bare token).
  if left.as_ref().is_some_and(has_absent_relop_operand) && has_absent_relop_operand(&right) {
    return Err("formulae_apply: both operands are fragments (not complete formulae)".into());
  }
  // Also check inside an existing formulae Dual being extended
  if let Some(XM::Dual(ref content, ..)) = left {
    if let XM::Apply(_, ref args, ..) = **content {
      for arg in &args.0 {
        if arg.as_ref().is_some_and(has_absent_relop_operand) {
          return Err("formulae_apply: formulae contains fragment with absent".into());
        }
      }
    }
  }
  // Period separator always creates formulae (it's a hard formula boundary).
  // Comma separator requires at least one relational item.
  let sep_is_period = sep.get_value(ctxt.nodes).ok().is_some_and(|v| v == ".");
  if !left_rel && !right_rel && !sep_is_period {
    return Err("formulae_apply: no relational items, use list_apply instead".into());
  }

  // Reject when right is non-relational and left is relational, BUT only for
  // comma separators. Period is a hard formula boundary and should NOT trigger
  // list grouping. This forces `a=b, c` via `formula relop formula_list`
  // (producing `a = list(b,c)`) but allows `a=b. c` to produce `formulae(a=b, c)`.
  let sep_is_period = match &sep {
    XM::Lexeme(ref lex, _) => lex.starts_with("PERIOD:"),
    XM::Token(ref props, _) => props.role.as_deref() == Some("PERIOD"),
    _ => false,
  };
  if left_rel && !right_rel && !sep_is_period {
    return Err(
      "formulae_apply: non-relational right after relational left — use formula_list RHS".into(),
    );
  }

  let meaning = "formulae"; // always

  // If left is already a formulae Dual, extend it
  if let Some(XM::Dual(ref mut content, ref mut pres, ..)) = left {
    if let XM::Apply(ref op, ref mut op_args, ..) = **content {
      if let XM::Token(ref props, _) = *op.0 {
        if props.meaning.as_deref() == Some(meaning) {
          let new_ref = create_xmrefs(&mut [&mut right], ctxt)?;
          op_args.0.extend(new_ref.into_iter().map(Option::Some));
          if let XM::Wrap(ref mut items, ..) = **pres {
            items.push(sep);
            items.push(right);
          }
          return Ok(left);
        }
      }
    }
  }
  list_or_formulae_create(left.unwrap(), sep, right, meaning, ctxt)
}

fn list_or_formulae_create(
  mut left: XM,
  sep: XM,
  mut right: XM,
  meaning: &'static str,
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  let op = XProps {
    meaning: Some(Cow::Borrowed(meaning)),
    ..XProps::default()
  };
  let ref_args = create_xmrefs(&mut [&mut left, &mut right], ctxt)?;
  Ok(Some(XM::Dual(
    Box::new(XM::Apply(
      op.into(),
      Args(ref_args.into_iter().map(Option::Some).collect()),
      XProps::default(),
      Meta::default(),
    )),
    Box::new(XM::Wrap(
      vec![left, sep, right],
      XProps::default(),
      Meta::default(),
    )),
    XProps::default(),
    Meta::default(),
  )))
}

/// Restructure flat formulae with \quad separators into right-recursive nesting.
/// Perl's moreRHS/maybeColRHS builds right-recursive formulae@(f1, formulae@(f2, formulae@(f3,
/// f4))) for \quad-separated relational expressions. The left-recursive Marpa grammar produces flat
/// formulae@(f1, f2, f3, f4). This function converts flat to right-recursive after parsing.
///
/// The XMWrap items alternate: [item, sep, item, sep, item, ...]
/// Only restructures when ALL separators are \quad-type (have name containing "uad").
pub fn restructure_formulae_right(xm: &mut XM) -> Result<(), Box<dyn Error>> {
  // Recurse into children first (bottom-up)
  match xm {
    XM::Apply(_, ref mut args, ..) => {
      for arg in args.0.iter_mut().flatten() {
        restructure_formulae_right(arg)?;
      }
    },
    XM::Wrap(ref mut items, ..) => {
      for item in items.iter_mut() {
        restructure_formulae_right(item)?;
      }
    },
    XM::Dual(ref mut content, ref mut pres, ..) => {
      restructure_formulae_right(content)?;
      restructure_formulae_right(pres)?;

      // Check if this is a flat formulae Dual with \quad separators
      if let XM::Apply(ref op, ref op_args, ..) = **content {
        if let XM::Token(ref props, _) = *op.0 {
          if props.meaning.as_deref() == Some("formulae") && op_args.0.len() > 2 {
            if let XM::Wrap(ref items, ..) = **pres {
              // Check if ALL separators are \quad-type
              let all_quad = items
                .iter()
                .enumerate()
                .filter(|(i, _)| i % 2 == 1) // odd indices are separators
                .all(|(_, sep)| is_quad_separator(sep));
              if all_quad {
                restructure_flat_to_right(xm)?;
              }
            }
          }
        }
      }
    },
    XM::Choices(ref mut choices) => {
      for choice in choices.iter_mut() {
        restructure_formulae_right(choice)?;
      }
    },
    XM::Arg(ref mut items) => {
      for item in items.iter_mut() {
        restructure_formulae_right(item)?;
      }
    },
    _ => {},
  }
  Ok(())
}

/// Check if an XM node is a \quad-type separator (XMHint PUNCT with name containing "uad").
fn is_quad_separator(xm: &XM) -> bool {
  match xm {
    XM::Token(ref props, _) => props.name.as_deref().is_some_and(|n| n.contains("uad")),
    XM::Lexeme(ref lex, _) => lex.split(':').nth(1).is_some_and(|t| t.contains("uad")),
    _ => false,
  }
}

/// Convert a flat formulae XMDual to right-recursive nesting.
/// Input: formulae@(f1,f2,...,fn) with XMWrap [f1,s1,f2,s2,...,fn]
/// Output: formulae@(f1, formulae@(f2, ..., formulae@(fn-1, fn)))
/// Items are MOVED (not cloned) to avoid DOM node aliasing.
fn restructure_flat_to_right(xm: &mut XM) -> Result<(), Box<dyn Error>> {
  // Take ownership of the flat Dual's contents
  let old = std::mem::replace(xm, XM::Token(XProps::default(), Meta::default()));
  if let XM::Dual(content, pres, _props, _meta) = old {
    if let XM::Apply(_, op_args, ..) = *content {
      if let XM::Wrap(wrap_items, ..) = *pres {
        let n_refs = op_args.0.len();
        if n_refs <= 2 {
          // Already binary — put it back
          *xm = XM::Dual(
            Box::new(XM::Apply(
              XProps {
                meaning: Some(Cow::Borrowed("formulae")),
                ..XProps::default()
              }
              .into(),
              op_args,
              XProps::default(),
              Meta::default(),
            )),
            Box::new(XM::Wrap(wrap_items, XProps::default(), Meta::default())),
            _props,
            _meta,
          );
          return Ok(());
        }
        // Split XMWrap items into (items, separators).
        // wrap_items = [f1, s1, f2, s2, f3, s3, f4] (alternates item/sep).
        // Pre-size: ~half each, bounded by `wrap_items.len()`.
        let wrap_len = wrap_items.len();
        let mut items: Vec<XM> = Vec::with_capacity(wrap_len.div_ceil(2));
        let mut seps: Vec<XM> = Vec::with_capacity(wrap_len / 2);
        for (i, item) in wrap_items.into_iter().enumerate() {
          if i % 2 == 0 {
            items.push(item);
          } else {
            seps.push(item);
          }
        }
        let mut refs: Vec<Option<XM>> = op_args.0.into_iter().collect();

        // Build right-recursive from right to left
        // Start with last two items: formulae@(f_{n-1}, f_n)
        let last_item = items.pop().unwrap();
        let last_ref = refs.pop().unwrap();
        let last_sep = seps.pop().unwrap();
        let second_last_item = items.pop().unwrap();
        let second_last_ref = refs.pop().unwrap();

        let mut result = build_formulae_pair(
          second_last_item,
          last_sep,
          last_item,
          second_last_ref,
          last_ref,
        );

        // Wrap from right to left
        while let Some(item) = items.pop() {
          let sep = seps.pop().unwrap();
          let item_ref = refs.pop().unwrap();
          // Assign xmkey to inner result so outer can reference it
          let key = get_xmarg_id()?.to_string();
          if let XM::Dual(_, _, ref mut props, _) = result {
            props.xmkey = Some(Cow::Owned(key.clone()));
          }
          let inner_ref = Some(XM::Ref(XProps {
            xmkey: Some(Cow::Owned(key)),
            ..XProps::default()
          }));
          result = build_formulae_pair(item, sep, result, item_ref, inner_ref);
        }

        *xm = result;
        return Ok(());
      }
    }
  }
  // If pattern didn't match, this shouldn't happen since we checked before calling
  Ok(())
}

/// Build a 2-item formulae XMDual with pre-existing refs.
fn build_formulae_pair(
  left: XM,
  sep: XM,
  right: XM,
  left_ref: Option<XM>,
  right_ref: Option<XM>,
) -> XM {
  let op = XProps {
    meaning: Some(Cow::Borrowed("formulae")),
    ..XProps::default()
  };
  XM::Dual(
    Box::new(XM::Apply(
      op.into(),
      Args(vec![left_ref, right_ref]),
      XProps::default(),
      Meta::default(),
    )),
    Box::new(XM::Wrap(
      vec![left, sep, right],
      XProps::default(),
      Meta::default(),
    )),
    XProps::default(),
    Meta::default(),
  )
}

/// Post-processing: rename `list` to `vector`/`set` when delimiters wrap the list.
/// Perl's Fence receives flat (open, item, punct, ..., close) and uses encloseN tables.
/// Our grammar builds `list` via `list_apply` before fencing can see the items.
/// This pass walks the tree and checks if a list Dual's presentation XMWrap
/// starts with OPEN and ends with CLOSE, then applies the encloseN meaning.
pub fn rename_fenced_lists(
  xm: &mut XM,
  nodes: &[libxml::tree::Node],
) -> Result<(), Box<dyn Error>> {
  match xm {
    XM::Apply(_, ref mut args, ..) => {
      for arg in args.0.iter_mut().flatten() {
        rename_fenced_lists(arg, nodes)?;
      }
    },
    XM::Wrap(ref mut items, ..) => {
      for item in items.iter_mut() {
        rename_fenced_lists(item, nodes)?;
      }
      // Check if this Wrap contains [OPEN, list_Dual, CLOSE] — rename list meaning
      // Handles script content like ^{(1+,0+,1-,0-)} where OPEN/CLOSE are siblings
      // of the list Dual in the script's presentation Wrap.
      if items.len() >= 3 {
        let first_role = get_xm_role(&items[0]);
        let last_role = get_xm_role(items.last().unwrap());
        if first_role.as_deref() == Some("OPEN") && last_role.as_deref() == Some("CLOSE") {
          let o_val: String = items[0].get_value(nodes).unwrap_or_default().into_owned();
          let c_val: String = items
            .last()
            .unwrap()
            .get_value(nodes)
            .unwrap_or_default()
            .into_owned();
          // Find and rename any list Dual among the inner items
          let len = items.len();
          for item in items[1..len - 1].iter_mut() {
            if let XM::Dual(ref mut content, ..) = item {
              if let XM::Apply(ref mut op, ref args, ..) = **content {
                if let XM::Token(ref mut props, _) = *op.0 {
                  if props.meaning.as_deref() == Some("list") {
                    let n = args.0.len();
                    let new_meaning = match (o_val.as_ref(), c_val.as_ref()) {
                      ("(", ")") if n == 2 => Some("open-interval"),
                      ("[", "]") if n == 2 => Some("closed-interval"),
                      ("(", "]") if n == 2 => Some("open-closed-interval"),
                      ("[", ")") if n == 2 => Some("closed-open-interval"),
                      ("{", "}") => Some("set"),
                      ("(", ")") => Some("vector"), // n >= 3
                      _ => None,
                    };
                    if let Some(m) = new_meaning {
                      props.meaning = Some(Cow::Borrowed(m));
                    }
                  }
                }
              }
            }
          }
        }
      }
    },
    XM::Dual(ref mut content, ref mut pres, ..) => {
      rename_fenced_lists(content, nodes)?;
      rename_fenced_lists(pres, nodes)?;
    },
    XM::Choices(ref mut trees) => {
      for tree in trees.iter_mut() {
        rename_fenced_lists(tree, nodes)?;
      }
    },
    _ => {},
  }
  Ok(())
}

/// Post-processing: combine adjacent SUPOP tokens in script content.
/// Perl MathGrammar L720-723: supops = SUPOP(s) → prime2, prime3, etc.
/// Marpa often parses `\prime\prime` as `list@(prime, prime)` or `times(prime, prime)`
/// instead of using the `supops` grammar rule. This pass detects these patterns
/// and replaces them with combined `prime{N}` tokens.
pub fn combine_supop_post(xm: &mut XM, nodes: &[libxml::tree::Node]) -> Result<(), Box<dyn Error>> {
  match xm {
    XM::Apply(_, ref mut args, ..) => {
      for arg in args.0.iter_mut().flatten() {
        combine_supop_post(arg, nodes)?;
      }
    },
    XM::Wrap(ref mut items, ..) => {
      for item in items.iter_mut() {
        combine_supop_post(item, nodes)?;
      }
    },
    XM::Dual(ref mut content, ref mut pres, ..) => {
      combine_supop_post(content, nodes)?;
      combine_supop_post(pres, nodes)?;
      // Check if content is Apply(list/times, args) where pres Wrap has all SUPOP items
      if let XM::Apply(ref op, ref args, ..) = **content {
        let is_list_or_times = if let XM::Token(ref props, _) = *op.0 {
          props.meaning.as_deref() == Some("list") || props.meaning.as_deref() == Some("times")
        } else {
          false
        };
        if is_list_or_times && args.0.len() >= 2 {
          // Check presentation Wrap: all non-separator items should be SUPOP
          if let XM::Wrap(ref items, ..) = **pres {
            // Items at even indices (0, 2, 4, ...) are the actual tokens
            // Items at odd indices (1, 3, ...) are separators (PUNCT, MULOP)
            let all_supop = items
              .iter()
              .enumerate()
              .filter(|(i, _)| i % 2 == 0) // actual items at even positions
              .all(|(_, item)| get_xm_role(item).as_deref() == Some("SUPOP"));
            if all_supop {
              let count = args.0.len();
              let text: String = items
                .iter()
                .enumerate()
                .filter(|(i, _)| i % 2 == 0)
                .map(|(_, item)| {
                  item
                    .get_value(nodes)
                    .unwrap_or(Cow::Borrowed("′"))
                    .into_owned()
                })
                .collect();
              // Replace the Dual with a single combined SUPOP Token
              *xm = XM::Token(
                XProps {
                  role: Some(Cow::Borrowed("SUPOP")),
                  name: Some(Cow::Owned(format!("prime{count}"))),
                  content: Some(Cow::Owned(text)),
                  ..XProps::default()
                },
                Meta::default(),
              );
              return Ok(());
            }
          }
        }
      }
    },
    XM::Choices(ref mut trees) => {
      for tree in trees.iter_mut() {
        combine_supop_post(tree, nodes)?;
      }
    },
    _ => {},
  }
  Ok(())
}

fn get_xm_role(xm: &XM) -> Option<String> {
  match xm {
    XM::Lexeme(l, _) => l.split(':').next().map(|s| s.to_string()),
    XM::Token(p, _) => p.role.as_ref().map(|r| r.to_string()),
    _ => None,
  }
}

/// application with trailing elision, as in `x \cdot y \cdot\cdot\cdot`
pub fn infix_apply_and_elide(
  rule_id: i32,
  mut args: Vec<Option<XM>>,
  p: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => arg1, infixop, arg2, elision);
  // check if "left" is already an application of infix op, in which case we can do n-ary apply.
  if let Some(XM::Apply(new_op, mut new_args, props, meta)) =
    infix_apply_nary(rule_id, vec![arg1, infixop, arg2], p, ctxt)?
  {
    new_args.0.push(elision);
    Ok(Some(XM::Apply(new_op, new_args, props, meta)))
  } else {
    Ok(None)
  }
}

// infix_apply in the base case,
// but when chained, using the flat "multirelation" behavior of latexml
pub fn infix_relation(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => left, infixop, right);
  // Reject multirelation when the left formula's last operand is a list Dual.
  // For `a = b, c = d`: the wrong parse creates `a = list(b,c)` then tries
  // to extend with `= d`. If the left formula has a list as its last arg,
  // the comma should have been a formula boundary, not an expression list.
  // Rejecting here forces Marpa to use formula_list instead.
  if let Some(ref left_xm) = left {
    fn last_arg_is_list(xm: &XM) -> bool {
      match xm {
        XM::Apply(ref op, ref args, ..) => {
          // Check if this is a relational Apply (has RELOP/ARROW operator)
          let is_rel = match &*op.0 {
            XM::Token(ref props, _) => {
              props.meaning.as_deref() == Some("multirelation")
                || props
                  .role
                  .as_deref()
                  .is_some_and(|r| r.contains("RELOP") || r.contains("ARROW"))
            },
            XM::Lexeme(ref lex, _) => lex
              .split(':')
              .next()
              .is_some_and(|r| r.contains("RELOP") || r.contains("ARROW")),
            _ => false,
          };
          if is_rel {
            // Check last argument
            if let Some(Some(XM::Dual(ref content, ..))) = args.0.last() {
              if let XM::Apply(ref inner_op, ..) = **content {
                if let XM::Token(ref props, _) = *inner_op.0 {
                  return props.meaning.as_deref() == Some("list");
                }
              }
            }
          }
          false
        },
        _ => false,
      }
    }
    if last_arg_is_list(left_xm) {
      return Err(
        "infix_relation: left formula ends with list (comma should be formula boundary)".into(),
      );
    }
  }
  // if left has a "multirelation" already, add right in.
  // if left applies a relation, flatten it out to infix form.
  // base case - build a simple infix apply
  let mut left = left;
  match left {
    Some(XM::Apply(ref op, ref mut left_args, _, ref _left_meta)) => {
      if let XM::Token(ref tok, _) = *op.0 {
        if tok.meaning == Some(Cow::Borrowed("multirelation")) {
          left_args.0.push(infixop);
          left_args.0.push(right);
          Ok(left)
        } else {
          Ok(Some(XM::Apply(
            infixop.into(),
            Args(vec![left, right]),
            XProps::default(),
            Meta::default(),
          )))
        }
      } else if let XM::Lexeme(ref lex, ref _left_meta) = *op.0 {
        let first_part = lex.split(':').next().unwrap();
        if first_part.contains("RELOP") || first_part.contains("ARROW") {
          // first multirelation need is here.
          let multirel_tok = XProps {
            meaning: Some(Cow::Borrowed("multirelation")),
            ..XProps::default()
          };
          let mut drained_left_args = left_args.0.drain(..);
          let left_1 = drained_left_args.next().unwrap();
          let left_2 = drained_left_args.next().unwrap();
          let moved_op = (*op.0).clone();
          Ok(Some(XM::Apply(
            multirel_tok.into(),
            Args(vec![left_1, Some(moved_op), left_2, infixop, right]),
            XProps::default(),
            Meta::default(),
          )))
        } else {
          Ok(Some(XM::Apply(
            infixop.into(),
            Args(vec![left, right]),
            XProps::default(),
            Meta::default(),
          )))
        }
      } else {
        Ok(Some(XM::Apply(
          infixop.into(),
          Args(vec![left, right]),
          XProps::default(),
          Meta::default(),
        )))
      }
    },
    _ => Ok(Some(XM::Apply(
      infixop.into(),
      Args(vec![left, right]),
      XProps::default(),
      Meta::default(),
    ))),
  }
}

pub fn infix_apply_nary(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => left, infixop, right);
  let mut left = left;
  // Early prune: `Apply(OPERATOR, [single_unfenced_arg]) * simple_rhs`
  // — narrow form of the wider-absorption rule. E.g. `D@(x) * y * z`
  // should be `D@(x*y*z)`. Reject here so the grammar's
  // `prefix_apply_applyop` path wins. (Mirrors the OPERATOR check
  // in `apply_invisible_times` below.)
  let infixop_is_mulop = match infixop {
    Some(XM::Lexeme(ref lex, _)) => {
      let role = lex.split(':').next().unwrap_or("");
      role == "MULOP"
    },
    Some(XM::Token(ref p, _)) => p.role.as_deref() == Some("MULOP"),
    _ => false,
  };
  if infixop_is_mulop {
    if let Some(XM::Apply(Operator(ref left_op), ref left_args, _, ref left_meta)) = left {
      let op_role = match &**left_op {
        XM::Token(p, _) => p.role.as_deref().map(String::from),
        XM::Lexeme(lex_id, _) => {
          if let Some(id) = lex_id.split(':').next_back().and_then(|s| s.parse::<usize>().ok()) {
            if id > 0 && id <= ctxt.nodes.len() {
              ctxt.nodes[id - 1].get_attribute("role")
            } else {
              None
            }
          } else {
            None
          }
        },
        _ => None,
      };
      if op_role.as_deref() == Some("OPERATOR")
        && left_meta.fenced.is_none()
        && left_args.trees().len() == 1
      {
        let arg_unfenced = left_args.trees().first().map(|a| a.get_meta().fenced.is_none()).unwrap_or(true);
        let rhs_unfenced = right.as_ref().map(|r| r.get_meta().fenced.is_none()).unwrap_or(false);
        let rhs_is_simple = match right.as_ref() {
          Some(XM::Lexeme(..)) | Some(XM::Token(..)) | Some(XM::Wrap(..)) => true,
          Some(XM::Apply(Operator(ref rhs_op), ..)) => {
            let r = match &**rhs_op {
              XM::Token(p, _) => p.role.as_deref().unwrap_or(""),
              XM::Lexeme(lex, _) => lex.split(':').next().unwrap_or(""),
              _ => "",
            };
            r == "SUPERSCRIPTOP" || r == "SUBSCRIPTOP"
          },
          _ => false,
        };
        if arg_unfenced && rhs_is_simple && rhs_unfenced {
          return Err(
            "infix_apply_nary: left is applied OPERATOR — prefer wider absorption via \
             prefix_apply_applyop"
              .into(),
          );
        }
      }
    }
  }
  // left-to-right associative:
  // 1. if "left" is already an application of "infixop",
  // 2. then tuck "right" inside it.
  if let Some(XM::Apply(ref left_op, ref mut left_args, _, ref _m)) = left {
    if let XM::Lexeme(left_op_lex, _xmeta) = &*left_op.0 {
      if let Some(XM::Lexeme(ref infix_op_lex, _)) = infixop {
        let left_op_pieces: Vec<_> = left_op_lex.split(':').collect();
        let infix_op_pieces: Vec<_> = infix_op_lex.split(':').collect();
        if left_op_pieces.len() == 3
          && infix_op_pieces.len() == 3
          && left_op_pieces[0] == infix_op_pieces[0]
          && left_op_pieces[1] == infix_op_pieces[1]
          // Perl's LeftRec doesn't flatten prefix applications (1 arg = unary prefix)
          // Only flatten when left already has 2+ args (binary or n-ary)
          && left_args.0.len() >= 2
        {
          left_args.0.push(right);
          return Ok(left);
        }
      }
    }
  }
  // Perl left-to-right: explicit MULOP only takes one factor on the right.
  // a/bc → (a/b)*c, F×G dx → (F×G)*dx. NOT a/(b*c) or F×(G*dx).
  // When any explicit (non-invisible) MULOP has a right operand that is an invisible-times
  // application, extract just the first factor and chain the rest.
  // Transform: Apply(op, left, Apply(⁢, first, rest...)) → Apply(⁢, Apply(op, left, first), rest...)
  // Detect explicit (visible) MULOPs: /, ×, etc. — NOT invisible times (⁢ U+2062).
  let is_explicit_mulop = match &infixop {
    Some(XM::Lexeme(lex, _)) => {
      let role = lex.split(':').next().unwrap_or("");
      let symbol = lex.split(':').nth(2).unwrap_or("");
      role == "MULOP" && symbol != "\u{2062}" // not invisible times char
    },
    Some(XM::Token(props, _)) => {
      props.role.as_deref() == Some("MULOP") && props.content.as_deref() != Some("\u{2062}")
    },
    _ => false,
  };
  if is_explicit_mulop {
    let right_is_invisible_times = match &right {
      Some(XM::Apply(ref op, ref args, ..)) => {
        let op_is_times = match &*op.0 {
          XM::Lexeme(lex, _) => lex.split(':').nth(1) == Some("times"),
          XM::Token(props, _) => props.meaning.as_deref() == Some("times"),
          _ => false,
        };
        op_is_times && args.0.len() >= 2
      },
      _ => false,
    };
    if right_is_invisible_times {
      if let Some(XM::Apply(right_op, right_args, right_props, right_meta)) = right {
        let mut factors = right_args.0;
        let first = factors.remove(0);
        // Build Apply(/, left, first)
        let div_result = Some(XM::Apply(
          infixop.into(),
          Args(vec![left, first]),
          XProps::default(),
          Meta::default(),
        ));
        // Rebuild: Apply(times, div_result, rest...)
        let mut new_args = vec![div_result];
        new_args.extend(factors);
        return Ok(Some(XM::Apply(
          right_op,
          Args(new_args),
          right_props,
          right_meta,
        )));
      }
    }
  }

  // base case: new apply tree
  let apply_tree = XM::Apply(
    infixop.into(),
    Args(vec![left, right]),
    XProps::default(),
    Meta::default(),
  );
  Ok(Some(apply_tree))
}

pub fn prefix_apply(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => prefixop, arg1);
  // Perl: when a FUNCTION applies to a fenced arg (from `fenced` semantic action),
  // lift the XMDual to wrap the entire application:
  //   Apply(func, Dual(Ref, Wrap)) → Dual(Apply(Ref(func), Ref), Apply(func, Wrap))
  // This matches Perl's ApplyFunction XMDual structure.
  let is_function_role = match &prefixop {
    Some(XM::Lexeme(lex, _)) => {
      let role = lex.split(':').next().unwrap_or("");
      matches!(role, "FUNCTION" | "OPFUNCTION" | "TRIGFUNCTION")
    },
    Some(XM::Token(props, _)) => props
      .role
      .as_deref()
      .is_some_and(|r| matches!(r, "FUNCTION" | "OPFUNCTION" | "TRIGFUNCTION")),
    _ => false,
  };
  if is_function_role {
    if let Some(XM::Dual(ref content, ref pres, ..)) = arg1 {
      if matches!(**content, XM::Ref(_)) && matches!(**pres, XM::Wrap(..)) {
        let mut func = prefixop.unwrap();
        let arg1_inner = arg1.unwrap();
        let XM::Dual(content_box, pres_box, ..) = arg1_inner else {
          unreachable!()
        };
        let content_ref = *content_box;
        let pres_wrap = *pres_box;
        let func_refs = create_xmrefs(&mut [&mut func], ctxt)?;
        let func_ref = func_refs.into_iter().next().unwrap();
        let content_apply = XM::Apply(
          func_ref.into(),
          Args(vec![Some(content_ref)]),
          XProps::default(),
          Meta::default(),
        );
        let pres_apply = XM::Apply(
          func.into(),
          Args(vec![Some(pres_wrap)]),
          XProps::default(),
          Meta::default(),
        );
        return Ok(Some(XM::Dual(
          Box::new(content_apply),
          Box::new(pres_apply),
          XProps::default(),
          Meta::default(),
        )));
      }
    }
  }
  Ok(Some(XM::Apply(
    prefixop.into(),
    Args(vec![arg1]),
    XProps::default(),
    Meta::default(),
  )))
}
/// Perl: ApplyDelimited — function application with parenthesized arguments.
/// Creates XMDual with content=Apply(XMRef(f),XMRef(args)) and
/// presentation=Apply(f, XMWrap(open, args, close)).
///
/// Uses _xmkey for deferred ID resolution: sets _xmkey on the original
/// DOM nodes (via lookup_lex_node), creates XMRef with matching _xmkey.
/// The resolve_xmkeys step after DOM insertion resolves these to idref.
/// Perl: ApplyDelimited — function application with parenthesized arguments.
/// Produces Apply(func, content) — same as prefix_apply for now.
/// Perl MathGrammar: function(args) → XMDual(Apply(XMRef, XMRef), Apply(func, XMWrap(open, args,
/// close))) Produces XMDual wrapping that preserves both semantic and presentation forms.
/// Content: Apply(XMRef(func), XMRef(args)) — pure semantic
/// Presentation: Apply(func, XMWrap(open, args, close)) — visual with delimiters
pub fn apply_delimited(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => func, open, content, close);
  let mut func_node = func.unwrap();
  let mut content_node = content.unwrap();
  // Create XMRefs for the semantic (content) branch
  let mut xmrefs = create_xmrefs(&mut [&mut func_node, &mut content_node], ctxt)?;
  let func_ref = xmrefs.remove(0);
  let content_ref = xmrefs.remove(0);
  // Content branch: Apply(XMRef(func), XMRef(content))
  let content_apply = XM::Apply(
    func_ref.into(),
    Args(vec![Some(content_ref)]),
    XProps::default(),
    Meta::default(),
  );
  // Presentation branch: Apply(func, XMWrap(open, content, close))
  let pres_wrap = XM::Wrap(
    vec![open.unwrap(), content_node, close.unwrap()],
    XProps::default(),
    Meta::default(),
  );
  let pres_apply = XM::Apply(
    func_node.into(),
    Args(vec![Some(pres_wrap)]),
    XProps::default(),
    Meta::default(),
  );
  // XMDual(content, presentation)
  Ok(Some(XM::Dual(
    Box::new(content_apply),
    Box::new(pres_apply),
    XProps::default(),
    Meta::default(),
  )))
}
/// Perl: standalone modifier `\mod expr` → Apply(mod, Absent, expr)
/// The absent first operand represents the missing left side.
pub fn modifier_prefix_apply(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => modop, arg1);
  let absent = XM::Token(
    XProps {
      meaning: Some(Cow::Borrowed("absent")),
      ..XProps::default()
    },
    Meta::default(),
  );
  Ok(Some(XM::Apply(
    modop.into(),
    Args(vec![Some(absent), arg1]),
    XProps::default(),
    Meta::default(),
  )))
}

/// Perl: postfix modifier `expr \pmod{3}` → Apply(annotated, expr, modifier)
/// The modifier annotates the preceding expression.
pub fn postfix_modifier_apply(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => expr, modifier);
  let annotated = XM::Token(
    XProps {
      meaning: Some(Cow::Borrowed("annotated")),
      ..XProps::default()
    },
    Meta::default(),
  );
  Ok(Some(XM::Apply(
    annotated.into(),
    Args(vec![expr, modifier]),
    XProps::default(),
    Meta::default(),
  )))
}

/// Perl MathGrammar L224-233: addExpressionModifier with parenthesized relop/modifierop
/// `x(>0)` → `Apply(annotated, x, Fence(OPEN, Apply(relop, Absent, 0), CLOSE))`
/// `h(\in C)` → `Apply(annotated, h, Fence(OPEN, Apply(\in, Absent, C), CLOSE))`
pub fn annotated_fenced_modifier(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => expr, open, op, inner_expr, close);
  let annotated = XM::Token(
    XProps {
      meaning: Some(Cow::Borrowed("annotated")),
      ..XProps::default()
    },
    Meta::default(),
  );
  let absent = XM::Token(
    XProps {
      meaning: Some(Cow::Borrowed("absent")),
      ..XProps::default()
    },
    Meta::default(),
  );
  // Build Apply(op, Absent, inner_expr) for the modifier content
  let mut modifier_apply = XM::Apply(
    op.into(),
    Args(vec![Some(absent), inner_expr]),
    XProps::default(),
    Meta::default(),
  );
  // Fence the modifier: Dual(XMRef, XMWrap(OPEN, Apply(...), CLOSE))
  // matching Perl's Fence() which creates XMDual for parenthesized groups
  let mut fenced_xmrefs = create_xmrefs(&mut [&mut modifier_apply], ctxt)?;
  let fenced = XM::Dual(
    Box::new(fenced_xmrefs.remove(0)),
    Box::new(XM::Wrap(
      vec![open.unwrap(), modifier_apply, close.unwrap()],
      XProps::default(),
      Meta::default(),
    )),
    XProps::default(),
    Meta::default(),
  );
  Ok(Some(XM::Apply(
    annotated.into(),
    Args(vec![expr, Some(fenced)]),
    XProps::default(),
    Meta::default(),
  )))
}

/// Perl MathGrammar L223: expression PUNCT OPEN relop/modifierop Expression CLOSE
/// Semicolon annotation: a;(<e) → annotated(a, Fence((, absent < e, )))
/// Drops the PUNCT arg and delegates to annotated_fenced_modifier.
pub fn annotated_punct_fenced_modifier(
  rule_id: i32,
  mut args: Vec<Option<XM>>,
  p: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  // args: [expr, punct, open, op, inner_expr, close]
  // Drop punct (index 1) and delegate
  args.remove(1); // remove punct
  annotated_fenced_modifier(rule_id, args, p, ctxt)
}

/// Speculative prefix application for `unknown fenced_factor`.
/// Produces an XMApp tree competing with the invisible-times interpretation
/// in Marpa's ambiguous forest. The pragmatic layer selects the
/// mathematically-consistent winner (see `FencedLettersAreFunctionArguments`).
///
/// **Intentional Rust divergence from Perl**: In Perl (Parse::RecDescent), speculation
/// only marks tokens with `possibleFunction='yes'` and falls back to invisible-times
/// multiplication. The Rust Marpa grammar directly produces the function application
/// parse `f@(x)`, which is the semantically superior interpretation — it avoids an
/// artificial invisible MULOP token that was a crutch for Parse::RecDescent's
/// backtracking parser. See docs/OXIDIZED_DESIGN.md.
pub fn speculative_prefix_apply(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => prefixop, arg1);
  // Mirror of `prefix_apply_applyop`: when arg1 is a fenced
  // modifier expression (`(>0)`, `(\in C)`), reject — the
  // legitimate parse goes through `annotated_fenced_modifier`.
  if let Some(ref arg) = arg1 {
    if is_fenced_modifier_dual(arg) {
      return Err(
        "speculative_prefix_apply: arg is a fenced modifier expression — \
         prefer annotated_fenced_modifier"
          .into(),
      );
    }
    // K-12 algebra: `letter |x|` reads as multiplication
    // (`letter * |x|`), NOT function application (`letter @
    // |x|`). The grammar admits speculative function-app via
    // `unknown fenced_factor → speculative_prefix_apply` for any
    // fenced_factor; when the fenced_factor is a bilaterally-
    // vertbar-fenced absolute-value / norm / stretchy-abs shape,
    // that speculation is mathematically wrong. Reject here so
    // `tight_term factor → apply_invisible_times` wins, giving a
    // unique multiplication parse for `a|a|+b|b|+c|c|`.
    //
    // Implication: QM-context cases like `<a|f|b>` and
    // `\langle B|sum|C\rangle` that rely on the speculative
    // function-app for `letter |x|` lose that reading. The
    // affected tests (qm/mathtools/count_parses/physics) are
    // re-blessed to the multiplication interpretation.
    if is_vertbar_fenced_dual(arg) {
      return Err(
        "speculative_prefix_apply: arg is vertbar-fenced — \
         prefer K-12 multiplication via apply_invisible_times"
          .into(),
      );
    }
  }
  Ok(Some(XM::Apply(
    prefixop.into(),
    Args(vec![arg1]),
    XProps::default(),
    Meta::default(),
  )))
}

/// Detect `XM::Dual(_, Wrap[OPEN-vertbar, …, CLOSE-vertbar])` — a
/// bilaterally-vertbar-fenced absolute-value or norm shape. The
/// `fenced` action produces this for `|expr|`, `||expr||`, and
/// `\left|expr\right|`. We use it to reject these as candidates
/// for function-application speculation; K-12 algebra reads
/// `letter |x|` as multiplication, not function-app.
fn is_vertbar_fenced_dual(arg: &XM) -> bool {
  let XM::Dual(_, ref presentation, _, _) = *arg else {
    return false;
  };
  let XM::Wrap(ref items, _, _) = **presentation else {
    return false;
  };
  let is_vertbar = |x: Option<&XM>| -> bool {
    match x {
      Some(XM::Token(p, _)) => {
        p.content.as_deref() == Some("|") || p.content.as_deref() == Some("‖")
      },
      Some(XM::Lexeme(name, _)) => {
        name.starts_with("VERTBAR:") || name.starts_with("STRETCHY_VERTBAR:")
      },
      _ => false,
    }
  };
  is_vertbar(items.first()) && is_vertbar(items.last())
}
/// Perl: limit-from@(number, sign) — directional limits like 0+, 1-
/// Matches factor_base followed by addop. Semantic checks:
/// 1. The addop must be + or - (not other ADDOP like ⊕)
/// 2. The factor_base should be a number or simple ID (not a compound expression)
///
/// If checks fail, prunes the parse so Marpa tries addition instead.
pub fn limit_from_apply(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  // Check the addop is + or -
  let is_plus_minus = args.get(1).and_then(|a| a.as_ref()).is_some_and(|xm| {
    let val = match xm {
      XM::Lexeme(lex, _) => lookup_lex_node(lex, ctxt.nodes)
        .ok()
        .and_then(|n| n.get_attribute("meaning")),
      XM::Token(props, _) => props.meaning.as_ref().map(|c| c.to_string()),
      _ => None,
    };
    matches!(val.as_deref(), Some("plus") | Some("minus"))
  });
  if !is_plus_minus {
    return Err("limit_from_apply: addop is not +/-, pruning".into());
  }
  unp!(args => base, sign);
  let op = XM::Token(
    XProps {
      meaning: Some(Cow::Borrowed("limit-from")),
      ..XProps::default()
    },
    Meta::default(),
  );
  Ok(Some(XM::Apply(
    op.into(),
    Args(vec![base, sign]),
    XProps::default(),
    Meta::default(),
  )))
}

/// Perl IntFactor: diffd ATOM_OR_ID/UNKNOWN => Apply(DIFFOP(d), var)
/// Matches `d` followed by a factor. The semantic action checks that the first
/// token's text content is literally "d" (case-sensitive). If not, prunes the parse.
/// When matched, annotates the `d` token with role=DIFFOP, meaning=differential-d.
pub fn diffop_apply(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  // Check that the first token is literally "d"
  let is_d = args
    .first()
    .and_then(|a| a.as_ref())
    .is_some_and(|xm| match xm {
      XM::Token(props, _) => props.content.as_deref() == Some("d"),
      XM::Lexeme(lex, _) => lex.split(':').nth(1) == Some("d"),
      _ => false,
    });
  if !is_d {
    return Err("diffop_apply: first token is not 'd', pruning parse".into());
  }
  // Perl: diffd is only recognized inside IntOpArgFactors (integral context).
  // Check if there's an INTOP token in the lexeme stream.
  let has_intop = ctxt
    .nodes
    .iter()
    .any(|n| n.get_attribute("role").as_deref() == Some("INTOP"));
  if !has_intop {
    return Err("diffop_apply: no INTOP in context, pruning parse".into());
  }
  unp!(args => diffd, arg1);
  // Annotate the d token: role=DIFFOP, meaning=differential-d
  let annotated = match diffd {
    Some(XM::Token(mut props, meta)) => {
      props.role = Some(Cow::Borrowed("DIFFOP"));
      props.meaning = Some(Cow::Borrowed("differential-d"));
      Some(XM::Token(props, meta))
    },
    Some(XM::Lexeme(lex, meta)) => {
      // Lexeme from Marpa: create a new Token with DIFFOP annotation
      // Preserve the original lexeme reference for into_xmath node lookup
      let mut props = XProps {
        content: Some(Cow::Borrowed("d")),
        role: Some(Cow::Borrowed("DIFFOP")),
        meaning: Some(Cow::Borrowed("differential-d")),
        ..XProps::default()
      };
      // Store original lexeme id in _xmkey for node reference
      props.xmkey = lex.split(':').nth(2).map(|s| Cow::Owned(s.to_string()));
      Some(XM::Token(props, meta))
    },
    other => other,
  };
  Ok(Some(XM::Apply(
    annotated.into(),
    Args(vec![arg1]),
    XProps::default(),
    Meta::default(),
  )))
}
/// APPLYOP explicit application: operator APPLYOP term => Apply(operator, term)
/// The APPLYOP token is consumed/discarded.
pub fn prefix_apply_applyop(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => prefixop, _applyop, arg1);
  // Early-action prune: when arg1 is a fenced modifier expression
  // — i.e. a Dual whose content is a relop/metarelop Apply with
  // `absent` as the FIRST argument (the `prefix_relop_apply`
  // shape) — we must NOT treat it as a function argument. The
  // grammar's `expression lparen relop expression rparen →
  // annotated_fenced_modifier` already builds the correct
  // `annotated@(x, fenced-modifier)` tree. Function-application
  // here corrupts to `x@(absent > 0)` which is meaningless.
  if let Some(ref arg) = arg1 {
    if is_fenced_modifier_dual(arg) {
      return Err(
        "prefix_apply_applyop: arg is a fenced modifier expression — \
         prefer annotated_fenced_modifier over function application"
          .into(),
      );
    }
  }
  Ok(Some(XM::Apply(
    prefixop.into(),
    Args(vec![arg1]),
    XProps::default(),
    Meta::default(),
  )))
}

/// Detect the "fenced modifier" shape: an `XM::Dual` whose
/// fenced content is a RELOP/METARELOP Apply with `absent` as the
/// first argument (the `prefix_relop_apply` shape produced by
/// `relop expression → prefix_relop_apply`).
///
/// The Dual produced by `fenced` / `annotated_fenced_modifier` has
/// shape `Dual(Ref(id), Wrap[OPEN, Apply(op, absent, expr), CLOSE])`
/// — the semantic Apply lives inside the **presentation Wrap**,
/// referenced by Ref from the content. So we look there for the
/// absent-prefixed Apply.
///
/// Used in `prefix_apply_applyop` and `apply_invisible_times` to
/// reject treating `(>0)` / `(\in C)` as a function argument when
/// the legitimate parse is `annotated@(x, fenced-modifier)`.
fn is_fenced_modifier_dual(arg: &XM) -> bool {
  let XM::Dual(ref content, ref presentation, _, _) = *arg else {
    return false;
  };
  let has_absent_prefix = |args: &Args| -> bool {
    let first = args.trees().first().copied().cloned();
    matches!(first,
      Some(XM::Token(p, _)) if p.meaning.as_deref() == Some("absent"))
  };
  // Path A — content is an Apply directly:
  if let XM::Apply(_, args, _, _) = &**content {
    return has_absent_prefix(args);
  }
  // Path B — content is a Ref; find the Apply in the presentation Wrap.
  if let XM::Wrap(items, _, _) = &**presentation {
    for item in items {
      if let XM::Apply(_, args, _, _) = item {
        if has_absent_prefix(args) {
          return true;
        }
      }
    }
  }
  false
}

/// Perl: moreTerms2 trailing-operator → Apply(New('limit-from'), term, addop)
/// Perl MathGrammar L720-723: Combine SUPOP tokens (\prime\prime → prime2).
/// Left-recursive: `supops supop` accumulates count.
pub fn combine_supops(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  let left = args.remove(0).unwrap();
  let right = args.remove(0).unwrap();
  // Count existing primes: if left is already a combined supops Token, extract count
  let (left_count, left_text) = match &left {
    XM::Token(props, _) => {
      let name = props.name.as_deref().unwrap_or("");
      if let Some(n_str) = name.strip_prefix("prime") {
        let content_str = props.content.as_deref().unwrap_or("′");
        (
          n_str.parse::<usize>().unwrap_or(1),
          Some(content_str.to_string()),
        )
      } else {
        (1, left.get_value(ctxt.nodes).ok().map(|c| c.into_owned()))
      }
    },
    XM::Lexeme(lex, _) => (
      1,
      lookup_lex_node(lex, ctxt.nodes)
        .ok()
        .map(|n| n.get_content()),
    ),
    _ => (1, None),
  };
  let right_text = match &right {
    XM::Lexeme(lex, _) => lookup_lex_node(lex, ctxt.nodes)
      .ok()
      .map(|n| n.get_content()),
    _ => None,
  };
  let count = left_count + 1;
  let combined_text = match (left_text, right_text) {
    (Some(l), Some(r)) => format!("{l}{r}"),
    (Some(l), None) => format!("{l}′"),
    (None, Some(r)) => format!("′{r}"),
    (None, None) => "′′".to_string(),
  };
  Ok(Some(XM::Token(
    XProps {
      role: Some(Cow::Borrowed("SUPOP")),
      name: Some(Cow::Owned(format!("prime{count}"))),
      content: Some(Cow::Owned(combined_text)),
      ..XProps::default()
    },
    Meta::default(),
  )))
}

/// Handles `a+` (limit from above) and similar trailing operators.
/// When the expression is an n-ary Apply with the same operator (e.g. a+b+c+),
/// only wraps the LAST term in limit-from (matching Perl behavior):
///   Apply(+, a, b, c) + → Apply(+, a, b, Apply(limit-from, c, +))
pub fn postfix_apply(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => arg, op);
  let limit_from = XM::Token(
    XProps {
      meaning: Some(Cow::Borrowed("limit-from")),
      ..XProps::default()
    },
    Meta::default(),
  );
  // Check if arg is an n-ary Apply with the same operator as op
  if let (Some(XM::Apply(ref app_op, ref app_args, ref app_props, ref app_meta)), Some(ref op_xm)) =
    (&arg, &op)
  {
    let same_op = match (app_op.0.as_ref(), op_xm) {
      // Compare Lexemes: both are lexeme strings like "ADDOP:plus:+"
      (XM::Lexeme(ref a_lex, _), XM::Lexeme(ref o_lex, _)) => {
        // Compare role:meaning prefix (ignoring the actual symbol after last :)
        let a_parts: Vec<_> = a_lex.splitn(3, ':').collect();
        let o_parts: Vec<_> = o_lex.splitn(3, ':').collect();
        a_parts.len() >= 2
          && o_parts.len() >= 2
          && a_parts[0] == o_parts[0]
          && a_parts[1] == o_parts[1]
      },
      // Compare realized Tokens
      (XM::Token(ref a_props, _), XM::Token(ref o_props, _)) => {
        a_props.meaning == o_props.meaning && a_props.meaning.is_some()
      },
      _ => false,
    };
    if same_op && app_args.0.len() >= 2 {
      // Wrap only the last argument in limit-from
      let mut new_args = app_args.0.clone();
      let last_arg = new_args.pop().unwrap();
      let wrapped = XM::Apply(
        limit_from.into(),
        Args(vec![last_arg, op]),
        XProps::default(),
        Meta::default(),
      );
      new_args.push(Some(wrapped));
      return Ok(Some(XM::Apply(
        app_op.clone(),
        Args(new_args),
        app_props.clone(),
        app_meta.clone(),
      )));
    }
  }
  // Fallback: wrap the entire expression
  Ok(Some(XM::Apply(
    limit_from.into(),
    Args(vec![arg, op]),
    XProps::default(),
    Meta::default(),
  )))
}

/// Perl MathGrammar L423: POSTFIX operator (e.g. n! → factorial@(n))
/// Takes (base, postfix_op) and produces Apply(op, base).
pub fn apply_postfix(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => base, op);
  Ok(Some(XM::Apply(
    op.into(),
    Args(vec![base]),
    XProps::default(),
    Meta::default(),
  )))
}

/// Perl MathGrammar L709-711: TwoPartRelop — combines two adjacent relops.
/// E.g. `>=` → "greater-than-or-equals", `<<` → "much-less-than"
pub fn two_part_relop_combine(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => op1, op2);
  // Extract meanings from the lexeme nodes
  let (m1, content1) = if let Some(XM::Lexeme(ref lex, _)) = op1 {
    let node = lookup_lex_node(lex, ctxt.nodes)?;
    let m = node.get_attribute("meaning").unwrap_or_default();
    let c = node.get_content();
    (m, c)
  } else {
    (String::new(), String::new())
  };
  let (m2, content2) = if let Some(XM::Lexeme(ref lex, _)) = op2 {
    let node = lookup_lex_node(lex, ctxt.nodes)?;
    let m = node.get_attribute("meaning").unwrap_or_default();
    let c = node.get_content();
    (m, c)
  } else {
    (String::new(), String::new())
  };
  // Perl: TwoPartRelop logic
  let meaning = if m1 == m2 {
    format!("much-{m1}")
  } else {
    format!("{m1}-or-{m2}")
  };
  let content = format!("{content1}{content2}");
  Ok(Some(XM::Token(
    XProps {
      role: Some(Cow::Borrowed("RELOP")),
      meaning: Some(Cow::Owned(meaning)),
      content: Some(Cow::Owned(content)),
      ..XProps::default()
    },
    Meta::default(),
  )))
}

pub fn fenced(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => open_opt, arg_opt, close_opt);
  let mut arg = arg_opt.unwrap();
  let open = open_opt.unwrap();
  let close = close_opt.unwrap();
  // let xmrefs = create_xmrefs(&[&arg], ctxt)?.remove(0);
  // Ok(Some(
  //   XM::Dual(Box::new(xmrefs), Box::new(
  //     XM::Wrap(vec![open_opt.unwrap(),arg,close_opt.unwrap()], XProps::default(),
  // Meta::default())   ), XProps::default(), Meta::default())
  // ))
  let o = open.get_value(ctxt.nodes)?;
  let c = close.get_value(ctxt.nodes)?;
  let op_name = format!("delimited-{}{}", o, c);

  // TODO: For now assume a single argument in arg; specialize in other functions such as
  // "open_interval",       for the other cases from the classic MathParser.pm
  if op_name == "delimited-()" {
    // Check if arg is a multi-item list (XM::Dual from list_apply/formulae_apply).
    // If so, use interpret_delimited for per-item XMRefs (matching Perl's NewFenced).
    let is_multi_item = match &arg {
      XM::Dual(ref content, ..) => matches!(&**content, XM::Apply(ref op_box, ref args, ..)
        if args.0.len() >= 2 && matches!(&*op_box.0,
          XM::Token(ref p, _) if matches!(p.meaning.as_deref(),
            Some("vector") | Some("list") | Some("formulae")))),
      XM::Apply(ref op_box, ref args, ..) => {
        args.0.len() >= 2
          && matches!(&*op_box.0,
        XM::Token(ref p, _) if matches!(p.meaning.as_deref(),
          Some("vector") | Some("list") | Some("formulae")))
      },
      _ => false,
    };
    if is_multi_item {
      // Extract meaning and items from either Dual(Apply(...), Wrap(...)) or bare Apply
      let (meaning_str, items) = match arg {
        XM::Dual(content, pres, ..) => {
          // For Dual: use the presentation Wrap's children (which have xml:ids)
          let m = if let XM::Apply(ref op_box, ..) = *content {
            if let XM::Token(ref p, _) = *op_box.0 {
              p.meaning.as_deref().unwrap_or("vector").to_string()
            } else {
              "vector".to_string()
            }
          } else {
            "vector".to_string()
          };
          // Extract items from the presentation Wrap (skip separators)
          let pres_items = if let XM::Wrap(wrap_items, ..) = *pres {
            // Items are at even indices (0, 2, 4, ...) — separators at odd
            wrap_items
              .into_iter()
              .enumerate()
              .filter(|(i, _)| i % 2 == 0)
              .map(|(_, item)| item)
              .collect::<Vec<_>>()
          } else {
            vec![]
          };
          (m, pres_items)
        },
        XM::Apply(op_box, args_inner, ..) => {
          let m = if let XM::Token(ref p, _) = *op_box.0 {
            p.meaning.as_deref().unwrap_or("vector").to_string()
          } else {
            "vector".to_string()
          };
          let items: Vec<XM> = args_inner.0.into_iter().flatten().collect();
          (m, items)
        },
        _ => unreachable!(),
      };
      // Determine meaning from delimiter + item count (matching Perl's fence lookup).
      // Note: 2-item parens are NOT auto-labeled as "open-interval" here —
      // the dedicated `interval` semantic handles intervals via the
      // `lparen term punct term rparen => interval` grammar rule at term
      // level. `fenced` is the general "list-in-parens" path and produces
      // list-like meaning; the forest retains both interpretations and
      // pragmatics picks based on context (fenced=list wins in function
      // argument context, interval wins standalone).
      let n = items.len();
      let fence_meaning: Cow<'static, str> = match (o.as_ref(), n) {
        ("(", _) => Cow::Borrowed("vector"),
        ("{", _) => Cow::Borrowed("set"),
        _ => Cow::Owned(meaning_str),
      };
      let op = XProps {
        meaning: Some(fence_meaning),
        ..XProps::default()
      };
      // Build stuff: [open, item1, comma, item2, comma, ..., close]
      let comma = XM::Token(
        XProps {
          role: Some(Cow::Borrowed("PUNCT")),
          content: Some(Cow::Borrowed(",")),
          ..XProps::default()
        },
        Meta::default(),
      );
      let mut stuff = vec![open];
      for (i, item) in items.into_iter().enumerate() {
        if i > 0 {
          stuff.push(comma.clone());
        }
        stuff.push(item);
      }
      stuff.push(close);
      interpret_delimited(op.into(), stuff, ctxt).map(Option::Some)
    } else {
      // Single arg: XMDual(XMRef(arg), XMWrap((,arg,)))
      // create_xmrefs skips ephemeral variants (XMHint, and the default
      // skip-without-warning arm), so refs may come back empty when the
      // delimited body was just a spacing hint. In that case the Dual
      // with an XMRef to nothing would be meaningless, so fall back to
      // a bare Wrap (arxiv hep-ph/9210235 hit this on `\lparen \,
      // \rparen` where the sole arg was an XMHint that got filtered).
      let mut arg_xmrefs = create_xmrefs(&mut [&mut arg], ctxt)?;
      if arg_xmrefs.is_empty() {
        return Ok(Some(XM::Wrap(
          vec![open, arg, close],
          XProps::default(),
          Meta::default(),
        )));
      }
      Ok(Some(XM::Dual(
        Box::new(arg_xmrefs.remove(0)),
        Box::new(XM::Wrap(
          vec![open, arg, close],
          XProps::default(),
          Meta::default(),
        )),
        XProps::default(),
        Meta::default(),
      )))
    }
  } else if op_name == "delimited-{}" {
    // Perl enclose1: {expr} => set
    let op = XProps {
      meaning: Some(Cow::Borrowed("set")),
      ..XProps::default()
    };
    interpret_delimited(op.into(), vec![open, arg, close], ctxt).map(Option::Some)
  } else if op_name == "delimited-||" {
    let op = XProps {
      meaning: Some(Cow::Borrowed("absolute-value")),
      ..XProps::default()
    };
    let open_m = morph_vertbar(open, "OPEN", ctxt.nodes);
    let close_m = morph_vertbar(close, "CLOSE", ctxt.nodes);
    interpret_delimited(op.into(), vec![open_m, arg, close_m], ctxt).map(Option::Some)
  } else {
    // Check for known delimiter meanings
    let meaning = match (o.as_ref(), c.as_ref()) {
      ("\u{230A}", "\u{230B}") => Some("floor"),   // ⌊ ⌋
      ("\u{2308}", "\u{2309}") => Some("ceiling"), // ⌈ ⌉
      ("\u{2016}", "\u{2016}") => Some("norm"),    // ‖ ‖
      ("lfloor", "rfloor") => Some("floor"),       // name-based match
      ("lceil", "rceil") => Some("ceiling"),       // name-based match
      _ => None,
    };
    if let Some(m) = meaning {
      let op = XProps {
        meaning: Some(Cow::Borrowed(m)),
        ..XProps::default()
      };
      interpret_delimited(op.into(), vec![open, arg, close], ctxt).map(Option::Some)
    } else {
      let op = xnew(op_name);
      interpret_delimited(op.into(), vec![open, arg, close], ctxt).map(Option::Some)
    }
  }
}

// Empty fenced expression: OPEN CLOSE with no content => list()
// Perl: Apply(List, []) wrapped in XMDual with XMWrap(OPEN, CLOSE)
pub fn empty_fenced(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => open_opt, close_opt);
  let open = open_opt.unwrap();
  let close = close_opt.unwrap();
  let list_op = XProps {
    meaning: Some(Cow::Borrowed("list")),
    ..XProps::default()
  };
  // Build: Dual(Apply(list), Wrap(open, close))
  Ok(Some(XM::Dual(
    Box::new(XM::Apply(
      list_op.into(),
      vec![].into(),
      XProps::default(),
      Meta::default(),
    )),
    Box::new(XM::Wrap(
      vec![open, close],
      XProps::default(),
      Meta::default(),
    )),
    XProps::default(),
    Meta::default(),
  )))
}

// similar to fenced but the operator is a kind of tuple or interval, such as "open-interval"
// and the arguments are delimited with a comma
pub fn interval(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => open_opt, arg1_opt, sep_opt, arg2_opt, close_opt);
  let open = open_opt.unwrap();
  let mut arg1 = arg1_opt.unwrap();
  let sep = sep_opt.unwrap();
  let mut arg2 = arg2_opt.unwrap();
  let close = close_opt.unwrap();

  // Extract text values from lexemes (like fenced does)
  let o = open.get_value(ctxt.nodes)?;
  let c = close.get_value(ctxt.nodes)?;

  // Determine interval type from delimiter pair
  let op_meaning = match (o.as_ref(), c.as_ref()) {
    ("(", ")") | ("]", "[") => "open-interval",
    ("[", "]") => "closed-interval",
    ("[", ")") => "closed-open-interval",
    ("(", "]") => "open-closed-interval",
    ("⟨", "⟩") => "list", // angle brackets: ⟨a,b⟩ → list, not tuple
    _ => "tuple",
  };

  // Create operator as XM::Token with meaning attribute
  let op: XM = XProps {
    meaning: Some(Cow::Borrowed(op_meaning)),
    ..XProps::default()
  }
  .into();

  let ref_args = create_xmrefs(&mut [&mut arg1, &mut arg2], ctxt)?;

  Ok(Some(XM::Dual(
    Box::new(XM::Apply(
      op.into(),
      ref_args.into(),
      XProps::default(),
      Meta::default(),
    )),
    Box::new(XM::Wrap(
      vec![open, arg1, sep, arg2, close],
      XProps::default(),
      Meta::default(),
    )),
    XProps::default(),
    Meta::default(),
  )))
}

/// Perl's Fence (MathParser.pm): generalized fenced expression with
/// comma-separated items. Determines meaning from delimiter+punctuation
/// pattern using the Perl enclose tables.
/// Receives: open, item1, punct1, item2, [punct2, item3, ...], close
pub fn fence(
  _rule_id: i32,
  args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  // Collect all non-None args into a flat stuff vector
  let stuff: Vec<XM> = args.into_iter().flatten().collect();
  let open = &stuff[0];
  let close = &stuff[stuff.len() - 1];
  let o = open.get_value(ctxt.nodes)?;
  let c = close.get_value(ctxt.nodes)?;
  // Count items (every other element between open and close is an item)
  let n = (stuff.len() - 2).div_ceil(2); // number of items
  // Get first punctuation value for enclose2/encloseN lookup
  let p = if n >= 2 {
    stuff[2].get_value(ctxt.nodes).ok()
  } else {
    None
  };
  let p_str = p.as_deref().unwrap_or(",");

  // Perl's enclose tables: determine operator meaning from delimiters + punctuation
  let op_meaning = match n {
    0 => "list",
    1 => match (o.as_ref(), c.as_ref()) {
      ("{", "}") => "set",
      ("|", "|") => "absolute-value",
      ("\u{2308}", "\u{2309}") => "ceiling",             // ⌈ ⌉
      ("\u{230A}", "\u{230B}") => "floor",               // ⌊ ⌋
      ("\u{2016}", "\u{2016}") | ("||", "||") => "norm", // ‖ ‖
      _ => return Ok(None),                              // fall through, shouldn't happen
    },
    2 => match (o.as_ref(), p_str, c.as_ref()) {
      ("{", ",", "}") => "set",
      ("{", ":", "}") | ("{", "|", "}") => "conditional-set",
      ("(", "|", ")") => "conditional",
      ("(", ",", ")") => "open-interval",
      ("[", ",", "]") => "closed-interval",
      ("(", ",", "]") => "open-closed-interval",
      ("[", ",", ")") => "closed-open-interval",
      _ => "list",
    },
    _ => match (o.as_ref(), p_str, c.as_ref()) {
      ("{", ",", "}") => "set",
      ("(", ",", ")") => "vector",
      _ => "list",
    },
  };

  let op: XM = XProps {
    meaning: Some(Cow::Borrowed(op_meaning)),
    ..XProps::default()
  }
  .into();
  // Change VERTBAR separators inside fences to MIDDLE role
  // (Perl: Fence sets middle delimiter role to MIDDLE)
  // Separators are at odd indices in [open, item, sep, item, ..., close]
  let mut stuff = stuff;
  for i in (2..stuff.len().saturating_sub(1)).step_by(2) {
    match &mut stuff[i] {
      XM::Token(ref mut props, _) if props.role.as_deref() == Some("VERTBAR") => {
        props.role = Some(Cow::Borrowed("MIDDLE"));
      },
      XM::Lexeme(ref lex, ref meta) if lex.starts_with("VERTBAR:") => {
        // For lexemes, change the role on the underlying DOM node
        if let Some(ref cv) = meta.curry_level {
          let cv_str = cv.to_string();
          if let Some(idx_str) = cv_str.strip_prefix(':') {
            if let Ok(lex_idx) = idx_str.parse::<usize>() {
              let idx = if lex_idx > 0 { lex_idx - 1 } else { 0 };
              if idx < ctxt.nodes.len() {
                let mut node = ctxt.nodes[idx].clone();
                let _ = node.set_attribute("role", "MIDDLE");
              }
            }
          }
        }
      },
      _ => {},
    }
  }
  interpret_delimited(op, stuff, ctxt).map(Option::Some)
}

/// This is similar, but "interprets" a delimited list as being the
/// application of some operator to the items in the list.
fn interpret_delimited(
  op: XM,
  mut stuff: Vec<XM>,
  ctxt: ActionContext,
) -> Result<XM, Box<dyn Error>> {
  let upto = stuff.len() - 1;
  let (_seps, mut args) = extract_separators(&mut stuff[1..upto]);
  let ref_args = create_xmrefs(&mut args, ctxt)?;
  Ok(XM::Dual(
    Box::new(XM::Apply(
      op.into(),
      ref_args.into(),
      XProps::default(),
      Meta::default(),
    )),
    Box::new(XM::Wrap(stuff, XProps::default(), Meta::default())),
    XProps::default(),
    Meta::default(),
  ))
}

/// A trailing presentational embellishment,
/// represent by containing it in the presentation arm of an XMDual
pub fn postfix_embellished(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  let mut arg = args.remove(0).unwrap();
  let trailer = args.remove(0).unwrap();
  // Perl: trailing comma wraps content in list@(...), trailing period in formulae@(...)
  // This matches Perl's endPunct(?) behavior in script content parsing.
  let is_comma = trailer.get_value(ctxt.nodes).ok().is_some_and(|v| v == ",");
  let is_period = trailer.get_value(ctxt.nodes).ok().is_some_and(|v| v == ".");
  let mut ref_arg = create_xmrefs(&mut [&mut arg], ctxt)?;
  let content = if is_comma || is_period {
    // Perl: trailing comma/period wraps content in list@(ref)
    // Period as separator creates formulae; as trailing punct, still wraps in list.
    Box::new(XM::Apply(
      XProps {
        meaning: Some(Cow::Borrowed("list")),
        ..XProps::default()
      }
      .into(),
      Args(vec![Some(ref_arg.remove(0))]),
      XProps::default(),
      Meta::default(),
    ))
  } else {
    Box::new(ref_arg.remove(0))
  };
  Ok(Some(XM::Dual(
    content,
    Box::new(XM::Wrap(
      vec![arg, trailer],
      XProps::default(),
      Meta::default(),
    )),
    XProps::default(),
    Meta::default(),
  )))
}
/// Wrap start_script + parsed content together.
/// Returns XM::Wrap([start_script, content]) so new_script can use the parsed
/// content instead of re-reading from DOM (which loses XMDual structures).
pub fn faux_wrap(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => start_script, content, _end_script);
  // Bundle both the script wrapper lexeme and the parsed expression.
  // new_script_inner will detect this Wrap and use the parsed content.
  Ok(Some(XM::Wrap(
    vec![
      start_script.unwrap(),
      content.unwrap_or(XM::Token(XProps::default(), Meta::default())),
    ],
    XProps::default(),
    Meta::default(),
  )))
}

pub fn standalone_script(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => start_script, _content, _end_script);
  // TODO: it looks like we need properties on each XM::Apply,
  // and porting NewScript is a head-scratcher.
  // for now, just keep the property if it's there.
  new_script(None, start_script.unwrap(), ctxt)
}

pub fn postfix_script(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => base, op);
  // 3-arg rules (e.g. mulop postsubarg postsuperarg): chain both scripts
  let op2 = args.pop().flatten();
  let intermediate = new_script(base, op.unwrap(), ActionContext {
    nodes:    ctxt.nodes,
    document: &mut *ctxt.document,
  })?;
  if let Some(op2) = op2 {
    new_script(intermediate, op2, ctxt)
  } else {
    Ok(intermediate)
  }
}

pub fn prefix_script(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => op, base);
  new_script(base, op.unwrap(), ctxt)
}

/// Like prefix_script but forces "pre" position for POST scripts used as pre-scripts.
/// Perl: parse_kludgeScripts_rec calls NewScript($base, $script, 'pre') for POST scripts
/// that follow FLOAT scripts from the same empty {} base.
pub fn prefix_script_pre(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => op, base);
  new_script_forced_pre(base, op.unwrap(), ctxt)
}

/// Parse a scriptpos string like "post2" into position type and level.
/// Follows Perl's: ($sx, $sl) = ($scriptpos || 'post') =~ /^(pre|mid|post)?(\d+)?$/
fn parse_scriptpos(s: &str) -> (&'static str, u32) {
  let s = if s.is_empty() { "post" } else { s };
  let x = if s.starts_with("pre") {
    "pre"
  } else if s.starts_with("mid") {
    "mid"
  } else {
    "post"
  };
  let l: u32 = s
    .trim_start_matches(|c: char| c.is_ascii_alphabetic())
    .parse()
    .unwrap_or(1);
  (x, l)
}

/// This is loosely in the lines of MathParser::NewScript, but taking into account
/// the realities of our new data structures.
pub fn new_script(
  base: Option<XM>,
  script: XM,
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  new_script_inner(base, script, ctxt, false)
}

/// Like new_script but forces "pre" position (Perl: NewScript($base, $script, 'pre')).
/// Used for POST scripts kludged into pre-script position. Sets "pre" without _wasfloat.
fn new_script_forced_pre(
  base: Option<XM>,
  script: XM,
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  new_script_inner(base, script, ctxt, true)
}

fn new_script_inner(
  base: Option<XM>,
  script: XM,
  ctxt: ActionContext,
  force_pre: bool,
) -> Result<Option<XM>, Box<dyn Error>> {
  // faux_wrap now returns XM::Wrap([start_script_lexeme, parsed_content]).
  // Extract both pieces. The lexeme provides role/scriptpos metadata;
  // the parsed content is used instead of re-reading from DOM.
  let (script_lex, parsed_content) = match script {
    XM::Wrap(mut items, ..) if items.len() == 2 => {
      let content = items.pop().unwrap();
      let lex = items.pop().unwrap();
      (lex, Some(content))
    },
    XM::Lexeme(..) => (script, None),
    other => panic!(
      "new_script expects faux_wrap Wrap or Lexeme, got {:?}",
      other
    ),
  };

  if let XM::Lexeme(ref lex, _) = script_lex {
    let script_wrap = lookup_lex_node(lex, ctxt.nodes)?;
    let node_role = script_wrap.get_attribute("role").unwrap();
    let is_float = !force_pre && node_role.starts_with("FLOAT");
    let is_super = node_role.ends_with("SUPERSCRIPT");
    let role = Cow::Borrowed(if is_super {
      "SUPERSCRIPTOP"
    } else {
      "SUBSCRIPTOP"
    });

    // Perl: Extract base's scriptpos to determine binding level
    let (bx, mut bl, base_wasfloat, base_bumplevel) = extract_base_scriptpos(&base, &ctxt);

    // Read scriptpos from the script node
    let raw_sp = script_wrap.get_attribute("scriptpos").unwrap_or_default();
    let (sx, mut sl) = parse_scriptpos(&raw_sp);
    let sx_defined = !raw_sp.is_empty() && sx != "post";

    if bl == 0 {
      bl = if sl > 0 { sl } else { 1 };
    }
    if sl == 0 {
      sl = if bl > 0 { bl } else { 1 };
    }

    let sx = if sx_defined {
      sx
    } else if bl == sl {
      bx
    } else {
      "post"
    };

    let x = if force_pre || is_float {
      "pre"
    } else if bl == sl {
      bx
    } else {
      if !sx.is_empty() { sx } else { "post" }
    };

    let mut l = if sl > 0 {
      sl
    } else if bl > 0 {
      bl
    } else {
      0
    };

    let mut bumped = false;
    if base_wasfloat {
      l += 1;
      bumped = true;
    } else if base_bumplevel > 0 {
      l = base_bumplevel;
    }

    let scriptpos: Cow<'static, str> = format!("{x}{l}").into();
    let op = new_props(
      None,
      None,
      Some(raw_map!("role"=>role, "scriptpos"=>scriptpos)),
    );
    // Use parsed content if available, otherwise fall back to obtain_arg (DOM re-read)
    let script_arg = if let Some(content) = parsed_content {
      Some(content)
    } else {
      obtain_arg(script_lex, 0, ctxt)?
    };
    let mut meta = if bumped {
      Meta::with_bumplevel(l)
    } else {
      Meta::default()
    };
    if is_float {
      meta.set_wasfloat();
    }
    // Perl: NewScript(Absent(), ...) when base is None (standalone floating scripts)
    let base_arg = base.or_else(|| {
      Some(XM::Token(
        XProps {
          meaning: Some(Cow::Borrowed("absent")),
          ..XProps::default()
        },
        Meta::default(),
      ))
    });
    Ok(Some(XM::Apply(
      op.into(),
      Args(vec![base_arg, script_arg]),
      XProps::default(),
      meta,
    )))
  } else {
    panic!(
      "new_script expects Lexeme inside faux_wrap, got {:?}",
      script_lex
    );
  }
}

/// Extract scriptpos info from the base of a script operation.
/// Returns (position_string, level, was_float, bump_level)
fn extract_base_scriptpos(
  base: &Option<XM>,
  ctxt: &ActionContext,
) -> (&'static str, u32, bool, u32) {
  match base {
    Some(XM::Apply(ref op, _, _props, ref meta)) => {
      // Check if the operator is a SCRIPTOP
      if let XM::Token(ref op_props, _) = *op.0 {
        let role = op_props.role.as_deref().unwrap_or("");
        if role.ends_with("SCRIPTOP") {
          let sp = op_props.scriptpos.as_deref().unwrap_or("post");
          let (bx, bl) = parse_scriptpos(sp);
          let wasfloat = meta.wasfloat();
          let bumplevel = meta.bumplevel();
          return (bx, bl, wasfloat, bumplevel);
        }
      }
      ("post", 0, false, 0)
    },
    // For Lexeme bases (e.g., \sum with scriptpos="mid"),
    // look up the XML node to get scriptpos
    Some(XM::Lexeme(ref lex, _)) => {
      if let Ok(node) = lookup_lex_node(lex, ctxt.nodes) {
        let sp = node.get_attribute("scriptpos").unwrap_or_default();
        if !sp.is_empty() {
          let (bx, bl) = parse_scriptpos(&sp);
          return (bx, bl, false, 0);
        }
      }
      ("post", 0, false, 0)
    },
    Some(XM::Token(ref props, _)) => {
      let sp = props.scriptpos.as_deref().unwrap_or("post");
      let (bx, bl) = parse_scriptpos(sp);
      (bx, bl, false, 0)
    },
    _ => ("post", 0, false, 0),
  }
}

// Get n-th arg of an XMApp.
// However, this is really only used to get the script out of a sub/super script
pub fn obtain_arg(tree: XM, n: usize, ctxt: ActionContext) -> Result<Option<XM>, Box<dyn Error>> {
  match &tree {
    XM::Lexeme(lex, _) => {
      let lex_node = lookup_lex_node(lex, ctxt.nodes)?;
      let args = element_nodes(lex_node);
      let nth = args.get(n).map(XM::from);
      Ok(nth)
      // TODO:
      // Tricky case: if $node is an XMRef, we'll want to reference the SUB node too
      // and not just use it directly; else that node will be duplicated in both branches of XMDual
      // if ($nth && !$node->isSameNode($onode)) {
      //   return LaTeXML::Package::createXMRefs($LaTeXML::MathParser::DOCUMENT, $nth); }
    },
    XM::Apply(_, ref args, ..) => match args.0.get(n) {
      Some(t) => Ok(t.clone()),
      None => Ok(None),
    },
    // Other XM variants (Token, Dual, Wrap, Choices, Arg, Ref) don't
    // carry positional args — Perl's obtain_arg returns undef for these.
    _ => Ok(None),
  }
}

pub fn apply_invisible_times(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => left, right);
  let mut left = left;
  let mut right = right;
  // OPFUNCTION/TRIGFUNCTION/FUNCTION tokens absorb the next argument via prefix_apply,
  // NOT via invisible times. When these appear as left of invisible_times (because
  // tight_term includes factor which includes opfunction), prune in favor of prefix_apply.
  if let Some(ref l) = left {
    let role = match l {
      XM::Token(props, _) => props.role.as_deref().map(String::from),
      XM::Lexeme(lex_id, _) => {
        if let Some(id) = lex_id
          .split(':')
          .next_back()
          .and_then(|s| s.parse::<usize>().ok())
        {
          if id > 0 && id <= ctxt.nodes.len() {
            ctxt.nodes[id - 1].get_attribute("role")
          } else {
            None
          }
        } else {
          None
        }
      },
      // For scripted functions/operators (XM::Apply with SCRIPTOP operator):
      // check the base token's role. E.g. \log_e → Apply(SUBSCRIPTOP, [log, e])
      // where log has role OPFUNCTION — should still prefer prefix_apply.
      // Also for compound operators: \nabla\log → Apply(nabla, [log])
      // where nabla is OPERATOR — the compound result should absorb args.
      XM::Apply(ref op, ref args, ..) => {
        let op_role = match &*op.0 {
          XM::Token(ref p, _) => p.role.as_deref().map(String::from),
          XM::Lexeme(lex, _) => {
            // Extract role from lexeme string prefix: "OPERATOR:nabla:1" → "OPERATOR"
            lex.split(':').next().map(String::from)
          },
          _ => None,
        };
        let op_role_str = op_role.as_deref().unwrap_or("");
        if op_role_str.ends_with("SCRIPTOP") {
          // Scripted: check base token's role (e.g. \log_e → SUBSCRIPTOP over OPFUNCTION)
          args
            .0
            .first()
            .and_then(|base| base.as_ref())
            .and_then(|base| match base {
              XM::Token(props, _) => props.role.as_deref().map(String::from),
              XM::Lexeme(lex_id, _) => lex_id
                .split(':')
                .next_back()
                .and_then(|s| s.parse::<usize>().ok())
                .and_then(|id| {
                  if id > 0 && id <= ctxt.nodes.len() {
                    ctxt.nodes[id - 1].get_attribute("role")
                  } else {
                    None
                  }
                }),
              _ => None,
            })
        } else if op_role_str == "OPERATOR" {
          // Compound operator: \nabla\log → Apply(OPERATOR, [OPFUNCTION])
          // Should absorb next arg via prefix_apply, not invisible-times.
          // BUT: applied operator D@(a) should allow invisible-times for D@(a)*(b).
          // Distinguish: compound = arg[0] is function/operator/trig; applied = arg is regular.
          let first_arg_role = args
            .0
            .first()
            .and_then(|a| a.as_ref())
            .and_then(|a| match a {
              XM::Token(p, _) => p.role.as_deref().map(String::from),
              XM::Lexeme(lex, _) => lex.split(':').next().map(String::from),
              _ => None,
            });
          let is_compound = matches!(
            first_arg_role.as_deref(),
            Some("OPFUNCTION") | Some("TRIGFUNCTION") | Some("FUNCTION") | Some("OPERATOR")
          );
          if is_compound { op_role.clone() } else { None }
        } else {
          None
        }
      },
      _ => None,
    };
    if matches!(
      role.as_deref(),
      Some("OPFUNCTION") | Some("TRIGFUNCTION") | Some("FUNCTION") | Some("OPERATOR")
    ) {
      // Exception 1: when the RIGHT side is also a FUNCTION/OPFUNCTION/TRIGFUNCTION,
      // prefer invisible_times (multiplication). Perl: `fgh` with all FUNCTION → f·g·h.
      let rhs_is_function = right
        .as_ref()
        .map(|r| {
          let rr = match r {
            XM::Token(props, _) => props.role.as_deref().map(String::from),
            XM::Lexeme(lex_id, _) => lex_id
              .split(':')
              .next_back()
              .and_then(|s| s.parse::<usize>().ok())
              .and_then(|id| {
                if id > 0 && id <= ctxt.nodes.len() {
                  ctxt.nodes[id - 1].get_attribute("role")
                } else {
                  None
                }
              }),
            _ => None,
          };
          matches!(
            rr.as_deref(),
            Some("OPFUNCTION") | Some("TRIGFUNCTION") | Some("FUNCTION")
          )
        })
        .unwrap_or(false);
      // Exception 2: OPERATOR * fenced → allow (compound_operator grammar rule generates
      // the prefix_apply tree, but it's not always available; invisible_times serves as
      // fallback for D(a)(b) patterns where D is OPERATOR).
      if !rhs_is_function {
        return Err(
          "apply_invisible_times: left is OPFUNCTION/TRIGFUNCTION/FUNCTION, prefer prefix_apply"
            .into(),
        );
      }
    }
  }
  // Wider-absorption variant for **applied** OPERATOR Applies:
  // `Apply(OPERATOR, [single_unfenced_arg]) * simple_RHS` should
  // prefer the parse where the operator absorbs more — i.e., `D x y z`
  // means `D@(x*y*z)`, not `D@(x) * y * z`. The block above prunes
  // BARE OPERATOR tokens on the LHS but the "applied" case
  // (compound-operator's first arg is regular content, not another
  // function/operator) was deliberately left alone. Pruning HERE
  // forces the absorption path via `prefix_apply_applyop`.
  if let Some(XM::Apply(Operator(ref left_op), ref left_args, _, ref left_meta)) = left {
    // Resolve the role of LHS's operator. For Tokens it's a direct
    // field; for Lexemes the lexeme's last `:N` field indexes into
    // `ctxt.nodes` to get the DOM node's `role` attribute (same
    // lookup mechanism the OPFUNCTION block above uses).
    let op_role = match &**left_op {
      XM::Token(p, _) => p.role.as_deref().map(String::from),
      XM::Lexeme(lex_id, _) => {
        if let Some(id) = lex_id.split(':').next_back().and_then(|s| s.parse::<usize>().ok()) {
          if id > 0 && id <= ctxt.nodes.len() {
            ctxt.nodes[id - 1].get_attribute("role")
          } else {
            None
          }
        } else {
          None
        }
      },
      _ => None,
    };
    if op_role.as_deref() == Some("OPERATOR")
      && left_meta.fenced.is_none()
      && left_args.trees().len() == 1
    {
      // Only prune if the SINGLE arg is non-fenced (e.g. `D x` not `D(a)`)
      // and the RHS is a simple unfenced factor or scripted factor.
      let arg_is_unfenced = left_args
        .trees()
        .first()
        .map(|a| a.get_meta().fenced.is_none())
        .unwrap_or(true);
      let rhs_is_simple = match right.as_ref() {
        Some(XM::Lexeme(..)) | Some(XM::Token(..)) | Some(XM::Wrap(..)) => true,
        Some(XM::Apply(Operator(ref rhs_op), ..)) => {
          let rhs_role = match &**rhs_op {
            XM::Token(p, _) => p.role.as_deref().unwrap_or(""),
            XM::Lexeme(lex, _) => lex.split(':').next().unwrap_or(""),
            _ => "",
          };
          rhs_role == "SUPERSCRIPTOP" || rhs_role == "SUBSCRIPTOP"
        },
        _ => false,
      };
      let rhs_unfenced =
        right.as_ref().map(|r| r.get_meta().fenced.is_none()).unwrap_or(false);
      if arg_is_unfenced && rhs_is_simple && rhs_unfenced {
        return Err(
          "apply_invisible_times: left is applied OPERATOR — \
           prefer wider absorption via prefix_apply_applyop"
            .into(),
        );
      }
    }
  }
  // Early-action prune for the fenced-modifier shape on the RHS:
  // `x (>0)` / `x (\in C)` — the legitimate parse is
  // `annotated_fenced_modifier`, NOT `x * (>0)` (implicit-times)
  // and NOT `x@(>0)` (function-app, handled in `prefix_apply_applyop`).
  if let Some(ref r) = right {
    if is_fenced_modifier_dual(r) {
      return Err(
        "apply_invisible_times: right is a fenced modifier expression — \
         prefer annotated_fenced_modifier"
          .into(),
      );
    }
  }
  // Bigop application results should not participate in invisible-times on their right.
  // When ∫_0^∞ x^2 dx is parsed, both `∫_0^∞(x^2 dx)` (absorption) and
  // `∫_0^∞(x^2) * dx` (flat) exist. Prune the flat parse by rejecting
  // invisible-times where the left is Apply(bigop, ...).
  // Perl: addIntOpArgs/addOpArgs absorbs the full integrand; we match by pruning.
  if let Some(ref l) = left {
    if is_bigop_or_scripted_bigop(l, ctxt.nodes) {
      return Err("apply_invisible_times: left is bigop/scripted bigop, prefer absorption".into());
    }
  }
  // Note: bare OPFUNCTION absorption (diffd@(x) vs diffd*x) is handled by
  // the FunctionsPreferWiderAbsorption pragmatic, which compares competing
  // trees and rejects the narrow parse.
  // Perl: scripted function application — f^2(a), f'(a), g_n(x).
  // When left is a scripted Apply whose base has FUNCTION/OPFUNCTION/TRIGFUNCTION role,
  // and right is a fenced XMDual (from parenthesized fencing), produce function
  // application with XMDual wrapping instead of invisible times.
  if let Some(ref l) = left {
    if let Some(ref r) = right {
      let is_scripted_function = is_scripted_function_head(l, ctxt.nodes);
      let is_fenced_dual = matches!(r, XM::Dual(ref c, ref p, _, _)
        if matches!(**c, XM::Ref(_)) && matches!(**p, XM::Wrap(..)));
      if is_scripted_function && is_fenced_dual {
        // Lift the fenced XMDual: Apply(f^2, Dual(Ref, Wrap)) → Dual(Apply(Ref(f^2), Ref(arg)),
        // Apply(f^2, Wrap))
        let mut func = left.take().unwrap();
        let arg_dual = right.take().unwrap();
        let XM::Dual(content_box, pres_box, ..) = arg_dual else {
          unreachable!()
        };
        let content_ref = *content_box;
        let pres_wrap = *pres_box;
        let func_refs = create_xmrefs(&mut [&mut func], ctxt)?;
        let func_ref = func_refs.into_iter().next().unwrap();
        let content_apply = XM::Apply(
          func_ref.into(),
          Args(vec![Some(content_ref)]),
          XProps::default(),
          Meta::default(),
        );
        let pres_apply = XM::Apply(
          func.into(),
          Args(vec![Some(pres_wrap)]),
          XProps::default(),
          Meta::default(),
        );
        return Ok(Some(XM::Dual(
          Box::new(content_apply),
          Box::new(pres_apply),
          XProps::default(),
          Meta::default(),
        )));
      }
    }
  }

  // Perl: trigBarearg greedily absorbs ALL following bare factors: \sin xyz → sin(x*y*z).
  // Reject invisible_times(trig_app(args), bare_factor) — the factor should be absorbed
  // into the trig argument via trig_arg rule, not multiplied outside.
  if let Some(XM::Apply(ref op, _, _, ref meta)) = left {
    if meta.fenced.is_none() {
      let op_name = op.0.base_operator_name();
      if op_name.starts_with("TRIGFUNCTION") {
        if let Some(ref r) = right {
          let is_bare_factor = match r {
            XM::Lexeme(_, ref rm) => rm.fenced.is_none(),
            XM::Token(_, ref rm) => rm.fenced.is_none(),
            _ => false,
          };
          if is_bare_factor {
            return Err(
              "apply_invisible_times: trig function should absorb bare factor via trig_arg".into(),
            );
          }
        }
      }
    }
  }
  // Perl: MaybeFunction — mark UNKNOWN tokens as possibleFunction when MATHPARSER_SPECULATE is set
  // and the right side is a delimited group (parenthesized)
  maybe_mark_possible_function(&mut left, &right, ctxt.nodes);

  // left-to-right associative -- if "left" is already a "times", tuck "right" in:
  if let Some(XM::Apply(ref op, ref mut left_args, _, ref _m)) = left {
    if let XM::Token(xop, _xmeta) = &*op.0 {
      match xop.meaning {
        Some(ref name) if name == "times" => {
          left_args.0.push(right);
          return Ok(left);
        },
        _ => {},
      }
    }
  }
  // Mixed number detection: NUMBER followed by FRACOP → invisible plus
  // Perl: 2\frac{3}{4} = 2 + 3/4; 123\frac{12}{34} = 123 + 12/34 (all-integer)
  // But 123.456\frac{12}{34} = 123.456 × (12/34) (decimal prefix → not mixed)
  let l_num = is_number(&left);
  let l_integer = l_num && is_integer_number(&left);
  let mut r_frac = is_fracop(&right);
  // Also check via nodes: if right is a Lexeme pointing to a DOM node with FRACOP inside
  if l_num && !r_frac {
    if let Some(XM::Lexeme(ref _lex, ref meta)) = right {
      // Use curry_level to find the node — it encodes the node position
      if let Some(ref cv) = meta.curry_level {
        let cv_str = cv.to_string();
        // Extract index from ":N" format — node index is N-1 (lexeme counter is 1-based)
        if let Some(idx_str) = cv_str.strip_prefix(':') {
          if let Ok(lex_idx) = idx_str.parse::<usize>() {
            let idx = if lex_idx > 0 { lex_idx - 1 } else { 0 };
            if idx < ctxt.nodes.len() {
              let node = &ctxt.nodes[idx];
              if node.get_name() == "XMApp" {
                for child in node.get_child_elements() {
                  if child.get_attribute("role").as_deref() == Some("FRACOP") {
                    r_frac = true;
                    break;
                  }
                }
              }
            }
          }
        }
      }
    }
  }
  // Mixed number only when BOTH sides of the fraction are pure integers.
  // Perl: 2\frac{3}{4} → 2+3/4, but 123\frac{12.0}{34} → 123×(12.0/34)
  let mut is_mixed_number = l_integer && r_frac;
  if is_mixed_number {
    // Check the fraction's numerator/denominator are pure integers via DOM nodes.
    // The fraction is often opaque in the XM tree (represented as an ATOM Lexeme).
    // We need to examine the DOM node's XMArg children for non-NUMBER content.
    let frac_node_opt = find_fracop_node(&right, ctxt.nodes);
    if let Some(frac_node) = frac_node_opt {
      // Check all non-operator children (numerator, denominator) for pure integer content.
      // Children are bare XMTok elements, not wrapped in XMArg.
      for child in frac_node.get_child_elements() {
        let role = child.get_attribute("role").unwrap_or_default();
        if role == "FRACOP" {
          continue;
        } // skip the operator
        let content = child.get_content();
        if role != "NUMBER" || content.contains('.') {
          is_mixed_number = false;
          break;
        }
      }
    }
  }
  let op = if is_mixed_number {
    invisible_plus()
  } else {
    invisible_times()
  };

  Ok(Some(XM::Apply(
    op.into(),
    Args(vec![left, right]),
    XProps::default(),
    Meta::default(),
  )))
}

pub fn compound_operator_2(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => op1, op2);
  // invisible comma:
  let comma = invisible_comma();
  // TODO: We need to extend that rule to the n-ary case
  // Currently following the original MathGrammar and creating a List XMDual
  new_list(vec![op1.unwrap(), comma.into(), op2.unwrap()], ctxt)
}

pub fn new_props(
  meaning: Option<Cow<'static, str>>,
  content: Option<Cow<'static, str>>,
  props_opt: Option<HashMap<&'static str, Cow<'static, str>>>,
) -> XProps {
  let mut props = props_opt.unwrap_or_default();
  let role = props.remove("role");
  let name = props.remove("name");
  let id = props.remove("id");
  let idref = props.remove("idref");
  let fontref = props.remove("_font");
  let scriptpos = props.remove("scriptpos");
  // TODO: explicit "font" prop path not yet wired — current callers never
  // pass it. If ever hit, fall through to the content-based specialization
  // (same as None), which is an approximation but won't crash.
  let font = match props.remove("font") {
    Some(_) | None => {
      if let Some(ref text) = content {
        if !text.is_empty() && !text.chars().all(|c| c.is_whitespace()) {
          font::FONT_TEXT_DEFAULT.specialize(text)
        } else {
          Font::default()
        }
      } else {
        Font::default()
      }
    },
  };
  XProps {
    meaning,
    content,
    role,
    name,
    scriptpos,
    id,
    idref,
    fontref,
    font: Some(font),
    ..Default::default()
  }
}

pub fn new_list(mut pieces: Vec<XM>, ctxt: ActionContext) -> Result<Option<XM>, Box<dyn Error>> {
  // drop placeholder token for missing trailing punct, if any
  if pieces.len() > 1 {
    let last_meaning_opt = pieces.last().unwrap().get_token_meaning(ctxt.nodes)?;
    if let Some(last_meaning) = last_meaning_opt {
      if last_meaning == "absent" {
        pieces.pop();
      }
    }
  }
  if pieces.len() == 1 {
    Ok(pieces.pop())
  } else {
    let (_seps, mut items) = extract_separators(&mut pieces);
    Ok(Some(XM::Dual(
      Box::new(XM::Apply(
        new_props(Some(Cow::Borrowed("list")), None, None).into(),
        create_xmrefs(&mut items, ctxt)?.into(),
        XProps::default(),
        Meta::default(),
      )),
      Box::new(XM::Wrap(pieces, XProps::default(), Meta::default())),
      XProps::default(),
      Meta::default(),
    )))
  }
}

/// Given  alternating expressions & separators (punctuation,...)
/// extract the separators as a concatenated string,
/// returning (separators, args...)
/// But note that the separators are never used for anything!?
fn extract_separators(items: &mut [XM]) -> (Vec<&mut XM>, Vec<&mut XM>) {
  // TODO: consider using the separators at some point, but not for now
  let punct = Vec::new();
  // `items` alternates [arg, sep, arg, sep, …, arg]; args count is
  // `ceil(items.len() / 2)`. Pre-size to skip Vec doublings.
  let mut args = Vec::with_capacity(items.len().div_ceil(2));
  let mut items_iter = items.iter_mut();
  while let Some(arg) = items_iter.next() {
    args.push(arg);
    let _discard_punct = items_iter.next();
  }
  (punct, args)
}

// Some handy shorthands.
/// Morph a VERTBAR token to OPEN or CLOSE (or MIDDLE/PUNCT) — mirrors Perl's MorphVertbar.
/// For delimiter roles (OPEN, CLOSE, MIDDLE), `|` stays `|`.
/// For operator roles (PUNCT), `|` becomes `⁣` (U+2223 DIVIDES).
fn morph_vertbar(xm: XM, role: &'static str, nodes: &[XMLNode]) -> XM {
  // Character substitution: for delimiter category keep `|` as-is.
  // For operator category: `|` → `⁣` (U+2223).
  let is_delimiter = matches!(role, "OPEN" | "CLOSE" | "MIDDLE");
  match xm {
    XM::Lexeme(lex, meta) => {
      if let Ok(node) = lookup_lex_node(&lex, nodes) {
        let mut props = XProps::from(node);
        props.role = Some(Cow::Borrowed(role));
        if !is_delimiter {
          if let Some(ref c) = props.content.clone() {
            if c == "|" {
              props.content = Some(Cow::Borrowed("\u{2223}"));
            }
          }
        }
        XM::Token(props, meta)
      } else {
        XM::Lexeme(lex, meta)
      }
    },
    XM::Token(mut props, meta) => {
      props.role = Some(Cow::Borrowed(role));
      if !is_delimiter {
        if let Some(ref c) = props.content.clone() {
          if c == "|" {
            props.content = Some(Cow::Borrowed("\u{2223}"));
          }
        }
      }
      XM::Token(props, meta)
    },
    xm => xm,
  }
}

/// Perl: MaybeFunction — when MATHPARSER_SPECULATE is set and an UNKNOWN token
/// is used as the left operand of invisible times with a delimited right side,
/// mark the token with possibleFunction="yes".
fn maybe_mark_possible_function(left: &mut Option<XM>, right: &Option<XM>, nodes: &[XMLNode]) {
  // Only active when MATHPARSER_SPECULATE is set. Use with_value to
  // avoid cloning the Stored envelope on every invisible-times probe
  // (runs per token in the math parser).
  let speculate = latexml_core::state::with_value("MATHPARSER_SPECULATE", |v| {
    matches!(v, Some(latexml_core::state::Stored::Bool(true)))
  });
  if !speculate {
    return;
  }
  // Check if right side contains delimiters (parenthesized group)
  let right_has_delimiters = matches!(right, Some(XM::Dual(..)) | Some(XM::Wrap(..)));
  if !right_has_delimiters {
    return;
  }
  // Navigate through XMApp wrappers to find the innermost token (matching Perl's descent)
  mark_inner_possible_function(left, nodes);
}

fn mark_inner_possible_function(xm: &mut Option<XM>, nodes: &[XMLNode]) {
  match xm {
    Some(XM::Token(ref mut props, _)) if props.role.as_deref() == Some("UNKNOWN") => {
      props.possible_function = Some(Cow::Borrowed("yes"));
    },
    Some(XM::Lexeme(ref lex, _)) if lex.starts_with("UNKNOWN:") => {
      // Lexemes are "ROLE:content:id" references to XML nodes.
      // Set the attribute directly on the underlying XML node.
      if let Some(id_str) = lex.split(':').next_back() {
        if let Ok(id) = id_str.parse::<usize>() {
          if id > 0 && id <= nodes.len() {
            let mut node = nodes[id - 1].clone();
            let _ = node.set_attribute("possibleFunction", "yes");
          }
        }
      }
    },
    Some(XM::Apply(_, ref mut args, ..)) => {
      if let Some(first) = args.0.first_mut() {
        mark_inner_possible_function(first, nodes);
      }
    },
    _ => {},
  }
}

/// Check if an XM NUMBER has integer content (no decimal point)
fn is_integer_number(xm: &Option<XM>) -> bool {
  match xm {
    Some(XM::Token(props, _)) => {
      props.role.as_deref() == Some("NUMBER")
        && props.meaning.as_ref().is_none_or(|m| !m.contains('.'))
    },
    Some(XM::Lexeme(lex, _)) => lex.starts_with("NUMBER:") && !lex.contains('.'),
    _ => false,
  }
}

fn is_number(xm: &Option<XM>) -> bool {
  match xm {
    Some(XM::Token(props, _)) => props.role.as_deref() == Some("NUMBER"),
    Some(XM::Lexeme(lex, _)) => lex.starts_with("NUMBER:"),
    _ => false,
  }
}

/// Find the DOM node for a fraction from an XM tree
fn find_fracop_node<'a>(
  xm: &Option<XM>,
  nodes: &'a [libxml::tree::Node],
) -> Option<&'a libxml::tree::Node> {
  // Try via Lexeme curry_level (node index)
  let meta = match xm {
    Some(XM::Lexeme(_, ref m)) => Some(m),
    Some(XM::Apply(_, _, _, ref m)) => Some(m),
    _ => None,
  };
  if let Some(meta) = meta {
    if let Some(ref cv) = meta.curry_level {
      let cv_str = cv.to_string();
      if let Some(idx_str) = cv_str.strip_prefix(':') {
        if let Ok(lex_idx) = idx_str.parse::<usize>() {
          let idx = if lex_idx > 0 { lex_idx - 1 } else { 0 };
          if idx < nodes.len() && nodes[idx].get_name() == "XMApp" {
            return Some(&nodes[idx]);
          }
        }
      }
    }
    // Try via Lexeme content: "ROLE:meaning:N" → index N-1
    if let Some(XM::Lexeme(ref lex, _)) = xm {
      if let Some(idx_str) = lex.rsplit(':').next() {
        if let Ok(lex_idx) = idx_str.parse::<usize>() {
          let idx = if lex_idx > 0 { lex_idx - 1 } else { 0 };
          if idx < nodes.len() && nodes[idx].get_name() == "XMApp" {
            return Some(&nodes[idx]);
          }
        }
      }
    }
  }
  None
}

fn is_fracop(xm: &Option<XM>) -> bool {
  match xm {
    Some(XM::Apply(op, ..)) => {
      if let XM::Token(props, _) = &*op.0 {
        props.role.as_deref() == Some("FRACOP")
      } else if let XM::Lexeme(lex, _) = &*op.0 {
        lex.starts_with("FRACOP:")
      } else {
        false
      }
    },
    _ => false,
  }
}

fn invisible_plus() -> XProps {
  XProps {
    meaning: Some(Cow::Borrowed("plus")),
    role: Some(Cow::Borrowed("ADDOP")),
    content: Some(Cow::Borrowed("\u{2064}")), // INVISIBLE PLUS
    font: Some(font::FONT_TEXT_DEFAULT.specialize("\u{2064}")),
    ..XProps::default()
  }
}

fn invisible_times() -> XProps {
  XProps {
    meaning: Some(Cow::Borrowed("times")),
    role: Some(Cow::Borrowed("MULOP")),
    content: Some(Cow::Borrowed("\u{2062}")),
    font: Some(font::FONT_TEXT_DEFAULT.specialize("\u{2062}")),
    ..XProps::default()
  }
}

fn invisible_comma() -> XProps {
  XProps {
    role: Some(Cow::Borrowed("PUNCT")),
    content: Some(Cow::Borrowed("\u{2063}")),
    font: Some(font::FONT_TEXT_DEFAULT.specialize("\u{2063}")),
    ..XProps::default()
  }
}

fn xnew(text: String) -> XProps {
  XProps {
    meaning: Some(Cow::Owned(text)),
    ..XProps::default()
  }
}

/// Check if an XM node is a bigop or a scripted bigop (at any depth).
/// e.g. ∫, ∫_0, ∫_0^∞, {}_a^b∫_c^d — all contain a bigop at the base.
fn is_bigop_or_scripted_bigop(xm: &XM, nodes: &[libxml::tree::Node]) -> bool {
  match xm {
    XM::Token(props, _) => {
      matches!(
        props.role.as_deref(),
        Some("INTOP") | Some("BIGOP") | Some("SUMOP") | Some("LIMITOP") | Some("DIFFOP")
      )
    },
    XM::Lexeme(lex_id, _) => {
      matches!(
        get_lexeme_role(lex_id, nodes).as_deref(),
        Some("INTOP") | Some("BIGOP") | Some("SUMOP") | Some("LIMITOP") | Some("DIFFOP")
      )
    },
    XM::Apply(ref op, ref args, ..) => {
      let op_role = get_operator_role(op, nodes);
      // Direct bigop application: Apply(INTOP, ...)
      if matches!(
        op_role.as_deref(),
        Some("INTOP") | Some("BIGOP") | Some("SUMOP") | Some("LIMITOP") | Some("DIFFOP")
      ) {
        return true;
      }
      // Scripted: Apply(SUBSCRIPTOP/SUPERSCRIPTOP, base, script)
      // Recursively check the base (first arg)
      if matches!(
        op_role.as_deref(),
        Some("SUBSCRIPTOP")
          | Some("SUPERSCRIPTOP")
          | Some("POSTSUBSCRIPT")
          | Some("POSTSUPERSCRIPT")
      ) {
        if let Some(Some(ref base)) = args.0.first() {
          return is_bigop_or_scripted_bigop(base, nodes);
        }
      }
      false
    },
    _ => false,
  }
}

/// Check if an XM node is a scripted function head: Apply(SCRIPTOP, FUNCTION_base, script)
/// at any nesting depth. e.g. f^2, f', f_n, sin^2 — all have a FUNCTION at the base.
fn is_scripted_function_head(xm: &XM, nodes: &[libxml::tree::Node]) -> bool {
  match xm {
    // A bare function token is not "scripted" — handled by the earlier check
    XM::Token(..) | XM::Lexeme(..) => false,
    XM::Apply(ref op, ref args, ..) => {
      let op_role = get_operator_role(op, nodes);
      // Must be a script operator (SUBSCRIPTOP, SUPERSCRIPTOP, etc.)
      if matches!(
        op_role.as_deref(),
        Some("SUBSCRIPTOP")
          | Some("SUPERSCRIPTOP")
          | Some("POSTSUBSCRIPT")
          | Some("POSTSUPERSCRIPT")
      ) {
        // Check the base (first arg) — is it a FUNCTION or another scripted function?
        if let Some(Some(ref base)) = args.0.first() {
          return is_function_role_item(base, nodes) || is_scripted_function_head(base, nodes);
        }
      }
      false
    },
    _ => false,
  }
}

/// Check if an XM item has a FUNCTION/OPFUNCTION/TRIGFUNCTION role.
fn is_function_role_item(xm: &XM, nodes: &[libxml::tree::Node]) -> bool {
  let role = match xm {
    XM::Token(props, _) => props.role.as_deref().map(String::from),
    XM::Lexeme(lex_id, _) => get_lexeme_role(lex_id, nodes),
    _ => None,
  };
  matches!(
    role.as_deref(),
    Some("FUNCTION") | Some("OPFUNCTION") | Some("TRIGFUNCTION")
  )
}

/// Extract the role of an XM operator (Token or Lexeme).
fn get_operator_role(op: &Operator, nodes: &[libxml::tree::Node]) -> Option<String> {
  match &*op.0 {
    XM::Token(props, _) => props.role.as_deref().map(String::from),
    XM::Lexeme(lex_id, _) => get_lexeme_role(lex_id, nodes),
    _ => None,
  }
}

/// Extract the role from a lexeme ID by looking up the DOM node.
fn get_lexeme_role(lex_id: &str, nodes: &[libxml::tree::Node]) -> Option<String> {
  lex_id
    .split(':')
    .next_back()
    .and_then(|s| s.parse::<usize>().ok())
    .and_then(|id| {
      if id > 0 && id <= nodes.len() {
        nodes[id - 1].get_attribute("role")
      } else {
        None
      }
    })
}

fn absent() -> XM {
  let props = XProps {
    meaning: Some(Cow::Borrowed("absent")),
    ..XProps::default()
  };
  props.into()
}

/// Prefix arrow: `→ expr` becomes `Apply(→, absent, expr)` — matching Perl's `AnyOp Expression`
/// Perl: MorphVertbar — expression VERTBAR expression treated as conditional/modifier
/// e.g. `x | y,z,t` → `conditional@(x, list@(y,z,t))`
pub fn vertbar_modifier(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => left, _vertbar, right);
  // Morph the VERTBAR to MODIFIEROP with meaning="conditional"
  // Use text default font (not math italic) — Perl MorphVertbar produces unfonted |
  let modop = XProps {
    meaning: Some(Cow::Borrowed("conditional")),
    role: Some(Cow::Borrowed("MODIFIEROP")),
    stretchy: Some(Cow::Borrowed("false")),
    content: Some(Cow::Borrowed("|")),
    font: Some(font::FONT_TEXT_DEFAULT.specialize("|")),
    ..XProps::default()
  };
  Ok(Some(XM::Apply(
    modop.into(),
    Args(vec![left, right]),
    XProps::default(),
    Meta::default(),
  )))
}

/// Perl moreRelations: consecutive relops without intervening terms.
/// `A ∈ ∞ ∋` → Apply(∈, A*∞, ∋) where ∋ is appended without absent.
pub fn consecutive_relop_chain(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => left, relop1, relop2);
  // Build a multirelation or extend existing one, appending both relops
  let mut left = left;
  if let Some(XM::Apply(ref op, ref mut left_args, ..)) = left {
    if let XM::Token(ref tok, _) = *op.0 {
      if tok.meaning == Some(Cow::Borrowed("multirelation")) {
        left_args.0.push(relop1);
        left_args.0.push(relop2);
        return Ok(left);
      }
    }
    // If left is Apply(RELOP, a, b), convert to multirelation
    let is_relop = match &*op.0 {
      XM::Lexeme(ref lex, _) => lex.split(':').next().unwrap().contains("RELOP"),
      XM::Token(ref tok, _) => matches!(tok.role.as_deref(), Some("RELOP")),
      _ => false,
    };
    if is_relop {
      let multirel_tok = XProps {
        meaning: Some(Cow::Borrowed("multirelation")),
        ..XProps::default()
      };
      let mut drained = left_args.0.drain(..);
      let l1 = drained.next().unwrap();
      let l2 = drained.next().unwrap();
      let moved_op = (*op.0).clone();
      return Ok(Some(XM::Apply(
        multirel_tok.into(),
        Args(vec![l1, Some(moved_op), l2, relop1, relop2]),
        XProps::default(),
        Meta::default(),
      )));
    }
  }
  // Base case: left is an expression, relop1+relop2 are consecutive
  // Apply(relop1, left, relop2) — relop2 becomes the right operand
  Ok(Some(XM::Apply(
    relop1.into(),
    Args(vec![left, relop2]),
    XProps::default(),
    Meta::default(),
  )))
}

/// Perl: formula relop (no right operand) — trailing relop with implied absent right
/// e.g. `y < 2 <` → `multirelation(y, <, 2, <, absent)`
pub fn postfix_relop(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => left, relop);
  let right = Some(absent());
  // Reuse infix_relation logic: if left is already a relation, convert to multirelation
  let mut left = left;
  if let Some(XM::Apply(ref op, ref mut left_args, ..)) = left {
    if let XM::Token(ref tok, _) = *op.0 {
      if tok.meaning == Some(Cow::Borrowed("multirelation")) {
        left_args.0.push(relop);
        left_args.0.push(right);
        return Ok(left);
      }
    }
    if let XM::Lexeme(ref lex, _) = *op.0 {
      if lex.split(':').next().unwrap().contains("RELOP") {
        let multirel_tok = XProps {
          meaning: Some(Cow::Borrowed("multirelation")),
          ..XProps::default()
        };
        let mut drained = left_args.0.drain(..);
        let l1 = drained.next().unwrap();
        let l2 = drained.next().unwrap();
        let moved_op = (*op.0).clone();
        return Ok(Some(XM::Apply(
          multirel_tok.into(),
          Args(vec![l1, Some(moved_op), l2, relop, right]),
          XProps::default(),
          Meta::default(),
        )));
      }
    }
  }
  // Simple case: just apply relop to left and absent
  Ok(Some(XM::Apply(
    relop.into(),
    Args(vec![left, right]),
    XProps::default(),
    Meta::default(),
  )))
}

/// Perl: METARELOP Formula — prefix metarelop with implied absent left operand
/// e.g. `\vdash x = 0` → `absent proves (x = 0)`
pub fn prefix_metarelop_apply(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => metarelop, right);
  Ok(Some(XM::Apply(
    metarelop.into(),
    Args(vec![Some(absent()), right]),
    XProps::default(),
    Meta::default(),
  )))
}

/// Perl: AnyOp Expression => Apply(AnyOp, Absent(), Expression)
/// Leading relop with implied absent left operand (e.g. `= e + f + g` in eqnarray)
pub fn prefix_relop_apply(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => relop, right);
  // Early-action prune: when the operator is METARELOP (e.g.
  // `\vdash`, `:`, `\models`), the formula-level
  // `metarelop expression → prefix_relop_apply` rule competes with
  // the statement-level `metarelop formula → prefix_metarelop_apply`
  // rule. Legacy prefers the statement-level grouping
  // (`Apply(vdash, formula)`) over the formula-level chain
  // (`multirelation(vdash, ..., =, 0)`). Reject the formula-level
  // METARELOP prefix here so the parse goes through the
  // statement-level rule.
  let is_metarelop = relop.as_ref().is_some_and(|op| match op {
    XM::Lexeme(l, _) => l.split(':').next() == Some("METARELOP"),
    XM::Token(p, _) => p.role.as_deref() == Some("METARELOP"),
    _ => false,
  });
  if is_metarelop {
    return Err(
      "prefix_relop_apply: METARELOP prefix at formula-level — \
       prefer statement-level prefix_metarelop_apply"
        .into(),
    );
  }
  // For BINOP prefix usage (e.g. \mathbin{|}x), Perl produces op@(x) without absent.
  // For RELOP prefix (e.g. = b, < c), keep absent as first arg.
  let is_binop = relop.as_ref().is_some_and(|op| match op {
    XM::Lexeme(l, _) => l.split(':').next() == Some("BINOP"),
    XM::Token(p, _) => p.role.as_deref() == Some("BINOP"),
    _ => false,
  });
  let args = if is_binop {
    Args(vec![right])
  } else {
    Args(vec![Some(absent()), right])
  };
  Ok(Some(XM::Apply(
    relop.into(),
    args,
    XProps::default(),
    Meta::default(),
  )))
}

pub fn prefix_arrow_apply(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => arrowop, right);
  Ok(Some(XM::Apply(
    arrowop.into(),
    Args(vec![Some(absent()), right]),
    XProps::default(),
    Meta::default(),
  )))
}

/// Arrow-wrapped content from amscd XMWrap role="ARROW":
/// start_ARROW arrow expression end_ARROW → Apply(arrow, absent, expression)
pub fn arrow_wrap_apply(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => _start, arrowop, content, _end);
  Ok(Some(XM::Apply(
    arrowop.into(),
    Args(vec![Some(absent()), content]),
    XProps::default(),
    Meta::default(),
  )))
}

/// Arrow-wrapped solo (no expression): start_ARROW arrow end_ARROW → just the arrow
pub fn arrow_wrap_solo(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => _start, arrowop, _end);
  Ok(arrowop)
}

/// OPEN expr (without CLOSE) — e.g. \{ array → cases-like wrapping.
/// Perl: factorOpen handles unmatched OPEN by consuming the expression.
/// For { delimiter, produces XMDual: content=Apply(cases, XMRef), pres=XMWrap({, expr).
pub fn open_fenced(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => open_opt, arg_opt);
  let open = open_opt.unwrap();
  let mut arg = arg_opt.unwrap();
  // Perl: Fence({, content) → XMDual(Apply(cases, XMRef(content)), XMWrap({, content))
  let o = open.get_value(ctxt.nodes)?;
  if o == "{" {
    let op = XProps {
      meaning: Some(Cow::Borrowed("cases")),
      ..XProps::default()
    };
    let refs = create_xmrefs(&mut [&mut arg], ctxt)?;
    let content = XM::Apply(
      Operator::from(op),
      Args(refs.into_iter().map(Option::Some).collect()),
      XProps::default(),
      Meta::default(),
    );
    // Perl: XMWrap(open, content, absent_close) — absent marks missing close delimiter
    let absent_close = XM::Token(
      XProps {
        meaning: Some(Cow::Borrowed("absent")),
        ..XProps::default()
      },
      Meta::default(),
    );
    let pres = XM::Wrap(
      vec![open, arg, absent_close],
      XProps::default(),
      Meta::default(),
    );
    Ok(Some(XM::Dual(
      Box::new(content),
      Box::new(pres),
      XProps::default(),
      Meta::default(),
    )))
  } else {
    // Non-brace open without close — just wrap
    let absent_close = XM::Token(
      XProps {
        meaning: Some(Cow::Borrowed("absent")),
        ..XProps::default()
      },
      Meta::default(),
    );
    Ok(Some(XM::Wrap(
      vec![open, arg, absent_close],
      XProps::default(),
      Meta::default(),
    )))
  }
}

/// expr CLOSE (without OPEN) — e.g. array \} → cases-like wrapping.
/// For } delimiter, produces XMDual: content=Apply(cases, XMRef), pres=XMWrap(expr, }).
pub fn close_fenced(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => arg_opt, close_opt);
  let mut arg = arg_opt.unwrap();
  let close = close_opt.unwrap();
  // Perl: Fence(content, }) → XMDual(Apply(cases, XMRef(content)), XMWrap(content, }))
  let c = close.get_value(ctxt.nodes)?;
  if c == "}" {
    let op = XProps {
      meaning: Some(Cow::Borrowed("cases")),
      ..XProps::default()
    };
    let refs = create_xmrefs(&mut [&mut arg], ctxt)?;
    let content = XM::Apply(
      Operator::from(op),
      Args(refs.into_iter().map(Option::Some).collect()),
      XProps::default(),
      Meta::default(),
    );
    // Perl: XMWrap(absent_open, content, close) — absent marks missing open delimiter
    let absent_open = XM::Token(
      XProps {
        meaning: Some(Cow::Borrowed("absent")),
        ..XProps::default()
      },
      Meta::default(),
    );
    let pres = XM::Wrap(
      vec![absent_open, arg, close],
      XProps::default(),
      Meta::default(),
    );
    Ok(Some(XM::Dual(
      Box::new(content),
      Box::new(pres),
      XProps::default(),
      Meta::default(),
    )))
  } else {
    let absent_open = XM::Token(
      XProps {
        meaning: Some(Cow::Borrowed("absent")),
        ..XProps::default()
      },
      Meta::default(),
    );
    Ok(Some(XM::Wrap(
      vec![absent_open, arg, close],
      XProps::default(),
      Meta::default(),
    )))
  }
}

/// Double-fenced: <<expr>> or <<list>> — double angle brackets as a single
/// semantic unit. Used in quantum mechanics (<<a|b>>), operator theory, etc.
/// Produces Apply(delimited-<<>>, content).
#[allow(dead_code)]
pub fn double_fenced(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  // Args: open1, open2, content, close1, close2
  unp!(args => _open1, _open2, arg_opt, _close1, _close2);
  let arg = arg_opt.unwrap();
  let op = XProps {
    meaning: Some(Cow::Borrowed("delimited-\\langle\\langle\\rangle\\rangle")),
    ..XProps::default()
  };
  Ok(Some(XM::Apply(
    Operator::from(op),
    Args(vec![Some(arg)]),
    XProps::default(),
    Meta::default(),
  )))
}

/// Perl MathGrammar L259-260, MathParser.pm L1656-1668: NewEvalAt
/// Handles `a|_{x=0}`, `f(x)|_{0}^{1}`, `\left.xyz\right|_{0}^{2}`
/// Pattern: base evalAtOp sub [sup]
///
/// Content arm:  XMApp(evaluated-at, base_ref, sub_ref?, sup_ref?)
/// Presentation: XMApp(SUBSCRIPTOP, XMWrap(base, bar[CLOSE]), sub_content)
///               optionally wrapped in XMApp(SUPERSCRIPTOP, ..., sup_content)
pub fn eval_at(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  // Args: base, vertbar, sub, [sup] — 3 or 4 args depending on rule
  let mut base = args.remove(0).unwrap();
  let vertbar = args.remove(0).unwrap();
  // Remaining args: sub and optionally sup scripts (faux_wrap'd start_POSTSUBSCRIPT tokens)
  // Perl: maybeEvalAt handles SUB then SUP or SUP then SUB
  let (sub_script, sup_script) = if args.len() == 2 {
    let s1 = args.remove(0);
    let s2 = args.remove(0);
    // Determine which is sub and which is super by checking the role.
    // faux_wrap returns Wrap([lexeme, content]) — extract the lexeme first.
    let s1_is_sub = s1.as_ref().is_none_or(|xm| {
      let lex = match xm {
        XM::Lexeme(ref l, _) => Some(&**l),
        XM::Wrap(ref items, ..) if !items.is_empty() => {
          if let XM::Lexeme(ref l, _) = items[0] {
            Some(&**l)
          } else {
            None
          }
        },
        _ => None,
      };
      lex
        .and_then(|l| lookup_lex_node(l, ctxt.nodes).ok())
        .map(|n| n.get_attribute("role").unwrap_or_default().contains("SUB"))
        .unwrap_or(true)
    });
    if s1_is_sub { (s1, s2) } else { (s2, s1) }
  } else {
    (args.remove(0), None)
  };

  // Pre-extract all values from ctxt.nodes before create_xmrefs consumes ctxt
  let bar_close = morph_vertbar(vertbar, "CLOSE", ctxt.nodes);
  let sub_content_xm = get_script_child_xm(&sub_script, ctxt.nodes);
  let sup_content_xm = sup_script
    .as_ref()
    .and_then(|s| get_script_child_xm(&Some(s.clone()), ctxt.nodes));
  let sub_content_xm2 = get_script_child_xm(&sub_script, ctxt.nodes);
  let sup_content_xm2 = sup_script
    .as_ref()
    .and_then(|s| get_script_child_xm(&Some(s.clone()), ctxt.nodes));

  // Build content arm FIRST: this sets _xmkey/xml:id on base and sub/sup
  // so the presentation arm (built after) gets the references.
  let eval_tok = XM::Token(
    XProps {
      meaning: Some(Cow::Borrowed("evaluated-at")),
      ..XProps::default()
    },
    Meta::default(),
  );

  let mut sub_for_ref = sub_content_xm;
  let mut sup_for_ref = sup_content_xm;

  let mut content_args: Vec<&mut XM> = vec![&mut base];
  if let Some(ref mut sc) = sub_for_ref {
    content_args.push(sc);
  }
  if let Some(ref mut sc) = sup_for_ref {
    content_args.push(sc);
  }
  let ref_args = create_xmrefs(&mut content_args, ctxt)?;

  let content = XM::Apply(
    eval_tok.into(),
    ref_args.into(),
    XProps::default(),
    Meta::default(),
  );

  // Build presentation arm AFTER content (so base has _xmkey set)
  let wrap = XM::Wrap(vec![base, bar_close], XProps::default(), Meta::default());

  let sub_op = XM::Token(
    XProps {
      role: Some(Cow::Borrowed("SUBSCRIPTOP")),
      scriptpos: Some(Cow::Borrowed("post1")),
      ..XProps::default()
    },
    Meta::default(),
  );
  let mut pres = XM::Apply(
    sub_op.into(),
    Args(vec![Some(wrap), sub_content_xm2]),
    XProps::default(),
    Meta::default(),
  );

  if let Some(sup_xm) = sup_content_xm2 {
    let sup_op = XM::Token(
      XProps {
        role: Some(Cow::Borrowed("SUPERSCRIPTOP")),
        scriptpos: Some(Cow::Borrowed("post1")),
        ..XProps::default()
      },
      Meta::default(),
    );
    pres = XM::Apply(
      sup_op.into(),
      Args(vec![Some(pres), Some(sup_xm)]),
      XProps::default(),
      Meta::default(),
    );
  }

  Ok(Some(XM::Dual(
    Box::new(content),
    Box::new(pres),
    XProps::default(),
    Meta::default(),
  )))
}

/// Get the first child element of a script wrapper as an XM.
/// Handles both old-style Lexeme and new-style Wrap([lexeme, content]) from faux_wrap.
fn get_script_child_xm(script_opt: &Option<XM>, nodes: &[XMLNode]) -> Option<XM> {
  let script = script_opt.as_ref()?;
  // New format: Wrap([lexeme, content]) — return the parsed content directly
  if let XM::Wrap(ref items, ..) = script {
    if items.len() == 2 {
      return Some(items[1].clone());
    }
  }
  // Old format: bare Lexeme — look up from DOM
  if let XM::Lexeme(ref lex, _) = script {
    let node = lookup_lex_node(lex, nodes).ok()?;
    let children = node.get_child_elements();
    if let Some(first_child) = children.first() {
      for (i, n) in nodes.iter().enumerate() {
        if n == first_child {
          return Some(XM::Lexeme(Rc::from(format!("{}", i + 1).as_str()), Meta::default()));
        }
      }
      return Some(XM::from(first_child));
    }
  }
  None
}

/// Dirac bra-ket notation helpers.
/// All produce Apply(meaning, args) wrapped in XMDual with appropriate presentation.
fn qm_fenced(
  meaning: &'static str,
  mut args_xm: Vec<Option<XM>>,
  mut stuff: Vec<XM>,
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  let op = XProps {
    meaning: Some(Cow::Borrowed(meaning)),
    ..XProps::default()
  };
  let mut arg_refs: Vec<&mut XM> = args_xm.iter_mut().filter_map(|a| a.as_mut()).collect();
  let refs: Vec<Option<XM>> = create_xmrefs(arg_refs.as_mut_slice(), ctxt)?
    .into_iter()
    .map(Option::Some)
    .collect();
  // Propagate xmkey from content args to matching presentation stuff items.
  // create_xmrefs sets xmkey on the args in args_xm, but stuff was cloned
  // before that. The presentation-side elements need _xmkey so that the
  // base_xmath createXMRefs handler can resolve the content-side XMRef.
  for arg in args_xm.iter().flatten() {
    let arg_xmkey = match arg {
      XM::Token(p, _) | XM::Apply(_, _, p, _) | XM::Dual(_, _, p, _) | XM::Wrap(_, p, _) => {
        p.xmkey.clone()
      },
      _ => None,
    };
    if let Some(ref key) = arg_xmkey {
      // Find the corresponding non-delimiter item in stuff
      for item in stuff.iter_mut() {
        match item {
          XM::Token(props, _)
          | XM::Apply(_, _, props, _)
          | XM::Dual(_, _, props, _)
          | XM::Wrap(_, props, _)
            if props.xmkey.is_none() && props.id.is_none() =>
          {
            props.xmkey = Some(key.clone());
            break;
          },
          _ => {},
        }
      }
    }
  }
  Ok(Some(XM::Dual(
    Box::new(XM::Apply(
      op.into(),
      Args(refs),
      XProps::default(),
      Meta::default(),
    )),
    Box::new(XM::Wrap(stuff, XProps::default(), Meta::default())),
    XProps::default(),
    Meta::default(),
  )))
}

/// `<a>` → expectation@(a) — Perl enclose1: '<@>' => 'expectation'
#[allow(dead_code)]
pub fn qm_expectation(
  _: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  let stuff: Vec<XM> = args.iter().flatten().cloned().collect();
  unp!(args => _open, expr, _close);
  qm_fenced("expectation", vec![expr], stuff, ctxt)
}

/// `<a|` → bra@(a) — Perl enclose1: '<@|' => 'bra'
pub fn qm_bra(
  _: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  let stuff: Vec<XM> = args.iter().flatten().cloned().collect();
  unp!(args => _open, expr, _bar);
  qm_fenced("bra", vec![expr], stuff, ctxt)
}

/// `|b>` → ket@(b) — Perl enclose1: '|@>' => 'ket'
pub fn qm_ket(
  _: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  let stuff: Vec<XM> = args.iter().flatten().cloned().collect();
  unp!(args => _bar, expr, _close);
  qm_fenced("ket", vec![expr], stuff, ctxt)
}

/// `<a|b>` → inner-product@(a, b) — Perl MathGrammar L382-386
pub fn qm_braket(
  _: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  let stuff: Vec<XM> = args.iter().flatten().cloned().collect();
  unp!(args => _open, left, _bar, right, _close);
  qm_fenced("inner-product", vec![left, right], stuff, ctxt)
}

/// `<a|f|b>` → quantum-operator-product@(a, f, b) — Perl MathGrammar L387-393
pub fn qm_bracket(
  _: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  let stuff: Vec<XM> = args.iter().flatten().cloned().collect();
  unp!(args => _open, left, _bar1, mid, _bar2, right, _close);
  qm_fenced(
    "quantum-operator-product",
    vec![left, mid, right],
    stuff,
    ctxt,
  )
}

/// Perl MathGrammar L294: `|| exp ||` → norm
/// Merges two single vertbar `|` tokens into double `‖` and fences as norm.
pub fn norm_fenced(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  // Args: |1 |2 expression |3 |4
  unp!(args => open1_opt, _open2_opt, arg_opt, _close1_opt, _close2_opt);
  let arg = arg_opt.unwrap();
  // Merge each pair of | into ‖
  let open = merge_vertbar_pair(open1_opt.unwrap(), "OPEN", ctxt.nodes);
  let close = merge_vertbar_pair(_close1_opt.unwrap(), "CLOSE", ctxt.nodes);
  let op = XProps {
    meaning: Some(Cow::Borrowed("norm")),
    ..XProps::default()
  };
  interpret_delimited(op.into(), vec![open, arg, close], ctxt).map(Option::Some)
}

/// Merge two single `|` tokens into `‖` (U+2016) with the given role.
/// Perl CatSymbols: concatenates two delimiters into a combined symbol.
fn merge_vertbar_pair(xm: XM, role: &'static str, nodes: &[XMLNode]) -> XM {
  let mut props = match xm {
    XM::Lexeme(ref lex, _) => {
      if let Ok(node) = lookup_lex_node(lex, nodes) {
        XProps::from(node)
      } else {
        XProps::default()
      }
    },
    XM::Token(ref p, _) => p.clone(),
    _ => XProps::default(),
  };
  props.role = Some(Cow::Borrowed(role));
  props.content = Some(Cow::Borrowed("\u{2016}")); // ‖
  props.stretchy = None; // remove stretchy=false from individual |
  XM::Token(props, Meta::default())
}
