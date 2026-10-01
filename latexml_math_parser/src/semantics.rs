use std::{borrow::Cow, cell::Cell, error::Error, rc::Rc};

use latexml_core::{
  common::{
    font::{self, Font},
    xml::element_nodes,
  },
  document::Document,
  raw_map,
};
use libxml::tree::Node as XMLNode;
use marpa::{lexer::token::Token, stack::*, thin::Value, tree_builder::*};
use rustc_hash::FxHashMap as HashMap;

use self::tree::lookup_lex_node;
pub use self::tree::{Args, Operator, XM, XProps};
use crate::{
  parser::{p_get_value, realize_xmnode},
  pragmatics::{ValidationPragmatics, is_invisible_times_op},
  util::create_xmrefs,
};

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
          // No registered action for this rule id with multiple children — the
          // ambiguous grammar's fallback keeps the first parse (the "aggressively
          // prune" design). Route it through the math-parser diagnostic path
          // (`Info:` — captured in the log floor, `--quiet`-gated on stderr, and
          // NOT inflating the warning verdict) instead of a raw `eprintln!` that
          // bypassed the logger and printed unconditionally.
          log_math_info!(
            "ambiguous",
            "action",
            format!("rule {id:?}: returning first of {more} children; dropped {args:?}")
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
      Node::Leaf(ref tok) => Ok(Some(XM::Lexeme(
        Rc::from(tok.to_string().as_str()),
        Meta::default(),
      ))),
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
  if let Some(XM::Lexeme(ref lex, _)) = infixop
    && lex.contains(":compose:")
  {
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
  if let Some(XM::Apply(op, args, ..)) = xm {
    // An Apply with a function/trigfunction/opfunction operator that has arguments
    // is a ground-level application, not a function value.
    if !args.0.is_empty()
      && let XM::Lexeme(ref lex, _) = *op.0
    {
      return lex.starts_with("TRIGFUNCTION:")
        || lex.starts_with("OPFUNCTION:")
        || lex.starts_with("EXPECTATION:")
        || lex.starts_with("FUNCTION:");
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
  let right = right.unwrap();
  let sep = sep.unwrap();

  // The separator decides the wrapper:
  //  * `\quad` (WIDE_PUNCT) separates whole FORMULAS at the top level — a
  //    `\quad`-list is a flat `formulae@(…)` whose items may be of ANY kind
  //    (relational or not), e.g. `a\op b \quad a\rel b \quad …`. (Perl emits a
  //    right-NESTED `formulae@(a, formulae@(b, …))`; we keep it FLAT — an
  //    intentional, user-directed divergence; see docs/parity/KNOWN_PERL_ERRORS.md.)
  //  * a plain comma separates a `list@(…)`. A MIXED comma-list — a relational
  //    item plus plain items — IS a valid list: a bare (unparenthesized)
  //    comma-list is a top-level list, and a relation is a legitimate item. So
  //    `0 < x, y` → `list(0<x, y)` and `a = b, c, d` → `list(a=b, c, d)`, NOT
  //    `lt(0, list(x,y))` / `eq(a, list(b,c,d))` (also an intentional divergence
  //    from Perl, which wrongly reads the comma-list as a single bare relation
  //    operand). The grammar's deleted `formula relop formula_list` rule used to
  //    manufacture that bogus reading.
  let is_quad = is_quad_separator(&sep);
  let left_rel = left.as_ref().is_some_and(is_relational_item);
  let right_rel = is_relational_item(&right);
  // For a COMMA list, an all-relational pair (`a=b, c=d`) is a `formulae@`, not a
  // list — defer to `formulae_apply`. `\quad` accepts relational pairs directly
  // (it builds `formulae@` itself). A comma list with exactly one relational item
  // (mixed) is accepted here as a `list@`.
  if !is_quad && left_rel && right_rel {
    return Err("list_apply: both items relational in a comma list, use formulae_apply".into());
  }
  // Rule 3: an `absent` relop operand marks an equation FRAGMENT — a leading or
  // trailing relop whose other operand lives on a different align line. How
  // strictly a fragment disqualifies an item depends on the separator:
  //
  //  * `\quad` (WIDE_PUNCT): reject if EITHER item is a fragment. A
  //    `\quad`-separated run of align continuations is one broken-up equation,
  //    not a list, and admitting it produces actively WRONG trees — the pinned
  //    `tests/math/sampler` case `\displaystyle=f(x)+\phantom{g(x)}+h(x)`
  //    (`\phantom` lexes as WIDE_PUNCT) parses to
  //    `fragments@(absent = limit-from@(f@(x), +), + h@(x))`, which mis-groups
  //    the `+`. `ltx_math_unparsed` is the honest outcome there.
  //  * comma: reject only when BOTH items are fragments. This mirrors the
  //    relaxation `formulae_apply` already carries below ("the earlier strict
  //    rule (reject if EITHER is a fragment) was correct only for inner
  //    contexts … at the top level these pairings are well-formed and need to
  //    survive pragmatic prune").
  //
  // The comma half matters because `list_apply` kept the strict form for both,
  // and with the `formula relop formula_list` rule deliberately gone
  // (KNOWN_PERL_ERRORS #37) a leading-relop item followed by a comma then had NO
  // derivation at all: `$>50,000$` fell out as `ltx_math_unparsed` even though
  // both halves parse alone (`>x` ✓, `a,b` ✓, `a>50,000` ✓). Pragmas prune; they
  // must not empty the forest. With this split, `>50,000` reaches the #37 reading
  // `list@(absent>50, 000)` — the same shape `a>50,000` already produced as
  // `list@(a>50, 000)`. (Perl instead builds `>(absent, list(50,000))` via the
  // rule #37 removed; ours is the intended divergence.) Witness: arXiv 2605.17646.
  let left_fragment = left.as_ref().is_some_and(has_absent_relop_operand);
  let right_fragment = has_absent_relop_operand(&right);
  let fragment_reject = if is_quad {
    left_fragment || right_fragment
  } else {
    left_fragment && right_fragment
  };
  if fragment_reject {
    return Err("list_apply: fragment item (absent relop operand) in this list".into());
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
          XM::Token(p, _) | XM::Ref(p) => p.meaning.as_deref(),
          XM::Lexeme(name, _) => Some(&**name),
          _ => None,
        };
        meaning == Some("conditional")
      },
      // Dual wrapping conditional: bare only if presentation is NOT
      // parens-fenced.
      XM::Dual(content, pres, ..) => {
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
          (matches!(items.first(),
            Some(XM::Token(p, _))
              if p.role.as_deref() == Some("OPEN")
                && p.content.as_deref() == Some("("))
            || matches!(items.first(),
              Some(XM::Lexeme(name, _)) if name.starts_with("OPEN:(:")))
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
  // `\quad` (WIDE_PUNCT) separates top-level FRAGMENTS — heterogeneous content
  // that needn't be formulas — so it gets its own flat `fragments@` class,
  // distinct from comma's `list@`/`formulae@`. A plain comma builds `list@`
  // (all-relational comma pairs were already routed to `formulae_apply` above).
  list_apply_core(left, sep, right, ctxt)
}

/// Does a list Dual's presentation hold its `items` alone — `[item, separator, item, …]`, `2n - 1`
/// nodes? A list fenced with its own delimiters (a bracket or brace list since 57au, Perl's `Fence`:
/// `[open, item, separator, …, close]`) does not: it is one closed item, which a comma after it does
/// not extend, a relation after it does not split, and parens around it do not rename or unpack —
/// each read it as a bare run and lost or unparsed the formula (`\left([a;b]\right)`
/// `vector@([, ;, ])`, 57aw; `W=[w_1,\ldots,w_d]\in\mathbb{R}^d` unparsed, 233 formulas of the 57ax
/// train A/B, 2605.03399, 2605.29816).
fn presents_its_items_alone(presentation: &XM, items: usize) -> bool {
  matches!(presentation, XM::Wrap(wrapped, ..) if items > 0 && wrapped.len() == 2 * items - 1)
}

/// Core list/fragments construction for `list_apply`, AFTER its admission
/// checks (relational pairs, absent-relop, Rule-4 bare-conditional). Extracted
/// so `vertbar_modifier_listlhs` can append a conditional as the LAST list item
/// WITHOUT Rule-4's rejection: in the bar-after-comma context (`a,b | c`) the
/// conditional `b|c` IS a legitimate item (Perl `list@(a, conditional@(b,c))`),
/// unlike the bar-first `x|y,z` wrap case (`conditional@(x, list@(y,z))`) that
/// Rule 4 guards against. `list_apply` itself runs the checks then calls this.
fn list_apply_core(
  mut left: Option<XM>,
  sep: XM,
  mut right: XM,
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  let is_quad = is_quad_separator(&sep);
  let meaning = if is_quad { "fragments" } else { "list" };

  // If left is already a list/formulae/fragments Dual without its own delimiters, extend it (flat
  // accumulation — all three classes are kept flat; `presents_its_items_alone`). A
  // `distribute_list_relation` dual has `formulae` content but a relation-Apply presentation, and
  // extending it here would strand a keyless bare ref (see the matching guard + rationale in
  // `formulae_apply`, EXPECTED_ID_XMREF_DESIGN 26v); a fenced list is one closed item, which a comma
  // after it does not extend (`[a,b,c],d`, 57ay; 2605.03399).
  if let Some(XM::Dual(ref mut content, ref mut pres, ..)) = left
    && let XM::Apply(ref op, ref mut op_args, ..) = **content
    && presents_its_items_alone(pres, op_args.0.len())
    && let XM::Token(ref props, _) = *op.0
    && matches!(
      props.meaning.as_deref(),
      Some("list") | Some("formulae") | Some("fragments")
    )
  {
    // Extend: add ref for new item to content args
    let new_ref = create_xmrefs(&mut [&mut right], ctxt)?;
    op_args.0.extend(new_ref.into_iter().map(Some));
    // Add separator and new item to presentation wrap
    if let XM::Wrap(ref mut items, ..) = **pres {
      items.push(sep);
      items.push(right);
    }
    return Ok(left);
  }

  list_or_formulae_create(left.unwrap(), sep, right, meaning, ctxt)
}

/// ASF migration item 5 (Option A): build a comma-list of
/// `modified_term` items. Mirrors `list_apply` but explicitly
/// ACCEPTS relational items — that's the whole point. Used for
/// function argument lists where each argument carries its own
/// single-relop expression, e.g. `P(x=0, y<0)`.
///
/// Rejects the same non-relop pathologies `list_apply` rejects
/// (bare conditional, etc.) but does NOT force formulae_apply on
/// relations. The grammar rules that route to this action
/// (`formula_list += modified_term punct …`) only fire when at
/// least one side is a `modified_term`, so the relation case is
/// the expected one.
pub fn modified_list_apply(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => left, sep, right);
  let mut left = left;
  let mut right = right.unwrap();
  let sep = sep.unwrap();
  let meaning = "list";

  // If left is already a list/formulae Dual, extend it (mirrors the
  // flat-accumulation behaviour at the end of `list_apply`).
  if let Some(XM::Dual(ref mut content, ref mut pres, ..)) = left
    && let XM::Apply(ref op, ref mut op_args, ..) = **content
    && presents_its_items_alone(pres, op_args.0.len())
    && let XM::Token(ref props, _) = *op.0
    && (props.meaning.as_deref() == Some("list") || props.meaning.as_deref() == Some("formulae"))
  {
    let new_ref = create_xmrefs(&mut [&mut right], ctxt)?;
    op_args.0.extend(new_ref.into_iter().map(Some));
    if let XM::Wrap(ref mut items, ..) = **pres {
      items.push(sep);
      items.push(right);
    }
    return Ok(left);
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
  if let Some(Some(XM::Dual(content, ..))) = args.first()
    && let XM::Apply(ref op, ref op_args, ..) = **content
    && let XM::Token(ref props, _) = *op.0
    && props.meaning.as_deref() == Some("list")
  {
    // Check if any list item is relational
    for arg in &op_args.0 {
      if arg.as_ref().is_some_and(is_relational_item) {
        return Err("formula_list_apply: list contains relational items".into());
      }
    }
  }
  list_apply(rule_id, args, p, ctxt)
}

/// Check if an XM tree contains `absent` as a direct operand of a relop.
/// Absent operands are valid at the top level (equation fragments like `= f(x)`)
/// but should be pruned when inside inner rules (lists, fenced expressions, function args).
fn has_absent_relop_operand(xm: &XM) -> bool {
  if let XM::Apply(op, args, ..) = xm
    && (is_multirelation(&op.0) || is_relational_op(&op.0))
  {
    for arg in &args.0 {
      if let Some(XM::Token(props, _)) = arg
        && props.meaning.as_deref() == Some("absent")
      {
        return true;
      }
    }
  }
  false
}

/// Check if an XM tree is a relational formula (contains RELOP or multirelation).
/// Used to distinguish Perl's "formulae" (comma-separated relations at top level)
/// from "list" (comma-separated plain expressions).
pub(crate) fn is_relational_item(xm: &XM) -> bool {
  match xm {
    XM::Apply(op, ..) => is_multirelation(&op.0) || is_relational_op(&op.0),
    // A formulae XMDual is inherently relational (it wraps relational items)
    XM::Dual(content, ..) => {
      if let XM::Apply(ref op, ..) = **content
        && let XM::Token(ref props, _) = *op.0
      {
        return props.meaning.as_deref() == Some("formulae");
      }
      false
    },
    _ => false,
  }
}

/// The #37 hole: a comma list that opens with two relations and goes on with a plain item
/// (`x=0, y=1, z`; `x_i\ge 0,\ i=1,\ldots,n`). The relational pair can only be a `formulae`
/// (`list_apply` refuses a comma pair of two relations), and `formulae_apply` refuses a plain item
/// after relations, so the list had no derivation (unparsed; Perl reads it). A flat `formulae`
/// container followed by a plain item after a comma (a WIDE_PUNCT comma too) is extended as a list is —
/// `list_apply`'s checks — and made a `list@(x = 0, y = 1, z)` (#37).
/// `attach_enumerations` then attaches an enumeration to its relation. A relational item after it
/// is `formulae_apply`'s; a non-flat formulae (`distribute_list_relation`'s) is refused, as
/// `list_apply_core` refuses to extend it. Its grammar rule takes `relation_pairs` and a plain
/// expression only, so no other comma list gets a derivation. Witnesses 2605.00467, 2605.24797.
pub fn formulae_then_item_apply(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmatics: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  let flat_formulae = matches!(args.first(), Some(Some(XM::Dual(content, pres, ..)))
    if matches!(&**content, XM::Apply(op, items, ..)
      if matches!(&*op.0, XM::Token(props, _) if props.meaning.as_deref() == Some("formulae"))
        && presents_its_items_alone(pres, items.0.len())));
  if !flat_formulae {
    return Err("formulae_then_item_apply: not a flat formulae container".into());
  }
  if args
    .get(2)
    .and_then(Option::as_ref)
    .is_none_or(is_relational_item)
  {
    return Err("formulae_then_item_apply: a relation after formulae is formulae_apply's".into());
  }
  // A comma only (a WIDE_PUNCT comma too): a `\quad`-list reads as fragments already, and a second
  // derivation for it would make its reading depend on enumeration order.
  if !args.get(1).and_then(Option::as_ref).is_some_and(|sep| {
    !is_quad_separator(sep) && realized_value(sep, &ctxt).is_ok_and(|v| v == ",")
  }) {
    return Err("formulae_then_item_apply: a comma list only".into());
  }
  let mut out = list_apply(rule_id, args, pragmatics, ctxt)?;
  if let Some(XM::Dual(content, ..)) = out.as_mut()
    && let XM::Apply(op, ..) = &mut **content
    && let XM::Token(props, _) = &mut *op.0
    && props.meaning.as_deref() == Some("formulae")
  {
    props.meaning = Some(Cow::Borrowed("list"));
  }
  Ok(out)
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
  if let Some(XM::Dual(ref content, ..)) = left
    && let XM::Apply(_, ref args, ..) = **content
  {
    for arg in &args.0 {
      if arg.as_ref().is_some_and(has_absent_relop_operand) {
        return Err("formulae_apply: formulae contains fragment with absent".into());
      }
    }
  }
  // Period separator always creates formulae (it's a hard formula boundary).
  // Comma separator requires at least one relational item.
  // The separator's value read through an XMRef, as a gathered/split row's content branch holds it.
  let sep_is_period = realized_value(&sep, &ctxt).is_ok_and(|v| v == ".");
  if !left_rel && !right_rel && !sep_is_period {
    return Err("formulae_apply: no relational items, use list_apply instead".into());
  }

  // Reject when right is non-relational and left is relational, BUT only for
  // comma separators. Period is a hard formula boundary and should NOT trigger
  // list grouping. This forces `a=b, c` via `formula relop formula_list`
  // (producing `a = list(b,c)`) but allows `a=b. c` to produce `formulae(a=b, c)`.
  let sep_is_period = match &sep {
    XM::Lexeme(lex, _) => lex.starts_with("PERIOD:"),
    XM::Token(props, _) => props.role.as_deref() == Some("PERIOD"),
    _ => false,
  };
  if left_rel && !right_rel && !sep_is_period {
    return Err(
      "formulae_apply: non-relational right after relational left — a list (formulae_then_item_apply)".into(),
    );
  }

  let meaning = "formulae"; // always

  // If left is already a formulae Dual, extend it — BUT only when its
  // presentation holds its items alone (`presents_its_items_alone`: the flat-list `XMWrap`
  // without delimiters of its own, which a fenced list has, 57ay). A
  // `distribute_list_relation` dual ALSO has `meaning="formulae"` content yet a
  // RELATION-`Apply` presentation (`(a,b)=c`), not a Wrap; extending that here
  // would push a content ref but silently skip the presentation update (the
  // `if let XM::Wrap` below fails), stranding the ref as a keyless bare `<XMRef/>`
  // → the dominant `expected:id` "Missing idref" cluster (370 papers; witness
  // `0704.2334`, `a,\quad b=c,\quad d=e`). Requiring a Wrap presentation makes
  // such a left fall through to `list_or_formulae_create`, which builds a fresh
  // dual whose content refs BOTH resolve. See EXPECTED_ID_XMREF_DESIGN 2026-06-26v.
  if let Some(XM::Dual(ref mut content, ref mut pres, ..)) = left
    && let XM::Apply(ref op, ref mut op_args, ..) = **content
    && presents_its_items_alone(pres, op_args.0.len())
    && let XM::Token(ref props, _) = *op.0
    && props.meaning.as_deref() == Some(meaning)
  {
    let new_ref = create_xmrefs(&mut [&mut right], ctxt)?;
    op_args.0.extend(new_ref.into_iter().map(Some));
    if let XM::Wrap(ref mut items, ..) = **pres {
      items.push(sep);
      items.push(right);
    }
    return Ok(left);
  }
  // Comma-list LEFT of a relation: `a,b \in A` / `x,y \le z`. The grammar reaches
  // here with `left` a bare (non-relational) item and `right` a binary RELOP
  // relation `Apply(∈,[b,A])`. The plain `formulae@(a, b∈A)` is semantically wrong
  // (∈ binds only `b`, splitting `a` off). Build the user-specified XMDual
  // (2026-06-22): content DISTRIBUTES the relation over the list, presentation
  // wraps the list as the relation's LHS. (List-RIGHT `0<x,y` is the separate
  // `formula relop formula_list` path → `list(0<x,y)`, untouched.)
  // Not over a factor-level conditional, whose condition binds the last item only (`a,b\mid c\sim d`,
  // `\Delta;\Gamma\mid\Phi\vdash P`), nor from a text label beside math (`\text{Poisson:}\quad
  // y\mid\lambda\sim…`, `\text{where}\quad G=W\times A`, 2605.01549; 57cc review): an item holding text
  // distributes only over a relation whose left operand holds text too, its kin (`\textbf{A},\textbf{B}
  // \in\mathbb{R}^n` 2605.12082, `\mbox{det}\,G,\mbox{det}\,G_h\geq c_0`; 57cc's A/B).
  let kin = |l: &XM| {
    !holds_text(l, &ctxt)
      || matches!(&right, XM::Apply(_, Args(a), ..)
        if a.first().and_then(Option::as_ref).is_some_and(|lhs| holds_text(lhs, &ctxt)))
  };
  if !left_rel
    && !sep_is_period
    && left
      .as_ref()
      .is_some_and(|l| !matches!(l, XM::Dual(..)) && kin(l))
    && matches!(&right, XM::Apply(op, Args(a), ..) if a.len() == 2 && op_is_relop(op))
    && !relates_a_conditional(&right)
  {
    return Ok(Some(distribute_list_relation(
      left.unwrap(),
      sep,
      right,
      ctxt,
    )?));
  }
  list_or_formulae_create(left.unwrap(), sep, right, meaning, ctxt)
}

/// True iff `op` is a binary RELOP operator (`∈`, `≤`, `=`, …) — NOT a
/// `multirelation` chain. Used to gate `distribute_list_relation`.
fn op_is_relop(op: &Operator) -> bool {
  !is_multirelation(&op.0) && operator_category(&op.0).is_some_and(|r| r.contains("RELOP"))
}

/// `a,b \in A` → the user-specified XMDual (2026-06-22, surpass-Perl): the content
/// branch DISTRIBUTES the relation over each list element —
/// `formulae@(∈(a,A), ∈(b,A))`, sharing XMRefs to the relop (∈) and RHS (A) — while
/// the presentation branch wraps the list `a,b` in an `<XMWrap>` as the relation's
/// LHS — `Apply(∈, XMWrap(a,',',b), A)`. `right` MUST be a binary RELOP relation
/// (caller guarantees via `op_is_relop`).
fn distribute_list_relation(
  mut left: XM,
  sep: XM,
  right: XM,
  ctxt: ActionContext,
) -> Result<XM, Box<dyn Error>> {
  let XM::Apply(Operator(r_op_box), Args(mut r_args), ..) = right else {
    unreachable!("distribute_list_relation: caller guarantees a binary relation")
  };
  let mut rhs = r_args.pop().flatten().unwrap();
  let mut lhs = r_args.pop().flatten().unwrap();
  let mut r_op = *r_op_box;
  // Ref the four presentation nodes (left, lhs, rhs, relop); rhs and relop are
  // shared across the two distributed content relations.
  let refs = create_xmrefs(&mut [&mut left, &mut lhs, &mut rhs, &mut r_op], ctxt)?;
  let mut it = refs.into_iter();
  let ref_left = it.next().unwrap();
  let ref_lhs = it.next().unwrap();
  let ref_rhs = it.next().unwrap();
  let ref_rop = it.next().unwrap();
  let mk_rel = |op: XM, l: XM, r: XM| {
    XM::Apply(
      Operator(Box::new(op)),
      Args(vec![Some(l), Some(r)]),
      XProps::default(),
      Meta::default(),
    )
  };
  // content: formulae@(∈(a,A), ∈(b,A))
  let rel1 = mk_rel(ref_rop.clone(), ref_left, ref_rhs.clone());
  let rel2 = mk_rel(ref_rop, ref_lhs, ref_rhs);
  let formulae_op: XM = XProps {
    meaning: Some(Cow::Borrowed("formulae")),
    ..XProps::default()
  }
  .into();
  let content = XM::Apply(
    Operator(Box::new(formulae_op)),
    Args(vec![Some(rel1), Some(rel2)]),
    XProps::default(),
    Meta::default(),
  );
  // presentation: ∈(XMWrap(a,',',b), A)
  let pres_wrap = XM::Wrap(vec![left, sep, lhs], XProps::default(), Meta::default());
  let pres = XM::Apply(
    Operator(Box::new(r_op)),
    Args(vec![Some(pres_wrap), Some(rhs)]),
    XProps::default(),
    Meta::default(),
  );
  Ok(XM::Dual(
    Box::new(content),
    Box::new(pres),
    XProps::default(),
    Meta::default(),
  ))
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
      Args(ref_args.into_iter().map(Some).collect()),
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
    XM::Apply(_, args, ..) => {
      for arg in args.0.iter_mut().flatten() {
        restructure_formulae_right(arg)?;
      }
    },
    XM::Wrap(items, ..) => {
      for item in items.iter_mut() {
        restructure_formulae_right(item)?;
      }
    },
    XM::Dual(content, pres, ..) => {
      restructure_formulae_right(content)?;
      restructure_formulae_right(pres)?;
      // NOTE: `\quad`-separated `formulae@(…)` are kept FLAT. We used to
      // right-nest them here (`formulae@(f1, formulae@(f2, …))`) to mirror
      // Perl's `moreRHS`/`maybeColRHS`, but a flat `formulae@(f1, f2, …)` is the
      // correct shape — an intentional, user-directed divergence from Perl (see
      // docs/parity/KNOWN_PERL_ERRORS.md).
    },
    XM::Choices(choices) => {
      for choice in choices.iter_mut() {
        restructure_formulae_right(choice)?;
      }
    },
    XM::Arg(items) => {
      for item in items.iter_mut() {
        restructure_formulae_right(item)?;
      }
    },
    _ => {},
  }
  Ok(())
}

/// The names of the ellipsis tokens: `\ldots` (and `\hdots`, `...`), `\dots` and amsmath's
/// `\dotsc`/`\dotsb`/`\dotsm`/`\dotsi`/`\dotso`, `\cdots`.
const ELLIPSIS_NAMES: [&str; 8] = [
  "ldots", "dots", "dotsc", "dotsb", "dotsm", "dotsi", "dotso", "cdots",
];

/// An ellipsis item: an ELIDEOP (`\cdots`), or a token named as an ellipsis (`\ldots` and `\dots`
/// are IDs, math_common.rs), read through an XMRef.
pub(crate) fn is_ellipsis(xm: &XM, ctxt: &ActionContext) -> bool {
  if operator_category(xm) == Some("ELIDEOP") {
    return true;
  }
  match xm {
    XM::Token(props, _) => props
      .name
      .as_deref()
      .is_some_and(|name| ELLIPSIS_NAMES.contains(&name)),
    XM::Lexeme(lex, _) => lookup_lex_node(lex, ctxt.nodes).ok().is_some_and(|node| {
      let node = crate::data::resolve_xmref(node).unwrap_or_else(|| node.clone());
      node.get_name() == "XMTok"
        && (node.get_attribute("role").as_deref() == Some("ELIDEOP")
          || node
            .get_attribute("name")
            .is_some_and(|name| ELLIPSIS_NAMES.contains(&name.as_str())))
    }),
    _ => false,
  }
}

/// A separator that ends a segment of a comma container: `\quad`-spaced punctuation (a `\quad`
/// hint, or a comma the lexer read as WIDE_PUNCT for its `rpadding`), `;`, a period.
fn ends_a_segment(sep: &XM, ctxt: &ActionContext) -> bool {
  is_quad_separator(sep)
    || matches!(operator_category(sep), Some("WIDE_PUNCT" | "PERIOD"))
    || realized_value(sep, ctxt).is_ok_and(|value| value == ";" || value == ".")
}

/// A missing comma beside an ellipsis (user ruling 2026-09-29: split it off; Perl, and the parse,
/// read `\{l_1,\cdots l_k\}` as `set@(l _ 1, cdots * l _ k)`): in a comma container, an item whose
/// first-argument spine — through unfenced invisible products, sums and relations — reaches an
/// invisible product that starts with an ellipsis gives the ellipsis up as an item of its own before
/// it (`set@(l _ 1, cdots, l _ k)`, `k=0,1,\cdots L-1` `list@(k = 0, 1, cdots, L - 1)`); an item
/// ending in one, before a comma, gives it up after it (`\{30,40\ldots,80\}`). The item as it would
/// read with the comma typed, an invisible comma (U+2063) between — the separator Perl supplies for
/// a missing one (`InvisibleComma`, MathParser.pm:1182-1183) — so `attach_enumerations`, which runs
/// next, reads it as it reads the comma'd list. A post-pass like it: a grammar rule would need an
/// ellipsis-item twin of every list action. Not an ellipsis before a visible operator (`x_1,\cdots
/// +x_n` elides a sum), nor a decimal's trailing dots in a relation (`x=0.325\ldots`).
/// Witnesses 2605.12837, 2605.12555, 2605.02211, 2605.00390, 2605.25695.
pub fn separate_ellipsis_items(
  xm: &mut XM,
  ctxt: &mut ActionContext,
) -> Result<(), Box<dyn Error>> {
  match xm {
    XM::Apply(op, args, ..) => {
      separate_ellipsis_items(&mut op.0, ctxt)?;
      for arg in args.0.iter_mut().flatten() {
        separate_ellipsis_items(arg, ctxt)?;
      }
    },
    XM::Dual(content, pres, ..) => {
      separate_ellipsis_items(content, ctxt)?;
      separate_ellipsis_items(pres, ctxt)?;
    },
    XM::Wrap(items, ..) | XM::Arg(items) | XM::Choices(items) => {
      for item in items.iter_mut() {
        separate_ellipsis_items(item, ctxt)?;
      }
    },
    _ => {},
  }
  separate_in_container(xm, ctxt)
}

/// The meanings of a comma container the split reads: a bare list or formulae, or a Fence-named one
/// (`\{l_1,\cdots l_k\}` is `set@` already, `(a,\cdots b)` a pair).
const COMMA_CONTAINERS: [&str; 8] = [
  "list",
  "formulae",
  "set",
  "vector",
  "open-interval",
  "closed-interval",
  "open-closed-interval",
  "closed-open-interval",
];

fn separate_in_container(xm: &mut XM, ctxt: &mut ActionContext) -> Result<(), Box<dyn Error>> {
  let XM::Dual(content, pres, _, dual_meta) = xm else {
    return Ok(());
  };
  let XM::Apply(op, refs, ..) = &mut **content else {
    return Ok(());
  };
  match (&mut *op.0, &mut **pres) {
    // A comma container: `list@(…)` and its kin, presented as its items.
    (XM::Token(op_props, _), XM::Wrap(wrapped, ..))
      if op_props
        .meaning
        .as_deref()
        .is_some_and(|m| COMMA_CONTAINERS.contains(&m)) =>
    {
      if separate_items(&mut refs.0, wrapped, ctxt)? {
        match op_props.meaning.as_deref() {
          // A comma formulae that gains a plain ellipsis item is named once the enumerations are
          // attached (`name_an_elided_formulae`), as its comma'd twin reads.
          Some("formulae") => dual_meta.elided_formulae = true,
          // A pair that became three items is named as Perl's Fence names three (`encloseN`,
          // MathParser.pm:1368-1377): between parentheses a vector, else a list.
          Some(
            "vector"
            | "open-interval"
            | "closed-interval"
            | "open-closed-interval"
            | "closed-open-interval",
          ) => {
            let parens = matches!((wrapped.first(), wrapped.last()), (Some(open), Some(close))
              if realized_value(open, ctxt).is_ok_and(|v| v == "(") && realized_value(close, ctxt).is_ok_and(|v| v == ")"));
            op_props.meaning = Some(Cow::Borrowed(if parens { "vector" } else { "list" }));
          },
          _ => {},
        }
      }
    },
    // A head applied to its arguments between delimiters (`\max(a_1,\ldots a_n)`): the content
    // applies the head's ref to the argument refs, the presentation the head to the fenced wrap.
    (XM::Ref(_), XM::Apply(_, pres_args, ..)) => {
      if let [Some(XM::Wrap(wrapped, ..))] = pres_args.0.as_mut_slice()
        && wrapped.len() == 2 * refs.0.len() + 1
      {
        separate_items(&mut refs.0, wrapped, ctxt)?;
      }
    },
    _ => {},
  }
  Ok(())
}

/// Split the ellipses off the items of one comma container: its item refs and its presentation, the
/// items with their separators, bare or between delimiters. Whether any item gave one up.
fn separate_items(
  refs: &mut Vec<Option<XM>>,
  wrapped: &mut Vec<XM>,
  ctxt: &mut ActionContext,
) -> Result<bool, Box<dyn Error>> {
  let n = refs.len();
  if n < 2 || refs.iter().any(Option::is_none) {
    return Ok(false);
  }
  let fenced = wrapped.len() == 2 * n + 1;
  if !fenced && wrapped.len() != 2 * n - 1 {
    return Ok(false);
  }
  let offset = usize::from(fenced);
  let is_comma =
    |sep: &XM, ctxt: &ActionContext| realized_value(sep, ctxt).is_ok_and(|value| value == ",");
  // Item k, with the ellipses it gives up before and after it.
  let mut new_wrapped: Vec<XM> = Vec::with_capacity(wrapped.len() + 4);
  let mut new_refs: Vec<Option<XM>> = Vec::with_capacity(n + 2);
  let old = std::mem::take(wrapped);
  let mut old_refs = std::mem::take(refs);
  let mut changed = false;
  let mut items: Vec<Option<XM>> = Vec::with_capacity(n);
  let mut separators: Vec<XM> = Vec::with_capacity(n);
  let (mut open, mut close) = (None, None);
  for (i, node) in old.into_iter().enumerate() {
    if fenced && i == 0 {
      open = Some(node);
    } else if fenced && i == 2 * n {
      close = Some(node);
    } else if (i - offset) % 2 == 0 {
      items.push(Some(node));
    } else {
      separators.push(node);
    }
  }
  let comma_before: Vec<bool> = (0..n)
    .map(|k| k > 0 && is_comma(&separators[k - 1], ctxt))
    .collect();
  let comma_after: Vec<bool> = (0..n)
    .map(|k| k + 1 < n && is_comma(&separators[k], ctxt))
    .collect();
  if let Some(open) = open {
    new_wrapped.push(open);
  }
  for k in 0..n {
    if k > 0 {
      new_wrapped.push(separators[k - 1].clone());
    }
    let mut item = items[k]
      .take()
      .ok_or("separate_ellipsis_items: item taken twice")?;
    let mut item_ref = old_refs[k].take();
    let leading = if comma_before[k] || (k == 0 && comma_after[k]) {
      take_edge_ellipsis(&mut item, true, true, ctxt)
    } else {
      None
    };
    let trailing = if comma_after[k] || (k + 1 == n && comma_before[k]) {
      take_edge_ellipsis(&mut item, false, true, ctxt)
    } else {
      None
    };
    if leading.is_some() || trailing.is_some() {
      changed = true;
      // The item may have collapsed to one factor: point its ref at what stands now (the item's own
      // key or id where it kept one).
      item_ref = create_xmrefs(&mut [&mut item], ActionContext {
        nodes:    ctxt.nodes,
        document: &mut *ctxt.document,
      })?
      .into_iter()
      .next();
    }
    if let Some(mut ellipsis) = leading {
      let ellipsis_ref = create_xmrefs(&mut [&mut ellipsis], ActionContext {
        nodes:    ctxt.nodes,
        document: &mut *ctxt.document,
      })?;
      new_wrapped.push(ellipsis);
      new_wrapped.push(XM::Token(invisible_comma(), Meta::default()));
      new_refs.extend(ellipsis_ref.into_iter().map(Some));
    }
    new_wrapped.push(item);
    new_refs.push(item_ref);
    if let Some(mut ellipsis) = trailing {
      let ellipsis_ref = create_xmrefs(&mut [&mut ellipsis], ActionContext {
        nodes:    ctxt.nodes,
        document: &mut *ctxt.document,
      })?;
      new_wrapped.push(XM::Token(invisible_comma(), Meta::default()));
      new_wrapped.push(ellipsis);
      new_refs.extend(ellipsis_ref.into_iter().map(Some));
    }
  }
  if let Some(close) = close {
    new_wrapped.push(close);
  }
  *wrapped = new_wrapped;
  *refs = new_refs;
  Ok(changed)
}

/// Take the ellipsis at the leading (or trailing) edge of `xm`'s spine: through unfenced invisible
/// products, sums and relations, to an invisible product of two or more factors whose edge factor is
/// an ellipsis. Stops at anything else — a fence, a Dual, a script, a node with its own key below the
/// root (another ref points at it). A product left with one factor is that factor.
fn take_edge_ellipsis(xm: &mut XM, leading: bool, root: bool, ctxt: &ActionContext) -> Option<XM> {
  let XM::Apply(Operator(op), Args(args), props, meta) = xm else {
    return None;
  };
  if meta.fenced.is_some() || (!root && (props.xmkey.is_some() || props.id.is_some())) {
    return None;
  }
  let edge = if leading {
    args.iter().position(Option::is_some)?
  } else {
    args.iter().rposition(Option::is_some)?
  };
  let invisible_product = matches!(&**op, XM::Token(p, _) if p.content.as_deref() == Some("\u{2062}"))
    || is_invisible_times_lexeme(op);
  if invisible_product && args.iter().flatten().count() >= 2 {
    let is_edge_ellipsis = args[edge]
      .as_ref()
      .is_some_and(|factor| is_ellipsis(factor, ctxt));
    if !is_edge_ellipsis {
      return take_edge_ellipsis(args[edge].as_mut()?, leading, false, ctxt);
    }
    // A trailing ellipsis leaves one factor only (`40\ldots,80`): after a longer product it elides the
    // product (`p_1p_2\cdots, q`), and after a decimal it elides digits (`x=0.325\ldots`).
    if !leading && (args.iter().flatten().count() != 2 || ends_in_a_decimal(args, edge, ctxt)) {
      return None;
    }
    let ellipsis = args.remove(edge)?;
    if args.iter().flatten().count() == 1 {
      // The one factor left is the item; the caller makes its ref anew (the root's key is orphaned,
      // which `resolve_xmkeys` drops).
      *xm = args.iter_mut().find_map(Option::take)?;
    }
    return Some(ellipsis);
  }
  if matches!(operator_category(op), Some("ADDOP" | "BINOP")) || is_relational_op(op) {
    return take_edge_ellipsis(args[edge].as_mut()?, leading, false, ctxt);
  }
  None
}

/// `x=0.325\ldots`, `x, 0.3\ldots`: the factor before a trailing ellipsis is a decimal — digit
/// elision, not a missing comma.
fn ends_in_a_decimal(args: &[Option<XM>], edge: usize, ctxt: &ActionContext) -> bool {
  edge > 0
    && args[..edge]
      .iter()
      .rev()
      .flatten()
      .next()
      .is_some_and(|before| {
        operator_category(before) == Some("NUMBER")
          && realized_value(before, ctxt).is_ok_and(|v| v.contains('.'))
      })
}

fn is_invisible_times_lexeme(op: &XM) -> bool {
  matches!(op, XM::Lexeme(lex, _) if lex.starts_with("MULOP:") && lex.contains('\u{2062}'))
}

/// The #37 ellipsis exception (user ruling 2026-09-29): an enumeration attaches to its relation. A
/// run of plain comma items holding an ellipsis is one operand, as Perl reads it (`maybeColRHS`,
/// `maybeRHS`, MathGrammar:146-170): the right operand of the relation before it when the run
/// reaches the end of its segment or ends in an item (`i=1,\ldots,n` `i = list@(1, ldots, n)`,
/// `i=1,\ldots,n, j=1,\ldots,m`), the left operand of the relation after it when it opens its
/// segment (`x_1,\ldots,x_n\in X` `list@(x _ 1, ldots, x _ n) element-of X`); a run ending in its
/// ellipsis between two relations stays (relation elision, `a_1=0,\ldots,a_n=0`; a bridging chain,
/// `G=G_0,G_1,\dots,G_L=G'`). Segments end at `\quad`, `;` and a period
/// (`\beta\in\mathbb{Q},\qquad\lambda_1,\ldots,\lambda_L\in\mathbb{C}`). A post-pass on the chosen
/// parse, like `rename_fenced_lists`: no parse trees are added (a grammar rule would need the whole
/// segment for its choice, and #37's over-parse would come back). The run items keep their keys —
/// their refs move from the container to the enumeration; a container left with one relation is
/// that relation, which takes the container's key. Plain lists keep #37 (`a=b,c,d`, `i=1,2`).
/// Where the run is plainly not the relation's operand it stays an item (57bv.1, the guards in
/// `attach_in_container`): a tuple component's equation between delimiters, a left operand past a
/// text, a run after a scripted member (`P_0=I,P_1,\dots`), bridging to one, or after a relation that
/// closes an elided run of relations, and "and so on" before a repeated statement.
/// Witnesses 2605.12085, 2605.00329 (left), 2605.00515 (set-builder), 2605.21039 (43 formulas).
pub fn attach_enumerations(xm: &mut XM, ctxt: &mut ActionContext) -> Result<(), Box<dyn Error>> {
  attach_enumerations_in(xm, ctxt, false)
}

/// `attach_enumerations` on `xm`, which stands between delimiters of its own when `delimited` (the
/// middle of a fence's `Wrap(open, xm, close)`, `fenced`).
fn attach_enumerations_in(
  xm: &mut XM,
  ctxt: &mut ActionContext,
  delimited: bool,
) -> Result<(), Box<dyn Error>> {
  match xm {
    XM::Apply(op, args, ..) => {
      attach_enumerations_in(&mut op.0, ctxt, false)?;
      for arg in args.0.iter_mut().flatten() {
        attach_enumerations_in(arg, ctxt, false)?;
      }
    },
    XM::Dual(content, pres, ..) => {
      attach_enumerations_in(content, ctxt, false)?;
      attach_enumerations_in(pres, ctxt, false)?;
    },
    XM::Wrap(items, ..) => {
      let fence = items.len() == 3
        && operator_category(&items[0]) == Some("OPEN")
        && operator_category(&items[2]) == Some("CLOSE");
      for (k, item) in items.iter_mut().enumerate() {
        attach_enumerations_in(item, ctxt, fence && k == 1)?;
      }
    },
    XM::Arg(items) | XM::Choices(items) => {
      for item in items.iter_mut() {
        attach_enumerations_in(item, ctxt, false)?;
      }
    },
    _ => {},
  }
  if let Some(relation) = attach_in_container(xm, ctxt, delimited)? {
    *xm = relation;
  } else {
    name_an_elided_formulae(xm, ctxt);
  }
  Ok(())
}

/// A comma formulae that `separate_ellipsis_items` gave a plain ellipsis item (`Meta::elided_formulae`)
/// and that still holds one, once the enumerations are attached, is a list, as its comma'd twin reads
/// (`x_{1}=0,\dots x_{n}=0` as `a_1=0,\ldots,a_n=0`); one whose ellipsis a relation took stays
/// formulae (`\{\epsilon_i, i=1, 2,\ldots n\}` as its twin set@(formulae@(ε_i, i = list@(1, 2, ldots,
/// n))); 57bv A/B, 2605.00514). A formulae the grammar built keeps its name.
fn name_an_elided_formulae(xm: &mut XM, ctxt: &ActionContext) {
  if let XM::Dual(content, pres, _, meta) = xm
    && meta.elided_formulae
    && let XM::Apply(op, ..) = &mut **content
    && let XM::Token(op_props, _) = &mut *op.0
    && op_props.meaning.as_deref() == Some("formulae")
    && let XM::Wrap(wrapped, ..) = &**pres
    && wrapped.iter().any(|item| is_ellipsis(item, ctxt))
  {
    op_props.meaning = Some(Cow::Borrowed("list"));
  }
}

/// One block of a rebuilt container: an item kept as it stood, or a relation with the enumeration
/// runs it takes (item indices), left and right.
struct EnumerationBlock {
  item:  usize,
  left:  Vec<usize>,
  right: Vec<usize>,
}

fn attach_in_container(
  xm: &mut XM,
  ctxt: &mut ActionContext,
  delimited: bool,
) -> Result<Option<XM>, Box<dyn Error>> {
  let XM::Dual(content, pres, dual_props, _) = xm else {
    return Ok(None);
  };
  let XM::Apply(op, refs, ..) = &mut **content else {
    return Ok(None);
  };
  let XM::Token(op_props, _) = &mut *op.0 else {
    return Ok(None);
  };
  if !matches!(
    op_props.meaning.as_deref(),
    Some("list" | "formulae" | "fragments")
  ) {
    return Ok(None);
  }
  let XM::Wrap(wrapped, ..) = &mut **pres else {
    return Ok(None);
  };
  let n = refs.0.len();
  if n < 2 || refs.0.iter().any(Option::is_none) {
    return Ok(None);
  }
  let fenced = wrapped.len() == 2 * n + 1;
  if !fenced && wrapped.len() != 2 * n - 1 {
    return Ok(None);
  }
  let offset = usize::from(fenced);
  let item = |k: usize| &wrapped[offset + 2 * k];
  let separator = |k: usize| &wrapped[offset + 2 * k + 1];
  let relational: Vec<bool> = (0..n)
    .map(|k| {
      matches!(item(k), XM::Apply(_, args, ..) if args.0.len() >= 2) && is_relational_item(item(k))
    })
    .collect();
  let ellipsis: Vec<bool> = (0..n).map(|k| is_ellipsis(item(k), ctxt)).collect();
  if !ellipsis.iter().any(|e| *e) || !relational.iter().any(|r| *r) {
    return Ok(None);
  }
  // A run: maximal plain items holding an ellipsis, within a segment.
  let mut owner: Vec<Option<(usize, bool)>> = vec![None; n]; // (relation, attached on its right)
  // A relation whose left run is refused (`takes_left`): a tuple component, which takes no run on its
  // right either (`(0,\ldots,0,k_\ell=k,0,\ldots,0)`, 2605.09683).
  let mut stranded = vec![false; n];
  // A plain run (no ellipsis) beside an attached enumeration: `i=1,\ldots,4,\,j=0,1,2` and
  // `d=0,1,\,k=1,\ldots,K` (2605.18167) attach it too (user ruling 2026-09-29, both directions;
  // `plain_runs` below, divergence #378), as Perl's `maybeColRHS` attaches every run (MathGrammar:165-172).
  let mut ranged = vec![false; n];
  let mut plain_runs: Vec<(usize, usize, usize)> = Vec::new(); // (first, last, relation)
  let mut start = 0;
  while start < n {
    let mut end = start;
    while end + 1 < n && !ends_a_segment(separator(end), ctxt) {
      end += 1;
    }
    let mut k = start;
    while k <= end {
      if relational[k] {
        k += 1;
        continue;
      }
      let first = k;
      while k <= end && !relational[k] {
        k += 1;
      }
      let last = k - 1;
      let plain = !(first..=last).any(|j| ellipsis[j]);
      let before = (first > start && relational[first - 1]).then(|| first - 1);
      let after = (last < end && relational[last + 1]).then_some(last + 1);
      // A lone ellipsis after a chain of relations is "and so on" (`x=0, y=1, \ldots`), not the last
      // relation's value (57bv review); after one relation it is (`i=1,\ldots`).
      // So is one before a break whose next relation repeats the last one's left operand — the
      // statements go on (`f(v_1)=f(v_2),\dots,\quad f(v_{k-1})=f(v_k)`, 2605.00553;
      // `p\leftarrow…,\ \cdots,\quad p\leftarrow…`, 2605.09708).
      let repeats_its_statement = |relation: usize| {
        end + 1 < n
          && relational[end + 1]
          && same_progression(
            relation_operands(item(relation)).0,
            relation_operands(item(end + 1)).0,
            ctxt,
          )
      };
      let lone_and_so_on = first == last
        && ellipsis[first]
        && before.is_some_and(|relation| {
          relation > start && relational[relation - 1] || repeats_its_statement(relation)
        });
      let run: Vec<&XM> = (first..=last).filter(|&j| !ellipsis[j]).map(item).collect();
      // A relation takes the run on its right when it opens it: not when it closes an elided run of
      // relations (`A_1\lhd B_1,\ldots,A_n\lhd B_n,C_1,\ldots`, 2605.14476), nor when its left operand
      // is a scripted member of the run — its letter a scripted run item's — and its right is none
      // (`P_0=I,P_1,\dots,P_n`, 2605.23874; `i=1,\ldots,i_{\max}`, `l''=l,\ldots,l'`, `T_{train}\subseteq
      // 1,\ldots,T` attach, 2605.24906), nor after a stranded run.
      let takes_right = |relation: usize| {
        let (left, right) = relation_operands(item(relation));
        let closes_an_elided_run =
          relation >= start + 2 && ellipsis[relation - 1] && relational[relation - 2];
        let scripted_run: Vec<&XM> = run
          .iter()
          .copied()
          .filter(|item| script_base(item).is_some())
          .collect();
        let member_left = left.is_some_and(|left| script_base(left).is_some())
          && shares_progression(left, &scripted_run, ctxt)
          && !shares_progression(right, &run, ctxt);
        !closes_an_elided_run && !member_left && !stranded[relation]
      };
      // A relation takes the run on its left (Perl's `maybeRHS`, MathGrammar:146-150), except past a
      // text (`\theta_1,\ldots,\theta_k\text{ s.t. gaps }\geq\delta`, 2605.23087) and except a tuple
      // component's equation: between delimiters, an `=` whose left operand does not continue the run
      // or whose right is one of it (`(j_1,\ldots,j_L,j_{L+1}=j_1)`, `\{v_1,\ldots,u=\{v_0,v_k\}\}`,
      // `(0,\ldots,0,k_\ell=k,0,\ldots,0)`; 2605.18633, 2605.24348, 2605.09683).
      let takes_left = |relation: usize| {
        let (left, right) = relation_operands(item(relation));
        let component = (fenced || delimited)
          && matches!(item(relation), XM::Apply(Operator(op), ..)
            if realized_meaning(op, ctxt).as_deref() == Some("equals"))
          && (!continues_progression(left, &run, ctxt) || shares_progression(right, &run, ctxt));
        !left.is_some_and(|left| holds_text(left, ctxt)) && !component
      };
      // A run between two relations is the first one's value unless it bridges a chain to the next,
      // whose left operand is a scripted member (`a=a_0,a_1,\dots,a_{n-1},a_n=b` as
      // `G=G_0,G_1,\dots,G_L=G'`; 2605.24348, 2605.17185; `i=1,\ldots,n,\ n\geq 2` attaches).
      let bridges = |next: usize| {
        let left = relation_operands(item(next)).0;
        left.is_some_and(|left| script_base(left).is_some()) && shares_progression(left, &run, ctxt)
      };
      // A plain run continues the relation before it in its own segment — each item of its right
      // operand's progression, a number after a number, the same letter under scripts, a plain letter
      // after plain letters — and bridges no chain to the next (`i=1,\ldots,n,\ x=0, y`,
      // `x_1,\dots,x_n=0, y` keep theirs as items); attached after the segment loop, if another
      // relation of the container took an enumeration on its right.
      if plain {
        if let Some(relation) = before
          && takes_right(relation)
          && after.is_none_or(|next| !bridges(next))
          && let (_, Some(value)) = relation_operands(item(relation))
          && (first..=last).all(|j| continues_progression(Some(item(j)), &[value], ctxt))
        {
          plain_runs.push((first, last, relation));
        }
        continue;
      }
      let target = match (before, after) {
        (Some(_), None) if lone_and_so_on => None,
        (Some(relation), None) if takes_right(relation) => Some((relation, true)),
        (Some(relation), Some(next))
          if !ellipsis[last] && !bridges(next) && takes_right(relation) =>
        {
          Some((relation, true))
        },
        (None, Some(relation)) if first == start => {
          if takes_left(relation) {
            Some((relation, false))
          } else {
            stranded[relation] = true;
            None
          }
        },
        _ => None,
      };
      if let Some(target) = target {
        if target.1 {
          ranged[target.0] = true;
        }
        for slot in owner.iter_mut().take(last + 1).skip(first) {
          *slot = Some(target);
        }
      }
    }
    start = end + 1;
  }
  for (first, last, relation) in plain_runs {
    if ranged
      .iter()
      .enumerate()
      .any(|(other, &ranged)| ranged && other != relation)
    {
      for slot in owner.iter_mut().take(last + 1).skip(first) {
        *slot = Some((relation, true));
      }
    }
  }
  if owner.iter().all(Option::is_none) {
    return Ok(None);
  }
  // Blocks in source order: each relation with its runs, each other item alone.
  let mut blocks: Vec<EnumerationBlock> = Vec::new();
  for (k, slot) in owner.iter().enumerate() {
    match *slot {
      Some((relation, right)) => {
        let index = blocks.iter().position(|b| b.item == relation);
        let block = match index {
          Some(i) => &mut blocks[i],
          None => {
            blocks.push(EnumerationBlock {
              item:  relation,
              left:  Vec::new(),
              right: Vec::new(),
            });
            blocks.last_mut().unwrap()
          },
        };
        if right {
          block.right.push(k);
        } else {
          block.left.push(k);
        }
      },
      None if blocks.iter().any(|b| b.item == k) => {},
      None => blocks.push(EnumerationBlock {
        item:  k,
        left:  Vec::new(),
        right: Vec::new(),
      }),
    }
  }
  let block_start = |b: &EnumerationBlock| b.left.first().copied().unwrap_or(b.item);
  // Take the pieces out of the container.
  let mut items: Vec<Option<XM>> = Vec::with_capacity(n);
  let mut separators: Vec<Option<XM>> = Vec::with_capacity(n);
  let old = std::mem::take(wrapped);
  let (open, close) = if fenced {
    (old.first().cloned(), old.last().cloned())
  } else {
    (None, None)
  };
  for (i, node) in old.into_iter().enumerate() {
    if fenced && (i == 0 || i == 2 * n) {
      continue;
    }
    if (i - offset) % 2 == 0 {
      items.push(Some(node));
    } else {
      separators.push(Some(node));
    }
  }
  let mut item_refs: Vec<Option<XM>> = std::mem::take(&mut refs.0);
  let mut new_wrapped: Vec<XM> = Vec::with_capacity(2 * blocks.len() + 1);
  let mut new_refs: Vec<Option<XM>> = Vec::with_capacity(blocks.len());
  if let Some(open) = open {
    new_wrapped.push(open);
  }
  let mut all_relational = true;
  for (b, block) in blocks.iter().enumerate() {
    if b > 0
      && let Some(sep) = separators[block_start(block) - 1].take()
    {
      new_wrapped.push(sep);
    }
    let mut node = items[block.item]
      .take()
      .ok_or("attach_enumerations: item taken twice")?;
    if !block.left.is_empty() || !block.right.is_empty() {
      let XM::Apply(_, args, ..) = &mut node else {
        return Err("attach_enumerations: relation is not an application".into());
      };
      if !block.left.is_empty()
        && let Some(slot) = args.0.iter_mut().find(|a| a.is_some())
      {
        let mut operand = slot.take().ok_or("attach_enumerations: no left operand")?;
        let operand_ref = create_xmrefs(&mut [&mut operand], ActionContext {
          nodes:    ctxt.nodes,
          document: &mut *ctxt.document,
        })?;
        let mut wrap = Vec::new();
        let mut run_refs = Vec::new();
        for &k in &block.left {
          wrap.push(
            items[k]
              .take()
              .ok_or("attach_enumerations: run item taken twice")?,
          );
          wrap.push(
            separators[k]
              .take()
              .ok_or("attach_enumerations: run separator taken twice")?,
          );
          run_refs.push(item_refs[k].take());
        }
        wrap.push(operand);
        run_refs.extend(operand_ref.into_iter().map(Some));
        *slot = Some(enumeration(run_refs, wrap));
      }
      if !block.right.is_empty()
        && let Some(slot) = args.0.iter_mut().rev().find(|a| a.is_some())
      {
        let mut operand = slot.take().ok_or("attach_enumerations: no right operand")?;
        let operand_ref = create_xmrefs(&mut [&mut operand], ActionContext {
          nodes:    ctxt.nodes,
          document: &mut *ctxt.document,
        })?;
        let mut wrap = vec![operand];
        let mut run_refs: Vec<Option<XM>> = operand_ref.into_iter().map(Some).collect();
        let mut previous = block.item;
        for &k in &block.right {
          wrap.push(
            separators[previous]
              .take()
              .ok_or("attach_enumerations: run separator taken twice")?,
          );
          wrap.push(
            items[k]
              .take()
              .ok_or("attach_enumerations: run item taken twice")?,
          );
          run_refs.push(item_refs[k].take());
          previous = k;
        }
        *slot = Some(enumeration(run_refs, wrap));
      }
    } else {
      all_relational &= relational[block.item];
    }
    new_wrapped.push(node);
    new_refs.push(item_refs[block.item].take());
  }
  if let Some(close) = close {
    new_wrapped.push(close);
  }
  if new_refs.len() == 1 && !fenced {
    // One relation left: it is the formula, under the container's key.
    let mut relation = new_wrapped
      .pop()
      .ok_or("attach_enumerations: empty container")?;
    // Its own key was the container's ref to it, now gone: it takes the container's (an outer ref's
    // target), or none, so no orphan id is minted.
    if let XM::Apply(_, _, props, _) = &mut relation {
      props.xmkey = dual_props.xmkey.clone();
      if dual_props.id.is_some() {
        props.id = dual_props.id.clone();
      }
    }
    return Ok(Some(relation));
  }
  *wrapped = new_wrapped;
  refs.0 = new_refs;
  if all_relational && op_props.meaning.as_deref() == Some("list") {
    op_props.meaning = Some(Cow::Borrowed("formulae"));
  }
  Ok(None)
}

/// A comma container of statements a bare condition cannot hold: two relations then an index range —
/// an ellipsis with an item after the last relation (`\theta_i\sim P,\ i=1,\ldots,n`) — or a relation and a
/// `\quad`-spaced separator (`,\quad`). An elided run of relations (`X_n=i,\ldots,X_0=i_0`), "and so on"
/// after them (`X_n=i_n,X_{n-1}=i_{n-1},\ldots`), one relation's range (`\sum_{j|j=1,\ldots,n}`) and a `;`
/// (`y|x=1;\theta`) stay conditions.
fn is_statement_list(xm: &XM, ctxt: &ActionContext) -> bool {
  let XM::Dual(content, pres, ..) = xm else {
    return false;
  };
  let XM::Apply(op, refs, ..) = &**content else {
    return false;
  };
  if !matches!(&*op.0, XM::Token(props, _)
    if matches!(props.meaning.as_deref(), Some("list" | "formulae" | "fragments")))
  {
    return false;
  }
  let XM::Wrap(wrapped, ..) = &**pres else {
    return false;
  };
  let n = refs.0.len();
  let fenced = wrapped.len() == 2 * n + 1;
  if n < 2 || !fenced && wrapped.len() != 2 * n - 1 {
    return false;
  }
  let offset = usize::from(fenced);
  let items: Vec<&XM> = (0..n).map(|k| &wrapped[offset + 2 * k]).collect();
  let relations: Vec<usize> = (0..n)
    .filter(|&k| {
      matches!(items[k], XM::Apply(_, args, ..) if args.0.len() >= 2)
        && is_relational_item(items[k])
    })
    .collect();
  let Some(&last_relation) = relations.last() else {
    return false;
  };
  let tail = &items[last_relation + 1..];
  let index_range = relations.len() >= 2
    && tail.iter().any(|item| is_ellipsis(item, ctxt))
    && tail.iter().any(|item| !is_ellipsis(item, ctxt));
  let crosses_a_segment = (0..n - 1).any(|k| {
    let separator = &wrapped[offset + 2 * k + 1];
    is_quad_separator(separator) || operator_category(separator) == Some("WIDE_PUNCT")
  });
  index_range || crosses_a_segment
}

/// A relation's outer operands: its first and last (`x_n\in X` → `x_n`, `X`).
fn relation_operands(relation: &XM) -> (Option<&XM>, Option<&XM>) {
  match relation {
    XM::Apply(_, Args(args), ..) => (
      args.iter().flatten().next(),
      args.iter().flatten().next_back(),
    ),
    _ => (None, None),
  }
}

/// What makes an item a member of an enumeration's progression: its letter under its scripts
/// (`x_1`, `x^{(n)}`, `l''` → x, x, l), a number, or an applied letter (`f(x_1)` → f).
fn progression_key(xm: &XM, ctxt: &ActionContext) -> Option<String> {
  let nucleus = script_nucleus(xm);
  match nucleus {
    XM::Lexeme(..) | XM::Token(..) => {
      if operator_category(nucleus) == Some("NUMBER") {
        Some("#".to_string())
      } else {
        realized_value(nucleus, ctxt)
          .ok()
          .filter(|v| !v.is_empty())
          .map(|v| v.into_owned())
      }
    },
    XM::Apply(Operator(head), Args(args), ..)
      if args.len() == 1
        && matches!(**head, XM::Lexeme(..) | XM::Token(..))
        && matches!(operator_category(head), Some("UNKNOWN" | "FUNCTION")) =>
    {
      realized_value(head, ctxt).ok().map(|v| format!("@{v}"))
    },
    // a signed number is a number: `j=-1,0,1` (57cc review; `\mathcal J\in\{-1,0,1\}^d` shapes)
    XM::Apply(Operator(head), Args(args), ..)
      if operator_category(head) == Some("ADDOP")
        && matches!(args.as_slice(), [Some(number)] if operator_category(number) == Some("NUMBER")) =>
    {
      Some("#".to_string())
    },
    _ => None,
  }
}

/// Do two operands share a progression key (`f(v_1)`, `f(v_{k-1})`)?
fn same_progression(a: Option<&XM>, b: Option<&XM>, ctxt: &ActionContext) -> bool {
  let key = |xm: Option<&XM>| xm.and_then(|xm| progression_key(xm, ctxt));
  key(a).is_some_and(|a| key(b).as_ref() == Some(&a))
}

/// Does `xm` hold a text (an `XMText` lexeme) outside its scripts? A scripted item is its base
/// (`x_{\text{out}}`, `\underbrace{…}_{\textrm{…}}` hold none; 57bv.1 review, 2605.11818, 2605.01053).
fn holds_text(xm: &XM, ctxt: &ActionContext) -> bool {
  if let Some(base) = script_base(xm) {
    return holds_text(base, ctxt);
  }
  match xm {
    XM::Lexeme(lex, _) => {
      lookup_lex_node(lex, ctxt.nodes).is_ok_and(|node| node.get_name() == "XMText")
    },
    XM::Apply(op, args, ..) => {
      holds_text(&op.0, ctxt) || args.0.iter().flatten().any(|arg| holds_text(arg, ctxt))
    },
    XM::Dual(_, pres, ..) => holds_text(pres, ctxt),
    XM::Wrap(items, ..) | XM::Arg(items) => items.iter().any(|item| holds_text(item, ctxt)),
    _ => false,
  }
}

/// Does `operand` share a progression key with an item of the run?
fn shares_progression(operand: Option<&XM>, run: &[&XM], ctxt: &ActionContext) -> bool {
  operand
    .and_then(|operand| progression_key(operand, ctxt))
    .is_some_and(|key| {
      run
        .iter()
        .any(|item| progression_key(item, ctxt).as_ref() == Some(&key))
    })
}

/// Does `operand` continue the run: of its progression, or a plain letter after plain letters
/// (`x,y,\ldots,z`)?
fn continues_progression(operand: Option<&XM>, run: &[&XM], ctxt: &ActionContext) -> bool {
  let plain_letter = |xm: &XM| {
    matches!(xm, XM::Lexeme(..) | XM::Token(..))
      && operator_category(xm) == Some("UNKNOWN")
      && realized_value(xm, ctxt).is_ok_and(|v| v.chars().count() == 1)
  };
  shares_progression(operand, run, ctxt)
    || operand
      .is_some_and(|operand| plain_letter(operand) && run.iter().any(|item| plain_letter(item)))
}

/// An enumeration: a flat `list@(…)` whose presentation holds the items as written.
fn enumeration(refs: Vec<Option<XM>>, wrapped: Vec<XM>) -> XM {
  let op = XProps {
    meaning: Some(Cow::Borrowed("list")),
    ..XProps::default()
  };
  XM::Dual(
    Box::new(XM::Apply(
      op.into(),
      Args(refs),
      XProps::default(),
      Meta::default(),
    )),
    Box::new(XM::Wrap(wrapped, XProps::default(), Meta::default())),
    XProps::default(),
    Meta::default(),
  )
}

/// Check if an XM node is a \quad-type separator (XMHint PUNCT with name containing "uad").
fn is_quad_separator(xm: &XM) -> bool {
  match xm {
    XM::Token(props, _) => props.name.as_deref().is_some_and(|n| n.contains("uad")),
    XM::Lexeme(lex, _) => lex.split(':').nth(1).is_some_and(|t| t.contains("uad")),
    _ => false,
  }
}

/// Post-processing: name the fenced lists and paren pairs of the chosen parse.
/// Perl's Fence receives flat (open, item, punct, ..., close) and uses encloseN tables.
/// Our grammar builds `list` via `list_apply` before fencing can see the items, so a list Dual whose
/// presentation XMWrap starts with OPEN and ends with CLOSE takes the encloseN meaning here; and a
/// two-item comma pair between parentheses is named by the slot it fills (`paren_pair_meaning`), once,
/// on the parse already chosen, so its name no longer depends on which derivation Marpa enumerated
/// first (`interval_term`'s open-interval, `fenced`'s vector).
pub fn rename_fenced_lists(xm: &mut XM, ctxt: &ActionContext) -> Result<(), Box<dyn Error>> {
  name_fenced_lists(xm, PairSlot::Other, ctxt)
}

/// The slot a paren pair fills (`operand_slots`): one that holds a set (`x\in(0,1)`), a function's
/// argument group (`f(a,b)`, `u(x,\infty)`), or any other.
#[derive(Clone, Copy, PartialEq)]
enum PairSlot {
  Set,
  Argument,
  Other,
}

fn name_fenced_lists(
  xm: &mut XM,
  slot: PairSlot,
  ctxt: &ActionContext,
) -> Result<(), Box<dyn Error>> {
  match xm {
    XM::Apply(op, args, ..) => {
      let slots = operand_slots(&op.0, &args.0, slot, ctxt);
      for (arg, slot) in args.0.iter_mut().zip(slots) {
        if let Some(arg) = arg {
          name_fenced_lists(arg, slot, ctxt)?;
        }
      }
    },
    XM::Wrap(items, ..) => {
      // A single item between parentheses is transparent to a set slot: `\subset((-0.28,0))`
      // (2605.01702); an argument group's inner pair is an item of its own (`\mu((x,\infty))`).
      let transparent = items.len() == 3 && is_paren_wrap(items, ctxt);
      for (i, item) in items.iter_mut().enumerate() {
        let inner = if slot == PairSlot::Set && transparent && i == 1 {
          PairSlot::Set
        } else {
          PairSlot::Other
        };
        name_fenced_lists(item, inner, ctxt)?;
      }
      // Check if this Wrap is exactly [OPEN, list_Dual, CLOSE] — rename list meaning
      // Handles script content like ^{(1+,0+,1-,0-)} where OPEN/CLOSE are siblings
      // of the list Dual in the script's presentation Wrap. Only then: a list among other items
      // is not the delimiters' own — `p(x|y,z)`'s `y,z` is the conditional's condition, not an
      // open interval (57bd; ≤3,964 formulas in 511 papers of the 57bb A/B, 2605.00161
      // `q(x_s\mid x_t,x_0)`; Perl `conditional@(x, list@(y, z))`).
      if items.len() == 3 {
        let first_role = get_xm_role(&items[0]);
        let last_role = get_xm_role(items.last().unwrap());
        if first_role.as_deref() == Some("OPEN") && last_role.as_deref() == Some("CLOSE") {
          let o_val: String = realized_value(&items[0], ctxt)?.into_owned();
          let c_val: String = realized_value(items.last().unwrap(), ctxt)?.into_owned();
          // Rename the list Dual between the delimiters (the wrap's one inner item)
          if let Some(item) = items.get_mut(1) {
            // A bare list only (its presentation `[item, separator, item, …]`): a list that carries
            // its own delimiters is fenced already (`\left([a;b]\right)`: `list@(a, b)`, not the
            // parens' `open-interval`; 57av train A/B, 2605.08815).
            // And a comma list only: Perl's enclose tables are keyed by the first separator and name
            // comma lists alone (MathParser.pm:1368-1377), `x^{(a;b)}` `x ^ (list@(a, b))` (57ay review).
            if let XM::Dual(content, presentation, ..) = item
              && let XM::Apply(ref mut op, ref args, ..) = **content
              && let XM::Token(ref mut props, _) = *op.0
              && props.meaning.as_deref() == Some("list")
              && presents_its_items_alone(presentation, args.0.len())
              && let XM::Wrap(ref separated, ..) = **presentation
              && separated
                .iter()
                .skip(1)
                .step_by(2)
                .map(|separator| realized_value(separator, ctxt))
                .find(|value| !value.as_deref().is_ok_and(|v| v == "\u{2063}"))
                .is_some_and(|value| value.is_ok_and(|v| v == ","))
            {
              let n = args.0.len();
              let new_meaning = match (o_val.as_ref(), c_val.as_ref()) {
                ("(", ")") if n == 2 => {
                  Some(paren_pair_meaning(slot, &separated[0], &separated[2], ctxt))
                },
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
    },
    XM::Dual(content, pres, ..) => {
      if is_paren_pair(content, pres, ctxt) {
        if let XM::Wrap(ref wrapped, ..) = **pres
          && let XM::Apply(ref mut op, ..) = **content
          && let XM::Token(ref mut props, _) = *op.0
        {
          props.meaning = Some(Cow::Borrowed(paren_pair_meaning(
            slot,
            &wrapped[1],
            &wrapped[3],
            ctxt,
          )));
        }
        name_fenced_lists(content, PairSlot::Other, ctxt)?;
        name_fenced_lists(pres, PairSlot::Other, ctxt)?;
      } else {
        name_fenced_lists(content, slot, ctxt)?;
        name_fenced_lists(pres, slot, ctxt)?;
      }
    },
    XM::Choices(trees) => {
      for tree in trees.iter_mut() {
        name_fenced_lists(tree, slot, ctxt)?;
      }
    },
    XM::Arg(items) => {
      for item in items.iter_mut() {
        name_fenced_lists(item, PairSlot::Other, ctxt)?;
      }
    },
    _ => {},
  }
  Ok(())
}

/// A two-item comma pair between parentheses: a Dual applying `vector` or `open-interval` to two items,
/// presented `( a , b )`.
fn is_paren_pair(content: &XM, presentation: &XM, ctxt: &ActionContext) -> bool {
  matches!(content, XM::Apply(Operator(op), Args(args), ..)
    if args.len() == 2
      && matches!(&**op, XM::Token(props, _)
        if matches!(props.meaning.as_deref(), Some("vector" | "open-interval"))))
    && matches!(presentation, XM::Wrap(wrapped, ..)
      if wrapped.len() == 5
        && is_paren_wrap(wrapped, ctxt)
        && realized_value(&wrapped[2], ctxt).is_ok_and(|v| v == ","))
}

/// A two-item comma pair between parentheses is an open interval where the slot holds a set, a vector
/// — a pair, a point — elsewhere (user ruling 2026-09-29, divergence #371; Perl names every one
/// `open-interval`, `%enclose2` MathParser.pm:1369). Outside a set or argument slot an infinite
/// endpoint on its own side makes an interval (`(0,\infty)`, `(-\infty,0]`, `C^1((0,\infty))`,
/// 2605.00581); a function's argument pair stays a vector (`u(x,\infty)`, `\Pi(M^2,\infty)`,
/// 2605.28015) unless it runs from −∞ to ∞ or to a number (`S(-\infty,\infty)`, 57bx), and so does
/// `(\infty,1)` (higher categories, 2605.30648).
fn paren_pair_meaning(
  slot: PairSlot,
  first: &XM,
  second: &XM,
  ctxt: &ActionContext,
) -> &'static str {
  let interval = match slot {
    PairSlot::Set => true,
    // A function's argument pair is a pair (`u(x,\infty)`, `\Pi(M^2,\infty)`), unless it runs from −∞
    // to ∞ or to a single-token number — an interval the function takes (`S(-\infty,\infty)`,
    // `f(-\infty,-3)`, 57bx; 2605.16086, 2605.01702; `u(-\infty,t)`, `f(\pm\infty,0)` stay pairs).
    PairSlot::Argument => {
      infinity_sign(first, ctxt).is_some_and(|sign| matches!(sign, '-' | '\u{2212}'))
        && (infinity_sign(second, ctxt)
          .is_some_and(|sign| matches!(sign, ' ' | '+' | '\u{B1}' | '\u{2213}'))
          || is_signed_number(second))
    },
    PairSlot::Other => {
      infinity_sign(first, ctxt)
        .is_some_and(|sign| matches!(sign, '-' | '\u{2212}' | '\u{B1}' | '\u{2213}'))
        || infinity_sign(second, ctxt)
          .is_some_and(|sign| matches!(sign, ' ' | '+' | '\u{B1}' | '\u{2213}'))
    },
  };
  if interval { "open-interval" } else { "vector" }
}

/// A number, signed or not (`3`, `-3`).
fn is_signed_number(xm: &XM) -> bool {
  match xm {
    XM::Apply(op, args, ..) if args.0.len() == 1 && operator_category(&op.0) == Some("ADDOP") => {
      args.0[0].as_ref().is_some_and(is_signed_number)
    },
    XM::Dual(_, pres, ..) => is_signed_number(pres),
    XM::Lexeme(..) | XM::Token(..) => operator_category(xm) == Some("NUMBER"),
    _ => false,
  }
}

/// The sign of an infinite endpoint: `∞` (`' '`), or `∞` under a sign (`-\infty`, `+\infty`,
/// `\pm\infty`, `\mp\infty`).
fn infinity_sign(xm: &XM, ctxt: &ActionContext) -> Option<char> {
  match xm {
    XM::Apply(op, args, ..) if args.0.len() == 1 && operator_category(&op.0) == Some("ADDOP") => {
      let sign = realized_value(&op.0, ctxt).ok()?.chars().next()?;
      args.0[0]
        .as_ref()
        .and_then(|arg| infinity_sign(arg, ctxt))
        .filter(|inner| *inner == ' ')
        .map(|_| sign)
    },
    XM::Dual(_, pres, ..) => infinity_sign(pres, ctxt),
    XM::Lexeme(..) | XM::Token(..) => realized_value(xm, ctxt)
      .is_ok_and(|v| v == "\u{221E}")
      .then_some(' '),
    _ => None,
  }
}

/// A wrap between a `(` and a `)`, each read through the node it names.
fn is_paren_wrap(items: &[XM], ctxt: &ActionContext) -> bool {
  let (Some(open), Some(close)) = (items.first(), items.last()) else {
    return false;
  };
  realized_value(open, ctxt).is_ok_and(|v| v == "(")
    && realized_value(close, ctxt).is_ok_and(|v| v == ")")
}

/// The slots an application's operands fill: a set — the right of `∈`/`∉` (unless both sides of
/// the `∈` are paren pairs, a componentwise membership: `(x,a)\in(\mathcal X,\mathcal A)`,
/// 2605.06977, 2605.09849; a pair left of a set product keeps the set slot,
/// `(t,x)\in(0,T)\times\Omega`, 2605.25978), the left of `∋`, both of a subset or superset relation; in a multirelation an operand
/// whose neighbouring relation asks for a set on its side; and every operand of a set operator (`×`
/// as written, `∪`, `∩`, `∖`, big or scripted: `\bigcup_n`) or the base of a power, when the
/// application itself fills a set slot (`x\in(0,1)^d`, `t\in(-\delta,\delta)\setminus\{0\}`,
/// 2605.01633); a function's argument group (`f(a,b)`, `\Pi(M^2,\infty)`; a scripted or accented head
/// parses as a product, so `\Pi_1(a,b)` does not reach here); else none.
fn operand_slots(
  op: &XM,
  args: &[Option<XM>],
  slot: PairSlot,
  ctxt: &ActionContext,
) -> Vec<PairSlot> {
  let n = args.len();
  let set_if = |set: bool| if set { PairSlot::Set } else { PairSlot::Other };
  if is_multirelation(op) {
    let sides = |i: usize| {
      args
        .get(i)
        .and_then(Option::as_ref)
        .map_or((false, false), |relation| {
          relation_set_sides(relation, ctxt)
        })
    };
    return (0..n)
      .map(|i| set_if(i % 2 == 0 && ((i > 0 && sides(i - 1).1) || sides(i + 1).0)))
      .collect();
  }
  let (left, right) = relation_set_sides(op, ctxt);
  if (left || right) && n == 2 {
    // A pair right of a pair is componentwise membership, `(x,a)\in(\mathcal X,\mathcal A)`: only that
    // right-hand pair stays a vector; `(t,x)\in(0,T)\times\Omega` keeps its set slot (57br re-review;
    // 2605.25978, 2605.01547 `(r,t)\in(0,5)\times\mathbb R`, 2605.26054).
    let is_pair = |arg: &Option<XM>| match arg {
      Some(XM::Dual(content, pres, ..)) => is_paren_pair(content, pres, ctxt),
      _ => false,
    };
    let componentwise = !left && is_pair(&args[0]) && is_pair(&args[1]);
    return vec![set_if(left), set_if(right && !componentwise)];
  }
  if slot == PairSlot::Set {
    if operator_role(op, ctxt.nodes).as_deref() == Some("SUPERSCRIPTOP") {
      return (0..n).map(|i| set_if(i == 0)).collect();
    }
    let passes_the_slot = match realized_meaning(op, ctxt).as_deref() {
      Some("union" | "intersection" | "set-minus") => true,
      Some("times") => realized_value(op, ctxt).is_ok_and(|v| v == "\u{D7}"),
      _ => false,
    };
    if passes_the_slot {
      return vec![PairSlot::Set; n];
    }
  }
  if matches!(
    operator_category(script_nucleus(op)),
    Some("FUNCTION" | "UNKNOWN" | "ID" | "OPFUNCTION" | "TRIGFUNCTION" | "OPERATOR" | "ATOM")
  ) {
    return vec![PairSlot::Argument; n];
  }
  vec![PairSlot::Other; n]
}

/// Which sides of a relation hold sets: `(left, right)`.
fn relation_set_sides(relation: &XM, ctxt: &ActionContext) -> (bool, bool) {
  let Some(meaning) = realized_meaning(relation, ctxt) else {
    return (false, false);
  };
  if meaning.starts_with("element-of") || meaning.starts_with("not-element-of") {
    (false, true)
  } else if matches!(
    meaning.as_str(),
    "contains" | "not-contains" | "not-contains-nor-equals"
  ) {
    (true, false)
  } else if meaning.contains("subset") || meaning.contains("superset") {
    (true, true)
  } else {
    (false, false)
  }
}

/// Perl `p_getTokenMeaning(realizeXMNode($x))` (MathParser.pm:1090, :135-150): a token's meaning, a
/// lexeme's node's read through an XMRef, as a gathered/split row's content branch holds them
/// (2605.00284); a scripted or decorated operator's, its base's (`\bigcup_n`, `\in_{\mathcal A}`).
fn realized_meaning(xm: &XM, ctxt: &ActionContext) -> Option<String> {
  if let Some(base) = script_base(xm) {
    return realized_meaning(base, ctxt);
  }
  match xm {
    XM::Lexeme(lex, _) => {
      let node = lookup_lex_node(lex, ctxt.nodes).ok()?;
      realize_xmnode(node, ctxt.document).get_attribute("meaning")
    },
    XM::Token(props, _) | XM::Ref(props) => props.meaning.as_deref().map(String::from),
    _ => None,
  }
}

/// Post-processing: combine adjacent SUPOP tokens in script content.
/// Perl MathGrammar L720-723: supops = SUPOP(s) → prime2, prime3, etc.
/// Marpa often parses `\prime\prime` as `list@(prime, prime)` or `times(prime, prime)`
/// instead of using the `supops` grammar rule. This pass detects these patterns
/// and replaces them with combined `prime{N}` tokens.
pub fn combine_supop_post(xm: &mut XM, nodes: &[libxml::tree::Node]) -> Result<(), Box<dyn Error>> {
  match xm {
    XM::Apply(_, args, ..) => {
      for arg in args.0.iter_mut().flatten() {
        combine_supop_post(arg, nodes)?;
      }
    },
    XM::Wrap(items, ..) => {
      for item in items.iter_mut() {
        combine_supop_post(item, nodes)?;
      }
    },
    XM::Dual(content, pres, ..) => {
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
    XM::Choices(trees) => {
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
///
/// Not after a head or its bare application (`reaches_a_following_ellipsis`), whose argument the ellipsis opens or goes
/// on (a trig function's `trig_ellipses`, an OPFUNCTION's or operator's `op_bare_base`, a big operator's operand) or
/// which it multiplies: `a+\cos\cdots` is a + cos@(⋯), `\sin\cdots-\cos\cdots` sin@(⋯) − cos@(⋯), `a+2\log\cdots`,
/// `a+\nabla\cdots` (57cj.12 review), `a+\sum\cdots` a + ∑@(⋯), `a+\sum_n\cdots`, `a+\int\cdots`, `a+\lim\cdots`,
/// `\sin\cdots+\det\cdots`, `a+\partial_x\cdots` (57cj.13 review), `a+\cos\cdots\cdots` a + cos@(⋯·⋯), `a+\det A\cdots`
/// a + det@(A·⋯), `a+\log x\cdots`, `a+\sin x\cdots` a + sin@(x)·⋯ (a trailing ellipsis leaves a trig argument, user ruling
/// 2026-09-29: `\sin x\cdots` sin@(x)·⋯; between two items it stays inside, `trig_elided`), as Perl or its product, where an elided sum read a + cos + ⋯ and showed a `+` the source
/// does not have (latent, no corpus witness in the 3,003 A/B sources). A closed factor keeps the elision reading
/// (`a+b\cdots` a + b + ⋯, `a+\sin(x)\cdots`: SYNC_STATUS "Math-parse residuals of the 57cj train" (9)).
pub fn infix_apply_and_elide(
  rule_id: i32,
  mut args: Vec<Option<XM>>,
  p: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => arg1, infixop, arg2, elision);
  if arg2
    .as_ref()
    .is_some_and(|arg2| reaches_a_following_ellipsis(product_end(arg2, true)))
  {
    return Err(
      "infix_apply_and_elide: the ellipsis belongs to the head before it or its argument".into(),
    );
  }
  // … and one after another continues their run (`ends_an_elided_run`): `a+x\cdots\cdots` is a + x·⋯·⋯
  if arg2
    .as_ref()
    .is_some_and(|arg2| ends_an_elided_run(arg2, &ctxt))
  {
    return Err("infix_apply_and_elide: the ellipsis continues the run before it".into());
  }
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
  // Reject `relop` applied to a comma-list whose items include a
  // relation (single-relop modified_term). For `P(x = 0, y < 0)`
  // both grammar paths reach the action:
  //   * Path A (current): `formula = formula relop formula_list` → `x = list(0, y<0)` (this branch)
  //   * Path B (new): `formula_list = modified_term punct modified_term` → `list(x=0, y<0)` wrapped
  //     by fenced+function-app
  //
  // Path B is the surpass-Perl interpretation (math convention: a
  // comma-list of relations inside a function-arg group). Rejecting
  // Path A here forces Marpa to commit to Path B. ASF migration
  // item 5 (Option A), 2026-05-19.
  fn list_has_relation(xm: &XM) -> bool {
    // Returns true when `xm` is a `list@(...)` (XMApp directly, or
    // XMDual wrapping one) and any presentation item is a relation
    // Apply. We check the presentation wrap (not the content args)
    // because list-Apply args are XMRefs after `create_xmrefs`; the
    // actual relation Applies live in the presentation branch.
    let (content_args, pres_items) = match xm {
      XM::Apply(op, args, ..) => {
        if let XM::Token(ref props, _) = *op.0 {
          if props.meaning.as_deref() == Some("list") {
            (Some(args), None)
          } else {
            (None, None)
          }
        } else {
          (None, None)
        }
      },
      XM::Dual(content, pres, ..) => {
        if let XM::Apply(ref op, ref args, ..) = **content {
          if let XM::Token(ref props, _) = *op.0 {
            if props.meaning.as_deref() == Some("list") {
              // Only a bare list's presentation is its items; a fenced list is one closed item (57ay).
              let pres_items = match **pres {
                XM::Wrap(ref items, ..) if presents_its_items_alone(pres, args.0.len()) => {
                  Some(items)
                },
                _ => None,
              };
              (Some(args), pres_items)
            } else {
              (None, None)
            }
          } else {
            (None, None)
          }
        } else {
          (None, None)
        }
      },
      _ => (None, None),
    };
    if let Some(args) = content_args
      && args
        .0
        .iter()
        .any(|a| a.as_ref().is_some_and(is_relational_item))
    {
      return true;
    }
    if let Some(items) = pres_items
      && items.iter().any(is_relational_item)
    {
      return true;
    }
    false
  }
  if let Some(ref right_xm) = right
    && list_has_relation(right_xm)
  {
    return Err(
      "infix_relation: right is list@ containing relations — prefer list-of-modified_terms path"
        .into(),
    );
  }
  // Reject multirelation when the left formula's last operand is a list Dual without delimiters of
  // its own (`presents_its_items_alone`). For `a = b, c = d`: the wrong parse creates `a = list(b,c)`
  // then tries to extend with `= d`. If the left formula has a bare list as its last arg, the comma
  // should have been a formula boundary, not an expression list. Rejecting here forces Marpa to use
  // formula_list instead. A fenced list is one closed item and splits nothing: `x=[a,b,c]=y` (57ay;
  // 2605.03399, 2605.29816).
  if let Some(ref left_xm) = left {
    fn last_arg_is_list(xm: &XM) -> bool {
      match xm {
        XM::Apply(op, args, ..) => {
          // Check if this is a relational Apply (has RELOP/ARROW operator)
          let is_rel = is_multirelation(&op.0) || is_relational_op(&op.0);
          if is_rel {
            // Check last argument
            if let Some(Some(XM::Dual(content, presentation, ..))) = args.0.last()
              && let XM::Apply(ref inner_op, ref inner_args, ..) = **content
              && let XM::Token(ref props, _) = *inner_op.0
            {
              return props.meaning.as_deref() == Some("list")
                && presents_its_items_alone(presentation, inner_args.0.len());
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
  Ok(Some(chain_relation(left, infixop, right)))
}

pub fn infix_apply_nary(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => left, infixop, right);
  let mut left = left;
  // `a\cdot b\cdots` elides the operation (`infix_apply_and_elide`: a cdot b cdot cdots); a product
  // ending in a bare ellipsis as a visible operation's right operand is that reading's duplicate once
  // an ellipsis is a factor (57bs).
  if infixop
    .as_ref()
    .is_some_and(|op| matches!(operator_category(op), Some("MULOP" | "BINOP" | "ADDOP")))
    && right
      .as_ref()
      .is_some_and(|right| ends_in_a_bare_ellipsis(right, &ctxt))
  {
    return Err(
      "infix_apply_nary: the ellipsis elides the operation (infix_apply_and_elide)".into(),
    );
  }
  // Perl's greedy `barearg` (MathGrammar:321-337) takes a MulOp — a decorated `\otimes_k` too —
  // and the bare argument after it into an operator's argument: `\nabla u\cdot v` is ∇@(u·v).
  if infixop.as_ref().is_some_and(is_product_operator)
    && let (Some(l), Some(r)) = (&left, &right)
    && leaves_a_bare_argument(l, r, false, &ctxt)
  {
    return Err("infix_apply_nary: the operator on the left takes this bare argument".into());
  }
  // … and the trig function a differential operator takes, Perl's greedy `moreTrigBareargs`
  // (MathGrammar:351-357; `trig_arg mulop factor_base`): `\partial\sin x\times_i^2 y` is ∂(sin(x ×_i² y)),
  // Perl's golden t/parse/artefacts (#374).
  if infixop.as_ref().is_some_and(is_product_operator)
    && let (Some(l), Some(r)) = (&left, &right)
    && is_differential(product_end(l, true))
    && leaves_a_trig_bare_argument(l, r, &ctxt)
  {
    return Err(
      "infix_apply_nary: the differential operator's trig function takes this factor".into(),
    );
  }
  // … and any trig function's bare argument a MulOp or BinOp continues (`trig_arg mulop factor_base`), as Perl's
  // greedy `moreTrigBareargs` (`trig_argument_across_a_mulop`)
  if infixop.as_ref().is_some_and(is_product_operator)
    && let (Some(l), Some(r)) = (&left, &right)
    && trig_argument_across_a_mulop(l, r, &ctxt)
  {
    return Err("infix_apply_nary: the trig function's bare argument takes this factor".into());
  }
  // Divergence #374: a Leibniz quotient is one derivative (`leibniz_quotient`).
  let leibniz = matches!((&left, &infixop, &right), (Some(l), Some(op), Some(r))
    if is_divide(op) && is_differential(product_end(r, false)) && holds_a_leibniz_numerator(l));
  if leibniz {
    let (Some(l), Some(op), Some(r)) = (left, infixop, right) else {
      unreachable!()
    };
    return Ok(Some(leibniz_quotient(l, op, r, &ctxt)));
  }
  // left-to-right associative:
  // 1. if "left" is already an application of "infixop",
  // 2. then tuck "right" inside it.
  //
  // Note (ASF + LOSTNODES): in pure-Tree-iter parsing it would be
  // natural to record `ReplacedBy(infixop, left_op)` here so the
  // absorbed operator's xml:id is redirected to the kept operator's.
  // But this action runs inside the ASF Cartesian-product loop —
  // `action_on` fires for every candidate combo, including ones
  // ultimately discarded. Recording from action time would pollute
  // LOSTNODES with mappings from pruned parses, breaking the
  // top-level rewrite walk on the *final* tree. The recording
  // therefore lives at `parse_single` (after `into_xmath` +
  // `append_tree` commit the chosen tree), via the pre/post-snapshot
  // diff against the document idstore.
  //
  // Perl left-to-right: an explicit MULOP only takes one factor on the right (Perl's `moreFactors`,
  // MathGrammar:252-258: `MulOp Factor` and `Factor`, each `ApplyNary` on the product so far).
  // a/bc → (a/b)*c, F×G dx → (F×G)*dx — NOT a/(b*c) or F×(G*dx). When an explicit (visible) MULOP
  // has a right operand that is an invisible-times application, it takes just the first factor and the
  // rest multiply the result: Apply(op, left, Apply(⁢, first, rest...)) → Apply(⁢, Apply(op, left,
  // first), rest...). A decorated MULOP (`a\times_k DB` is `(a ×_k D) * B`, as Perl's `MulOp`) is
  // visible. A BINOP keeps its juxtaposed operand whole, where Perl's `MulOp : BINOP` (:688) takes one
  // factor: a `\mathbin` of unknown meaning is no product, and every reading the one-factor rule gave in
  // the corpus was wrong — `KX\mathbin{\|}(I-K)X` ‖(KX, I−K)·X (2605.31129), `[WX_i\mathbin{\|}WX_j]`
  // (2605.31315, 2605.08689, 2605.25490, 2605.26237, 2605.30618; 57cj.19.1 A/B km191, divergence #393;
  // Q11 extends it to the large MULOPs, below, divergence #396).
  //
  // A large product operator — ⊗, ⊙ and the circled and boxed family — keeps its juxtaposed operand whole too (user
  // ruling Q11, 2026-10-01: juxtaposition binds tighter than a large MULOP; divergence #396): `a\otimes 2b` a⊗(2b),
  // `2\Lambda_1\otimes\Lambda_1\otimes 2\Lambda_1\otimes\Lambda_1` (2Λ₁)⊗Λ₁⊗(2Λ₁)⊗Λ₁ (2605.17901), `g\otimes w\otimes\sigma'(\gamma_i)`
  // (2605.01702), `P\odot P\odot P_\theta(x|y)` (2605.00423), a decorated one too, `R\otimes_{\mathbb C}\mathbb C G` (2605.14864;
  // was (a ⊗_k D)·B as Perl's `MulOp`, `a\otimes_k DB`).
  let is_explicit_mulop = infixop.as_ref().is_some_and(|op| {
    operator_category(op) == Some("MULOP")
      && !is_invisible_times_operator(op, &ctxt)
      && !is_a_large_mulop(op, &ctxt)
  });
  // … except before an integral's differentials, which close the integrand: there a BINOP or a large MULOP takes the
  // integrand's factors before them, its juxtaposed operand (user rulings Q11 and 2026-10-01, the latter revisiting #393's
  // one-factor exception of 57cj.19.3-57cj.19.8): `\int f\otimes g h\,dx` ∫((f⊗(g h))·dx), `\int f\boxast g h\,dx`
  // ∫((f⧆(g h))·dx) (was ∫((f⧆g)·h·dx)), as `a\boxast g h` ⧆(a, g h); `\int f\boxast g\,dx` ∫((f⧆g)·dx) (was ∫(⧆(f, g·dx))
  // before 57cj.19), `\int_X f\boxast g\,d\mu(x)` ∫((f⧆g)·dμ·x); physics' `\dd x`, `\dd{x}`, `\dd^2 x` too; an operand
  // that opens with a differential stays whole (`integrand_split`; latent, the reviews' probes, no corpus witness). The
  // integrand closes before the letter `d` of a differential too, the letter twin a differential's reading meets, so
  // `LetterDsBeforeVariablesAreDifferentials` decides on the `d` alone (`\int_0^1 f\otimes g h\,dx` read ∫(f⊗(g·h·d·x))
  // beside ∫((f⊗(g h))·dx) and the first survived, 57cj.20.Q11 review).
  let before_differentials = infixop.as_ref().is_some_and(|op| {
    (operator_category(op) == Some("BINOP") || is_a_large_mulop(op, &ctxt))
      && integrand_split(&right, &ctxt).is_some()
  });
  let right = if before_differentials {
    integrand_before_differentials(right, &ctxt)
  } else {
    right
  };
  let takes_one_juxtaposed_factor =
    (is_explicit_mulop || before_differentials) && is_juxtaposed_product(&right, &ctxt);
  if let Some(XM::Apply(ref left_op, ref mut left_args, ref left_props, ref _m)) = left
    && let XM::Lexeme(left_op_lex, _xmeta) = &*left_op.0
    && let Some(ref infix @ XM::Lexeme(ref infix_op_lex, _)) = infixop
  {
    let left_op_pieces: Vec<_> = left_op_lex.split(':').collect();
    let infix_op_pieces: Vec<_> = infix_op_lex.split(':').collect();
    if left_op_pieces.len() == 3
          && infix_op_pieces.len() == 3
          && left_op_pieces[0] == infix_op_pieces[0]
          && left_op_pieces[1] == infix_op_pieces[1]
          // Perl's `ApplyNary` flattens only the same operator (`isSameExpr`, MathParser.pm:1506,
          // :1522-1539: meaning, value and mathstyle) — `a/b\div c` is ÷(/(a,b),c), never one
          // n-ary application that drops the ÷ (57cj.19 review).
          && same_operator_token(&left_op.0, infix, &ctxt)
          // … and never into an application that carries an id (Perl ApplyNary, MathParser.pm:1507-1509;
          // a defensive mirror: parse-built applications carry none)
          && left_props.id.is_none()
          // Perl's LeftRec doesn't flatten prefix applications (1 arg = unary prefix)
          // Only flatten when left already has 2+ args (binary or n-ary)
          && left_args.0.len() >= 2
    {
      // … and in an n-ary chain the explicit MulOp takes one factor too: `a\cdot b\cdot c d` is
      // (a·b·c)·d and `a/b/c d` ((a/b)/c)·d, as `a\cdot b c` (a·b)·c and as Perl (SYNC residual (14),
      // 57cj.19; was a·b·(c d); witnesses 2605.31500 `M\cdot D\cdot si`, 2605.00321
      // `\lambda\cdot\beta\cdot\hat{\delta}(\mathbf{p})`, 2605.14476 `12/z/ma`, was 12/z/(m·a)).
      if takes_one_juxtaposed_factor
        && let Some(XM::Apply(right_op, Args(mut factors), right_props, right_meta)) = right
      {
        left_args.0.push(factors.remove(0));
        let mut new_args = vec![left];
        new_args.extend(factors);
        return Ok(Some(XM::Apply(
          right_op,
          Args(new_args),
          right_props,
          right_meta,
        )));
      }
      left_args.0.push(right);
      return Ok(left);
    }
  }
  if takes_one_juxtaposed_factor
    && let Some(XM::Apply(right_op, Args(mut factors), right_props, right_meta)) = right
  {
    let first = factors.remove(0);
    // Apply(op, left, first) …
    let product = Some(XM::Apply(
      infixop.into(),
      Args(vec![left, first]),
      XProps::default(),
      Meta::default(),
    ));
    // … times the rest: Apply(⁢, product, rest...)
    let mut new_args = vec![product];
    new_args.extend(factors);
    return Ok(Some(XM::Apply(
      right_op,
      Args(new_args),
      right_props,
      right_meta,
    )));
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

/// A large product operator (user ruling Q11, 2026-10-01; divergence #396): ⊗ (`\otimes`, tensor-product), ⊙ (`\odot`,
/// direct-product) and the circled and boxed family of the same size, ⊘ `\oslash`, ⊚ `\circledcirc`, ⊛ `\circledast`,
/// ⊠ `\boxtimes`, ⊡ `\boxdot`, stmaryrd's ⦸ `\varobslash`; and (the Q11 scope ruling, 2026-10-01) the semidirect products
/// ⋉ `\ltimes`, ⋊ `\rtimes`, ⋋ `\leftthreetimes`, ⋌ `\rightthreetimes` (2605.11552, 2605.12221, 2605.15276, 2605.27086),
/// the coproduct ⨿ `\amalg` (the kernel's ∐), the circles ○ `\bigcirc` and ◯ `\varbigcirc`, mathabx's box product □
/// `\square` and its MULOP ⊕ `\pluscirc` (a MULOP only there: `\oplus` is an ADDOP, amsfonts' `\square` a symbol) — bare
/// or decorated (`\otimes_k`). Not `\cdot`, `\times`, `\star`, `\ast`, `\circ`, `/`. A default by glyph with its escape:
/// the operator's role, so a document that declares one otherwise reads it so.
fn is_a_large_mulop(op: &XM, ctxt: &ActionContext) -> bool {
  operator_category(op) == Some("MULOP")
    && realized_value(script_nucleus(op), ctxt).is_ok_and(|value| {
      matches!(
        value.as_ref(),
        "\u{2297}"
          | "\u{2299}"
          | "\u{2298}"
          | "\u{229A}"
          | "\u{229B}"
          | "\u{22A0}"
          | "\u{22A1}"
          | "\u{29B8}"
          | "\u{22C9}"
          | "\u{22CA}"
          | "\u{22CB}"
          | "\u{22CC}"
          | "\u{2210}"
          | "\u{2A3F}"
          | "\u{25CB}"
          | "\u{25EF}"
          | "\u{25A1}"
          | "\u{2295}"
      )
    })
}

/// Regroup an integrand product `g h\,dx\,dy` (`integrand_split`) as its factors before the first
/// differential, one juxtaposed product, then the rest: (g h)·dx·dy — what a large MULOP takes is the product's first
/// factor (Q11).
fn integrand_before_differentials(right: Option<XM>, ctxt: &ActionContext) -> Option<XM> {
  let split = integrand_split(&right, ctxt);
  let Some(XM::Apply(op, Args(factors), props, meta)) = right else {
    return right;
  };
  match split {
    Some(at) if at >= 2 => {
      let mut factors = factors;
      let rest = factors.split_off(at);
      let integrand = XM::Apply(
        op.clone(),
        Args(factors),
        XProps::default(),
        Meta::default(),
      );
      let mut regrouped = vec![Some(integrand)];
      regrouped.extend(rest);
      Some(XM::Apply(op, Args(regrouped), props, meta))
    },
    _ => Some(XM::Apply(op, Args(factors), props, meta)),
  }
}

/// Where an unfenced integrand product closes (`integrand_before_differentials`): its first factor after the first that
/// is an integral's differential, or the letter `d` before a variable its differential takes (the letter twin,
/// `letter_differential_sites`). None when the product opens with a differential — it closes nothing and stays whole
/// (#393): `a\mathbin{\#}\dd\omega\,\eta` #(a, dω·η), an exterior derivative, `a\mathbin{\#}\dd x\,\dd y`,
/// `\int f\boxast dx\,dy`, `\int f\mathbin{\#}\dd x\,g\,\dd y` (57cj.19.5-57cj.19.7 reviews) — or holds none. A differential is
/// a `d`-kind one's application, not a differential operator's (`\partial_t u`): a bare `d` only in an integral's operand
/// (`diffop_apply`, `util::in_an_integral_operand`); a bound differential anywhere — iopart's `\rmd`, elsart's `\d` (meaning
/// `differential-d`), physics' `\dd`/`\differential` (meaning `differential`, a dual over its symbol), braced too
/// (`\dd{x}`, `\dd[3]{x}`: a dual over its application).
fn integrand_split(right: &Option<XM>, ctxt: &ActionContext) -> Option<usize> {
  let Some(XM::Apply(Operator(op), Args(factors), props, meta)) = right else {
    return None;
  };
  if meta.fenced.is_some()
    || props.id.is_some()
    || factors.len() < 2
    || !is_invisible_times_operator(op, ctxt)
  {
    return None;
  }
  let is_differential_at = |at: usize| {
    factors[at].as_ref().is_some_and(|factor| {
      is_an_integral_differential_factor(factor, ctxt)
        || is_a_letter_differential_d(factor)
          && factors
            .get(at + 1)
            .and_then(Option::as_ref)
            .is_some_and(is_a_differential_variable)
    })
  };
  if is_differential_at(0) {
    return None;
  }
  (1..factors.len()).find(|&at| is_differential_at(at))
}

/// Is `right` a juxtaposed product — an invisible-times application of two or more factors — whose
/// first factor alone an explicit MulOp on its left takes (`infix_apply_nary`)? A visible `×` product
/// is none.
fn is_juxtaposed_product(right: &Option<XM>, ctxt: &ActionContext) -> bool {
  matches!(right, Some(XM::Apply(op, args, ..))
    if args.0.len() >= 2 && is_invisible_times_operator(&op.0, ctxt))
}

/// A factor that is an integral's `d`-kind differential (`integrand_split`): its application or token.
fn is_an_integral_differential_factor(factor: &XM, ctxt: &ActionContext) -> bool {
  match factor {
    XM::Apply(Operator(head), _, _, factor_meta) => {
      factor_meta.differential && is_a_differential(head, ctxt)
    },
    XM::Lexeme(..) => is_a_differential(factor, ctxt),
    _ => false,
  }
}

/// Is `xm` — scripted or not (`\dd^2`) — a `d`-kind differential's token, power, application or dual?
fn is_a_differential(xm: &XM, ctxt: &ActionContext) -> bool {
  if let Some(base) = script_base(xm) {
    return is_a_differential(base, ctxt);
  }
  match xm {
    XM::Lexeme(lex, _) => {
      lookup_lex_node(lex, ctxt.nodes).is_ok_and(|node| node_is_a_differential(node, ctxt.document))
    },
    other => is_a_differential_meaning(realized_meaning(other, ctxt).as_deref()),
  }
}

/// The meaning of a `d`-kind differential: a `d`'s (`differential-d`) or physics' `\differential` (`differential`)
/// — not a variation (`\variation`, δ) or a partial derivative.
fn is_a_differential_meaning(meaning: Option<&str>) -> bool {
  matches!(meaning, Some("differential-d" | "differential"))
}

/// Is `node` a `d`-kind differential — its token, its power (`functional-power`), its application, or a dual whose
/// content is one of these (physics' `\dd`, `\dd[3]`, `\dd{x}`, `\dd[3]{x}`)?
fn node_is_a_differential(node: &libxml::tree::Node, document: &Document) -> bool {
  let node = realize_xmnode(node, document);
  match node.get_name().as_str() {
    "XMTok" => is_a_differential_meaning(node.get_attribute("meaning").as_deref()),
    "XMApp" => {
      let children = node.get_child_elements();
      children.first().is_some_and(|head| {
        node_is_a_differential(head, document)
          || realize_xmnode(head, document)
            .get_attribute("meaning")
            .as_deref()
            == Some("functional-power")
            && children
              .get(1)
              .is_some_and(|base| node_is_a_differential(base, document))
      })
    },
    "XMDual" => node
      .get_child_elements()
      .first()
      .is_some_and(|content| node_is_a_differential(content, document)),
    _ => false,
  }
}

/// Is `op` the invisible times (U+2062) — the juxtaposition's operator, or a lexeme whose node's
/// value it is?
fn is_invisible_times_operator(op: &XM, ctxt: &ActionContext) -> bool {
  is_invisible_times_op(op)
    || is_invisible_times_lexeme(op)
    || matches!(op, XM::Lexeme(..))
      && realized_value(op, ctxt).is_ok_and(|value| value == "\u{2062}")
}

/// Perl `isSameExpr` (MathParser.pm:1522-1539) for two operator lexemes already alike in role and
/// meaning: the same value (`p_getValue`) and mathstyle, each read through `realizeXMNode`.
fn same_operator_token(a: &XM, b: &XM, ctxt: &ActionContext) -> bool {
  let mathstyle = |xm: &XM| match xm {
    XM::Lexeme(lex, _) => lookup_lex_node(lex, ctxt.nodes)
      .ok()
      .and_then(|node| realize_xmnode(node, ctxt.document).get_attribute("mathstyle")),
    _ => None,
  };
  matches!((realized_value(a, ctxt), realized_value(b, ctxt)), (Ok(x), Ok(y)) if x == y)
    && mathstyle(a) == mathstyle(b)
}

/// The division `/` (Perl `MULOP:divide`), bare or as a token.
fn is_divide(op: &XM) -> bool {
  match op {
    XM::Lexeme(lex, _) => lex.starts_with("MULOP:divide:"),
    XM::Token(props, _) => {
      props.role.as_deref() == Some("MULOP") && props.meaning.as_deref() == Some("divide")
    },
    _ => false,
  }
}

/// A Leibniz numerator's differential operator: applied, or standing alone (`\partial/\partial t`).
fn is_leibniz_numerator_item(xm: &XM) -> bool {
  is_differential(xm) || is_bare_differential_operator(xm)
}

/// Does `xm`, left of a `/`, hold a Leibniz numerator — a differential operator, alone or last but for
/// the factors it would take, in an unfenced product (`leibniz_numerator`)?
fn holds_a_leibniz_numerator(xm: &XM) -> bool {
  is_leibniz_numerator_item(xm)
    || matches!(xm, XM::Apply(Operator(op), Args(factors), props, meta)
      if is_invisible_times_op(op)
        && meta.fenced.is_none()
        && props.id.is_none()
        && factors.iter().flatten().any(is_leibniz_numerator_item))
}

/// The numerator of a Leibniz quotient in `left` (or in a `\frac` numerator, `regroup_leibniz_numerator`):
/// the last differential operator of an unfenced product, with the factors after it as its operand
/// (`\partial\rho u` ∂(ρu)), and the factors before it, which multiply the quotient (`T\,\partial F`
/// T·…); `left` back when it holds none.
fn leibniz_numerator(left: XM) -> Result<(Vec<Option<XM>>, XM), Box<XM>> {
  if is_leibniz_numerator_item(&left) {
    return Ok((Vec::new(), left));
  }
  match left {
    XM::Apply(op, Args(mut factors), props, meta)
      if is_invisible_times_op(&op.0) && meta.fenced.is_none() && props.id.is_none() =>
    {
      let Some(k) = factors
        .iter()
        .rposition(|factor| factor.as_ref().is_some_and(is_leibniz_numerator_item))
      else {
        return Err(Box::new(XM::Apply(op, Args(factors), props, meta)));
      };
      let after = factors.split_off(k + 1);
      let Some(Some(numerator)) = factors.pop() else {
        unreachable!()
      };
      if after.is_empty() {
        return Ok((factors, numerator));
      }
      // The differential operator takes the factors after it: `\partial\rho u` ∂(ρu).
      let (head, mut operand) = match numerator {
        XM::Apply(head, Args(mut args), _, meta) if meta.differential && args.len() == 1 => {
          (head, vec![args.pop().flatten()])
        },
        bare => (bare.into(), Vec::new()),
      };
      operand.extend(after);
      let operand = if operand.len() == 1 {
        operand.pop().flatten()
      } else {
        Some(XM::Apply(
          invisible_times().into(),
          Args(operand),
          XProps::default(),
          Meta::default(),
        ))
      };
      let numerator = XM::Apply(
        head,
        Args(vec![operand]),
        XProps::default(),
        Meta::for_differential(),
      );
      Ok((factors, numerator))
    },
    other => Err(Box::new(other)),
  }
}

/// A `\frac` numerator over a differential operator (`\frac{\partial\rho u}{\partial t}`, parser.rs
/// `is_leibniz_numerator_arg`): its last differential operator takes the factors after it, as in
/// `\partial\rho u/\partial t` (∂(ρu), divergence #374; Perl's greedy `bigop` reads it so; `\frac{\partial\Delta W}
/// {\partial B}` ∂(ΔW)/∂B, 2605.05995).
pub(crate) fn regroup_leibniz_numerator(numerator: XM) -> XM {
  match leibniz_numerator(numerator) {
    Ok((before, numerator)) if before.is_empty() => numerator,
    Ok((mut before, numerator)) => {
      before.push(Some(numerator));
      XM::Apply(
        invisible_times().into(),
        Args(before),
        XProps::default(),
        Meta::default(),
      )
    },
    Err(numerator) => *numerator,
  }
}

/// A `\frac` denominator of differentials under a Leibniz numerator (parser.rs `is_leibniz_denominator_arg`):
/// each differential takes the factors up to the next one, the variable it names — `\frac{\partial u_0}
/// {\partial\Delta\psi}` ∂u_0/∂(Δψ) (2605.23203, 2605.28495), `\frac{\partial\Delta W}{\partial B}` beside
/// `\frac{\partial\mathcal L}{\partial\Delta W}` reads ΔW once (2605.05995), `\frac{\partial^2 f}{\partial x\partial y}`
/// unchanged (57cj review); an ellipsis stands between two differentials (`\partial x_1^{\gamma_1}\dots\partial
/// x_d^{\gamma_d}`). The `\frac` delimits the denominator, which a slash quotient's does not
/// (`leibniz_quotient` counts the order there).
pub(crate) fn regroup_leibniz_denominator(denominator: XM, ctxt: &ActionContext) -> XM {
  let XM::Apply(op, Args(factors), props, meta) = denominator else {
    return denominator;
  };
  if !(is_invisible_times_op(&op.0) && meta.fenced.is_none() && props.id.is_none())
    || !factors
      .iter()
      .flatten()
      .any(|factor| is_partial_derivative(factor) && !differentiates_by_its_subscript(factor))
  {
    return XM::Apply(op, Args(factors), props, meta);
  }
  let mut grouped: Vec<Option<XM>> = Vec::new();
  // The operand the last differential gathers, while it takes factors.
  let mut open: Option<(Operator, Vec<Option<XM>>)> = None;
  let close = |open: Option<(Operator, Vec<Option<XM>>)>, grouped: &mut Vec<Option<XM>>| {
    if let Some((head, mut operand)) = open {
      let operand = if operand.len() == 1 {
        operand.pop().flatten()
      } else {
        Some(XM::Apply(
          invisible_times().into(),
          Args(operand),
          XProps::default(),
          Meta::default(),
        ))
      };
      grouped.push(Some(XM::Apply(
        head,
        Args(vec![operand]),
        XProps::default(),
        Meta::for_differential(),
      )));
    }
  };
  for factor in factors {
    match factor {
      Some(XM::Apply(head, Args(mut args), _, factor_meta))
        if factor_meta.differential
          && args.len() == 1
          && is_bare_differential_operator(&head.0)
          && !has_a_subscript(&head.0) =>
      {
        close(open.take(), &mut grouped);
        open = Some((head, vec![args.pop().flatten()]));
      },
      Some(ellipsis) if is_ellipsis(&ellipsis, ctxt) => {
        close(open.take(), &mut grouped);
        grouped.push(Some(ellipsis));
      },
      // a subscripted derivative is no variable's, it stands alone (`\frac{\partial f}{\partial x\,\partial_y g}`,
      // 57cj.2 review)
      Some(derivative) if is_partial_derivative(&derivative) => {
        close(open.take(), &mut grouped);
        grouped.push(Some(derivative));
      },
      other => match open.as_mut() {
        Some((_, operand)) => operand.push(other),
        None => grouped.push(other),
      },
    }
  }
  close(open.take(), &mut grouped);
  match grouped.pop() {
    Some(Some(only)) if grouped.is_empty() => only,
    last => {
      grouped.push(last.flatten());
      XM::Apply(op, Args(grouped), props, meta)
    },
  }
}

/// Does a scripted head carry a subscript (`\partial_x`, `\partial^2_{xy}`)? A Leibniz denominator's ∂
/// names its variable after it; a subscripted one is a derivative of its own (57cj.1 review; latent, the review's
/// probe: `\frac{\partial_t u\,v}{\partial_x u\,w}` keeps (∂_x u)·w).
fn has_a_subscript(head: &XM) -> bool {
  match head {
    XM::Apply(Operator(op), Args(args), ..)
      if matches!(operator_category(op), Some("SUBSCRIPTOP" | "SUPERSCRIPTOP")) =>
    {
      operator_category(op) == Some("SUBSCRIPTOP")
        || args
          .first()
          .and_then(Option::as_ref)
          .is_some_and(has_a_subscript)
    },
    _ => false,
  }
}

/// A partial derivative whose operator carries a subscript (`\partial_x u`).
fn differentiates_by_its_subscript(xm: &XM) -> bool {
  matches!(xm, XM::Apply(Operator(head), ..) if has_a_subscript(head))
}

/// The order a differential operator states: the number over it (`\partial^2`, `\partial^2_{xy}`), 1
/// unwritten, None when it is no number (`\partial^n`).
fn differential_order(head: &XM, ctxt: &ActionContext) -> Option<u32> {
  match head {
    XM::Apply(Operator(op), Args(args), ..)
      if matches!(operator_category(op), Some("SUBSCRIPTOP" | "SUPERSCRIPTOP")) =>
    {
      let [Some(base), Some(script)] = args.as_slice() else {
        return None;
      };
      if operator_category(op) == Some("SUPERSCRIPTOP") {
        written_number(script, ctxt)
      } else {
        differential_order(base, ctxt)
      }
    },
    _ => Some(1),
  }
}

/// A script that is a number, as a count (`2` in `\partial^2`, `x^2`).
fn written_number(xm: &XM, ctxt: &ActionContext) -> Option<u32> {
  (operator_category(xm) == Some("NUMBER"))
    .then(|| realized_value(xm, ctxt).ok()?.trim().parse().ok())
    .flatten()
}

/// The order a denominator's differential counts toward: its operator's times its variable's power
/// (`\partial x^2` 2), None when either is no number.
fn differential_power(xm: &XM, ctxt: &ActionContext) -> Option<u32> {
  let XM::Apply(Operator(head), Args(args), ..) = xm else {
    return Some(1);
  };
  let power = match args.as_slice() {
    [Some(XM::Apply(Operator(op), Args(variable), ..))]
      if operator_category(op) == Some("SUPERSCRIPTOP") =>
    {
      match variable.as_slice() {
        [_, Some(power)] => written_number(power, ctxt)?,
        _ => 1,
      }
    },
    _ => 1,
  };
  Some(differential_order(head, ctxt)? * power)
}

/// Divergence #374: a Leibniz quotient `\partial X/\partial Y` is one derivative, whatever stands around
/// it (user ruling 2026-09-29: ∂F/∂T is a derivative). The numerator is the left side's last differential
/// operator with the factors after it (`leibniz_numerator`: `T\,\partial F/\partial T` T·(∂F/∂T),
/// `\partial\rho u/\partial t` ∂(ρu)/∂t); the denominator the leading differentials the numerator's order
/// asks for — `\partial^2 f/\partial x\partial y` ∂²f/(∂x∂y), every one for an order that is no number
/// (`\partial^n f/\partial x_1\cdots\partial x_n`); what follows multiplies the quotient, as `a/bc` is
/// (a/b)c (`\partial u/\partial x\,v` (∂u/∂x)·v). Perl's greedy `bigop` reads ∂@(F/∂@(T))
/// (MathGrammar:717, :605-618; KNOWN_PERL_ERRORS #387; 2605.03741, 2605.08634, 2605.15405).
fn leibniz_quotient(left: XM, divide: XM, right: XM, ctxt: &ActionContext) -> XM {
  let (mut factors, numerator) = match leibniz_numerator(left) {
    Ok(split) => split,
    Err(left) => (Vec::new(), *left),
  };
  let order = match &numerator {
    XM::Apply(Operator(head), _, _, meta) if meta.differential => differential_order(head, ctxt),
    bare => differential_order(bare, ctxt),
  };
  let (denominator, after) = match right {
    XM::Apply(op, Args(mut denominators), props, meta)
      if is_invisible_times_op(&op.0) && meta.fenced.is_none() && props.id.is_none() =>
    {
      let mut taken = 0;
      let mut total = 0;
      // the numerator's kind of differential only: `\int_0^1\partial^n f/\partial x^n\,dx` keeps its `dx`
      let partial = is_partial_differential(&numerator);
      for (k, factor) in denominators.iter().enumerate() {
        match factor {
          Some(factor) if is_differential(factor) && is_partial_differential(factor) == partial => {
            taken = k + 1;
            if let (Some(order), Some(power)) = (order, differential_power(factor, ctxt)) {
              total += power;
              if total >= order {
                break;
              }
            }
          },
          // an ellipsis between differentials (`\partial x_1\cdots\partial x_n`)
          Some(factor) if is_ellipsis(factor, ctxt) => {},
          _ => break,
        }
      }
      let after = if taken == 0 {
        Vec::new()
      } else {
        denominators.split_off(taken)
      };
      let denominator = match denominators.pop() {
        Some(Some(only)) if denominators.is_empty() => only,
        last => {
          denominators.push(last.flatten());
          XM::Apply(op, Args(denominators), props, meta)
        },
      };
      (denominator, after)
    },
    right => (right, Vec::new()),
  };
  factors.push(Some(XM::Apply(
    divide.into(),
    Args(vec![Some(numerator), Some(denominator)]),
    XProps::default(),
    Meta::default(),
  )));
  factors.extend(after);
  match factors.pop() {
    Some(Some(quotient)) if factors.is_empty() => quotient,
    last => {
      factors.push(last.flatten());
      XM::Apply(
        invisible_times().into(),
        Args(factors),
        XProps::default(),
        Meta::default(),
      )
    },
  }
}

/// The arguments a known function takes from the group after it, spread as its direct
/// operands, or None when the group is one argument. Perl's `addEasyArgs`/`ApplyDelimited`
/// (MathGrammar:570-577; `requireArgs` :532-536 after an APPLYOP) reads `OPEN Argument
/// (argPunct Argument)* balancedClose`, `argPunct` being PUNCT, MIDDLE or VERTBAR (:656), and
/// drops the delimiters: `\max(a,b)` `maximum@(a, b)`, not `maximum@(vector@(a, b))`;
/// `\max(a;b)`, `\max[a;b]` (57ba), `\max\{a,b\}`, `\max[a,b]`, `\operatorname{E}[X]` `E@(X)`,
/// `\exp\{x\}` (57bb; the 57am probes' `\operatorname{E}\{…\}`, 2605.24123; golden
/// `tests/parse/fenced_lists.tex`, "A function takes the arguments between any delimiters").
/// A group reads so when it is a grouping fence — `delimited-…`, `list`, `set`, an interval, a
/// paren `vector` — opened by a grouping delimiter, parens, brackets or braces (angle brackets are
/// an inner product or an average, one argument: `\max\langle a,b\rangle`
/// `maximum@(delimited-⟨⟩@(list@(a, b)))`, `\log\langle Z\rangle` `logarithm@(delimited-⟨⟩@(Z))`;
/// divergence #363), and closed by its match or by nothing (a row break, a typo: `\exp((n-k)(\ln…`, 2605.25295,
/// where Perl, needing `balancedClose`, reads nothing); a close of another kind is no argument's
/// (`\max(a,b]` stays a product in Perl; `\exp\lfloor x\rceil` keeps its rounding), and separated by
/// `argPunct` only (`\max(a:b)` one argument, Perl `maximum@(a colon b)`). A named function of
/// its content stays one argument — `\log\lfloor x\rfloor` `logarithm@(floor@(x))`,
/// `\log\lvert x\rvert`, `\max\{x\mid x>0\}` a `conditional-set` — where Perl's grammar,
/// reading the arguments first, drops the floor (`logarithm@(x)`) and garbles the set
/// (`x * ket@(x) * 0`; divergence #363). One item that is itself a bare list gives its items
/// (`\Pr[X=1,Y=2]` Pr@(X = 1, Y = 2)), and the second value says so:
/// the list's own presentation then joins the function's (`splice_listed_arguments`); so does a
/// single paren item (a `Ref`) that is a bare list — a relation list, one `formulae` item since
/// 57bk. None for any other single paren item (the caller's own case) and a bare list, so `\sin(x)`
/// and the unknown-`f` apply divergence (OXIDIZED_DESIGN #18) are unaffected.
fn fenced_tuple_items(
  xm: &XM,
  presentation: &XM,
  ctxt: &ActionContext,
) -> Option<(Vec<Option<XM>>, bool)> {
  let arg_punct = |separators: &[XM]| {
    separators.iter().skip(1).step_by(2).all(|separator| {
      matches!(
        get_xm_role(separator).as_deref(),
        Some("PUNCT" | "MIDDLE" | "VERTBAR")
      )
    })
  };
  let items = match xm {
    XM::Apply(Operator(op), Args(items), ..) => {
      let XM::Token(props, _) = op.as_ref() else {
        return None;
      };
      let meaning = props.meaning.as_deref()?;
      let grouping = meaning.starts_with("delimited-")
        || matches!(
          meaning,
          "vector"
            | "list"
            | "set"
            | "open-interval"
            | "closed-interval"
            | "open-closed-interval"
            | "closed-open-interval"
        );
      if !grouping {
        return None;
      }
      // A bare paren `vector` (no delimiters of its own) spreads as before.
      if meaning == "vector" && presents_its_items_alone(presentation, items.len()) {
        return Some((items.clone(), false));
      }
      Some(items)
    },
    // A paren group of one item (`fenced`'s single-item Dual) gives arguments only when that item is
    // a bare list: a relation list, one `formulae` item (57bk) — `\Pr(X\le x,Y\le y)`
    // Pr@(X <= x, Y <= y), as Perl's `ApplyDelimited` (2605.00042, 2605.01907).
    XM::Ref(_) => None,
    _ => return None,
  };
  let XM::Wrap(fenced, ..) = presentation else {
    return None;
  };
  let [open, inner @ .., close] = fenced.as_slice() else {
    return None;
  };
  // Read through an `XMRef`, as a split or gathered row's content branch holds its lexemes (Perl
  // `isMatchingClose` realizes the node, MathParser.pm:1379-1384; 57bb review: `\max(a,b)` in a
  // split row stayed `maximum@(vector@(a, b))`, 2605.01929, 2605.04581).
  let (Ok(open_value), Ok(close_value)) = (realized_value(open, ctxt), realized_value(close, ctxt))
  else {
    return None;
  };
  // A close that is missing altogether (the lexer's empty CLOSE: a row break, a typo) still ends the
  // argument the author opened — `\operatorname{null}(G(\theta(t))` null@(G@(θ@(t))), 2605.00284;
  // a close of another kind pairs the open as something else: `\max(a,b]`, `\exp\lfloor x\rceil`.
  if inner.is_empty()
    // Angle brackets are an inner product, not an argument list: `\max_m\langle t,b_m\rangle`
    // is the maximum of ⟨t, b_m⟩ (57bf review of the scripted-head lift: 95 formulas in 38 papers
    // of the A/B, every one an inner product, 2605.02896, 2605.20551; Perl spreads them, divergence
    // #363), and `\log\langle Z\rangle` the logarithm of an average (Perl `logarithm@(Z)`).
    || !matches!(open_value.as_ref(), "(" | "[" | "{")
    || !(close_value.is_empty() || balanced_close(&open_value) == Some(close_value.as_ref()))
    || !arg_punct(inner)
  {
    return None;
  }
  // One item that is a bare list: the list's items are the arguments.
  if let [XM::Dual(list_content, list_presentation, ..)] = inner
    && let XM::Apply(Operator(list_op), Args(list_items), ..) = &**list_content
    && let XM::Token(list_props, _) = list_op.as_ref()
    && matches!(
      list_props.meaning.as_deref(),
      Some("list" | "formulae" | "vector")
    )
    && presents_its_items_alone(list_presentation, list_items.len())
    && let XM::Wrap(list_separated, ..) = &**list_presentation
    && arg_punct(list_separated)
  {
    return Some((list_items.clone(), true));
  }
  items.map(|items| (items.clone(), false))
}

/// The presentation of a group whose one item is a bare list, when a function takes the list's
/// items as its arguments (`fenced_tuple_items`): the list's items and separators take its
/// place between the delimiters, as Perl's `ApplyDelimited` lays out `open, arguments and
/// separators, close` flat — the list's own Wrap would stay behind, an id nothing refers to.
fn splice_listed_arguments(presentation: &mut XM) {
  if let XM::Wrap(fenced, ..) = presentation
    && let [_, XM::Dual(_, list_presentation, ..), _] = fenced.as_mut_slice()
    && let XM::Wrap(list_separated, ..) = &mut **list_presentation
  {
    let listed = std::mem::take(list_separated);
    fenced.splice(1..2, listed);
  }
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
  // This matches Perl's ApplyFunction XMDual structure. A multi-arg paren
  // comma-list (`Apply(vector, [refs])` content) is SPREAD into the operands
  // (`\max(a,b)` → `max@(a,b)`), per Perl ApplyDelimited.
  // A function, scripted or not, and an operator take the arguments between delimiters alike:
  // Perl attaches a head's scripts first (`preScripted[...]`, MathGrammar:282-284, :430-433) and
  // `ApplyDelimited` never looks at the head (MathParser.pm:1291-1299); an OPERATOR, nested or
  // not, reaches it through `addOpFunArgs` → `addEasyArgs` (:312-313, :553-558, :571-576), and one
  // item through `nestOperators`' `OPEN Expression balancedClose` (:669-671) — `\max_i\{a_i,b_i\}`
  // `(maximum _ i)@(a _ i, b _ i)`, physics `\Re[\frac XY]` `real-part@(X / Y)` (57bf; 2605.02221,
  // 2605.20994).
  let takes_arguments = prefixop.as_ref().is_some_and(takes_delimited_arguments);
  if takes_arguments
    && let Some(XM::Dual(ref content, ref pres, ..)) = arg1
    && (matches!(**content, XM::Ref(_)) || fenced_tuple_items(content, pres, &ctxt).is_some())
    && matches!(**pres, XM::Wrap(..))
  {
    let mut func = prefixop.unwrap();
    let arg1_inner = arg1.unwrap();
    let XM::Dual(content_box, pres_box, ..) = arg1_inner else {
      unreachable!()
    };
    let mut pres_wrap = *pres_box;
    // Single fenced arg `(x)` → one operand; n-ary paren comma-list `(a,b,…)`
    // → spread its items, dropping the implicit `vector` (Perl ApplyDelimited).
    let content_args = match fenced_tuple_items(&content_box, &pres_wrap, &ctxt) {
      Some((items, listed)) => {
        if listed {
          splice_listed_arguments(&mut pres_wrap);
        }
        items
      },
      None => vec![Some(*content_box)],
    };
    let func_refs = create_xmrefs(&mut [&mut func], ctxt)?;
    let func_ref = func_refs.into_iter().next().unwrap();
    let content_apply = XM::Apply(
      func_ref.into(),
      Args(content_args),
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
  Ok(Some(XM::Apply(
    prefixop.into(),
    Args(vec![arg1]),
    XProps::default(),
    Meta::default(),
  )))
}
/// An item of an operator's bare argument, Perl `aBarearg` (MathGrammar:323-331): see
/// `is_bare_item`.
pub fn bare_argument_item(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => item);
  if item.as_ref().is_some_and(is_bare_item) {
    Ok(item)
  } else {
    Err("bare_argument_item: not an aBarearg".into())
  }
}

/// A scripted item of a trig function's bare argument, Perl `aTrigBarearg` (MathGrammar:341-348):
/// an atom, identifier, unknown or number with its scripts (`preScripted['ATOM_OR_ID']`,
/// `preScripted['UNKNOWN']`, `NUMBER addScripts`) — see `is_trig_bare_item`.
pub fn trig_bare_argument_item(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => item);
  if item.as_ref().is_some_and(is_trig_bare_item) {
    Ok(item)
  } else {
    Err("trig_bare_argument_item: not an aTrigBarearg".into())
  }
}

/// `trig_arg` juxtaposed with its next item: the bare argument goes on, Perl `moreTrigBareargs`
/// (MathGrammar:351-357) — unless the argument ends before the item (`ends_trig_argument`, divergence
/// #367): `\sin\theta_W\,C_{uB}` is sin@(θ_W)·C_uB, `\sin\theta d\theta` sin@(θ)·dθ,
/// `\sin\theta\mathbf v` sin@(θ)·v, where Perl's greedy argument takes them.
pub fn trig_argument_juxtaposition(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  if let [Some(argument), Some(item)] = args.as_slice()
    && ends_trig_argument(&product_factors(argument), item, &ctxt)
  {
    return Err("trig_argument_juxtaposition: the bare argument ends before this item".into());
  }
  if let [Some(argument), Some(item)] = args.as_slice()
    && leaves_a_trig_argument_s_chain(argument, item, &ctxt)
  {
    return Err(
      "trig_argument_juxtaposition: the ellipsis leaves the chain ending the argument".into(),
    );
  }
  // … nor after a derivative of a numeric constant, whose monomial takes the item (`numeric_monomial`,
  // `ends_in_a_differentiated_constant`): `\sin\partial_t 2\pi i\,u\,v` has no sin@((∂_t 2)·π·i)
  if let [Some(argument), Some(item)] = args.as_slice()
    && differentiated_constant_takes(product_end(argument, true), product_end(item, false))
  {
    return Err("trig_argument_juxtaposition: the derivative's constant takes this item".into());
  }
  apply_invisible_times(rule_id, args, pragmas, ctxt)
}

/// `trig_arg mulop factor_base` (and `binop`): the bare argument goes on across a MulOp or BinOp, Perl's greedy
/// `moreTrigBareargs` (MathGrammar:351-357), but no ellipsis goes on a chain that ends it (`leaves_a_trig_argument_s_chain`:
/// `\sin\log x\cdot\ldots` sin@(log@(x))·…, as `\sin\log x\ldots`).
pub fn trig_argument_across_an_operator(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  if let [Some(argument), _, Some(item)] = args.as_slice()
    && leaves_a_trig_argument_s_chain(argument, item, &ctxt)
  {
    return Err(
      "trig_argument_across_an_operator: the ellipsis leaves the chain ending the argument".into(),
    );
  }
  infix_apply_nary(rule_id, args, pragmas, ctxt)
}

/// Is `item` an ellipsis after a trig argument that ends in an OPFUNCTION's or operator's bare application
/// (`\sin\log x`, `\cos\log_2 x`, `\sin\max_i a_i`)? The ellipsis is that chain's, never the trig argument's: between
/// two items the chain takes the run (`trig_op_bare_elided`, `leaves_a_bare_argument`: `\sin\log x\ldots y`
/// sin@(log@(x·…·y))), and a trailing run leaves every bare argument (user ruling 15, 2026-09-29: "a trailing ellipsis
/// leaves any bare argument … an ellipsis stays inside only between two items"): `\sin\log x\ldots` sin@(log@(x))·…, as
/// `\sin\log x\cdots` sin@(log@(x))·⋯ and `\log x\ldots` log@(x)·…, whatever the run's first macro — 57cj.17 kept an
/// ID-led run in the trig argument, sin@(log@(x)·…·⋯·…), and let a `\cdots`-led one leave (57cj.17 review; latent, no
/// corpus witness). The trig actions ask it of the same pair: `trig_argument_juxtaposition` and
/// `trig_argument_across_an_operator` refuse to join, `leaves_a_trig_bare_argument` and `trig_argument_across_a_mulop`
/// refuse no stop, so exactly one reading survives. Not after a big operator's application (`\sin\log\det A\ldots`,
/// SYNC_STATUS (13)), whose operand takes an ellipsis (Q9), nor after an application to a group, which closes
/// (`\sin\log(x)\ldots` reads as `\sin x\ldots`, Q10).
fn leaves_a_trig_argument_s_chain(argument: &XM, item: &XM, ctxt: &ActionContext) -> bool {
  is_ellipsis(product_end(item, false), ctxt)
    && is_bare_operator_application(product_end(argument, true))
}

/// `trig_elided`, an ELIDEOP run inside a trig argument (57cj.17, the 57cj.16 review), juxtaposed or joined by a MulOp
/// or BinOp: the argument goes on over the run to an item it chains, as over an ellipsis ID — `\sin x\cdots y`
/// sin@(x·⋯·y) as `\sin x\ldots y` sin@(x·…·y), as Perl. Not after an argument ending in an OPFUNCTION's application,
/// whose own chain takes the run (no `trig_elidable_arg`, the grammar's: `\sin\log x\cdots\dots\cdot c` stays
/// sin@(log@((x·⋯·…)·c)), as Perl, where a second tree sin@(log@(x)·⋯·…)·c survived; nor after a run of ELIDEOPs
/// opening it, juxtaposed, `trig_ellipses`' own), and not across what ends the argument
/// (`ends_trig_argument`: a space, `\sin x\,\cdots y` and `\sin x\cdots\,y` sin@(x)·⋯·y; a `d`, `\sin x\cdots dy`; a
/// symbol of another type, `\sin x\cdots\mathbf y`) — the questions `leaves_a_bare_argument` asks of the twin that leaves the run outside, so
/// exactly one reading survives.
pub fn trig_argument_elision(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  let (Some(Some(argument)), Some(Some(item))) = (args.first(), args.last()) else {
    return Err("trig_argument_elision: missing operand".into());
  };
  if ends_trig_argument(&product_factors(argument), item, &ctxt) {
    return Err("trig_argument_elision: the bare argument ends before this item".into());
  }
  if args.len() == 3 {
    infix_apply_nary(rule_id, args, pragmas, ctxt)
  } else {
    apply_invisible_times(rule_id, args, pragmas, ctxt)
  }
}

/// Does a trig argument end in an application whose own chain an ELIDEOP run after it goes on (or leaves) — an
/// OPFUNCTION's, a bound head's or an operator's bare application, or an OPFUNCTION before a big operator, whose
/// operand takes it (`\sin\log\det A\cdots y` sin@(log@(det@(A·⋯·y))), `\sin\log\sum_i x_i\cdots y`)? The arguments
/// `trig_elidable_arg` leaves out, so `leaves_a_bare_argument` refuses no twin of theirs (`\sin\log\det A\cdots y` read
/// sin@(log@(det@(A·⋯))·y) when only the bare application was looked through).
fn ends_in_its_own_chain(argument: &XM, nodes: &[XMLNode]) -> bool {
  let last = product_end(argument, true);
  is_bare_operator_application(last) || takes_a_big_operator(last, nodes)
}

/// A big operator's application (`\det A`, `\sum_i x_i`), or an OPFUNCTION's unfenced application ending in one
/// (`\log\det A`): `ends_in_its_own_chain`.
fn takes_a_big_operator(xm: &XM, nodes: &[XMLNode]) -> bool {
  match xm {
    XM::Apply(Operator(op), Args(args), _, meta) if meta.fenced.is_none() => {
      is_bigop_or_scripted_bigop(xm, nodes)
        || is_opfunction_head(op)
          && matches!(args.as_slice(), [Some(arg)]
            if takes_a_big_operator(product_end(arg, true), nodes))
    },
    _ => false,
  }
}

/// The argument of the trig function's bare application `before` that an ELIDEOP run after it would go on
/// (`trig_elided`), down its right edge as `leaves_a_trig_bare_argument` reads it (`trig_application_ending`:
/// `\partial\sin x\cdots y`, `\sin\cos x\cdots y`, `\log\sin x\cdots\dots`).
fn trig_argument_before_a_run(before: &XM) -> Option<&XM> {
  let application = trig_application_ending(before);
  let XM::Apply(Operator(op), Args(args), _, meta) = application else {
    return None;
  };
  let head = script_nucleus(op);
  match args.as_slice() {
    [Some(arg)]
      if meta.fenced.is_none()
        && matches!(head, XM::Lexeme(..) | XM::Token(..))
        && operator_category(head) == Some("TRIGFUNCTION")
        && is_trig_argument(arg) =>
    {
      Some(arg)
    },
    _ => None,
  }
}

/// What a derivative of a constant ending a trig argument takes after it (`differentiated_constant_takes`): a numeric
/// monomial's constants take any factor (`numeric_monomial`), a function's bare argument a bare item only (juxtaposed
/// OPFUNCTIONs are separate factors, and a trig function ends a trig argument).
#[derive(Clone, Copy, PartialEq, Eq)]
enum DifferentiatedConstant {
  Monomial,
  FunctionArgument,
}

/// Does `xm` end in a derivative of a numeric constant — a number, a constant monomial a number leads, or a function's
/// bare application to a constant, down its right edge (`right_edge`): `\partial_t 2\pi`, `\partial_x 2\log 2`,
/// `\partial_x\log 2`? Its monomial or its function's argument would take a factor after it (the constant run ends
/// nothing, `crosses_a_bare_argument_end`), so a trig argument neither continues after it nor ends there:
/// `\sin\partial_t 2\pi i\,u\,v` has one reading, sin@(∂_t(2πiu))·v, where the ∂-of-constant readings survived to the
/// pragma (57cj.8 review NIT 9; latent, its probes).
fn ends_in_a_differentiated_constant(xm: &XM) -> Option<DifferentiatedConstant> {
  right_edge(xm).into_iter().find_map(|(node, _)| match node {
    XM::Apply(Operator(head), Args(args), _, meta)
      if meta.differential && is_bare_differential_operator(head) =>
    {
      match args.as_slice() {
        [Some(operand)] if is_constant(operand) => {
          if is_numeric_lead(operand) || is_numeric_monomial(operand) {
            Some(DifferentiatedConstant::Monomial)
          } else if is_bare_operator_application(operand) {
            Some(DifferentiatedConstant::FunctionArgument)
          } else {
            None
          }
        },
        _ => None,
      }
    },
    _ => None,
  })
}

/// Does the derivative of a constant ending a trig argument (`ends_in_a_differentiated_constant`) take `item` after it?
/// A numeric monomial takes a bare item or a function's application (`\sin\partial_x 2\pi\,\log v` sin@(∂_x(2π·log v))),
/// a function's bare argument a bare item only (`\sin\partial_x\log 2\log v` keeps sin@(∂_x log 2)·log v: the grammar
/// has no other reading).
fn differentiated_constant_takes(argument: &XM, item: &XM) -> bool {
  match ends_in_a_differentiated_constant(argument) {
    Some(DifferentiatedConstant::Monomial) => {
      is_trig_bare_item(item) || is_function_application(item)
    },
    Some(DifferentiatedConstant::FunctionArgument) => is_trig_bare_item(item),
    None => false,
  }
}

/// `trig_arg`'s letter applied to a group (`speculative_prefix_apply`, #18) — not across explicit space after the
/// letter, which ends the argument (#367): `\cos\phi\,(1-x)` is cos@(φ)·(1−x), as `\cos\phi\,x` is cos@(φ)·x (57cj.7
/// review; 2605.29683 A1.E17 `\cos\phi\,\bigl(10-\cos(6\theta)\bigr)\,r^{6}`, which read cos@(φ@(10−cos 6θ)); 2605.11097
/// `\sin^{2}\beta\,\big(F(\dots)-F(\dots)\big)`, 2605.15566, 2605.27600 `\cosh^{2}Z\,(dZ^{2}+d\varphi^{2})`). A tuple too,
/// whatever its items: a trig argument is an angle, so the tuple is a vector the trig value scales —
/// `\mathbf v=\cos\alpha\,(v_x,0)+\sin\alpha\,(0,v_y)` cos α·(v_x,0) + sin α·(0,v_y) (as 57cj.11 and Perl), `\cos\phi\,(x,y)`
/// cos φ·(x,y) (as Perl; an application in 57cj.10-57cj.12) (decided 2026-09-30 by the main loop under the user's ruling "the mathematically correct reading in
/// context", 2026-09-29; divergence #367; latent, no corpus witness in the 3,003 A/B sources). A derivative's operand
/// keeps an argument list's application (`is_an_argument_list`: `\sin\partial_x u\,(x,0)` sin@(∂_x(u@(x,0)))).
pub fn trig_letter_application(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  if let [Some(letter), Some(_)] = args.as_slice()
    && ends_with_space(letter, ctxt.nodes)
  {
    return Err(
      "trig_letter_application: explicit space ends the argument before the group".into(),
    );
  }
  speculative_prefix_apply(rule_id, args, pragmas, ctxt)
}

/// `trig_arg += letter_postfixed`: a letter's postfixed application to a group, not across explicit space after the
/// letter either (`letter_group_across_space`), a tuple too (`trig_letter_application`): `\cos\phi\,(1-x)!`
/// cos@(φ)·(1−x)!, `\sin^2\phi\,(1-x)!`, `\cos\phi\,(x,0)!` (57cj.8 review; latent, its probes).
pub fn trig_letter_postfixed(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => item);
  if item
    .as_ref()
    .is_some_and(|item| letter_group_across_space(item, ctxt.nodes).is_some())
  {
    return Err("trig_letter_postfixed: explicit space ends the argument before the group".into());
  }
  Ok(item)
}

/// Does explicit space follow `xm` — a positive `rpadding` on its last item, which `filter_hints`
/// folds from `\,`, `\:`, `\;`, `~`, `\ `, a `\kern`/`\hspace` under 10pt onto the token or script
/// before it? (Wider spaces, `\quad`, split the formula as a PUNCT instead; a net zero or negative
/// space, `\!`, `\,\!`, is none.)
fn ends_with_space(xm: &XM, nodes: &[libxml::tree::Node]) -> bool {
  trailing_padding(xm, nodes).is_some_and(|width| crate::util::get_xmhint_spacing(&width) > 0.0)
}

/// The space after `xm`: its last token's (`ends_with_space`), down its right edge through products and every
/// unfenced application of one argument — a derivative's is its factor's (`\sin\partial_x u\,v` sin@(∂_x u)·v, 57cj
/// review), and a bare application's its argument's, however deep (`\sin\partial_x 2\,\partial_y u\,v`
/// sin@(∂_x(2·∂_y u))·v, `\sin\partial_x 2\sin u\,v`; 57cj.6 review) — unless the application carries its own; a
/// group's, its closing delimiter's, an application to one too (`\sin\partial_x(u)\,v` sin@(∂_x(u))·v, 57cj.1 review;
/// `\sin\log(x)\,y` sin@(log(x))·y, `\sin\exp\bigl(x\bigr)\,y`, 57cj.7 review; latent, their probes; the A/B witness of
/// the rule, 2605.29683 A1.E17: `\cos\phi\,\bigl(10-\cos(6\theta)\bigr)\,r^{6}` keeps r⁶ outside).
fn trailing_padding(xm: &XM, nodes: &[libxml::tree::Node]) -> Option<String> {
  let lexeme_padding = |lex: &str| {
    lookup_lex_node(lex, nodes)
      .ok()
      .and_then(|node| node.get_attribute("rpadding"))
  };
  match product_end(xm, true) {
    XM::Lexeme(lex, _) => lexeme_padding(lex),
    XM::Token(props, _) => props.rpadding.as_deref().map(str::to_string),
    XM::Apply(Operator(op), Args(args), props, meta) => {
      if let Some(padding) = props.rpadding.as_deref() {
        return Some(padding.to_string());
      }
      if operator_category(op) == Some("POSTFIX") {
        // A postfix's space is its lexeme's: `\sin x!\,y` is sin@(x!)·y (57ca).
        return match op.as_ref() {
          XM::Lexeme(lex, _) => lexeme_padding(lex),
          _ => None,
        };
      }
      match args.as_slice() {
        [Some(argument)] if meta.fenced.is_none() => trailing_padding(argument, nodes),
        _ => None,
      }
    },
    XM::Dual(_, presentation, props, _) => props
      .rpadding
      .as_deref()
      .map(str::to_string)
      .or_else(|| trailing_padding(presentation, nodes)),
    XM::Wrap(items, ..) => items.last().and_then(|item| trailing_padding(item, nodes)),
    _ => None,
  }
}

/// Perl `aTrigBarearg`'s atoms (MathGrammar:341-348, `ATOM_OR_ID : ATOM | ID | ARRAY`, :315): an
/// atom, identifier, array, unknown or number, with its scripts — what `trig_arg` takes bare and
/// chains. Not a differential `d` (`XDIFFUNK`), which ends the argument (#367): `\sin\theta d\theta`
/// is sin@(θ)·dθ, `d\cos\theta_1 d\cos\theta_2` d·cos@(θ₁)·d·cos@(θ₂), where Perl's greedy chain takes it.
/// Postfixed, as `addScripts` takes it (`\sin xy!` sin@(x·y!), 57ca).
fn is_trig_bare_item(xm: &XM) -> bool {
  let xm = postfixed_operand(xm).unwrap_or(xm);
  let nucleus = script_nucleus(xm);
  matches!(nucleus, XM::Lexeme(..) | XM::Token(..))
    && matches!(
      operator_category(nucleus),
      Some("UNKNOWN" | "ID" | "XDIFFID" | "ATOM" | "ARRAY" | "NUMBER")
    )
    && !is_differential_d(xm)
}

/// The differential letter `d`, bare or scripted: the lexer's `XDIFFUNK`, which it reads as a plain
/// unknown outside an integral's operand (`util::in_an_integral_operand`).
fn is_differential_d(xm: &XM) -> bool {
  matches!(script_nucleus(xm), XM::Lexeme(lex, _)
    if lex.starts_with("XDIFFUNK:d:") || lex.starts_with("UNKNOWN:d:"))
}

/// An ellipsis — an ELIDEOP (`\cdots`) or an ellipsis ID (`\ldots`, `\dots`) — or an unfenced product of them in any
/// order: the run of ellipses that opens a trig argument, which the next ELIDEOP continues whatever the macros — the
/// grammar derives every such run (`trig_ellipses`, `trig_ellipsis_ids`), so refusing sin@(run)·⋯ never leaves a
/// formula without its tree (`\sin\cdots\cdots x` sin@(⋯·⋯·x), 57cj.12 review; `\sin\ldots\cdots x` sin@(…·⋯·x),
/// 57cj.13 review; `\sin\ldots\ldots\cdots x` sin@(…·…·⋯·x), `\sin\cdots\ldots\cdots x`, 57cj.14 review).
fn is_an_ellipsis_run(xm: &XM) -> bool {
  match xm {
    XM::Apply(Operator(op), Args(factors), _, meta)
      if meta.fenced.is_none() && factors.len() >= 2 && is_product_operator(op) =>
    {
      factors
        .iter()
        .all(|factor| factor.as_ref().is_some_and(is_an_ellipsis_run))
    },
    XM::Lexeme(lex, _) if lex.starts_with("ID:") => lex
      .split(':')
      .nth(1)
      .is_some_and(|name| ELLIPSIS_NAMES.contains(&name)),
    _ => operator_category(xm) == Some("ELIDEOP"),
  }
}

/// An unfenced juxtaposed product of a trig argument's items — what a large MULOP's application takes as its operand in a
/// bare argument's chain (`apply_invisible_times`, Q11).
fn is_juxtaposed_trig_items(xm: &XM) -> bool {
  matches!(xm, XM::Apply(Operator(op), Args(factors), _, meta)
  if meta.fenced.is_none() && factors.len() >= 2 && is_invisible_times_op(op)
    && factors.iter().all(|factor| factor.as_ref().is_some_and(|factor| {
      is_trig_bare_item(factor) || operator_category(factor) == Some("ELIDEOP")
    })))
}

/// What `trig_arg` derives: a bare item or a function's application, or a product of them whose
/// later factors are bare items or ELIDEOPs (`trig_elided`: `\sin x\cdots y`) — or a run of ellipses opening it
/// (`trig_ellipses`: `\cos\cdots x`).
fn is_trig_argument(xm: &XM) -> bool {
  if is_an_ellipsis_run(xm) {
    return true;
  }
  match xm {
    XM::Apply(Operator(op), Args(factors), _, meta)
      if meta.fenced.is_none() && factors.len() >= 2 && is_product_operator(op) =>
    {
      // (a large MULOP's operand is the juxtaposed items after it, Q11: `\sin x\otimes 2\pi t` sin@(x⊗(2πt)), #396; any
      // visible product operator is admitted here, but only a large MULOP's application gets a juxtaposed operand in a
      // trig chain — `apply_invisible_times` pushes into no other — so `\sin x\cdot y z`, `\sin x\boxast y z` read as before)
      let explicit = !is_invisible_times_op(op);
      let mut factors = factors.iter();
      factors
        .next()
        .and_then(Option::as_ref)
        .is_some_and(is_trig_argument)
        && factors.all(|factor| {
          factor.as_ref().is_some_and(|factor| {
            is_trig_bare_item(factor)
              || operator_category(factor) == Some("ELIDEOP")
              || explicit && is_juxtaposed_trig_items(factor)
          })
        })
    },
    _ => {
      let xm = postfixed_operand(xm).unwrap_or(xm);
      is_trig_bare_item(xm)
        // (a partial derivative, `trig_arg += diffop_application`: `\sin\partial_x u`, 57cj review)
        || is_partial_derivative(xm)
        || matches!(xm, XM::Apply(..) | XM::Dual(..))
          && matches!(
            head_category(xm),
            Some("FUNCTION" | "OPFUNCTION" | "UNKNOWN" | "XDIFFUNK")
          )
    },
  }
}

/// Perl's trig bare argument is greedy (`moreTrigBareargs`, MathGrammar:351-357): a product
/// `left · right` whose `left` ends in a trig function's (bare or scripted) bare application and
/// whose `right` starts with an item `trig_arg` would take is not a parse — `\cos 2\theta_i` is
/// cos@(2·θ_i), `a\cos 2\theta` a·cos@(2θ), `\sin^2 2x` (sin²)@(2x) (57bo). Not where the argument
/// ends before that item (`ends_trig_argument`, the same question `trig_argument_juxtaposition` asks,
/// so exactly one reading survives; #367), nor after a group (`\sin(x)y`: `trig_factor_arg`, which
/// takes no chain).
fn leaves_a_trig_bare_argument(left: &XM, right: &XM, ctxt: &ActionContext) -> bool {
  let application = trig_application_ending(product_end(left, true));
  let XM::Apply(Operator(op), Args(args), _, meta) = application else {
    return false;
  };
  let head = script_nucleus(op);
  let item = product_end(right, false);
  meta.fenced.is_none()
    && matches!(head, XM::Lexeme(..) | XM::Token(..))
    && operator_category(head) == Some("TRIGFUNCTION")
    && matches!(args.as_slice(), [Some(arg)]
      // an unapplied OPFUNCTION ending the argument takes what it applies to (`trig_function_item`):
      // `\sin\log_2(x)` sin@(log₂(x)), not sin@(log₂)·x, `\sin\log\max(a,x)` sin@(log(max(a,x))) (57cj.8 review;
      // latent, no corpus witness)
      if right_edge(arg).last().is_some_and(|(leaf, _)| is_opfunction_head(leaf))
        && is_an_opfunction_argument(item)
        || is_trig_argument(arg)
          && (is_trig_bare_item(item)
            // (an ELIDEOP continues a run of them, any mix, `trig_ellipses`: `\sin\cdots\cdots x` has no sin@(⋯)·⋯·x,
            // `\sin\ldots\ldots\cdots x` no sin@(…·…)·⋯·x)
            || is_an_ellipsis_run(arg) && operator_category(item) == Some("ELIDEOP"))
          && (!ends_trig_argument(&product_factors(arg), item, ctxt)
            // … and a bound head's argument a mention of its variable (`ends_a_trig_argument_within`)
            || right_edge_binds(arg, item, ctxt))
          // (an ellipsis after a chain ending the argument is the chain's, `leaves_a_trig_argument_s_chain`)
          && !leaves_a_trig_argument_s_chain(arg, item, ctxt)
        // … and a derivative of a numeric constant ending it, what its monomial takes, across a space or a
        // type mark too (`ends_in_a_differentiated_constant`): `\sin\partial_t 2\pi i\,u\,v` sin@(∂_t(2πiu))·v
        || differentiated_constant_takes(arg, item))
}

/// A product `left ∘ right` across a MulOp or BinOp whose `left` ends in a trig function's bare application
/// (`trig_application_ending`) and whose `right` starts with an item its argument takes across the MulOp
/// (`trig_arg mulop factor_base`, `trig_scripted_item`, `trig_postfixed`): `\sin x\cdot y` is sin@(x·y), not
/// sin@(x)·y, as Perl's greedy `moreTrigBareargs` (MathGrammar:351-357) — the twin `leaves_a_trig_bare_argument`
/// refuses across juxtaposition. Where the argument ends before the item (`ends_trig_argument`: a space, a `d`, a
/// symbol of another type) both readings stay, the student pragmas choosing (the MulOp route asks nothing). Both trees
/// used to reach the pragmas, which chose by the letters around them: `\sin\log a_i\cdot\ldots\cdot\cdots\cdot\ldots`
/// read sin@(log@(a_i))·…·⋯·… where `\sin\log x\cdot\ldots\cdot\cdots\cdot\ldots` read sin@(log@(x)·…·⋯·…) (57cj.17).
fn trig_argument_across_a_mulop(left: &XM, right: &XM, ctxt: &ActionContext) -> bool {
  let XM::Apply(Operator(op), Args(args), _, meta) =
    trig_application_ending(product_end(left, true))
  else {
    return false;
  };
  let head = script_nucleus(op);
  let item = product_end(right, false);
  meta.fenced.is_none()
    && matches!(head, XM::Lexeme(..) | XM::Token(..))
    && operator_category(head) == Some("TRIGFUNCTION")
    && matches!(args.as_slice(), [Some(arg)]
      if is_trig_argument(arg)
        && is_trig_bare_item(item)
        && !ends_trig_argument(&product_factors(arg), item, ctxt)
        && !leaves_a_trig_argument_s_chain(arg, item, ctxt))
}

/// The bare application whose argument an item after `xm` would go on: `xm`, or down its right edge — the one factor a
/// differential operator takes (`\partial\sin x y` is ∂(sin(x y)), #374), a composition's inner trig function, whose
/// argument is the one that goes on (`trig_composed_arg`: `\sin\cos x y` is sin@(cos@(x y)), not sin@(cos@(x))·y,
/// 57cj.10 review), and the last item of an OPFUNCTION's or operator's bare argument, a trig function's application
/// whose argument goes on where that chain does not (`op_bare_next` takes no ellipsis): `\log\sin x\ldots\ldots` is
/// log@(sin@(x·…·…)), as Perl, where log@(sin@(x))·…·… survived beside it and a second site flipped the reading
/// (SYNC_STATUS "Math-parse residuals of the 57cj train" (10); 57cj.16 review), `\log\sin x\cdots\dots` log@(sin@(x·⋯·…))
/// (57cj.17).
fn trig_application_ending(xm: &XM) -> &XM {
  let application = through_differentials(xm);
  if let Some(inner) = composed_trig_application(application) {
    return trig_application_ending(inner);
  }
  if is_bare_operator_application(application)
    && let XM::Apply(_, Args(args), _, meta) = application
    && meta.fenced.is_none()
    && let [Some(argument)] = args.as_slice()
  {
    let inner = trig_application_ending(product_end(argument, true));
    if is_trig_application(inner) {
      return inner;
    }
  }
  application
}

/// A trig function's bare application to another's (`trig_composed_arg`: `\sin\cos x`, `\sin^2\cos x`): the inner
/// application.
fn composed_trig_application(xm: &XM) -> Option<&XM> {
  let is_trig_application = |xm: &XM| {
    matches!(xm, XM::Apply(Operator(op), _, _, meta)
      if meta.fenced.is_none() && is_bare_function_head(op) && head_category(op) == Some("TRIGFUNCTION"))
  };
  match xm {
    XM::Apply(_, Args(args), ..) if is_trig_application(xm) => match args.as_slice() {
      [Some(inner)] if is_trig_application(inner) => Some(inner),
      _ => None,
    },
    _ => None,
  }
}

/// An OPFUNCTION's or trig function's application (not a bare head): what a numeric monomial takes after its
/// constants (`\sin\partial_x 2\log 2\pi\,u\,v`).
fn is_function_application(xm: &XM) -> bool {
  !is_bare_function_head(xm)
    && matches!(xm, XM::Apply(..) | XM::Dual(..))
    && matches!(head_category(xm), Some("OPFUNCTION" | "TRIGFUNCTION"))
}

/// What an OPFUNCTION applies to right after it: a bare item or a group, scripted or not (`opfunction_application`,
/// `scripted_group`: `\sin\log_2(x)^2` sin@(log₂((x)²))).
fn is_an_opfunction_argument(item: &XM) -> bool {
  let item = script_nucleus(item);
  is_bare_item(item)
    || matches!(item, XM::Wrap(..))
    || matches!(item, XM::Dual(_, presentation, ..) if matches!(**presentation, XM::Wrap(..)))
}

/// Does a head on `argument`'s right edge — its last factor's, or one of the bare applications that factor ends in —
/// bind a variable `item` mentions (`mentions_a_bound_variable`)? Its scope follows the variable across a space in a
/// trig argument (`ends_a_trig_argument_within`): `\sin\max_i u_i\,v_i` sin@(max_i(u_i·v_i)) (57cj.8 review).
fn right_edge_binds(argument: &XM, item: &XM, ctxt: &ActionContext) -> bool {
  let mut xm = product_end(argument, true);
  while let XM::Apply(Operator(head), Args(args), _, meta) = xm
    && meta.fenced.is_none()
    && let [Some(operand)] = args.as_slice()
  {
    if mentions_a_bound_variable(head, item, ctxt) {
      return true;
    }
    xm = product_end(operand, true);
  }
  false
}

/// Does the bare argument of a trig function end before `item`? Perl's `moreTrigBareargs`
/// (MathGrammar:351-357) is greedy and never looks at an item's type; the argument ends here where
/// the source or the item's type says so (divergence #367):
///   - explicit space after the argument so far, or before a number (`\cos\theta\;1`: the space is
///     the number token's own text), or a differential `d` (`\sin\theta d\theta`);
///   - type evidence (user direction 2026-09-29: prune by the types in a compound argument): the
///     item carries a mark no scalar angle does (`non_scalar_mark`) — a vector's, operator's or
///     set's font or accent, a derivative, a fraction holding a trig function, an adjoint — and the
///     argument so far holds something besides numbers (`\sin 2\mathcal P` stays whole) and no item
///     with the same mark (`\cosh\mathcal K\mathcal S`, an all-upright document, stay whole).
///
/// `\cos\phi_m\vec e_{x_m}` is cos@(φ_m)·e⃗, `\sin\theta\mathrm P_1` sin@(θ)·P_1 (57bq; 2605.31180,
/// 2605.28946). Both trig actions ask this one question of the same pair — `trig_argument_juxtaposition`
/// (join) and `leaves_a_trig_bare_argument` (stop) — so exactly one reading survives.
///
/// `factors` are the argument's so far (`product_factors`), a derivative's monomial's up to the item
/// (`trig_derivative_item`).
fn ends_trig_argument(factors: &[&XM], item: &XM, ctxt: &ActionContext) -> bool {
  // A postfixed item is its operand, scripts kept (57ca).
  let mut item = item;
  while let Some(base) = postfix_base(item) {
    item = base;
  }
  if factors
    .last()
    .is_some_and(|last| ends_with_space(last, ctxt.nodes))
    || starts_with_space(item, ctxt.nodes)
    || is_differential_d(item)
  {
    return true;
  }
  let Some(mark) = non_scalar_mark(item, ctxt) else {
    return false;
  };
  constant_run(factors) < factors.len()
    && factors
      .iter()
      .all(|factor| non_scalar_mark(factor, ctxt) != Some(mark))
}

/// A constant (57cj.8 review; latent, its probes): a number — NUMBER, or the lexer's ATOM_NUMBER, a fraction, root or
/// power of numbers and π (util.rs `is_numeric_constant`, the same notion) — or π, raised to a constant power or not
/// (`2^3`, `\pi^2`, `\sqrt2^3`; not `\pi^a` or `\pi_a`, a field or a policy); a sum, quotient or product of constants,
/// an imaginary unit right after one counting (`2\pi i`, `constant_run`); a group holding one; a function applied to
/// one (`\log 2`, `\cos\frac{\pi}{4}`). What only scales an angle, the trig argument so far holding no angle yet
/// (`\cos 2\pi\mathbf k\cdot\mathbf r`, 57bq review); what a derivative's monomial takes as it takes a number, a derivative
/// of it differentiating a constant (`crosses_a_bare_argument_end`, `differentiates_a_number`; 57cj.7 review).
fn is_constant(xm: &XM) -> bool {
  match xm {
    XM::Lexeme(..) | XM::Token(..) => is_numeric_factor(xm) || is_pi(xm),
    XM::Apply(Operator(op), Args(args), ..) => {
      let args: Vec<&XM> = args.iter().flatten().collect();
      if args.is_empty() {
        return false;
      }
      match operator_category(op) {
        Some("SUPERSCRIPTOP") => args.len() == 2 && args.iter().all(|arg| is_constant(arg)),
        Some("ADDOP" | "FRACOP") => args.iter().all(|arg| is_constant(arg)),
        _ if is_product_operator(op) => constant_run(&args) == args.len(),
        _ => is_bare_function_head(op) && matches!(args.as_slice(), [arg] if is_constant(arg)),
      }
    },
    XM::Dual(_, presentation, ..) => is_constant(presentation),
    XM::Wrap(items, ..) => {
      let inner: Vec<&XM> = items
        .iter()
        .filter(|item| !matches!(operator_category(item), Some("OPEN" | "CLOSE" | "PUNCT")))
        .collect();
      !inner.is_empty() && inner.iter().all(|item| is_constant(item))
    },
    _ => false,
  }
}

/// π, its lexeme (`\pi`, name `pi`).
fn is_pi(xm: &XM) -> bool {
  matches!(xm, XM::Lexeme(lex, _) if lex.split(':').nth(1) == Some("pi"))
}

/// The imaginary unit, unscripted: `i`, upright or italic, `\imath`, or a token meaning it — a constant right after
/// another (`constant_run`; util.rs `is_imaginary_unit_token`, the lexer's). 373 formulas with π and an `i` after it in
/// the delta A/B's 3,003 papers, the review's sample of 25 all the unit (`\frac{1}{2\pi i}`, `e^{-2\pi i(kx+\eta v)}`;
/// 57cj.8 review).
fn is_imaginary_unit(xm: &XM) -> bool {
  matches!(xm, XM::Lexeme(lex, _)
    if matches!(lex.split(':').nth(1), Some("i" | "imath" | "imaginary-unit")))
}

/// How many of `factors` lead as constants (`is_constant`), an imaginary unit right after one counting
/// (`\sin\partial_t 2\pi i\,u\,v` sin@(∂_t(2πiu))·v, 57cj.8 review; a leading `i` is a letter, `\sin\partial_x i\,u\,v`).
fn constant_run(factors: &[&XM]) -> usize {
  let mut run = 0;
  for factor in factors {
    if is_constant(factor) || run > 0 && is_imaginary_unit(factor) {
      run += 1;
    } else {
      break;
    }
  }
  run
}

/// The factors of an unfenced product, `xm` alone when it is none.
fn product_factors(xm: &XM) -> Vec<&XM> {
  match xm {
    XM::Apply(Operator(op), Args(args), _, meta)
      if meta.fenced.is_none() && args.len() >= 2 && is_product_operator(op) =>
    {
      args.iter().flatten().flat_map(product_factors).collect()
    },
    _ => vec![xm],
  }
}

/// A number whose own text starts with space: the lexer keeps `\;` or `\,` before a number inside
/// the number token (`\cos\theta\;1`, 2605.14924, 2605.26410).
fn starts_with_space(xm: &XM, nodes: &[libxml::tree::Node]) -> bool {
  match script_nucleus(product_end(xm, false)) {
    XM::Lexeme(lex, _) if lex.starts_with("NUMBER:") => {
      lookup_lex_node(lex, nodes).ok().is_some_and(|node| {
        crate::data::resolve_xmref(node)
          .unwrap_or_else(|| node.clone())
          .get_content()
          .starts_with(char::is_whitespace)
      })
    },
    _ => false,
  }
}

/// A mark in the source that a symbol is no scalar angle: the class of font or accent the author gave
/// it, or its shape — bold, calligraphic, script, blackboard, fraktur, sans-serif, an upright Latin
/// letter (`\mathrm`, `\mathtt`); a `\vec`/`\overrightarrow`, `\hat`/`\widehat` or `\dot`/`\ddot` accent; a Leibniz
/// derivative `\frac{\partial Y}{\partial\theta}`; a fraction or other built atom holding a trig
/// function (Perl's `aTrigBarearg` refuses a trig function, MathGrammar:339); an adjoint or transpose
/// `A^\dagger`, `A^\top`, `A^{\mathrm T}`. Not `\tilde`, `\bar`, a Greek capital or a capital italic
/// letter, which name angles too (`\cos\omega\tilde t`, `\cos\omega T`). Read from the lexeme's node
/// through an XMRef; a scripted item's from its base (and its superscript, for the adjoint).
fn non_scalar_mark(item: &XM, ctxt: &ActionContext) -> Option<&'static str> {
  if let XM::Apply(Operator(op), Args(args), ..) = item
    && operator_category(op) == Some("SUPERSCRIPTOP")
    && let [Some(_), Some(script)] = args.as_slice()
    && is_adjoint_mark(script, ctxt)
  {
    return Some("adjoint");
  }
  if let Some(base) = script_base(item) {
    return non_scalar_mark(base, ctxt);
  }
  let XM::Lexeme(lex, _) = item else {
    return None;
  };
  let node = lookup_lex_node(lex, ctxt.nodes).ok()?;
  node_mark(
    &crate::data::resolve_xmref(node).unwrap_or_else(|| node.clone()),
    ctxt.document,
  )
}

/// The mark a lexeme's node carries (see `non_scalar_mark`). A token's font is its `_font`, decoded
/// (the `font` attribute is written after the parse).
fn node_mark(node: &libxml::tree::Node, document: &Document) -> Option<&'static str> {
  match node.get_name().as_str() {
    "XMTok" => token_font_mark(node, document),
    "XMDual" => node
      .get_child_elements()
      .get(1)
      .and_then(|presentation| node_mark(presentation, document)),
    "XMApp" => {
      let children = node.get_child_elements();
      let head = children.first()?;
      if head.get_attribute("role").as_deref() == Some("OVERACCENT") {
        return match head.get_attribute("name").as_deref() {
          Some("vec" | "overrightarrow") => Some("vector-accent"),
          Some("hat" | "widehat") => Some("hat-accent"),
          Some("dot" | "ddot") => Some("dot-accent"),
          _ => children.get(1).and_then(|base| node_mark(base, document)),
        };
      }
      if head.get_attribute("meaning").as_deref() == Some("divide")
        && is_leibniz_fraction(&children[1..])
      {
        return Some("derivative");
      }
      holds_trig_function(node).then_some("trig")
    },
    "XMWrap" | "XMArg" => holds_trig_function(node).then_some("trig"),
    _ => None,
  }
}

/// A token's font class: bold; a calligraphic, script, blackboard, fraktur or sans-serif family; or an
/// upright Latin letter (`\mathrm{P}`; an italic letter is the default).
fn token_font_mark(token: &libxml::tree::Node, document: &Document) -> Option<&'static str> {
  let font = token
    .get_attribute("_font")
    .and_then(|hash| document.decode_font(&hash))?;
  if font.get_series().is_some_and(|series| series == "bold") {
    return Some("bold");
  }
  if let Some(family) = font.get_family()
    && let Some(mark) = [
      "caligraphic",
      "script",
      "blackboard",
      "fraktur",
      "sansserif",
    ]
    .into_iter()
    .find(|mark| family == mark)
  {
    return Some(mark);
  }
  (font.get_shape().is_some_and(|shape| shape == "upright")
    && is_ascii_letter(&token.get_content()))
  .then_some("upright")
}

/// A fraction whose numerator and denominator each start with a differential — `d`, `∂`, `δ`, upright
/// or not — and whose denominator is a product or an application (`\frac{d\theta}{dt}`,
/// `\frac{\partial^2 u}{\partial x^2}`, not `\frac{d_1}{d_2}`). The parts are parsed by now: a
/// product's first factor, an application's head, a script's base lead them (57bq review).
fn is_leibniz_fraction(parts: &[libxml::tree::Node]) -> bool {
  let [numerator, denominator] = parts else {
    return false;
  };
  let is_differential = |part: &libxml::tree::Node| {
    leading_token(part).is_some_and(|token| {
      token.get_attribute("role").as_deref() == Some("DIFFOP")
        || matches!(token.get_content().as_str(), "d" | "\u{2202}" | "\u{3B4}")
    })
  };
  is_differential(numerator)
    && is_differential(denominator)
    && content_of(denominator).is_some_and(|node| {
      node.get_name() == "XMApp"
        && node.get_child_elements().first().is_some_and(|head| {
          !head
            .get_attribute("role")
            .is_some_and(|role| role.ends_with("SCRIPTOP"))
        })
    })
}

/// The node an `XMArg` or `XMWrap` of one item stands for.
fn content_of(node: &libxml::tree::Node) -> Option<libxml::tree::Node> {
  match node.get_name().as_str() {
    "XMArg" | "XMWrap" => match node.get_child_elements().as_slice() {
      [only] => content_of(only),
      _ => None,
    },
    _ => Some(node.clone()),
  }
}

/// The token a parsed part starts with: a product's first factor's, a script's base's, an
/// application's head's (a function or a differential operator), a Dual's presentation's.
fn leading_token(node: &libxml::tree::Node) -> Option<libxml::tree::Node> {
  match node.get_name().as_str() {
    "XMTok" => Some(node.clone()),
    "XMApp" => {
      let children = node.get_child_elements();
      let head = children.first()?;
      let role = head.get_attribute("role").unwrap_or_default();
      let is_product = head.get_attribute("meaning").as_deref() == Some("times") || role == "MULOP";
      if is_product || role.ends_with("SCRIPTOP") {
        children.get(1).and_then(leading_token)
      } else {
        leading_token(head)
      }
    },
    "XMDual" => node.get_child_elements().get(1).and_then(leading_token),
    _ => node.get_child_elements().first().and_then(leading_token),
  }
}

/// A single ASCII letter.
fn is_ascii_letter(text: &str) -> bool {
  let mut chars = text.chars();
  matches!((chars.next(), chars.next()), (Some(c), None) if c.is_ascii_alphabetic())
}

/// Does a built atom hold a trig function anywhere?
fn holds_trig_function(node: &libxml::tree::Node) -> bool {
  node.get_child_elements().iter().any(|child| {
    (child.get_name() == "XMTok" && child.get_attribute("role").as_deref() == Some("TRIGFUNCTION"))
      || holds_trig_function(child)
  })
}

/// A superscript that marks an adjoint or a transpose: `\dagger`, `\top`, `\intercal`, or an upright
/// or sans-serif `T`.
fn is_adjoint_mark(script: &XM, ctxt: &ActionContext) -> bool {
  let XM::Lexeme(lex, _) = script else {
    return false;
  };
  let Ok(node) = lookup_lex_node(lex, ctxt.nodes) else {
    return false;
  };
  // A split row's content branch reaches the script through an XMRef to its whole script node.
  let mut node = crate::data::resolve_xmref(node).unwrap_or_else(|| node.clone());
  while node.get_name() != "XMTok" {
    match node.get_child_elements().as_slice() {
      [.., last] if node.get_name() == "XMApp" => node = last.clone(),
      [only] => node = only.clone(),
      _ => return false,
    }
  }
  matches!(
    node.get_attribute("name").as_deref(),
    Some("dagger" | "top" | "intercal")
  ) || matches!(
    node.get_attribute("meaning").as_deref(),
    Some("top" | "transpose" | "adjoint")
  ) || matches!(
    node.get_content().as_str(),
    "\u{2020}" | "\u{22A4}" | "\u{22BA}"
  ) || node.get_content() == "T"
    && matches!(
      token_font_mark(&node, ctxt.document),
      Some("upright" | "sansserif")
    )
}

/// Perl `addOpFunArgs` (MathGrammar:553-558): an operator or an OPFUNCTION applies to a
/// parenthesized group or a bare argument (`APPLYOP(?) barearg`), never to an operator. While its nest is open — no
/// function nested yet — `nestOperators` (:663-671; `compound_operator`) takes a leading function
/// or operator into it instead, so the argument cannot start with one: `\nabla\log p\cdot v` is
/// (∇@log)@(p·v), `\nabla\nabla^2 u` (∇@∇²)@(u).
pub fn operator_bare_apply(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  if let [Some(head), Some(arg)] = args.as_slice()
    && (is_operator_head(arg)
      || nest_is_open(head) && {
        // through a postfix: `\nabla\log^2!` nests as `\nabla\log^2` does (57ca); an expectation's
        // application it takes whole (`open_op_head expectation_application`, builder.rs)
        let first = product_end(arg, false);
        matches!(
          head_category(postfixed_operand(first).unwrap_or(first)),
          Some("FUNCTION" | "OPFUNCTION" | "TRIGFUNCTION")
        )
      })
  {
    return Err("operator_bare_apply: the operator nests over it, or takes no operator".into());
  }
  // An ellipsis ends no bare argument (`is_bare_continuation`): an ID one, which `op_bare_next` reads
  // as any identifier, leaves as an ELIDEOP does — `\nabla u\ldots` ∇@(u)·…, `\log x_1\ldots\log x_n`
  // log@(x_1)·…·log@(x_n) (57cb review).
  if let [_, Some(arg)] = args.as_slice()
    && product_factors(arg).len() >= 2
    && {
      // under a script or a postfix too, as `is_bare_continuation` reads it: `\log x\ldots!`
      let end = product_end(arg, true);
      is_ellipsis(script_nucleus(postfixed_operand(end).unwrap_or(end)), &ctxt)
    }
  {
    return Err("operator_bare_apply: an ellipsis ends no bare argument".into());
  }
  // A derivative that starts the bare argument ends where the argument ends: its numeric monomial takes no
  // OPFUNCTION or operator the argument would not take (57cj.6 review: `\log\partial_x 2u\log v` is
  // log@(∂_x(2u))·log@(v), `\nabla\partial_x 2u\,\log v`, `\max_i\partial_x 2u_i\log v`), unless the head binds a
  // variable the factor mentions (`bound_application`: `\max_i\partial_x 2u_i\log v_i`).
  if let [Some(head), Some(arg)] = args.as_slice()
    && crosses_a_bare_argument_end(
      product_end(arg, false),
      ctxt.nodes,
      // (a function's or operator's bare argument in it ends where its own head ends it)
      &mut |at: &BareBoundaryAt| {
        let first = product_end(at.next, false);
        at.within == BareBoundary::Monomial
          && is_opfunction_or_operator(first)
          && !mentions_a_bound_variable(head, first, &ctxt)
      },
    )
  {
    return Err(
      "operator_bare_apply: the derivative's monomial runs past the argument's end".into(),
    );
  }
  prefix_apply(rule_id, args, pragmas, ctxt)
}

/// `trig_arg += diffop_application` (grammar/builder.rs): a derivative in a trig function's bare argument ends
/// where that argument ends — its numeric monomial crosses no explicit space, `d` or type mark (`ends_trig_argument`,
/// #367), and takes no function or operator after it, as juxtaposed trig functions are separate factors (57cj.6
/// review: `\sin\partial_x 2u\,v` sin@(∂_x(2u))·v as `\sin\partial_x u\,v` is sin@(∂_x u)·v, `\sin\partial_t 2\pi u\,v`,
/// `\sin\partial_x 2u\mathbf v`, `\sin\partial_x 2u\cos v`); a function's or operator's bare argument in it crosses
/// none of the argument's own ends either (57cj.7 review: `\sin\partial_x\log u\,v` sin@(∂_x(log u))·v,
/// `\sin\partial_x 2\log u\,v`), and takes what its own head takes.
pub fn trig_derivative_item(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => item);
  if let Some(derivative) = &item
    && crosses_a_bare_argument_end(derivative, ctxt.nodes, &mut |at: &BareBoundaryAt| {
      ends_a_trig_argument_within(at, &ctxt)
    })
  {
    return Err(
      "trig_derivative_item: the derivative's monomial runs past the argument's end".into(),
    );
  }
  Ok(item)
}

/// `trig_arg += trig_function_item` (grammar/builder.rs): an OPFUNCTION's application in a trig function's argument,
/// as any OPFUNCTION's (Perl `aTrigBarearg : preScripted['OPFUNCTION'] addOpFunArgs`, MathGrammar:341-343) — its bare
/// argument ending where the trig argument ends (`ends_a_trig_argument_within`): `\sin\log u\,v` sin@(log u)·v,
/// `\sin\log_2 x` sin@(log₂(x)), `\sin\log 2x` sin@(log(2x)), `\sin\max_i u_i\,v_i` sin@(max_i(u_i·v_i)) (57cj.8
/// review: Rust-only unparsed or misread before, Perl parses them; latent, no corpus witness).
pub fn trig_function_item(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => item);
  if let Some(application) = &item
    && crosses_within(
      application,
      BareBoundary::UnderAHead,
      false,
      ctxt.nodes,
      &mut |at: &BareBoundaryAt| ends_a_trig_argument_within(at, &ctxt),
    )
  {
    return Err("trig_function_item: the bare argument runs past the trig argument's end".into());
  }
  Ok(item)
}

/// Does the trig argument that holds `at` end there (`trig_derivative_item`, `trig_function_item`)? Where the trig
/// argument itself would (`ends_trig_argument`: explicit space, `d`, a type mark), and before a trig function, as
/// juxtaposed trig functions are separate factors (`\sin\log x\cos y` sin@(log x)·cos y, where an OPFUNCTION's
/// argument alone takes it, `\log x\cos y` log@(x·cos y)) — unless a bound head's variable is mentioned after it, the
/// head's scope following it (user ruling 2026-09-30; `\sin\partial_x\max_i u_i\,v_i` sin@(∂_x(max_i(u_i·v_i))), 57cj.8
/// review); in a derivative's own monomial also before an OPFUNCTION or operator (57cj.6 review).
fn ends_a_trig_argument_within(at: &BareBoundaryAt, ctxt: &ActionContext) -> bool {
  let first = product_end(at.next, false);
  if at
    .head
    .is_some_and(|head| mentions_a_bound_variable(head, first, ctxt))
  {
    return false;
  }
  ends_trig_argument(at.factors, at.next, ctxt)
    || matches!(head_category(first), Some("TRIGFUNCTION"))
    || at.within == BareBoundary::Monomial && is_opfunction_or_operator(first)
}

/// Where a boundary between two factors lies (`crosses_a_bare_argument_end`): in a derivative's numeric monomial the
/// enclosing argument holds, or under a function's or operator's head nested in it, whose own bare argument that head
/// ends — only what ends the whole enclosing argument ends it there.
#[derive(Clone, Copy, PartialEq, Eq)]
enum BareBoundary {
  Monomial,
  UnderAHead,
}

/// A boundary `crosses_a_bare_argument_end` meets: the factors so far, the next, where it lies, and the head whose
/// bare argument holds it (none in a monomial, or between a letter and its group).
struct BareBoundaryAt<'a> {
  factors: &'a [&'a XM],
  next:    &'a XM,
  within:  BareBoundary,
  head:    Option<&'a XM>,
}

/// Does a partial derivative's numeric monomial — or one a derivative in it holds, or a function's or operator's
/// bare argument in either, or a letter's application to a group — run past an enclosing bare argument's end, `ends`
/// at a boundary between two of its factors? Not where everything under the derivative before it is constant: ending
/// there would differentiate a constant, so a space there ends nothing (`\sin\partial_x 2\,\partial_y u`
/// sin@(∂_x(2·∂_y u)), `\sin\partial_x 2\,u\,v` sin@(∂_x(2u))·v, 57cj.6 review; `\sin\partial_x 2\pi\,u\,v`
/// sin@(∂_x(2πu))·v, 57cj.7 review), judged down the whole path — through a function's argument too (`\sin\partial_x
/// \log 2\,u\,v` sin@(∂_x(log(2u)))·v, `\sin\partial_x 2\log 2\pi\,u\,v`; 57cj.8 review; `is_constant`; latent, no
/// corpus witness). A letter's application to a group across a space crosses that space (`\sin\partial_x u\,(1-x)`
/// sin@(∂_x u)·(1−x), 57cj.8 review).
fn crosses_a_bare_argument_end(
  derivative: &XM,
  nodes: &[XMLNode],
  ends: &mut dyn FnMut(&BareBoundaryAt) -> bool,
) -> bool {
  is_partial_derivative(derivative)
    && crosses_within(derivative, BareBoundary::Monomial, true, nodes, ends)
}

/// `crosses_a_bare_argument_end` at `xm`, whose boundaries lie `within` a monomial the enclosing argument holds or
/// under a head nested in it — a partial derivative's operand, a function's or operator's bare argument, a letter's
/// group — with everything under the derivative before `xm` constant or not (`constant_before`).
fn crosses_within(
  xm: &XM,
  within: BareBoundary,
  constant_before: bool,
  nodes: &[XMLNode],
  ends: &mut dyn FnMut(&BareBoundaryAt) -> bool,
) -> bool {
  if let Some(base) = postfix_base(xm) {
    return crosses_within(base, within, constant_before, nodes, ends);
  }
  if let Some((letter, group)) = letter_application(xm)
    && !is_an_argument_list(group, nodes)
  {
    return ends(&BareBoundaryAt {
      factors: &[letter],
      next: group,
      within,
      head: None,
    });
  }
  let XM::Apply(Operator(head), Args(args), _, meta) = xm else {
    return false;
  };
  let [Some(operand)] = args.as_slice() else {
    return false;
  };
  if is_partial_derivative(xm) {
    // (a derivative's operand starts its own run: `\partial_y 3` is a derivative of a constant)
    return if is_numeric_monomial(operand) {
      product_crosses(operand, true, within, None, nodes, ends)
    } else {
      crosses_within(operand, within, true, nodes, ends)
    };
  }
  // a function's or operator's application to a bare argument: `\log u\,v`, `\nabla u\,v`
  meta.fenced.is_none()
    && !meta.differential
    && !is_bare_differential_operator(head)
    && matches!(
      head_category(xm),
      Some("OPFUNCTION" | "TRIGFUNCTION" | "OPERATOR")
    )
    && !matches!(operand, XM::Dual(..) | XM::Wrap(..))
    && !matches!(operand, XM::Apply(_, _, _, operand_meta) if operand_meta.fenced.is_some())
    && product_crosses(
      operand,
      constant_before,
      BareBoundary::UnderAHead,
      Some(head),
      nodes,
      ends,
    )
}

/// `crosses_within` over the factors of one bare product: any factor's own, or a boundary between them that ends
/// while something before it under the derivative is not constant (`constant_run`'s notion: an imaginary unit right
/// after a constant of this product counts).
fn product_crosses(
  product: &XM,
  constant_before: bool,
  within: BareBoundary,
  head: Option<&XM>,
  nodes: &[XMLNode],
  ends: &mut dyn FnMut(&BareBoundaryAt) -> bool,
) -> bool {
  // (the monomial's factors flat: `number tight_term` nests the tail's product, `2(πuv)`)
  let factors = product_factors(product);
  let mut constant = constant_before;
  let mut run = 0;
  for (k, factor) in factors.iter().enumerate() {
    if crosses_within(factor, within, constant, nodes, ends) {
      return true;
    }
    let constant_factor = is_constant(factor) || run > 0 && run == k && is_imaginary_unit(factor);
    run += usize::from(constant_factor && run == k);
    constant &= constant_factor;
    if !constant
      && let Some(next) = factors.get(k + 1)
      && ends(&BareBoundaryAt {
        factors: &factors[..=k],
        next,
        within,
        head,
      })
    {
      return true;
    }
  }
  false
}

/// A letter applied to its group (`speculative_prefix_apply`, #18: `u(1-x)`), as (letter, group).
fn letter_application(xm: &XM) -> Option<(&XM, &XM)> {
  match xm {
    XM::Apply(Operator(letter), Args(args), _, meta)
      if meta.fenced.is_none()
        && matches!(**letter, XM::Lexeme(..))
        && matches!(operator_category(letter), Some("UNKNOWN" | "XDIFFUNK")) =>
    {
      match args.as_slice() {
        [Some(group @ (XM::Dual(..) | XM::Wrap(..)))] => Some((&**letter, group)),
        _ => None,
      }
    },
    _ => None,
  }
}

/// The group of a letter applied to it across explicit space after the letter (`letter_application`,
/// `ends_with_space`), under its postfixes and scripts: `\phi\,(1-x)`, `\phi\,(1-x)!`. In a trig argument and a
/// derivative's operand the space ends the application (`trig_letter_postfixed`, `differential_operator_apply`; 57cj.8
/// review) — in a derivative's operand not before an argument list (`is_letter_application_across_space`) — elsewhere
/// the letter still applies (`k\,(x-y)` k@(x−y); a ruling is pending, divergence #18).
fn letter_group_across_space<'a>(xm: &'a XM, nodes: &[libxml::tree::Node]) -> Option<&'a XM> {
  let mut xm = xm;
  while let Some(base) = postfix_base(xm).or_else(|| script_base(xm)) {
    xm = base;
  }
  letter_application(xm)
    .filter(|(letter, _)| ends_with_space(letter, nodes))
    .map(|(_, group)| group)
}

/// A letter applied to a group across explicit space (`letter_group_across_space`) that is no argument list
/// (`is_an_argument_list`): what a derivative's operand refuses (`differential_operator_apply`).
fn is_letter_application_across_space(xm: &XM, nodes: &[libxml::tree::Node]) -> bool {
  letter_group_across_space(xm, nodes).is_some_and(|group| !is_an_argument_list(group, nodes))
}

/// A parenthesized comma list of variables — letters, scripted or accented letters, ellipses, placeholder slots:
/// `(x,t)`, `(x_1,\ldots,x_n)`, `(q,\dot q,t)`, `(\cdot,t)` — or an evaluation point, variables beside constants: `(0,t)`,
/// `(x,0)`, `(x,-1)`, `(x,\infty)`, `(0^+,t)` — is an argument list, which keeps a letter's application across a space in
/// a derivative's operand: no tuple holding a variable multiplies (57cj.9 review: `\partial_t u\,(x,t)` ∂_t(u@(x,t)),
/// `\sin\partial_x u\,(x,t)`; the #18 row's "an argument list keeps the application", 2605.24758; 57cj.11 review: a
/// boundary or initial condition `\partial_x u\,(0,t)=0` ∂_x(u@(0,t)), `\partial_q L\,(q,\dot q,t)`,
/// `\partial_t\psi\,(\vec r,t)`, as the bare letter's `u\,(x,0)=g(x)` u@(x,0); 57cj.12 review: `\partial_x u\,(x,\infty)`,
/// the standard `\partial_x u\,(\cdot,t)`, a one-sided limit `\partial_x u\,(0^+,t)`, `\partial_t f\,(\hat{x_1},t)`).
/// Any other group is a vector or a list the letter multiplies: constants alone (`\partial_x u\,(1,0)` (∂_x u)·(1,0),
/// where Perl reads ∂_x(u·(1,0)) and the ∂ one-factor ruling moves the tuple out, #374; π, e and i count as constants,
/// `\partial_t f\,(i,0)`), an application, a power, a sum or a scaled letter among the items (`\partial_x u\,(-x,t)`,
/// `\partial_x u\,(2x,t)`: meant as evaluation points, a residual), brackets or another separator
/// (`\partial_x u\,(x,t;\lambda)`, a parameter list, a residual) — 57cj.10-57cj.12 reviews; latent, no corpus witness.
/// A trig argument is an angle, so there a tuple after a space is a vector the trig value scales, whatever its items
/// (`trig_letter_application`, decided 2026-09-30): the rule is the derivative operand's only.
fn is_an_argument_list(group: &XM, nodes: &[XMLNode]) -> bool {
  let presentation = match group {
    XM::Dual(_, presentation, ..) => presentation.as_ref(),
    other => other,
  };
  let XM::Wrap(items, ..) = presentation else {
    return false;
  };
  let [open, inner @ .., close] = items.as_slice() else {
    return false;
  };
  let is_comma = |item: &XM| matches!(delimiter_role_text(item), Some(("PUNCT", ",")));
  let is_variable = |item: &XM| is_a_variable_item(item, nodes);
  matches!(delimiter_role_text(open), Some(("OPEN", "(")))
    && matches!(delimiter_role_text(close), Some(("CLOSE", ")")))
    && inner.iter().any(is_comma)
    && inner.iter().any(is_variable)
    && inner
      .iter()
      .all(|item| is_comma(item) || is_variable(item) || is_an_evaluation_constant(item))
}

/// A letter (an UNKNOWN, whatever its font: `x`, `\alpha`, `\mathbf{x}`; not π or an unscripted e or i, constants), an accented one
/// (`\hat x`, `\dot q`, `\vec r`, `\bar x`, `\hat{x_1}`), a scripted one (`x_1`, `x'`, `x^{(1)}`, `x_i^j`, `\hat x_1`), an
/// ellipsis (`\ldots`, `\dots`, `\cdots`) or a placeholder slot (`is_a_placeholder_slot`: `u(\cdot,t)`): an item of an
/// argument list (`is_an_argument_list`). A letter to a constant power is an expression, as a sum is:
/// `\partial_t u\,(x^2,y)` (∂_t u)·(x², y).
fn is_a_variable_item(item: &XM, nodes: &[XMLNode]) -> bool {
  is_a_variable(item, nodes)
    || operator_category(item) == Some("ELIDEOP")
    || is_a_placeholder_slot(item)
    || match item {
      XM::Lexeme(lex, _) => {
        lex.starts_with("ID:")
          && lex
            .split(':')
            .nth(1)
            .is_some_and(|name| ELLIPSIS_NAMES.contains(&name))
      },
      XM::Token(props, _) => props
        .name
        .as_deref()
        .is_some_and(|name| ELLIPSIS_NAMES.contains(&name)),
      _ => false,
    }
}

/// The mark of an argument's slot alone (a MULOP): `\cdot` or `\bullet` (`u(\cdot,t)`, `f(\bullet)`, as
/// `bar_placeholder`'s `\|\cdot\|`) or the wildcard `\ast` (`\Delta(\ast,\ast)`, a Hamming distance's wildcard slots,
/// 2605.02499; `(\ast,\ast,(\Delta^j)_{j=1}^K)`, 2605.18079 — the only operator alone in a tuple across the 3,003 A/B
/// sources). Not another product operator (`(\otimes,t)`, `(\times,t)`, `(\star,t)` stay a vector, as Perl's open
/// interval: Perl's `AnyOp` takes every operator alone before a PUNCT or CLOSE, MathGrammar:204-206, and singles out
/// none as a slot; 57cj.13 review).
fn is_a_placeholder_slot(item: &XM) -> bool {
  const SLOT_MARKS: [&str; 3] = ["cdot", "bullet", "ast"];
  matches!(operator_category(item), Some("MULOP" | "BINOP"))
    && match item {
      XM::Lexeme(lex, _) => lex
        .split(':')
        .nth(1)
        .is_some_and(|name| SLOT_MARKS.contains(&name)),
      XM::Token(props, _) => props
        .name
        .as_deref()
        .is_some_and(|name| SLOT_MARKS.contains(&name)),
      _ => false,
    }
}

/// A constant item of an evaluation point (`is_an_argument_list`): a constant (`is_constant`: a number, π, a fraction
/// or power of them), e or i (`is_e_or_i`), ∞, a signed one (`-1`, `-\infty`), or a number approached from one side,
/// a sign as its superscript (`0^+`, `L^-` is a scripted letter, a variable).
fn is_an_evaluation_constant(item: &XM) -> bool {
  match item {
    XM::Lexeme(lex, _) => {
      is_constant(item) || is_e_or_i(item) || lex.split(':').nth(1) == Some("infinity")
    },
    XM::Apply(Operator(op), Args(args), ..) => match (operator_category(op), args.as_slice()) {
      (Some("ADDOP"), [Some(operand)]) => is_an_evaluation_constant(operand),
      (Some("SUPERSCRIPTOP"), [Some(base), Some(script)]) => {
        is_constant(item) || is_constant(base) && operator_category(script) == Some("ADDOP")
      },
      _ => is_constant(item),
    },
    XM::Dual(_, presentation, ..) => is_constant(item) || is_an_evaluation_constant(presentation),
    _ => is_constant(item),
  }
}

/// An unscripted e or i, the exponential base or the imaginary unit — a constant of an evaluation point, as π is
/// (`\partial_t f\,(i,0)` a vector; `(x,i)`, `(i,j)` hold a variable).
fn is_e_or_i(xm: &XM) -> bool {
  is_imaginary_unit(xm)
    || matches!(xm, XM::Lexeme(lex, _)
      if matches!(lex.split(':').nth(1), Some("e" | "exponential-e")))
}

/// A letter, bare, accented (`is_accented_letter`) or scripted by anything but a constant power
/// (`is_a_variable_item`). An unscripted e or i is a constant (`is_e_or_i`); subscripted or primed it is a letter — an
/// index or a basis vector, `\partial_t a\,(i_1,\ldots,i_k)`, `(i',t)`, `(e_1,t)`, `(\mathbf e_1,t)` (57cj.13 review) —
/// and raised to any other power it is the exponential or a power of the imaginary unit, no variable: `(e^{x},t)`,
/// `(e^{i\theta},t)`, `(i^n,t)` are vectors, as Perl reads them (57cj.14 review). The whole script chain decides, not
/// its outer layer, so the order TeX renders alike does not matter: a letter if any layer is a subscript or a script
/// opening with a prime, and no layer a constant power — `(e^x_k,t)` as `(e_k^x,t)`, `e'^x` (TeX's one superscript
/// `e^{\prime x}`) as `{e'}^x`; `(x^2_k,t)`, `(e_1^2,t)` are powers (57cj.15 review).
fn is_a_variable(xm: &XM, nodes: &[XMLNode]) -> bool {
  if script_base(xm).is_some() {
    let mut indexes_a_letter = false;
    let mut layer = xm;
    while let XM::Apply(Operator(op), Args(args), ..) = layer
      && script_base(layer).is_some()
    {
      let [Some(base), Some(script)] = args.as_slice() else {
        return false;
      };
      let is_a_power = operator_category(op) == Some("SUPERSCRIPTOP")
        && (!matches!(script, XM::Wrap(..) | XM::Dual(..)) && is_constant(script)
          || raises_a_prime_to_a_constant(script, nodes));
      if is_a_power {
        return false;
      }
      indexes_a_letter |=
        operator_category(op) == Some("SUBSCRIPTOP") || opens_with_a_prime(script, nodes);
      layer = base;
    }
    return is_a_variable(layer, nodes) || indexes_a_letter && is_e_or_i(layer);
  }
  match xm {
    XM::Lexeme(lex, _) if operator_category(xm) == Some("ATOM") => {
      lookup_lex_node(lex, nodes).is_ok_and(is_accented_letter)
    },
    _ => operator_category(xm) == Some("UNKNOWN") && !is_pi(xm) && !is_e_or_i(xm),
  }
}

/// A superscript TeX merged a prime into, raising it to a constant: `e'^2` is TeX's one superscript `e^{\prime 2}`, the
/// constant power (e′)², which `is_constant` cannot see through the prime (`prime@(absent, 2)`, an ATOM lexeme standing
/// for the XMApp): `(e'^2,t)` is a vector as `(e_1^2,t)` is, and `(x'^2,t)` as `(x^2,y)` (57cj.16 review NIT 3; latent,
/// no primed-power tuple in the 3,003 A/B sources). What follows the prime is read as any power (`util::is_constant_node`:
/// `e'^{-1}`, `e'^{1/2}`, `e'^\pi`); `e'^x` keeps the letter, and so does a fenced superscript, an order as in
/// `is_a_variable`'s plain power (`(x'^{(2)},t)` as `(x^{(2)},t)`, 57cj.17 review NIT 4).
fn raises_a_prime_to_a_constant(script: &XM, nodes: &[XMLNode]) -> bool {
  let is_absent =
    |xm: &XM| matches!(xm, XM::Token(props, _) if props.meaning.as_deref() == Some("absent"));
  match script {
    XM::Apply(Operator(op), Args(args), ..) if operator_category(op) == Some("SUPOP") => {
      let raised: Vec<&XM> = args
        .iter()
        .flatten()
        .filter(|arg| !is_absent(arg))
        .collect();
      !raised.is_empty()
        && raised
          .iter()
          .all(|arg| !matches!(arg, XM::Wrap(..) | XM::Dual(..)) && is_constant(arg))
    },
    XM::Lexeme(lex, _) if operator_category(script) == Some("ATOM") => {
      lookup_lex_node(lex, nodes).is_ok_and(|node| {
        // (from the math idstore only, as `opens_with_a_prime`)
        let resolve = |node: &XMLNode| {
          crate::data::resolve_xmref_in_store(node).unwrap_or_else(|| node.clone())
        };
        let node = resolve(node);
        let children = element_nodes(&node);
        let Some((prime, raised)) = children.split_first() else {
          return false;
        };
        let raised: Vec<&XMLNode> = raised
          .iter()
          .filter(|child| child.get_attribute("meaning").as_deref() != Some("absent"))
          .collect();
        node.get_name() == "XMApp"
          && resolve(prime).get_attribute("role").as_deref() == Some("SUPOP")
          && !raised.is_empty()
          && raised.iter().all(|child| {
            !matches!(resolve(child).get_name().as_str(), "XMWrap" | "XMDual")
              && crate::util::is_constant_node(child)
          })
      })
    },
    _ => false,
  }
}

/// A script that is a prime or opens with one: `'`, `''` (one `prime2` SUPOP), and the superscript TeX merges a prime
/// into, `e'^x` = `e^{\prime x}` (the prime applied to what follows it, an XMApp the script's ATOM lexeme stands for).
fn opens_with_a_prime(script: &XM, nodes: &[XMLNode]) -> bool {
  let first = product_end(script, false);
  match first {
    XM::Apply(Operator(op), ..) => operator_category(op) == Some("SUPOP"),
    XM::Lexeme(lex, _) if operator_category(first) == Some("ATOM") => {
      lookup_lex_node(lex, nodes).is_ok_and(|node| {
        // (from the math idstore only, as `is_accented_letter`)
        let resolve = |node: &XMLNode| {
          crate::data::resolve_xmref_in_store(node).unwrap_or_else(|| node.clone())
        };
        let node = resolve(node);
        node.get_name() == "XMApp"
          && element_nodes(&node).first().is_some_and(|op| {
            let op = resolve(op);
            op.get_name() == "XMTok" && op.get_attribute("role").as_deref() == Some("SUPOP")
          })
      })
    },
    _ => operator_category(first) == Some("SUPOP"),
  }
}

/// An accent over a letter (`\hat x`, `\dot q`, `\vec r`, `\underline x`) or over a letter with a subscript or a
/// superscript that is no constant power, the test a bare letter's power takes (`is_a_variable`: `\hat{x_1}`, `\bar{x_i}`,
/// `\hat{x^n}`; `\hat{x^2}`, `\hat{x^{-1}}`, `\hat{x^{1/2}}` are powers, as `x^{-1}` is, 57cj.13 review): an application of an
/// OVERACCENT or UNDERACCENT to an UNKNOWN token or its script application, which lexes as one ATOM (util.rs
/// `node_to_grammar_lexemes_ctx`), read through XMRefs — a variable of an argument list (`is_a_variable`; 57cj.11
/// review: `\partial_q L\,(q,\dot q,t)`; 57cj.12 review: `\partial_t f\,(\hat{x_1},t)`).
fn is_accented_letter(node: &XMLNode) -> bool {
  // (from the math idstore only: a miss walks no document, util.rs `is_numeric_constant`'s lesson)
  let resolve =
    |node: &XMLNode| crate::data::resolve_xmref_in_store(node).unwrap_or_else(|| node.clone());
  let node = resolve(node);
  let role = |node: &XMLNode| {
    let node = resolve(node);
    (node.get_name() == "XMTok")
      .then(|| node.get_attribute("role"))
      .flatten()
  };
  let is_a_letter = |node: &XMLNode| {
    let node = resolve(node);
    role(&node).as_deref() == Some("UNKNOWN")
      || node.get_name() == "XMApp"
        && matches!(element_nodes(&node).as_slice(), [script_op, letter, script]
        if role(letter).as_deref() == Some("UNKNOWN")
          && match role(script_op).as_deref() {
            Some("SUBSCRIPTOP") => true,
            Some("SUPERSCRIPTOP") => {
              let script = resolve(script);
              matches!(script.get_name().as_str(), "XMWrap" | "XMDual")
                || !crate::util::is_constant_node(&script)
            },
            _ => false,
          })
  };
  node.get_name() == "XMApp"
    && matches!(element_nodes(&node).as_slice(), [accent, base]
      if matches!(role(accent).as_deref(), Some("OVERACCENT" | "UNDERACCENT"))
        && is_a_letter(base))
}

/// An OPFUNCTION or an operator, bare or applied (not a trig function, which continues an OPFUNCTION's bare
/// argument: `\log x\sin y` log@(x·sin@(y))).
fn is_opfunction_or_operator(xm: &XM) -> bool {
  is_operator_head(xm)
    || matches!(
      head_category(postfixed_operand(xm).unwrap_or(xm)),
      Some("OPFUNCTION" | "OPERATOR")
    )
}

/// `bound_application`, `bound_operator_application` (grammar/builder.rs): the bare application of a
/// head whose subscript binds a variable takes the next item of its argument, juxtaposed or after a
/// MulOp or BinOp — an OPFUNCTION's application that mentions a variable the head binds, or after one
/// any bare item (`op_bare_next`). User ruling 2026-09-30, "a scripted operator's scope follows its
/// bound variable": `\max_i a_i\log b_i` is max_i@(a_i·log@(b_i)), as Perl's greedy `barearg`
/// (MathGrammar:321-337, :553-558), and `\max_x f(x)\log y` stays max_x@(f@(x))·log@(y) (#376). The
/// argument and the item join as a bare argument's items do (`apply_invisible_times`,
/// `infix_apply_nary`: an application the argument ends in refuses what its own head would take, so the
/// innermost bound head takes a mention), and the head applies to the whole (`operator_bare_apply`).
pub fn bound_argument_extends(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  let (Some(Some(application @ XM::Apply(Operator(head), ..))), Some(Some(item))) =
    (args.first(), args.last())
  else {
    return Err("bound_argument_extends: no application or item".into());
  };
  if !is_bare_operator_application(application) {
    return Err("bound_argument_extends: not a bare argument".into());
  }
  if is_opfunction_application(item) && !mentions_a_bound_variable(head, item, &ctxt) {
    return Err(
      "bound_argument_extends: the application mentions no variable the head binds".into(),
    );
  }
  let mut args = args.into_iter();
  let Some(Some(XM::Apply(Operator(head), Args(mut argument), ..))) = args.next() else {
    return Err("bound_argument_extends: no application".into());
  };
  let mut joined = vec![argument.pop().flatten()];
  joined.extend(args);
  let joined = if joined.len() == 2 {
    apply_invisible_times(rule_id, joined, pragmas, ActionContext {
      nodes:    ctxt.nodes,
      document: &mut *ctxt.document,
    })?
  } else {
    infix_apply_nary(rule_id, joined, pragmas, ActionContext {
      nodes:    ctxt.nodes,
      document: &mut *ctxt.document,
    })?
  };
  operator_bare_apply(rule_id, vec![Some(*head), joined], pragmas, ctxt)
}

/// Perl `addEasyArgs` (MathGrammar:571-576): an OPFUNCTION applies to a balanced group, `OPEN …
/// CLOSE` — not a bar pair, an `aBarearg` (:330) and so the start of its greedy bare argument:
/// `\log(n)^2` is (log@(n))² (divergence #351), `\log|z|^{2}dz` log@(|z|²·dz), as Perl.
pub fn group_apply(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  if let [_, Some(arg)] = args.as_slice()
    && is_function_group(arg)
  {
    prefix_apply(rule_id, args, pragmas, ctxt)
  } else {
    Err("group_apply: not a balanced group".into())
  }
}

/// A scripted OPFUNCTION (`\min_w`, `\max_i`) applied to a scripted group, as a limit takes its
/// operand: `\min_w(y-w)^2` is min_w@((y−w)²) (divergence #351 covers only an unscripted head).
/// Only a group with scripts — a scripted bare item is the bare argument's.
pub fn scripted_group_apply(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  if let [_, Some(arg)] = args.as_slice()
    && script_base(arg).is_some()
    && is_function_group(script_nucleus(arg))
  {
    prefix_apply(rule_id, args, pragmas, ctxt)
  } else {
    Err("scripted_group_apply: not a scripted group".into())
  }
}

/// Is `head`'s nest of operators still open (Perl `nestOperators`, MathGrammar:663-671): an
/// operator, scripted or not, or one nested over operators only?
fn nest_is_open(head: &XM) -> bool {
  match (script_base(head), head) {
    (Some(base), _) => nest_is_open(base),
    (None, XM::Apply(Operator(op), args, ..)) if is_nested_operator(op, args) => {
      matches!(args.0.as_slice(),
        [Some(nested)] if !is_function_head(nested) && nest_is_open(nested))
    },
    (None, _) => is_operator_head(head),
  }
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
/// backtracking parser. See docs/parity/OXIDIZED_DESIGN.md.
pub fn speculative_prefix_apply(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => prefixop, arg1);
  // A letter whose own group holds it multiplies the group: `x(x+1)`, `n(n+1)/2`,
  // `\lambda(\lambda I-A)` are products, as Perl reads every unknown (`doubtArgs`,
  // MathGrammar:515-528) — a function is not applied to an expression in itself (57bz; ~1,190 of
  // the 1,219 formulas / 342 papers its delta A/B changed; 2605.28300, 2605.02279, 2605.04013,
  // 2605.30479, 2605.00539, 2605.31439).
  if let (Some(head), Some(group)) = (&prefixop, &arg1)
    && letter_recurs_in_its_group(head, group, &ctxt)
  {
    return Err("speculative_prefix_apply: the letter recurs in its group".into());
  }
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
    // `unknown group_factor → speculative_prefix_apply` for a group
    // only — the bar pairs are `bare_abs` — but a group between
    // bar glyphs, amsmath's `\lvert…\rvert`, is bars to the reader
    // too. Reject it so `tight_term factor → apply_invisible_times`
    // wins, giving a unique multiplication parse, as for
    // `a|a|+b|b|+c|c|`.
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
    // Nor a Dirac bracket: a letter before a bra, ket, inner product or operator product multiplies
    // it, `H|\psi\rangle` H·ket, `c\langle u|u\rangle` c·⟨u|u⟩, as Perl (which applies no unknown,
    // `doubtArgs`, MathGrammar:518) — divergence #18 applies a letter to a group (user ruling
    // 2026-09-29; 57bp, ~45 formulas / 15 papers, 2605.20339, 2605.23874, 2605.28949).
    if dirac_meaning(arg).is_some() {
      return Err("speculative_prefix_apply: a Dirac bracket is no argument".into());
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
  let XM::Dual(_, ref presentation, _, ref meta) = *arg else {
    return false;
  };
  if meta.bar_fence {
    return true;
  }
  let XM::Wrap(ref items, ..) = **presentation else {
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
  unp!(args => diffd, arg1);
  let annotated = differential_d(diffd, &ctxt)?;
  Ok(Some(XM::Apply(
    annotated.into(),
    Args(vec![arg1]),
    XProps::default(),
    Meta::for_differential(),
  )))
}

/// A differential's power before its variable, `d^3` in `\int d^3x\,f` (`raised_differential_d`, divergence #395;
/// 2605.29990, 2605.21314, 2605.23046): the annotated `d` (`differential_d`) takes the superscript as any base does,
/// `(differential-d ^ 3)` as a bound differential's `\rmd^3` reads. Only a count is a power (`is_a_power_count`), and
/// only in an integral's operand, where Perl reads `diffd` at all (`moreIntOpArgFactors`, MathGrammar:633-638), as every
/// letter `d` (the lexer's `XDIFFUNK`, `util::in_an_integral_operand`): a dimension outside the operand stays a letter's
/// power, before the integral sign, `d^2h^2\int_t^{t+h}…` (2605.07939), `\leq 9\tilde L_f^2d^2h\sum\int…` (2605.26800), or
/// after a relation, `\int f\le C d^2 n`.
pub fn differential_d_power(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => diffd, script);
  let Some(script) = script else {
    return Err("differential_d_power: no script".into());
  };
  if !matches!(&script, XM::Wrap(parts, ..) if parts.get(1).is_some_and(is_a_power_count)) {
    return Err("differential_d_power: the script is no power".into());
  }
  let annotated = differential_d(diffd, &ctxt)?;
  new_script(annotated, script, ctxt)
}

/// A differential's power (`differential_d_power`): a count — a number, a single-character letter (not the transpose
/// `T`; a named one, `alpha`, is no count), or a
/// sum, difference or product of counts, parsed or the lexer's atom of one (`d^{d-1}`, `d^{2N}`). Not a negative
/// number (`d^{-1}`), a group (`d^{(2)}`, an order as `x^{(2)}` is), a symbol (`d^\infty`, `d^\natural`, 2605.01646), an
/// accented letter (`d^{\hat n}`), a prime, a star or a dagger.
fn is_a_power_count(xm: &XM) -> bool {
  let text = |xm: &XM| -> Option<String> {
    match xm {
      XM::Lexeme(lex, _) => lex.split(':').nth(1).map(str::to_owned),
      XM::Token(props, _) => props.content.as_deref().map(str::to_owned),
      _ => None,
    }
  };
  match xm {
    XM::Lexeme(lex, _) if lex.starts_with("ATOM_NUMBER:") => {
      !lex["ATOM_NUMBER:".len()..].starts_with('-')
    },
    XM::Lexeme(..) | XM::Token(..) => match operator_category(xm) {
      Some("NUMBER") => true,
      Some("UNKNOWN" | "XDIFFUNK") => text(xm).is_some_and(|t| t.chars().count() == 1 && t != "T"),
      // the lexer's atom of a count — a number or a letter (a gathered row's content branch spells a script so,
      // `d^{4}` ATOM `4`, 2605.29990's split), or a sum or product of counts in prefix order: `-d1` for d−1, `⁢2N`
      // for 2N (a sign before one operand, `-n`, is no count)
      Some("ATOM") => text(xm).is_some_and(|t| {
        let mut chars = t.chars();
        let first = chars.next();
        let rest: Vec<char> = chars.collect();
        t.chars().all(|c| c.is_ascii_digit())
          || (t.chars().count() == 1 && t != "T" && t.chars().all(|c| c.is_ascii_alphabetic()))
          || (matches!(first, Some('-' | '+' | '\u{2062}'))
            && rest.len() >= 2
            && rest.iter().all(char::is_ascii_alphanumeric))
      }),
      _ => false,
    },
    XM::Apply(Operator(op), Args(args), ..) => {
      args.len() >= 2
        && (is_invisible_times_op(op) || operator_category(op) == Some("ADDOP"))
        && args
          .iter()
          .all(|arg| arg.as_ref().is_some_and(is_a_power_count))
    },
    _ => false,
  }
}

/// A differential's power applied to its variable, `(d³)@(x)` (`raised_differential_d`, divergence #395).
pub fn differential_power_apply(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => power, variable);
  Ok(Some(XM::Apply(
    power.into(),
    Args(vec![variable]),
    XProps::default(),
    Meta::for_differential(),
  )))
}

/// The letter `d` of an integral's differential, annotated as Perl's IntFactor does (MathGrammar:643-647):
/// role DIFFOP, meaning differential-d. Refused for any other letter; outside an integral's operand the lexer gave the
/// `d` no differential lexeme — Perl reads `diffd` only among an INTOP's arguments (`moreIntOpArgFactors`,
/// MathGrammar:633-638).
fn differential_d(diffd: Option<XM>, ctxt: &ActionContext) -> Result<Option<XM>, Box<dyn Error>> {
  // Check that the first token is literally "d"
  let is_d = diffd.as_ref().is_some_and(|xm| match xm {
    XM::Token(props, _) => props.content.as_deref() == Some("d"),
    XM::Lexeme(lex, _) => lex.split(':').nth(1) == Some("d"),
    _ => false,
  });
  if !is_d {
    return Err("diffop_apply: first token is not 'd', pruning parse".into());
  }
  // Perl: diffd is only recognized inside IntOpArgFactors (integral context): the lexer gives a `d` its differential
  // lexeme (`XDIFFUNK`/`XDIFFID`, the only one the grammar's differential rules take) in an integral's operand only,
  // `util::in_an_integral_operand` (SYNC (17)).
  // Annotate the d token: role=DIFFOP, meaning=differential-d
  Ok(match diffd {
    Some(XM::Token(mut props, meta)) => {
      props.role = Some(Cow::Borrowed("DIFFOP"));
      props.meaning = Some(Cow::Borrowed("differential-d"));
      Some(XM::Token(props, meta))
    },
    // Perl `Annotate` (MathParser.pm:1206-1235) annotates the node itself rather than a fresh
    // `d` — the token keeps its xml:id, so a content branch's XMRef to it still resolves, and
    // its font — and on an XMRef yields `<XMRef idref role meaning/>`, carrying the ref's
    // `_xmkey` as Perl `createXMRefs` does (Package.pm:1557-1561). The token's props are the
    // ones `XProps` reads. Golden
    // tests/parse/integrals_and_differentials.tex#gathered_row_keeps_its_differential (arXiv
    // 2605.01547, 2605.23309 split integrands).
    Some(XM::Lexeme(lex, meta)) => match lookup_lex_node(&lex, ctxt.nodes) {
      Ok(node) if node.get_name() == "XMRef" => Some(XM::Ref(XProps {
        id: node.get_attribute("idref").map(Cow::Owned),
        xmkey: node.get_attribute("_xmkey").map(Cow::Owned),
        role: Some(Cow::Borrowed("DIFFOP")),
        meaning: Some(Cow::Borrowed("differential-d")),
        ..XProps::default()
      })),
      Ok(node) => {
        let mut props = XProps::from(node);
        props.role = Some(Cow::Borrowed("DIFFOP"));
        props.meaning = Some(Cow::Borrowed("differential-d"));
        Some(XM::Token(props, meta))
      },
      Err(_) => Some(XM::Token(
        XProps {
          content: Some(Cow::Borrowed("d")),
          role: Some(Cow::Borrowed("DIFFOP")),
          meaning: Some(Cow::Borrowed("differential-d")),
          ..XProps::default()
        },
        meta,
      )),
    },
    other => other,
  })
}
/// Divergence #374 (user ruling 2026-09-29): a differential operator's application to the one factor
/// after it (`diffop_application`) — a finished factor, as a differential `d x` is (`diffop_apply`,
/// `Meta::differential`): the factors after it are its product's, not its operand's
/// (`\partial_x u\cdot v` (∂_x u)·v), where Perl's `bigop` takes them (MathGrammar:717, :605-618; 2605.03741,
/// 2605.08634, 2605.24774).
pub fn differential_operator_apply(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  // A letter's application to a group across explicit space is no operand of its own, nor a factor of a numeric
  // monomial's: `\partial_x u\,(1-x)` is (∂_x u)·(1−x), and `\partial_x 2u\,(1-x)` ∂_x(2·u·(1−x)), the monomial taking the
  // group as a factor (in a trig argument the space ends it: `\sin\partial_x 2u\,(1-x)` sin@(∂_x(2u))·(1−x)) — the space
  // ends the application, as in a trig argument (57cj.8 review; 2605.24151 `\partial_z\rho\,(\tfrac12|\nabla\theta|^2)`, 2605.18945;
  // `\partial_x u(1-x)` stays ∂_x(u(1−x)), #18).
  if let [_, Some(operand)] = args.as_slice() {
    let factors = if is_numeric_monomial(operand) {
      product_factors(operand)
    } else {
      vec![operand]
    };
    if factors
      .iter()
      .any(|factor| is_letter_application_across_space(factor, ctxt.nodes))
    {
      return Err(
        "differential_operator_apply: explicit space ends the letter's application".into(),
      );
    }
  }
  Ok(
    prefix_apply(rule_id, args, pragmas, ctxt)?.map(|applied| match applied {
      XM::Apply(op, args, props, _) => XM::Apply(op, args, props, Meta::for_differential()),
      other => other,
    }),
  )
}

/// A differential's or a differential operator's application (`Meta::differential`): `d x`,
/// `\partial_x u`.
fn is_differential(xm: &XM) -> bool { matches!(xm, XM::Apply(_, _, _, meta) if meta.differential) }

/// A differential operator standing alone, bare or scripted: `\partial`, `\partial^2`, `\partial_x`.
fn is_bare_differential_operator(xm: &XM) -> bool {
  let nucleus = script_nucleus(xm);
  matches!(nucleus, XM::Lexeme(..) | XM::Token(..)) && operator_category(nucleus) == Some("DIFFOP")
}

/// What a differential operator's application holds, through every differential operator
/// (`\partial\sin x` → `\sin x`): the one factor it takes ends as that factor does (divergence #374).
fn through_differentials(xm: &XM) -> &XM {
  match xm {
    XM::Apply(_, Args(args), _, meta) if meta.differential => match args.as_slice() {
      [Some(operand)] => through_differentials(operand),
      _ => xm,
    },
    _ => xm,
  }
}

/// Is `xm` — a differential operator, bare, scripted or applied — a partial one, `\partial` or an
/// accented `\bar\partial` (util.rs lexes it a DIFFOP), rather than a `d` (`d x`, iopart's `\rmd x`,
/// physics' `\dd`)? A Leibniz quotient's denominator takes the numerator's kind only (57cj review:
/// `\int_0^1\partial^n f/\partial x^n\,dx` keeps its `dx`; latent, the review's probe, no corpus witness).
fn is_partial_differential(xm: &XM) -> bool {
  let head = match xm {
    XM::Apply(Operator(head), _, _, meta) if meta.differential => head.as_ref(),
    other => other,
  };
  let nucleus = script_nucleus(head);
  operator_category(nucleus) == Some("DIFFOP")
    && match nucleus {
      XM::Lexeme(lex, _) => lex
        .split(':')
        .nth(1)
        .is_some_and(|name| name.contains("partial") || name.contains('\u{2202}')),
      XM::Token(props, _) => props
        .meaning
        .as_deref()
        .is_some_and(|meaning| meaning.contains("partial")),
      _ => false,
    }
}

/// A partial differential operator's application to its one factor (`diffop_application`,
/// `differential_operator_apply`): `\partial_x u` — a bare item an operator's, an OPFUNCTION's or a
/// trig function's bare argument takes (57cj review: `\nabla\partial_x u` ∇@(∂_x u), `\sin\partial_x u`
/// sin@(∂_x u)); not a differential `d x`, which ends a trig argument (#367).
fn is_partial_derivative(xm: &XM) -> bool {
  is_differential(xm)
    && is_partial_differential(xm)
    && matches!(xm, XM::Apply(Operator(head), ..) if is_bare_differential_operator(head))
}

/// A number standing as a factor: a NUMBER, or a fraction or root of numbers, which the lexer reads whole
/// (`\frac12`, `ATOM_NUMBER:12`, `\sqrt2`, util.rs `is_numeric_constant`).
fn is_numeric_factor(xm: &XM) -> bool {
  match xm {
    XM::Lexeme(lex, _) if lex.starts_with("ATOM_NUMBER:") => true,
    XM::Lexeme(..) | XM::Token(..) => operator_category(xm) == Some("NUMBER"),
    _ => false,
  }
}

/// A differential operator applied to a number alone, which the factors juxtaposed after it join
/// (`numeric_monomial`, 57cj review): the product `(\partial_x 2)\,u` ranks below ∂_x(2u)
/// (`differentiated_number_sites`, `DifferentiatedNumbersTakeTheirFactors`).
fn differentiates_a_number(xm: &XM) -> bool {
  matches!(xm, XM::Apply(Operator(head), Args(args), _, meta)
    if meta.differential
      && is_bare_differential_operator(head)
      && matches!(args.as_slice(), [Some(operand)]
        if is_numeric_lead(operand)
          || is_numeric_monomial(operand)
          // (a function's bare application to a constant, whose argument takes the factors after it: `\partial_x\log 2\,u`;
          // not a bare π, which leads no monomial — `\int f\,d\pi(x,y)` keeps its measure dπ, 57cj.9 A/B, 2605.00545)
          || is_constant(operand) && is_bare_operator_application(operand)))
}

/// A product a number leads (`numeric_monomial`): `2\pi`, `\frac12 u^2`, `2^3u` (`is_numeric_lead`).
fn is_numeric_monomial(xm: &XM) -> bool {
  matches!(xm, XM::Apply(Operator(op), Args(factors), _, meta)
    if meta.fenced.is_none()
      && is_invisible_times_op(op)
      && factors.first().and_then(Option::as_ref).is_some_and(is_numeric_lead))
}

/// What leads a numeric monomial (`numeric_monomial`): a number, or one raised to a constant power (`numeric_power`:
/// `\partial_x 2^3u` ∂_x(2³·u), `\partial_x\sqrt2^3u`; 57cj.8 review, latent, its probes).
fn is_numeric_lead(xm: &XM) -> bool {
  is_numeric_factor(xm)
    || matches!(xm, XM::Apply(Operator(op), Args(args), ..)
      if operator_category(op) == Some("SUPERSCRIPTOP")
        && matches!(args.as_slice(), [Some(base), Some(power)]
          // (a constant group raised to a constant power: `(2\pi)^{-3}`, 57cj.9 review)
          if (is_numeric_lead(base) || matches!(base, XM::Dual(..) | XM::Wrap(..)) && is_constant(base))
            && is_constant(power)))
}

/// `numeric_power`: a number raised to a constant power, which leads a numeric monomial (`is_numeric_lead`); another
/// power is a factor of the derivative alone (`\partial_x 2^n u` (∂_x 2ⁿ)·u, ∂ taking one factor).
pub fn numeric_power_script(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  let power = postfix_script(rule_id, args, pragmas, ctxt)?;
  if !power.as_ref().is_some_and(is_numeric_lead) {
    return Err("numeric_power_script: no constant power".into());
  }
  Ok(power)
}

/// A differential operator, applied or bare, the first factor of `xm`: what a numeric monomial with a
/// factor of its own stops before (`numeric_monomial_product`, `differentiates_a_number`).
fn starts_with_a_derivative(xm: &XM) -> bool {
  let first = product_end(xm, false);
  is_partial_derivative(first) || is_bare_differential_operator(first)
}

/// An integral's differential `d x` (`diffop_apply`), or its `d` read as a letter (XDIFFUNK), the first factor
/// of `xm`: what a numeric monomial always stops before (57cj.2 review: `\int\partial_x 2uv\,dx` keeps its `dx`).
fn starts_with_a_d_differential(xm: &XM) -> bool {
  let first = product_end(xm, false);
  (is_differential(first) && !is_partial_derivative(first) && !is_bare_differential_operator(first))
    // the integral's letter `d` only (the lexer's XDIFFUNK), not a `d` outside one (`\partial_x 2d` ∂_x(2d))
    || matches!(script_nucleus(first), XM::Lexeme(lex, _) if lex.starts_with("XDIFFUNK:d:"))
}

/// `DifferentiatedNumbersTakeTheirFactors` (latent, no corpus witness: the 57cj reviews' probes): how many times
/// does the product `xm` leave outside a differential operator's number a factor juxtaposed after it that the
/// number's monomial takes (`numeric_monomial`, 57cj review: `\partial_x 2u` is ∂_x(2u), not ∂_x(2)·u)? Not an
/// integral's differential, nor a derivative after a monomial with a factor of its own (`numeric_monomial_product`;
/// 57cj.1, 57cj.2 reviews), nor what no monomial takes, an operator or a function with no argument
/// (`\partial_x 2u\,\nabla\cdot v` (∂_x(2u)·∇)·v, 57cj.5 review; an applied one it takes, `\partial_x 2u\,\nabla\,\partial_y 3w`).
/// Every differentiated number on the left factor's right edge counts — through a differential operator's operand
/// and a function's or operator's bare argument, `\partial_x\partial_y 2u` ∂_x(∂_y(2u)), `\sin\partial_x 2\,\partial_y u`
/// sin@(∂_x(2·∂_y u)) (`right_edge`; 57cj.5 review). A soft, counting preference (57cj.3, 57cj.4 reviews): one site it
/// cannot hold does not make the others split (`…+\partial_y 3w` keeps ∂_y(3w)). A product between delimiters counts
/// as any other: the split inside `(\partial_y 3w)` is the same split. Only juxtaposition: the monomial takes no
/// factor across a MulOp (`\partial_x 2\cdot u`).
pub(crate) fn differentiated_number_sites(xm: &XM) -> usize {
  let XM::Apply(Operator(op), Args(factors), ..) = xm else {
    return 0;
  };
  if !is_invisible_times_op(op) {
    return 0;
  }
  factors
    .windows(2)
    .map(|pair| match pair {
      [Some(left), Some(right)] => right_edge(left)
        .into_iter()
        // (inside a function's or operator's bare argument, which has ended before `right`, a number with factors of
        // its own takes nothing more: only a differentiated constant counts there, a number or its coefficient run,
        // `\sin\partial_x 2\pi\,u\,v`; 57cj.6, 57cj.7 reviews)
        .filter(|(operator, in_a_bare_argument)| {
          differentiates_a_number(operator)
            && !(*in_a_bare_argument && differentiates_a_monomial(operator))
            && !(starts_with_a_d_differential(right)
              || starts_with_an_unapplied_head(right)
              || differentiates_a_monomial(operator) && starts_with_a_derivative(right))
        })
        .count(),
      _ => 0,
    })
    .sum()
}

/// `LetterDsBeforeVariablesAreDifferentials` (57cj.20, SYNC (16); 2605.28900, 2605.08899): how many times does the
/// product `xm` read an integral's letter `d` — or its power, `d^3` — as a factor before a variable, bare or post-scripted, that its
/// differential takes (`diffop_apply`, `differential_power_apply`)? Perl's IntFactor tries `diffd ATOM_OR_ID
/// addScripts` before `Factor` (MathGrammar:640-647), so a `d` the differential can take is one: `\int f\,dx_1`
/// f·differential-d@(x₁), not f·d·x₁. A soft, counting preference: the trees with the fewest such letters are kept, so
/// a site no tree reads as a differential (no INTOP: the lexer's `d` is a plain unknown, util.rs; a numerator,
/// `StandaloneDiffopsAreNotNumerators`) counts in every tree alike and decides nothing. Only juxtaposition, as
/// the grammar's `factor` rule: `d\cdot x_1` has no differential reading.
pub(crate) fn letter_differential_sites(xm: &XM) -> usize {
  let XM::Apply(Operator(op), Args(factors), ..) = xm else {
    return 0;
  };
  if !is_invisible_times_op(op) {
    return 0;
  }
  // (the factors meet at their edges: a slash quotient on the left ends in its denominator's last factor,
  // `dx/dt^2` (differential-d@(x)/d)·t² beside differential-d@(x)/differential-d@(t²), a bare argument in its own,
  // `ends_in_a_letter_differential_d`; a group ends at its delimiter)
  factors
    .windows(2)
    .filter(|pair| {
      matches!(pair, [Some(left), Some(right)]
        if ends_in_a_letter_differential_d(left)
          && is_a_differential_variable(unfenced_product_end(right, false)))
    })
    .count()
}

/// The first (`last` false) or last factor of an undelimited product, as `product_end`, not entering a group.
fn unfenced_product_end(xm: &XM, last: bool) -> &XM {
  match xm {
    XM::Apply(Operator(op), args, _, meta)
      if meta.fenced.is_none() && args.0.len() >= 2 && is_product_operator(op) =>
    {
      match if last { args.0.last() } else { args.0.first() } {
        Some(Some(factor)) => unfenced_product_end(factor, last),
        _ => xm,
      }
    },
    _ => xm,
  }
}

/// The integral's letter `d` (the lexer's `XDIFFUNK`/`XDIFFID`, which `diffop_apply` annotates), alone or raised to a
/// power (`raised_differential_d`): `d`, `\mathrm{d}`, `d^3`, `d^n`.
fn is_a_letter_differential_d(xm: &XM) -> bool {
  let is_d = |xm: &XM| matches!(xm, XM::Lexeme(lex, _) if lex.starts_with("XDIFFUNK:d:") || lex.starts_with("XDIFFID:d:"));
  is_d(xm)
    || matches!(xm, XM::Apply(Operator(op), Args(args), ..)
      if operator_category(op) == Some("SUPERSCRIPTOP")
        && matches!(args.as_slice(), [Some(base), Some(power)] if is_d(base) && is_a_power_count(power)))
}

/// Does `xm` end in an integral's letter `d` (`is_a_letter_differential_d`): its last factor, or the last factor of a
/// bare argument it ends in — `\int\cos d\theta` read cos@(d)·θ beside cos@(d·θ) holds the same letter before θ
/// (57cj.20 review; a trig argument takes no differential, #367). A group ends the walk.
fn ends_in_a_letter_differential_d(xm: &XM) -> bool {
  let mut node = unfenced_product_end(xm, true);
  loop {
    if is_a_letter_differential_d(node) {
      return true;
    }
    match node {
      XM::Apply(_, Args(args), _, meta) if meta.fenced.is_none() => match args.as_slice() {
        [Some(argument)]
          if !matches!(argument, XM::Dual(..) | XM::Wrap(..))
            && !matches!(argument, XM::Apply(_, _, _, inner) if inner.fenced.is_some()) =>
        {
          node = unfenced_product_end(argument, true);
        },
        _ => return false,
      },
      _ => return false,
    }
  }
}

/// What a differential takes after its `d` (`diffop_apply`'s `factor_base`, `differential_variable`): a letter,
/// identifier, atom, array or number, bare or with post-scripts of its own.
fn is_a_differential_variable(xm: &XM) -> bool {
  matches!(
    operator_category(script_nucleus(xm)),
    Some("UNKNOWN" | "ID" | "ATOM" | "ARRAY" | "NUMBER" | "XDIFFUNK" | "XDIFFID")
  ) && matches!(script_nucleus(xm), XM::Lexeme(..))
}

/// The nodes on `xm`'s right edge, outermost first: its last factor, and down through each unfenced application
/// of one undelimited argument (a differential operator's operand, a function's or operator's bare argument, a
/// sign) to that argument's last factor; a group's application ends the walk (`differentiated_number_sites`). Each
/// node comes with whether a function's or operator's bare argument holds it.
fn right_edge(xm: &XM) -> Vec<(&XM, bool)> {
  let mut edge = Vec::new();
  let mut node = product_end(xm, true);
  let mut in_a_bare_argument = false;
  loop {
    edge.push((node, in_a_bare_argument));
    match node {
      XM::Apply(Operator(head), Args(args), _, meta) if meta.fenced.is_none() => {
        match args.as_slice() {
          [Some(argument)]
            if !matches!(argument, XM::Dual(..) | XM::Wrap(..))
              && !matches!(argument, XM::Apply(_, _, _, argument_meta) if argument_meta.fenced.is_some()) =>
          {
            in_a_bare_argument |= !meta.differential
              && matches!(
                head_category(head),
                Some("OPFUNCTION" | "TRIGFUNCTION" | "OPERATOR" | "FUNCTION")
              );
            node = product_end(argument, true);
          },
          _ => break,
        }
      },
      _ => break,
    }
  }
  edge
}

/// Does `xm` start with an operator or a function that takes no argument there (`\nabla`, `\nabla^2`, `\log`): what
/// no numeric monomial takes (`numeric_monomial = number tight_term`; `bare_op_term`, `bare_opfunction_term`).
fn starts_with_an_unapplied_head(xm: &XM) -> bool {
  let first = product_end(xm, false);
  is_operator_head(first) || is_bare_function_head(first)
}

/// Is `xm` a differential operator applied to a numeric monomial with a factor of its own after its constants
/// (`\partial_x(2u)`, `\partial_t(\frac12|u|^2)`, `\partial_x(2\pi u)`), not to a constant alone (`\partial_x 2`,
/// `\partial_x 2\pi`, `\partial_x(2\pi i)`, which a differentiated number counts as; 57cj.7, 57cj.8 reviews, latent, no
/// corpus witness)?
fn differentiates_a_monomial(xm: &XM) -> bool {
  matches!(xm, XM::Apply(Operator(head), Args(args), _, meta)
    if meta.differential
      && is_bare_differential_operator(head)
      && matches!(args.as_slice(), [Some(operand)]
        if is_numeric_monomial(operand) && !is_constant(operand)))
}

/// A differential operator applied to an ellipsis alone (`\partial_i\ldots`).
fn differentiates_an_ellipsis(xm: &XM, ctxt: &ActionContext) -> bool {
  matches!(xm, XM::Apply(Operator(head), Args(args), _, meta)
    if meta.differential
      && is_bare_differential_operator(head)
      && matches!(args.as_slice(), [Some(operand)] if is_ellipsis(operand, ctxt)))
}

/// `numeric_monomial`'s first product: a number and the factors after it, one operand of a differential
/// operator (57cj review: `\partial_x\frac12 u^2` ∂_x(½u²), `\partial_x 2u` ∂_x(2u), `\partial_t 2\pi iu`, as
/// Perl's greedy `bigop` reads them) — up to an integral's differential (`\int\partial_t\frac12|u|^2\,dx`
/// ∫(∂_t(½|u|²)·dx), its `d` read as a letter too) and a derivative after a factor of its own (`\partial_x 2u\,\partial_y v`
/// (∂_x(2u))·∂_y v; 57cj.1 review); a derivative right after the number it takes (`\partial_x 2\,\partial_y u`
/// ∂_x(2·∂_y u), as 57ci and Perl; 57cj.2 review), or after its constant run (`\partial_x 2\pi\,\partial_y u`
/// ∂_x(2π·∂_y u), `\partial_t 2\pi i\,\partial_x u`; 57cj.7, 57cj.8 reviews, latent, no corpus witness).
pub fn numeric_monomial_product(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  if let [Some(number), Some(tail)] = args.as_slice() {
    let factors = product_factors(tail);
    // (past the number's constant run, `\partial_x 2\pi\,\partial_y u` ∂_x(2π·∂_y u); 57cj.7, 57cj.8 reviews; the
    // leading number is in the run, so the tail's share is one less)
    let run = constant_run(
      &std::iter::once(number)
        .chain(factors.iter().copied())
        .collect::<Vec<_>>(),
    );
    if factors
      .iter()
      .any(|factor| starts_with_a_d_differential(factor))
      || factors
        .iter()
        .skip(run.saturating_sub(1))
        .skip(1)
        .any(|factor| starts_with_a_derivative(factor))
    {
      return Err("numeric_monomial_product: a differential ends the monomial".into());
    }
  }
  apply_invisible_times(rule_id, args, pragmas, ctxt)
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
  if let Some(ref arg) = arg1
    && is_fenced_modifier_dual(arg)
  {
    return Err(
      "prefix_apply_applyop: arg is a fenced modifier expression — \
         prefer annotated_fenced_modifier over function application"
        .into(),
    );
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
  let XM::Dual(ref content, ref presentation, ..) = *arg else {
    return false;
  };
  let has_absent_prefix = |args: &Args| -> bool {
    let first = args.trees().first().copied().cloned();
    matches!(first,
      Some(XM::Token(p, _)) if p.meaning.as_deref() == Some("absent"))
  };
  // Path A — content is an Apply directly:
  if let XM::Apply(_, args, ..) = &**content {
    return has_absent_prefix(args);
  }
  // Path B — content is a Ref; find the Apply in the presentation Wrap.
  if let XM::Wrap(items, ..) = &**presentation {
    for item in items {
      if let XM::Apply(_, args, ..) = item
        && has_absent_prefix(args)
      {
        return true;
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
  if let (Some(XM::Apply(app_op, app_args, app_props, app_meta)), Some(op_xm)) = (&arg, &op) {
    let same_op = match (app_op.0.as_ref(), op_xm) {
      // Compare Lexemes: both are lexeme strings like "ADDOP:plus:+"
      (XM::Lexeme(a_lex, _), XM::Lexeme(o_lex, _)) => {
        // Compare role:meaning prefix (ignoring the actual symbol after last :)
        let a_parts: Vec<_> = a_lex.splitn(3, ':').collect();
        let o_parts: Vec<_> = o_lex.splitn(3, ':').collect();
        a_parts.len() >= 2
          && o_parts.len() >= 2
          && a_parts[0] == o_parts[0]
          && a_parts[1] == o_parts[1]
      },
      // Compare realized Tokens
      (XM::Token(a_props, _), XM::Token(o_props, _)) => {
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

/// The operand of a POSTFIX application (`n!` → `n`, `(n+1)!` → `(n+1)`): the factor the postfix
/// took, which the checks that read a factor see through (57bw).
pub(crate) fn postfix_base(xm: &XM) -> Option<&XM> {
  match xm {
    XM::Apply(Operator(op), Args(args), ..) if operator_category(op) == Some("POSTFIX") => {
      match args.as_slice() {
        [Some(base)] => Some(base),
        _ => None,
      }
    },
    _ => None,
  }
}

/// What a postfix took, under all its scripts and any further postfixes (`n(n-1)!^2` → `n(n-1)`,
/// `g(y)!_k^2` → `g(y)`); none when `xm` carries no postfix.
pub(crate) fn postfixed_operand(xm: &XM) -> Option<&XM> {
  let mut operand = script_nucleus(xm);
  let mut postfixed = false;
  while let Some(base) = postfix_base(operand) {
    operand = script_nucleus(base);
    postfixed = true;
  }
  postfixed.then_some(operand)
}

/// Perl MathGrammar:419-424 (`addScripts`): a POSTFIX applies to the factor before it, `n!` is
/// factorial@(n).
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

/// Perl `absExpression : <rulevar: local $forbidEvalAt = 1>` (MathGrammar:410; the evalAt
/// alternatives of `moreFactors` test it, :259-262): no evaluation bar is read inside a `|…|`
/// pair. `|\nabla a|_L|\nabla b|_L` is |∇a|_L·|∇b|_L, not an evaluation bar closing over both
/// (2605.12082, 2605.04766; repro math-parse/evaluated_at_stays_outside_absolute_bars).
/// Divergence #350 (OXIDIZED_DESIGN_DIVERGENCES): only single-bar pairs (`|…|`, `||…||`) — the
/// ones a bar can be misread in — and not within a nested group; Perl forbids it in `\|…\|`,
/// `\left|…\right|` and every nested group too, leaving `\|u|_{\Gamma}\|`, `|g(f|_{x=0})|`
/// unparsed (2605.01526, 2605.04708, 2605.07463). And a preference, not a prune: the reading with the
/// fewest marked fences wins (`XM::prefer_fewest_evaluation_bars_inside`), so a pair whose every
/// reading nests one keeps it — `|f(x)|_{0}^{1}|`, `\Big|\partial_a^k(\ldots)\Big|_{a=a_m}\Big|`, which
/// Perl leaves unparsed.
fn holds_evaluation_bar(xm: &XM) -> bool { holds_bar_reading(xm, &["evaluated-at"]) }

/// A `\left|` read as a divider or a ket's opening bar (`divider_bar`, `ket_bar`) whose next item
/// evaluates at a stretchy bar: TeX paired that `\left|` with the `\right|` the item evaluates at
/// (`\left(\epsilon\left|\nabla u\right|_{L^2}\right)`), so the reading is refuted. Only
/// `prefer_fewer_conditionals` had hidden it, and not beside another conditional (57ap review; 46
/// formulas of 2605 put a factor and `\left|…\right|_` in a fence). In parentheses and brackets Perl
/// reads the pair too, trying `Factor` before `evalAtOp` (MathGrammar:257-263); in braces it tries
/// the set-builder bar first (`scriptFactorOpen`, :486-491) and reads a conditional set — Rust keeps
/// TeX's pairing there (divergence #358). A plain `|` evaluation bar is TeX's to pair with no
/// `\left|`, so `\left\{x\left|f|_{x=0}>0\right.\right\}` divides, as Perl. (An evaluation bar inside a
/// nested group, `\left|(f\right|_{x=0})`, is not seen — groups that cross each other.)
fn left_bar_pairs_an_evaluation_bar(bar: &XM, next: &XM) -> bool {
  matches!(bar, XM::Lexeme(lex, _) if lex.starts_with("LEFT_STRETCHY_VERTBAR:"))
    && holds_stretchy_evaluation_bar(next)
}

/// Does `xm` evaluate at a stretchy bar — a `\right|`, the bar TeX pairs with a `\left|` — outside a
/// nested group? `eval_at` presents `base|_{sub}` as a script application on `[base, bar]`.
fn holds_stretchy_evaluation_bar(xm: &XM) -> bool {
  match xm {
    XM::Dual(content, presentation, ..)
      if matches!(&**content, XM::Apply(Operator(op), ..)
        if matches!(&**op, XM::Token(props, _) if props.meaning.as_deref() == Some("evaluated-at"))) =>
    {
      evaluation_bar_is_stretchy(presentation)
    },
    XM::Apply(Operator(op), args, ..) => {
      holds_stretchy_evaluation_bar(op)
        || args.0.iter().flatten().any(holds_stretchy_evaluation_bar)
    },
    // A nested group — a fence, a delimited argument — pairs its own bars.
    XM::Dual(_, presentation, ..) if matches!(**presentation, XM::Wrap(..)) => false,
    XM::Dual(content, presentation, ..) => {
      holds_stretchy_evaluation_bar(content) || holds_stretchy_evaluation_bar(presentation)
    },
    _ => false,
  }
}

/// The bar of an `eval_at` presentation — the last of the scripted `[base, bar]` wrap, under a
/// superscript and a subscript application — is stretchy.
fn evaluation_bar_is_stretchy(presentation: &XM) -> bool {
  match presentation {
    XM::Apply(_, args, ..) => args
      .0
      .first()
      .and_then(Option::as_ref)
      .is_some_and(|scripted| match scripted {
        XM::Wrap(items, ..) => items.last().is_some_and(
          |bar| matches!(bar, XM::Token(props, _) if props.stretchy.as_deref() == Some("true")),
        ),
        inner => evaluation_bar_is_stretchy(inner),
      }),
    _ => false,
  }
}

/// Does `xm` hold a pair of bars read as a fence — an absolute value, a norm, or any group a bar
/// opens (`\mid f|`, `delimited-∣|`)? Its presentation opens with a bar.
pub(crate) fn holds_bar_pair(xm: &XM) -> bool {
  match xm {
    XM::Dual(content, presentation, _, meta) => {
      let opened_by_a_bar = match &**presentation {
        XM::Wrap(items, ..) => items.first().is_some_and(is_a_bar),
        _ => false,
      };
      meta.bar_fence || opened_by_a_bar || holds_bar_pair(content) || holds_bar_pair(presentation)
    },
    XM::Apply(Operator(op), args, ..) => {
      holds_bar_pair(op) || args.0.iter().flatten().any(holds_bar_pair)
    },
    XM::Wrap(items, ..) => items.iter().any(holds_bar_pair),
    _ => false,
  }
}

/// Is `xm` a bar token: `|`, `\mid`, a stretchy bar?
fn is_a_bar(xm: &XM) -> bool {
  matches!(
    operator_category(xm),
    Some("VERTBAR" | "LEFT_STRETCHY_VERTBAR" | "MIDDLE")
  )
}

/// Does `xm`'s presentation open with a bar — its leftmost token, reached through the first
/// operand of an infix, relation or script application?
pub(crate) fn opens_with_a_bar(xm: &XM) -> bool {
  match xm {
    XM::Dual(_, presentation, _, meta) => meta.bar_fence || opens_with_a_bar(presentation),
    XM::Wrap(items, ..) => items.first().is_some_and(opens_with_a_bar),
    XM::Apply(_, args, ..) if args.0.len() >= 2 => args
      .0
      .first()
      .and_then(Option::as_ref)
      .is_some_and(opens_with_a_bar),
    XM::Token(..) | XM::Lexeme(..) => is_a_bar(xm),
    _ => false,
  }
}

/// Does `xm` read one of its bars as an operator of `meanings` — an evaluation bar, a conditional —
/// outside a nested group, which pairs its own bars?
fn holds_bar_reading(xm: &XM, meanings: &[&str]) -> bool {
  match xm {
    XM::Token(props, _) | XM::Ref(props) => props
      .meaning
      .as_deref()
      .is_some_and(|m| meanings.contains(&m)),
    XM::Apply(Operator(op), args, ..) => {
      holds_bar_reading(op, meanings)
        || args
          .0
          .iter()
          .flatten()
          .any(|a| holds_bar_reading(a, meanings))
    },
    // A nested group — a fence, a delimited argument — pairs its own bars.
    XM::Dual(_, presentation, ..) if matches!(**presentation, XM::Wrap(..)) => false,
    XM::Dual(content, presentation, ..) => {
      holds_bar_reading(content, meanings) || holds_bar_reading(presentation, meanings)
    },
    _ => false,
  }
}

/// The Dirac meaning of `xm` — a bra, ket, inner product or operator product (`qm_fenced`).
fn dirac_meaning(xm: &XM) -> Option<&str> {
  let XM::Dual(content, ..) = xm else {
    return None;
  };
  let XM::Apply(Operator(op), ..) = &**content else {
    return None;
  };
  let XM::Token(props, _) = op.as_ref() else {
    return None;
  };
  props.meaning.as_deref().filter(|m| {
    matches!(
      *m,
      "bra" | "ket" | "inner-product" | "quantum-operator-product"
    )
  })
}

/// Does `xm` hold a ket outside a nested group (a fence, a Dirac bracket, pairs its own)?
fn holds_ket(xm: &XM) -> bool {
  if let Some(meaning) = dirac_meaning(xm) {
    return meaning == "ket";
  }
  match xm {
    XM::Apply(Operator(op), args, ..) => holds_ket(op) || args.0.iter().flatten().any(holds_ket),
    XM::Dual(_, presentation, ..) if matches!(**presentation, XM::Wrap(..)) => false,
    XM::Dual(content, ..) => holds_ket(content),
    _ => false,
  }
}

/// Does `xm` hold an open bra outside a nested group: a factor ending in a bra that the next factor
/// does not close — a factor not opening with a bar? Perl's `maybeBra` reads a braket before a bra
/// (`ketExpression maybeBraket`, MathGrammar:373-379), so such a bra would have been a bracket's; a
/// bra before a bar, an operator's end or its group's end stays one (`|c\langle u|\rangle`
/// ket@(c·bra@(u)), `\langle|a\rangle\langle a|+|b\rangle\langle b|\rangle`).
fn holds_open_bra(xm: &XM) -> bool {
  if dirac_meaning(xm).is_some() {
    return false;
  }
  match xm {
    XM::Apply(Operator(op), Args(args), ..) => {
      let factors: Vec<&XM> = args.iter().flatten().collect();
      (is_product_operator(op)
        && factors
          .windows(2)
          .any(|pair| ends_with_bra(pair[0]) && !opens_with_a_bar(pair[1])))
        || holds_open_bra(op)
        || factors.into_iter().any(holds_open_bra)
    },
    XM::Dual(_, presentation, ..) if matches!(**presentation, XM::Wrap(..)) => false,
    XM::Dual(content, ..) => holds_open_bra(content),
    _ => false,
  }
}

/// Does `xm` end with a bra — its last factor, or the base of its last scripted item?
fn ends_with_bra(xm: &XM) -> bool {
  if dirac_meaning(xm) == Some("bra") {
    return true;
  }
  if let Some(base) = script_base(xm) {
    return ends_with_bra(base);
  }
  match xm {
    XM::Apply(_, args, ..) => args
      .0
      .last()
      .and_then(Option::as_ref)
      .is_some_and(ends_with_bra),
    _ => false,
  }
}

thread_local! {
  /// Perl `$LaTeXML::MathParser::MAX_ABS_DEPTH` (MathParser.pm:814): how deep bar fences may nest
  /// in the parse under way — 1, then 2 and 3 on the retries of `MathParser::parse_lexemes`.
  static MAX_ABS_DEPTH: Cell<u8> = const { Cell::new(1) };
  /// Perl `SawNotation('AbsFail')` (MathGrammar:412): a bar fence nested deeper than allowed.
  static ABS_FAIL: Cell<bool> = const { Cell::new(false) };
}

/// Set how deep bar fences may nest (Perl `MAX_ABS_DEPTH`), clearing `AbsFail`; the previous limit.
pub(crate) fn set_max_abs_depth(depth: u8) -> u8 {
  ABS_FAIL.with(|fail| fail.set(false));
  MAX_ABS_DEPTH.with(|max| max.replace(depth))
}

/// Did a bar fence nest deeper than allowed since the limit was set (Perl `AbsFail`)?
pub(crate) fn abs_fail() -> bool { ABS_FAIL.with(Cell::get) }

/// The depth of the bar fences `xm` holds, Perl's `absExpression` nesting (MathGrammar:410-412): a
/// bar fence's own (`Meta::abs_depth`), through groups and applications — not a script's, whose
/// `Subscript`/`Superscript` start the count over (:84-89).
fn abs_depth_within(xm: &XM) -> u8 {
  match xm {
    XM::Dual(.., meta) if meta.bar_fence => meta.abs_depth,
    XM::Dual(_, presentation, ..) => abs_depth_within(presentation),
    XM::Apply(Operator(op), args, ..)
      if operator_category(op).is_some_and(|c| c.ends_with("SCRIPTOP")) =>
    {
      args
        .0
        .first()
        .and_then(Option::as_ref)
        .map_or(0, abs_depth_within)
    },
    XM::Apply(Operator(op), args, ..) => args
      .0
      .iter()
      .flatten()
      .map(abs_depth_within)
      .fold(abs_depth_within(op), u8::max),
    XM::Wrap(items, ..) | XM::Arg(items) | XM::Choices(items) => {
      items.iter().map(abs_depth_within).max().unwrap_or(0)
    },
    XM::Token(..) | XM::Lexeme(..) | XM::Ref(_) => 0,
  }
}

/// Mark an absolute value or norm (`Meta::bar_fence`): one between two single `|`
/// (`Meta::single_bar_pair`), one whose single-bar content holds an evaluation bar
/// (`Meta::evaluation_bar_inside`, see [`holds_evaluation_bar`]). Perl's `absExpression`
/// (MathGrammar:410-412) nests no deeper than `MAX_ABS_DEPTH`, so a deeper fence fails the
/// reading, noting `AbsFail` for the retry (MathParser.pm:831-836): the bars of `\log|a|+\log|b|`
/// are two absolute values, not one around `a·|+\log|·b` (repro
/// math-parse/bar_pairs_nest_as_shallow_as_they_can).
fn mark_bar_fence(
  mut fence: XM,
  single_bar_pair: bool,
  evaluation_bar_inside: bool,
) -> Result<XM, Box<dyn Error>> {
  if let XM::Dual(_, presentation, _, meta) = &mut fence {
    let depth = abs_depth_within(presentation).saturating_add(1);
    if depth > MAX_ABS_DEPTH.with(Cell::get) {
      ABS_FAIL.with(|fail| fail.set(true));
      return Err("bar fence: nested deeper than MAX_ABS_DEPTH (Perl AbsFail)".into());
    }
    meta.bar_fence = true;
    meta.single_bar_pair = single_bar_pair;
    meta.evaluation_bar_inside = evaluation_bar_inside;
    meta.abs_depth = depth;
  }
  Ok(fence)
}

/// Is `xm` an absolute value between two single `|` (`Meta::single_bar_pair`)?
fn is_single_bar_pair(xm: &XM) -> bool { matches!(xm, XM::Dual(.., meta) if meta.single_bar_pair) }

pub fn fenced(
  rule_id: i32,
  mut args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
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
  let o = realized_value(&open, &ctxt)?;
  let c = realized_value(&close, &ctxt)?;
  let op_name = format!("delimited-{}{}", o, c);
  // An angle fence holds no open bra: Perl reads a `\langle` label before a bar as a braket first
  // (`maybeBra`: `ketExpression maybeBraket`, MathGrammar:373-379), so `\langle Hx|x\rangle=\langle
  // x|Hx\rangle` and `\langle Ax|y\rangle\langle x|Ay\rangle` are inner products, never one fence
  // whose inner `\langle` opens a bra (57bp; 2605.30176, 2605.16417). A bra ending a factor stays, as
  // Perl's list fence `\big\langle Tx_i,|\zeta_k\rangle\langle\zeta_j|\big\rangle` (2605.21982).
  if matches!(
    (o.as_ref(), c.as_ref()),
    ("\u{27E8}", "\u{27E9}") | ("langle", "rangle")
  ) && holds_open_bra(&arg)
  {
    return Err("fenced: an angle fence holds no open bra".into());
  }
  // Perl's grammar (MathGrammar:462-475) fences a bracket or brace list two ways: a list of
  // expressions is `Fence`'s many items, named by its tables (`fence`); a list holding a relation is
  // one `Formulae` item (:69), fenced as one — `\{x=1;y=2\}` set@(formulae@(…)), `[x=1,y=2]`
  // delimited-[]@(formulae@(…)), never an interval of equations.
  if matches!((o.as_ref(), c.as_ref()), ("[", "]") | ("{", "}")) {
    match fenced_list(&arg) {
      Some(FencedList::Expressions) => {
        let stuff = fenced_list_items(open, arg, close).into_iter().map(Some);
        return fence(rule_id, stuff.collect(), pragmas, ctxt);
      },
      Some(FencedList::Formulae) => arg = as_formulae(arg),
      None => {},
    }
  }

  // TODO: For now assume a single argument in arg; specialize in other functions such as
  // "open_interval",       for the other cases from the classic MathParser.pm
  if op_name == "delimited-()" {
    // A paren list holding a relation is one `Formulae` item too, `(a<1,b<2)` formulae@(a < 1, b < 2)
    // (Perl `OPEN Formulae CLOSE`, MathGrammar:69, parens around one item transparent, MathParser.pm:
    // 1412-1415; a bracket, brace or conditional list since 57au/57bh; was `vector@(…)`, 57bk,
    // 2605.00042, 2605.01907, 2605.00130). A function still takes its items (`fenced_tuple_items`).
    let relation_list = matches!(fenced_list(&arg), Some(FencedList::Formulae));
    if relation_list {
      arg = as_formulae(arg);
    }
    // Check if arg is a multi-item list (XM::Dual from list_apply/formulae_apply).
    // If so, use interpret_delimited for per-item XMRefs (matching Perl's NewFenced).
    // A bare list only: its presentation `[item, separator, item, …]`. A list already fenced —
    // `\left([a;b]\right)`, a bracket list since 57au one Dual whose presentation carries its own
    // delimiters — is one item, and parens around one item are transparent (Perl `Fence`,
    // MathParser.pm:1412-1415); reading its presentation's even places took the delimiters and
    // separators for the items (`vector@([, ;, ])`, 118 formulas of the 57av train A/B; 2605.08815
    // S2.E4, 2605.00444, 2605.00423).
    let is_multi_item = !relation_list
      && match &arg {
        XM::Dual(content, presentation, ..) => matches!(&**content, XM::Apply(op_box, args, ..)
        if args.0.len() >= 2 && matches!(&*op_box.0,
          XM::Token(p, _) if matches!(p.meaning.as_deref(),
            Some("vector") | Some("list") | Some("formulae")))
          && presents_its_items_alone(presentation, args.0.len())),
        XM::Apply(op_box, args, ..) => {
          args.0.len() >= 2
            && matches!(&*op_box.0,
        XM::Token(p, _) if matches!(p.meaning.as_deref(),
          Some("vector") | Some("list") | Some("formulae")))
        },
        _ => false,
      };
    if is_multi_item {
      // Perl `Fence` (MathParser.pm:1390-1417) names a fenced list by its delimiters and its FIRST
      // separator: the enclose tables (:1368-1377) hold comma lists only, so parens around a `;`
      // list make a `list` (`p(x;\theta)` Perl `p * list@(x, theta)`, `(a,b;c)` a `vector`), and
      // the presentation keeps the separators as written — rebuilding it with commas of our own
      // rendered `p(x;\theta)` as `p(x,θ)` (2605.00042 `J(H;\alpha)`, 2605.00130 `I(X;F')`; 57ay
      // review). Two items with a comma stay a `vector` here: the `interval` action offers the
      // open interval.
      let (inner, comma_list) = match arg {
        // A bare list Dual (`presents_its_items_alone`): its presentation is the items with their
        // separators between.
        XM::Dual(_, pres, ..) => {
          let XM::Wrap(wrap_items, ..) = *pres else {
            unreachable!("is_multi_item: a bare list presents its items in a Wrap")
          };
          let first_separator = realized_value(&wrap_items[1], &ctxt)?;
          let comma_list = first_separator == ",";
          (wrap_items, comma_list)
        },
        // A bare Apply carries no separators of its own: comma-joined. No producer of a Dual-less
        // list is known (57ba review); kept as the fallback the shape check admits.
        XM::Apply(_, args_inner, ..) => {
          let comma = XM::Token(
            XProps {
              role: Some(Cow::Borrowed("PUNCT")),
              content: Some(Cow::Borrowed(",")),
              ..XProps::default()
            },
            Meta::default(),
          );
          let mut inner = Vec::new();
          for (i, item) in args_inner.0.into_iter().flatten().enumerate() {
            if i > 0 {
              inner.push(comma.clone());
            }
            inner.push(item);
          }
          (inner, true)
        },
        _ => unreachable!(),
      };
      let op = XProps {
        meaning: Some(Cow::Borrowed(if comma_list { "vector" } else { "list" })),
        ..XProps::default()
      };
      let mut stuff = vec![open];
      stuff.extend(inner);
      stuff.push(close);
      interpret_delimited(op.into(), stuff, ctxt).map(Some)
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
    interpret_delimited(op.into(), vec![open, arg, close], ctxt).map(Some)
  } else if op_name == "delimited-||" {
    // Single bars only; `\left|…\right|` pairs are the lexer's (divergence #350).
    let single_bar_pair = operator_category(&open) == Some("VERTBAR");
    // Bars, not `\lvert…\rvert` (an OPEN/CLOSE pair, a group as in Perl).
    let bar_fence = matches!(
      operator_category(&open),
      Some("VERTBAR" | "LEFT_STRETCHY_VERTBAR")
    );
    let evaluation_bar_inside = single_bar_pair && holds_evaluation_bar(&arg);
    // Four single bars around `x` are a norm (`norm_fenced`), never |(|x|)|: Perl tries
    // `SINGLEVERTBAR SINGLEVERTBAR absExpression …` (MathGrammar:294) before `VERTBAR absExpression
    // VERTBAR` (:299) — `\log||x||_2^2` (57am review round 5).
    if single_bar_pair && is_single_bar_pair(&arg) {
      return Err("fenced: four single bars are a norm, not nested absolute values".into());
    }
    // A pair of single bars holds no ket: Perl reads each `|…\rangle` as a ket before it can close an
    // absolute value (its ordered alternatives, MathGrammar:298-303), so `|a\rangle\langle
    // a||b\rangle\langle b|` is a product of projectors, never an absolute value around
    // `|b\rangle\langle b|` (57bp review; `absExpression` itself only forbids evaluation bars, :410).
    if single_bar_pair && holds_ket(&arg) {
      return Err("fenced: an absolute value holds no ket".into());
    }
    // Absolute-value `|x|`. The kerned-stack `\left|\left|x\right|\right|`
    // (double-bar norm) and triple variant are now recognized at the
    // grammar level via stretchy_(norm|triple_norm)_fenced (task #263),
    // so we no longer need a post-hoc tree-inspecting promotion here.
    let op = XProps {
      meaning: Some(Cow::Borrowed("absolute-value")),
      ..XProps::default()
    };
    let open_m = morph_vertbar(open, "OPEN", ctxt.nodes);
    let close_m = morph_vertbar(close, "CLOSE", ctxt.nodes);
    let fence = interpret_delimited(op.into(), vec![open_m, arg, close_m], ctxt)?;
    Ok(Some(if bar_fence {
      mark_bar_fence(fence, single_bar_pair, evaluation_bar_inside)?
    } else {
      fence
    }))
  } else {
    // A `\left\|…\right\|` pair is bars, as in Perl (the lexer's stretchy bars; `\lVert…\rVert`
    // is an OPEN/CLOSE group).
    let bar_fence = operator_category(&open) == Some("LEFT_STRETCHY_VERTBAR");
    let mark = |fence: XM| {
      if bar_fence {
        mark_bar_fence(fence, false, false)
      } else {
        Ok(fence)
      }
    };
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
      mark(interpret_delimited(
        op.into(),
        vec![open, arg, close],
        ctxt,
      )?)
      .map(Some)
    } else {
      let op = xnew(op_name);
      mark(interpret_delimited(
        op.into(),
        vec![open, arg, close],
        ctxt,
      )?)
      .map(Some)
    }
  }
}

/// What a bracket or brace pair encloses, when it is a list of two or more items: a `list`/`formulae`
/// Dual whose presentation is `[item, separator, item, …]`.
enum FencedList {
  /// Every item an expression: Perl's `Fence` over the items (`OPEN Expression (punct
  /// Expression)+ CLOSE`, MathGrammar:462, 472-475).
  Expressions,
  /// An item is a relation: one `Formulae` item (`OPEN Formulae CLOSE`, MathGrammar:69).
  Formulae,
}

fn fenced_list(arg: &XM) -> Option<FencedList> {
  let XM::Dual(content, presentation, ..) = arg else {
    return None;
  };
  let XM::Apply(op, args, ..) = &**content else {
    return None;
  };
  let XM::Token(props, _) = &*op.0 else {
    return None;
  };
  let XM::Wrap(items, ..) = &**presentation else {
    return None;
  };
  let meaning = props.meaning.as_deref();
  if !matches!(meaning, Some("list" | "formulae"))
    || args.0.len() < 2
    || !presents_its_items_alone(presentation, args.0.len())
  {
    return None;
  }
  Some(
    if meaning == Some("formulae") || items.iter().step_by(2).any(is_relational_item) {
      FencedList::Formulae
    } else {
      FencedList::Expressions
    },
  )
}

/// `[open, items and separators as written, close]` from a list `fenced_list` admitted.
fn fenced_list_items(open: XM, arg: XM, close: XM) -> Vec<XM> {
  let XM::Dual(_, presentation, ..) = arg else {
    unreachable!("fenced_list admits only a Dual");
  };
  let XM::Wrap(items, ..) = *presentation else {
    unreachable!("fenced_list admits only a Wrap presentation");
  };
  let mut stuff = Vec::with_capacity(items.len() + 2);
  stuff.push(open);
  stuff.extend(items);
  stuff.push(close);
  stuff
}

/// A fenced list holding a relation, as Perl's `NewFormulae` names it: `formulae`, which
/// `rename_fenced_lists` leaves alone.
fn as_formulae(mut arg: XM) -> XM {
  if let XM::Dual(ref mut content, ..) = arg
    && let XM::Apply(ref mut op, ..) = **content
    && let XM::Token(ref mut props, _) = *op.0
  {
    props.meaning = Some(Cow::Borrowed("formulae"));
  }
  arg
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

/// Perl `%balanced` (MathParser.pm:1348-1356): the close each open delimiter
/// balances.
fn balanced_close(open: &str) -> Option<&'static str> {
  Some(match open {
    "(" => ")",
    "[" => "]",
    "{" => "}",
    "|" => "|",
    "||" => "||",
    "\u{2016}" => "\u{2016}",
    "\u{2980}" => "\u{2980}",
    "\u{230A}" => "\u{230B}", // lfloor, rfloor
    "\u{2308}" => "\u{2309}", // lceil, rceil
    "\u{2329}" => "\u{232A}", // angle brackets (deprecated code points)
    "\u{27E8}" => "\u{27E9}", // angle brackets
    "\u{2225}" => "\u{2225}", // lVert, rVert
    _ => return None,
  })
}

/// Perl `p_getValue(realizeXMNode($x))` (MathParser.pm:135-150, 1070-1078): a lexeme's value
/// read through the node it names when it is an `XMRef`, as a gathered/split row's content
/// branch holds them (arXiv 2605.00284 split `u(t,x)`, 2605.01361 `\{i\mid…\}`).
fn realized_value<'a>(xm: &'a XM, ctxt: &ActionContext) -> Result<Cow<'a, str>, Box<dyn Error>> {
  match xm {
    XM::Lexeme(lex, _) => {
      let node = lookup_lex_node(lex, ctxt.nodes)?;
      Ok(Cow::Owned(p_get_value(&realize_xmnode(
        node,
        ctxt.document,
      ))))
    },
    other => other.get_value(ctxt.nodes),
  }
}

/// Perl `isMatchingClose` (MathParser.pm:1379-1384): the close's value is the
/// one the open's value balances, each read through `realizeXMNode`, so a
/// delimiter that is an `XMRef` answers with the value of the node it names.
fn is_matching_close(open: &XM, close: &XM, ctxt: &ActionContext) -> bool {
  let (Ok(open), Ok(close)) = (realized_value(open, ctxt), realized_value(close, ctxt)) else {
    return false;
  };
  balanced_close(&open).is_some_and(|expect| expect == close)
}

/// An empty fence from GENERIC open and close delimiters: only when the close
/// balances the open (Perl MathGrammar:463-464 `balancedClose`, :729). Any other
/// pair fails the parse, as in Perl, which leaves the formula unparsed with its
/// content kept. `\mathopen{}\mathclose{\left(x\right)}` lexes as an empty OPEN
/// and a CLOSE that CARRIES the `x`; building `list@()` from them dropped the
/// `x` from the content tree (arXiv 2605.13448, 2605.22010, 2605.21750;
/// guard `perfect_kernel_batch56::empty_fence_needs_a_balanced_close`).
pub fn balanced_empty_fenced(
  rule_id: i32,
  args: Vec<Option<XM>>,
  prag: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  if let (Some(Some(open)), Some(Some(close))) = (args.first(), args.get(1))
    && !is_matching_close(open, close, &ctxt)
  {
    return Err("empty fence: the close does not balance the open (Perl balancedClose)".into());
  }
  empty_fenced(rule_id, args, prag, ctxt)
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
  let o = realized_value(&open, &ctxt)?;
  let c = realized_value(&close, &ctxt)?;
  // Perl's `%enclose2` (MathParser.pm:1368-1373) is keyed by the delimiters AND the separator, and
  // names comma pairs only; any other pair falls to `list` (`Fence`, :1405-1409): `(a;b)`, `[a;b]`,
  // `(a;b]` — `[\mathbf{a};\mathbf{b}]` read `closed-interval` (2605.00423), `K_t=[K_{t-1};k_t]`
  // (2605.00435; 57ay review).
  // A reversed pair is the French open interval, written with `;` as often as with `,` (divergence
  // #362): `]0;1[` stays `open-interval`, which Perl cannot parse at all.
  let reversed = (o.as_ref(), c.as_ref()) == ("]", "[");
  let op_meaning = if realized_value(&sep, &ctxt)? != "," && !reversed {
    "list"
  } else {
    match (o.as_ref(), c.as_ref()) {
      ("(", ")") | ("]", "[") => "open-interval",
      ("[", "]") => "closed-interval",
      ("[", ")") => "closed-open-interval",
      ("(", "]") => "open-closed-interval",
      ("⟨", "⟩") => "list", // angle brackets: ⟨a,b⟩ → list, not tuple
      _ => "tuple",
    }
  };

  // Create operator as XM::Token with meaning attribute
  let op: XM = XProps {
    meaning: Some(Cow::Borrowed(op_meaning)),
    ..XProps::default()
  }
  .into();

  let ref_args = create_xmrefs(&mut [&mut arg1, &mut arg2], ctxt)?;
  // Marked here, where the delimiters' values are known (see `is_unbalanced_fence`).
  let mut meta = Meta::default();
  meta.unbalanced_fence = balanced_close(&o) != Some(c.as_ref());

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
    meta,
  )))
}

/// Perl's Fence (MathParser.pm): generalized fenced expression with
/// comma-separated items. Determines meaning from delimiter+punctuation
/// pattern using the Perl enclose tables.
/// Receives: open, item1, punct1, item2, [punct2, item3, ...], close
pub fn fence(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  // Collect all non-None args into a flat stuff vector
  let stuff: Vec<XM> = args.into_iter().flatten().collect();
  if stuff.len() < 2 {
    // A fence needs at least an open + close delimiter. A degenerate match
    // (all-None args, or a single surviving delimiter) would panic on
    // `stuff[0]` / `stuff[len-1]` or underflow `len - 2` — prune this parse.
    return Err("fence: need at least open + close delimiters".into());
  }
  // Perl reads the delimiters and the punctuation through `realizeXMNode` (MathParser.pm:1398,
  // 1402): in a gathered/split row's content branch they are XMRefs, and `|y|` there is
  // `absolute-value@(y)` as inline (golden
  // tests/parse/bar_pairs.tex#named_vertbar_parses_as_absolute_value).
  let o = realized_value(&stuff[0], &ctxt)?;
  let c = realized_value(&stuff[stuff.len() - 1], &ctxt)?;
  // Count items (every other element between open and close is an item)
  let n = (stuff.len() - 2).div_ceil(2); // number of items
  // Get first punctuation value for enclose2/encloseN lookup
  let p = if n >= 2 {
    realized_value(&stuff[2], &ctxt).ok()
  } else {
    None
  };
  let p_str = p.as_deref().unwrap_or(",");
  // Canonicalize the single "such-that / divides / given" bar. `\mid` reaches
  // here as ∣ (U+2223) or the CS-name "mid" (def_math infers a name from the
  // CS, unlike Perl's `DefMathI('\mid', undef, …)`), whereas bare `|`/`\vert`
  // reach here as "|". They are the same separator for the conditional /
  // conditional-set readings — Perl's VERTBAR terminal is content-agnostic
  // (MathGrammar:797) — so without this, `\{x\mid P\}` / `(a\mid b)` fell to
  // `list` instead of `conditional-set` / `conditional`. Mirror the lexeme-side
  // canonicalization in util.rs (VERTBAR:mid → VERTBAR:|).
  let p_str = if matches!(p_str, "\u{2223}" | "mid") {
    "|"
  } else {
    p_str
  };
  // A metarelation between the items other than a colon is one relation, fenced as one item (a
  // colon is a set-builder's in braces, Perl `suchThatOp`; elsewhere a separator here, `(a:b)`
  // `list@(a, b)`, where Perl reads `a colon b` — #366): Perl reads `OPEN Formulae CLOSE` with a `metarelopFormula` (MathGrammar:69, :118-125) and
  // `Fence` of one item (MathParser.pm:1412-1415) — `(p\iff q)` p iff q, `\{a\iff b\}` set@(a iff b),
  // `\{\Gamma\vdash A,B\}` set@(Gamma proves list@(A, B)); `suchThatOp` takes a colon only (:499-501).
  // Naming the pair from the tables dropped the relation (`list@(p, q)`; `(S\not\vdash c)` lost its
  // negation; 57bk, 2605.08011, 2605.06214, 2605.25146, 2605.00251).
  if n == 2
    && p_str != ":"
    && matches!(&stuff[2], XM::Lexeme(lex, _) if lex.starts_with("METARELOP:"))
  {
    let [open, left, relop, right, close]: [XM; 5] = stuff
      .try_into()
      .map_err(|_| "fence: a metarelation between two items")?;
    // A list beside the metarelation holding a relation is Perl's `Formulae`, as beside a bar:
    // `\{a<1,b\vdash c\}` set@(proves@(formulae@(a < 1, b), c)) (57bk review).
    let (left, right) = (
      relation_list_as_formulae(left),
      relation_list_as_formulae(right),
    );
    let relation = infix_relation(
      rule_id,
      vec![Some(left), Some(relop), Some(right)],
      pragmas,
      ActionContext {
        nodes:    ctxt.nodes,
        document: &mut *ctxt.document,
      },
    )?
    .ok_or("fence: no relation")?;
    return fenced(
      rule_id,
      vec![Some(open), Some(relation), Some(close)],
      pragmas,
      ctxt,
    );
  }

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
  // Change VERTBAR separators inside fences to MIDDLE role — Perl's `MorphVertbar(…,'MIDDLE')` for
  // the set-builder bar (MathGrammar:500); Perl marks the `(a|b)` bar a MODIFIEROP `conditional`
  // (:261-263) instead, an older convention here. Separators are at the even indices ≥ 2 of
  // [open, item, sep, item, ..., close]. A stretchy `\left|`/`\right|` divider
  // (`P\left(\left.A\right|B\right)`, 57ap) is re-roled on a copy: the parse may still be pruned.
  let last = stuff.len().saturating_sub(1);
  if (2..last)
    .step_by(2)
    .any(|i| left_bar_pairs_an_evaluation_bar(&stuff[i], &stuff[i + 1]))
  {
    return Err("fence: a `\\left|` divider pairs the item's evaluation bar".into());
  }
  let mut stuff: Vec<XM> = stuff
    .into_iter()
    .enumerate()
    .map(|(i, item)| match item {
      XM::Lexeme(ref lex, _)
        if i >= 2
          && i < last
          && i % 2 == 0
          && (lex.starts_with("LEFT_STRETCHY_VERTBAR:")
            || lex.starts_with("RIGHT_STRETCHY_VERTBAR:")) =>
      {
        morph_vertbar(item, "MIDDLE", ctxt.nodes)
      },
      other => other,
    })
    .collect();
  for i in (2..stuff.len().saturating_sub(1)).step_by(2) {
    match &mut stuff[i] {
      XM::Token(props, _) if props.role.as_deref() == Some("VERTBAR") => {
        props.role = Some(Cow::Borrowed("MIDDLE"));
      },
      &mut XM::Lexeme(ref lex, ref meta) if lex.starts_with("VERTBAR:") => {
        // For lexemes, change the role on the underlying DOM node
        if let Some(ref cv) = meta.curry_level {
          let cv_str = cv.to_string();
          if let Some(idx_str) = cv_str.strip_prefix(':')
            && let Ok(lex_idx) = idx_str.parse::<usize>()
          {
            let idx = if lex_idx > 0 { lex_idx - 1 } else { 0 };
            if idx < ctxt.nodes.len() {
              let mut node = ctxt.nodes[idx].clone();
              let _ = node.set_attribute("role", "MIDDLE");
            }
          }
        }
      },
      _ => {},
    }
  }
  if matches!(op_meaning, "conditional" | "conditional-set") {
    stuff = stuff
      .into_iter()
      .enumerate()
      .map(|(i, item)| {
        if i % 2 == 1 {
          relation_list_as_formulae(item)
        } else {
          item
        }
      })
      .collect();
  }
  interpret_delimited(op, stuff, ctxt).map(Some)
}

/// A list beside a conditional bar that holds a relation is Perl's `Formulae`
/// (`FormulaNOBar suchThatOp Formulae`, MathGrammar:487-491; `NewFormulae`, MathParser.pm:1439-1448),
/// as a fenced one is (`fenced_list`): `\{x : a<1, b<2\}` is
/// `conditional-set@(x, formulae@(a < 1, b < 2))`, `\{x|y,z\}` keeps `list@(y, z)`.
fn relation_list_as_formulae(arg: XM) -> XM {
  if matches!(fenced_list(&arg), Some(FencedList::Formulae)) {
    as_formulae(arg)
  } else {
    arg
  }
}

/// `[a|b]` / `[a \mid b]` — a bracketed conditional. Perl produces
/// `delimited-[]@(conditional@(a,b))`. Unlike `(a|b)` (→ `conditional@`) and
/// `\{a|b\}` (→ `conditional-set@`), the bare `a|b` conditional reduces only at
/// statement level (not as an `expression`), so `[a|b]` had no fence rule and
/// fell to `ltx_math_unparsed` — even though `[(a|b)]` already works. Build the
/// inner `conditional@(a,b)` with a delimiter-less presentation, then wrap it in
/// `delimited-[]` via the same `fenced` path `[(a|b)]` uses. `E[X|Y]`
/// (conditional expectation) is the canonical witness.
pub fn bracket_conditional(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  let mut stuff: Vec<XM> = args.into_iter().flatten().collect();
  // Expect [lbracket, a, bar, b, rbracket]; bail defensively otherwise.
  if stuff.len() != 5 {
    return Ok(None);
  }
  let rbracket = stuff.pop().unwrap();
  let b = stuff.pop().unwrap();
  let bar = stuff.pop().unwrap();
  if left_bar_pairs_an_evaluation_bar(&bar, &b) {
    return Err("bracket_conditional: a `\\left|` divider pairs the item's evaluation bar".into());
  }
  let mut a = relation_list_as_formulae(stuff.pop().unwrap());
  let mut b = relation_list_as_formulae(b);
  let lbracket = stuff.pop().unwrap();
  // Inner conditional@(a,b) — refs created via a ctxt reborrow so the original
  // `ctxt` remains available for the outer `fenced` wrap.
  let inner_refs = {
    let inner_ctxt = ActionContext {
      nodes:    ctxt.nodes,
      document: &mut *ctxt.document,
    };
    create_xmrefs(&mut [&mut a, &mut b], inner_ctxt)?
  };
  let cond_op: XM = XProps {
    meaning: Some(Cow::Borrowed("conditional")),
    ..XProps::default()
  }
  .into();
  let inner = XM::Dual(
    Box::new(XM::Apply(
      cond_op.into(),
      inner_refs.into(),
      XProps::default(),
      Meta::default(),
    )),
    Box::new(XM::Wrap(
      vec![a, bar, b],
      XProps::default(),
      Meta::default(),
    )),
    XProps::default(),
    Meta::default(),
  );
  // Wrap in delimited-[] using the standard single-arg fenced path.
  fenced(
    rule_id,
    vec![Some(lbracket), Some(inner), Some(rbracket)],
    pragmas,
    ctxt,
  )
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

/// Does `trailer` close a script's content? Only Perl's `Subscript`/`Superscript` start rules end in
/// `endPunct(?)`, which wraps the content in a one-item list (MathGrammar:84-92, `endPunct : PUNCT |
/// PERIOD` :129; MathParser.pm:1463-1475): `x_{a,}` x _ (list@(a)), `x_{a;}` too. Elsewhere Perl has no
/// closing mark — `parse_single` sets it aside for `Anything,` (MathParser.pm:657-670), and plain
/// `Anything` (an `XMArg` or `XMWrap`, MathGrammar:64-82) fails on it — and Rust reads it as presentation
/// (OXIDIZED_DESIGN #361): `\boxed{a+b.}` a + b, where the mark's value once chose `list@` for any
/// container (763 formulas in 82 papers of the 57ar A/B; 2605.00380 `\boxed{…}`, 2605.01199
/// `\xrightarrow{a.s.}`). The script is the mark's container: its `rule` attribute (base_xmath.rs,
/// tex_math.rs `XMArg rule='Subscript'`) — a gathered row's content branch holds an XMRef, whose
/// container is no script. A wide space's stand-in mark is a detached node (`filter_hints`,
/// util.rs), placed right after the node it follows: its container is that node's (`x_{a\qquad}`,
/// 57ax review).
fn closes_a_script(trailer: &XM, ctxt: &ActionContext) -> bool {
  let XM::Lexeme(lex, _) = trailer else {
    return false;
  };
  let Ok(node) = lookup_lex_node(lex, ctxt.nodes) else {
    return false;
  };
  let container = node.get_parent().or_else(|| {
    let position = ctxt.nodes.iter().position(|n| n == node)?;
    ctxt.nodes.get(position.checked_sub(1)?)?.get_parent()
  });
  container
    .and_then(|container| container.get_attribute("rule"))
    .is_some_and(|rule| matches!(rule.as_str(), "Subscript" | "Superscript"))
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
  let in_a_script = closes_a_script(&trailer, &ctxt);
  let mut ref_arg = create_xmrefs(&mut [&mut arg], ctxt)?;
  if ref_arg.is_empty() {
    // create_xmrefs skips ephemeral variants (XMHint etc.), so refs come back
    // empty when the script content was spacing-only (e.g. `x^{\,,}`). A Dual
    // with an XMRef to nothing is meaningless and `remove(0)` would panic —
    // fall back to a bare Wrap, mirroring the `fenced` empty-refs guard
    // (semantics.rs:2138, hep-ph/9210235).
    return Ok(Some(XM::Wrap(
      vec![arg, trailer],
      XProps::default(),
      Meta::default(),
    )));
  }
  let content = if in_a_script {
    // Perl `NewList(item, punct)` for a script's `endPunct` (MathParser.pm:1463-1475): a one-item
    // list, whatever the mark.
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
  unp!(args => start_script, content, _end_script);
  // A float script of an absent base (Perl `NewScript(Absent(), …)`, MathGrammar:79-80), its script
  // the parsed content, bundled as `faux_wrap` does: re-reading the script from the DOM loses a
  // content XMDual (`\Gamma^{{}^{(2)}}_{ij}`, `{}^{(a,b)}_{c}` gave an XMRef with no idref, 57bj review).
  let script = XM::Wrap(
    vec![
      start_script.unwrap(),
      content.unwrap_or(XM::Token(XProps::default(), Meta::default())),
    ],
    XProps::default(),
    Meta::default(),
  );
  new_script(None, script, ctxt)
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

/// Perl `DecorateOperator` (MathParser.pm:1649-1654), applied per script by `addOpDecoration`
/// (MathGrammar:692-697): the operator takes the script as any base does (`NewScript`), and the
/// scripted operator keeps the operator's role, so it still reads as an operator of its kind —
/// `a\leq_k b` is `a <= _ k b`, a relation, not `(<= _ k)@(a, b)` (arXiv 2605.03594
/// `\lesssim_{\kappa,L,U}`, 2605.28533 `<_{FOSD}`, 2605.20841 `\equiv_D`, `\lor_G`).
pub fn decorate_operator(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => op, script);
  let role = op.as_ref().and_then(|op| operator_role(op, ctxt.nodes));
  let Some(script) = script else {
    return Err("decorate_operator: no script".into());
  };
  Ok(new_script(op, script, ctxt)?.map(|mut decorated| {
    if let XM::Apply(_, _, ref mut props, _) = decorated {
      props.role = role.map(Cow::Owned);
    }
    decorated
  }))
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

    // Perl NewScript L1624-1643: a pre-script donates its lpadding to the new
    // combined app, a post/mid script its rpadding. Capture now (owned), before
    // `script_lex` is consumed below, so a thinspace folded onto the script
    // marker by filter_hints (e.g. `\,` in `x^2\,dx`) rides onto the rebuilt app.
    let (lpad_xfer, rpad_xfer): (Option<String>, Option<String>) = if x == "pre" {
      (
        script_wrap
          .get_attribute("lpadding")
          .filter(|s| !s.is_empty()),
        None,
      )
    } else {
      (
        None,
        script_wrap
          .get_attribute("rpadding")
          .filter(|s| !s.is_empty()),
      )
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
    let app_props = XProps {
      lpadding: lpad_xfer.map(Cow::Owned),
      rpadding: rpad_xfer.map(Cow::Owned),
      ..XProps::default()
    };
    Ok(Some(XM::Apply(
      op.into(),
      Args(vec![base_arg, script_arg]),
      app_props,
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
    Some(XM::Apply(op, _, _props, meta)) => {
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
    Some(XM::Lexeme(lex, _)) => {
      if let Ok(node) = lookup_lex_node(lex, ctxt.nodes) {
        let sp = node.get_attribute("scriptpos").unwrap_or_default();
        if !sp.is_empty() {
          let (bx, bl) = parse_scriptpos(&sp);
          return (bx, bl, false, 0);
        }
      }
      ("post", 0, false, 0)
    },
    Some(XM::Token(props, _)) => {
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
    XM::Apply(_, args, ..) => match args.0.get(n) {
      Some(t) => Ok(t.clone()),
      None => Ok(None),
    },
    // Other XM variants (Token, Dual, Wrap, Choices, Arg, Ref) don't
    // carry positional args — Perl's obtain_arg returns undef for these.
    _ => Ok(None),
  }
}

/// The `meaning` on an `XM::Dual`'s content operator, if it is an `Apply`
/// whose operator is a meaning-bearing token. Used to recognise the
/// `qm_bra`/`qm_ket` fences (content `Apply(meaning="bra"/"ket")`).
fn dual_content_meaning(xm: &XM) -> Option<&str> {
  if let XM::Dual(content, ..) = xm
    && let XM::Apply(op, ..) = &**content
    && let XM::Token(props, _) = &*op.0
  {
    return props.meaning.as_deref();
  }
  None
}

/// True iff the leftmost factor of `xm` (descending through the first arg of
/// invisible-times/prefix applications) is a Dirac `bra` (`⟨…|`).
fn reaches_dirac_bra_on_left(xm: &XM) -> bool {
  if dual_content_meaning(xm) == Some("bra") {
    return true;
  }
  // A script closes its bra: `\langle 0|_Z|0\rangle` is (bra@(0))_Z·ket@(0), as Perl (57bp review).
  if script_base(xm).is_some() {
    return false;
  }
  if let XM::Apply(_, args, ..) = xm
    && let Some(Some(first)) = args.0.first()
  {
    return reaches_dirac_bra_on_left(first);
  }
  false
}

/// True iff the rightmost factor of `xm` (descending through the last arg of
/// applications) is a Dirac `ket` (`|…⟩`).
fn reaches_dirac_ket_on_right(xm: &XM) -> bool {
  if dual_content_meaning(xm) == Some("ket") {
    return true;
  }
  if script_base(xm).is_some() {
    return false;
  }
  if let XM::Apply(_, args, ..) = xm
    && let Some(Some(last)) = args.0.last()
  {
    return reaches_dirac_ket_on_right(last);
  }
  false
}

/// Does the product `left · right` split a Dirac bracket? Marpa also reads one `⟨…|…|…⟩` span as the
/// product bra `⟨…|` · (middle) · ket `|…⟩`, which splits its matched `⟨…⟩` across the factors: one
/// matrix element, `qm_bracket`'s (Perl's `maybeBraket`, MathGrammar:381-393). The product splits one
/// where a factor of `left` reaches a bra, `right` ends in a ket — scripted too, the script being the
/// bracket's (`\langle a|H|b\rangle_A`, Perl's `addScripts` after `maybeBraket`) — and the factors
/// between them are a label: not none, and none a label may not hold (`is_forbidden_dirac_label`, and
/// no open bra among them). A bra right before the ket, or before a ket or an open bra, is a factor of
/// its own: Perl's `maybeBra` falls back to the bra when no `ketExpression` follows its bar, which
/// cannot start one (`$forbidVertBar`, MathGrammar:301, :373-379) — `\langle x||y\rangle` bra@(x)·ket@(y),
/// `\langle x||y\rangle c` bra@(x)·ket@(y)·c, `\langle a||b\rangle|c\rangle` bra@(a)·ket@(b)·ket@(c) (57bt;
/// 2605.29622 `\langle ij||ab\rangle`, unparsed while every bra-then-ket product was refuted). Every factor
/// is looked at, not the first alone: `c\langle a|H|b\rangle` is c·⟨a|H|b⟩ only, as `\langle a|H|b\rangle c`
/// is ⟨a|H|b⟩·c. A ket before a bra (`|…⟩⟨…|`, an outer product) splits nothing.
fn splits_a_dirac_bracket(left: &XM, right: &XM) -> bool {
  let ket = script_base(right).unwrap_or(right);
  if !reaches_dirac_ket_on_right(ket) {
    return false;
  }
  // What `right` holds before its ket — an operator or a function applied to it (`H|b\rangle`, `\nabla|b\rangle`) —
  // is middle.
  let right_holds_more = dirac_meaning(ket) != Some("ket");
  let factors = product_factors(left);
  factors.iter().enumerate().any(|(i, factor)| {
    let middle = &factors[i + 1..];
    reaches_dirac_bra_on_left(factor)
      && (right_holds_more || !middle.is_empty())
      && !middle.iter().any(|item| is_forbidden_dirac_label(item))
      && !middle
        .windows(2)
        .any(|pair| ends_with_bra(pair[0]) && !opens_with_a_bar(pair[1]))
  })
}

/// `function_factor bigop_operand`: a function, scripted or not, times the bigop after it —
/// Perl's `Factor moreFactors` (`\min_\theta\sum_i \ell_i` is min_θ * ∑…, `\log\int f`
/// log * ∫f). Not `apply_invisible_times`, whose left-function pruning (a function applies to
/// what follows it) would refute the only reading: a bigop application is a term, never a
/// function's argument. Golden: tests/parse/bigop_operands.tex#function_before_a_bigop_is_a_factor.
///
/// Mid-term (`tight_term function_factor bigop_operand`), the factors before the function join the
/// same product, as Perl's left-flattening `ApplyNary` (MathParser.pm:1497-1517) builds it — unless
/// the left product is fenced or has an id (:1503-1509):
/// `2\sin\int f` is times(2, sin, ∫f), `2x\sin\int f` times(2, x, sin, ∫f).
pub fn function_times_bigop(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  let bigop = args.pop().flatten();
  let function = args.pop().flatten();
  let takes = matches!((&function, &bigop), (Some(function), Some(bigop))
    if takes_the_limit_operator(function, bigop, &ctxt));
  let mut factors = args
    .pop()
    .flatten()
    .map(open_product_factors)
    .unwrap_or_default();
  factors.push(function);
  factors.push(bigop);
  let product = XM::Apply(
    invisible_times().into(),
    Args(factors),
    XProps::default(),
    Meta::default(),
  );
  Ok(Some(if takes {
    function_takes_a_limit_operator(product)
  } else {
    product
  }))
}

/// The factors of an unfenced invisible product without an id, `xm` alone otherwise: Perl's
/// left-flattening `ApplyNary` (MathParser.pm:1497-1517), which keeps a product whole whose `enclose`
/// or `xml:id` is set (:1503-1509).
fn open_product_factors(xm: XM) -> Vec<Option<XM>> {
  match xm {
    XM::Apply(op, Args(factors), props, meta)
      if meta.fenced.is_none()
        && props.id.is_none()
        && matches!(&*op.0, XM::Token(props, _)
          if props.meaning.as_deref() == Some("times")
            && props.content.as_deref() == Some("\u{2062}")) =>
    {
      factors
    },
    xm => vec![Some(xm)],
  }
}

/// `tight_term bigop_operand`, `bare_opfunction_term bigop_operand`: the product
/// (`apply_invisible_times`), whose big operator an expectation ending the left side takes
/// (`expectation_takes_the_big_operator`): `\mathbb{E}\frac1n\sum_i X_i` 𝔼@((1/n)·∑…),
/// `\frac1n\mathbb{E}\sum_i X_i` (1/n)·𝔼@(∑…).
pub fn product_before_a_big_operator(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  let takes = matches!(args.as_slice(), [Some(left), Some(bigop)]
    if takes_the_limit_operator(product_end(left, true), bigop, &ctxt));
  let product =
    apply_invisible_times(rule_id, args, pragmas, ctxt)?.map(expectation_takes_the_big_operator);
  Ok(if takes {
    product.map(function_takes_a_limit_operator)
  } else {
    product
  })
}

/// May the function before `bigop` take it (`function_takes_a_limit_operator`)? Not a trig function when the limit-type
/// operator's operand crosses the end of the trig argument it would be (`ends_trig_argument`, #367): `\sin\det A\,y`
/// stays sin·det@(A·y), as Perl, not sin@(det(A·y)) across the space (57cj.8 review; latent, its probes).
fn takes_the_limit_operator(function: &XM, bigop: &XM, ctxt: &ActionContext) -> bool {
  if head_category(script_nucleus(function)) != Some("TRIGFUNCTION") {
    return true;
  }
  let XM::Apply(_, Args(args), _, meta) = through_differentials(bigop) else {
    return true;
  };
  let [Some(operand)] = args.as_slice() else {
    return true;
  };
  if meta.fenced.is_some() || matches!(operand, XM::Dual(..) | XM::Wrap(..)) {
    return true;
  }
  let factors = product_factors(operand);
  !factors
    .windows(2)
    .enumerate()
    .any(|(k, pair)| ends_trig_argument(&factors[..=k], pair[1], ctxt))
}

/// `function_factor bigop_operand`: `function_times_bigop`, unless the function is an expectation,
/// which takes the big operator (`expectation_takes_the_big_operator`): `\mathbb{E}\sum_i X_i`
/// 𝔼@(∑…), where `\log\sum_i x_i` stays log·∑.
pub fn function_before_a_big_operator(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  Ok(function_times_bigop(rule_id, args, pragmas, ctxt)?.map(expectation_takes_the_big_operator))
}

/// An expectation takes the summation-like big operator after it (user ruling 2026-09-30, "follow
/// the mathematical meaning": the expectation of a big-operator expression is of the whole of it,
/// parentheses or not). Alone: `\mathbb{E}\sum_i X_i` 𝔼@(∑_i X_i),
/// `\varepsilon\mathbb{E}\int_0^T|u(s)|^2\,ds` ε·𝔼@(∫…) (2605.13204 S2.Ex6.m1),
/// `\mathbb{E}\sup_g\int g\,d\mu` (2605.03300 S3.E17.m2). After the coefficients of its bare
/// argument: `\mathbb{E}_{\{…\}}\frac1n\sum_{i=1}^n[…]` 𝔼@((1/n)·∑…) (2605.02116 A3.Ex163.m2),
/// `\eta\mathbb{E}_{x\sim\rho}\gamma\sum_a…` η·𝔼@(γ·∑…) (2605.06977 A3.Ex74.m1). As a function's
/// whole bare argument, which nests it (57cb): `\max_\pi\mathbb{E}_{\tau\sim\pi}\sum_t`
/// max_π@(𝔼@(∑…)) (2605.11975 S6.E24.m1), `\Tr\mathbb{E}\prod H` (2605.02768 S1.Ex1.m1). Any other
/// OPFUNCTION keeps Perl's product before a sum or an integral (`Factor moreFactors`, MathGrammar:258; `\log\sum_i x_i`
/// log·∑; a limit-type operator it takes, `function_takes_a_limit_operator`, #390),
/// in the expectation's argument too: `\mathbb{E}_{z_j}\min_\mu\frac{\tau}{m}\sum_j`
/// 𝔼@(min_μ@(τ/m)·∑…) (2605.02116 A5.Ex283.m1).
///
/// The grammar derives each shape once, as the product it is for any OPFUNCTION; this reads that
/// derivation, so no rule and no refused tree is added (`parse_tree_count_limits`). `product`'s
/// last factor is the big operator, and the deepest expectation the factor before it ends in takes
/// it (`ends_in_an_expectation`). Not a differential operator (DIFFOP, as
/// `ends_in_a_bigop_application` reads it, divergence #374): `\int_0^T\mathbb{E}X_t\,\dd t` keeps
/// its differential. A group closes the expectation (`\mathbb{E}[X]\sum_i Y_i` 𝔼@(X)·∑), and an
/// explicit MulOp is another rule (`\mathbb{E}X\cdot\sum_i Y_i`, `term mulop bigop_operand`).
fn expectation_takes_the_big_operator(product: XM) -> XM {
  match product {
    XM::Apply(op, Args(mut factors), props, meta)
      if is_invisible_times_op(&op.0)
        && matches!(factors.as_slice(), [.., Some(before), Some(bigop)]
          if is_summation_like(bigop) && ends_in_an_expectation(before)) =>
    {
      if let (Some(Some(bigop)), Some(Some(before))) = (factors.pop(), factors.pop()) {
        let taken = take_the_big_operator(before, bigop);
        if factors.is_empty() {
          return taken;
        }
        factors.push(Some(taken));
      }
      XM::Apply(op, Args(factors), props, meta)
    },
    product => product,
  }
}

/// Divergence #390 (57cj.8 review; surpass, KNOWN_PERL_ERRORS #400): a function — an OPFUNCTION or a trig function,
/// bare or scripted, or an operator's nest over one — right before a limit-type operator's application takes it as
/// its argument: `\log\det\Sigma` log@(det@(Σ)), `\sin\det A`, `\cos\sup_t u`, `\log\inf_x u`, `\nabla_x\log\det(A)`
/// ((∇_x)@(log))@(det@(A)), `\operatorname*{arg\,max}_s\log\det(L_s)`, and one through a derivative
/// (`\sin\partial_x\det A` sin@(∂_x(det A))) — where Perl, whose OPFUNCTION takes no big operator (`aBarearg`,
/// MathGrammar:323-331) and LIMITOP is one (:717), multiplies them: logarithm * determinant@(A). 243 formulas in 55 of
/// the 3,003 A/B papers, most `\log\det\Sigma` (2605.00130, 2605.26554, 2605.02883, 2605.03984, 2605.24401, 2605.25592,
/// 2605.14289; `\max_j\sup_z` 2605.02556, `\Im\lim` 2605.28932). A sum or an integral
/// stays a factor of its own (`\log\sum_i x_i` log·∑…, as Perl; an expectation takes it,
/// `expectation_takes_the_big_operator`). The grammar derives the product once, as for any big operator; this reads it.
fn function_takes_a_limit_operator(product: XM) -> XM {
  match product {
    XM::Apply(op, Args(mut factors), props, meta)
      if is_invisible_times_op(&op.0)
        && matches!(factors.as_slice(), [.., Some(before), Some(bigop)]
          if ends_in_a_function_head(before)
            && is_a_limit_operator_application(bigop)
            && !qualifies_the_limit_operator(last_function_head(before), bigop)) =>
    {
      if let (Some(Some(bigop)), Some(Some(before))) = (factors.pop(), factors.pop()) {
        let taken = take_a_limit_operator(before, bigop);
        if factors.is_empty() {
          return taken;
        }
        factors.push(Some(taken));
      }
      XM::Apply(op, Args(factors), props, meta)
    },
    product => product,
  }
}

/// An unapplied OPFUNCTION or trig function, bare or scripted, an operator's nest over one (`\nabla_x\log`), or a
/// function's bare application to one (`\log\log`, `\min_\theta\log`, `\log\max_i`, `\arg\min_x\log`, 57cj.9 and
/// 57cj.10 reviews; 2605.14289 `\log\exp\sup_x f`): what takes a limit-type operator's application
/// (`function_takes_a_limit_operator`), unless the head right before the operator names its variant
/// (`qualifies_the_limit_operator`).
fn ends_in_a_function_head(xm: &XM) -> bool {
  is_bare_function_head(xm)
    || matches!(xm, XM::Apply(Operator(op), args, _, meta)
      if meta.fenced.is_none()
        && (is_nested_operator(op, args) || is_bare_function_head(op))
        && matches!(args.0.as_slice(), [Some(inner)] if ends_in_a_function_head(inner)))
}

/// The head right before a limit-type operator in `xm`, which `ends_in_a_function_head`: `xm` when it is a bare or
/// scripted head, the head its bare argument ends in otherwise (`\log\arg` → `\arg`, `\nabla_x\log` → `\log`).
fn last_function_head(xm: &XM) -> &XM {
  match xm {
    XM::Apply(_, Args(args), ..) if !is_bare_function_head(xm) => match args.as_slice() {
      [Some(inner)] => last_function_head(inner),
      _ => xm,
    },
    _ => xm,
  }
}

/// Does `head` name the variant of the limit-type operator `bigop` rather than a function of its value? `\arg` before
/// an infimum or a supremum — `\arg\inf f(\theta)` is one arg-inf, not the complex argument of an infimum (2605.30648,
/// 2605.16560) — and `\operatorname{ess}` before every limit-type operator (`\operatorname{ess}\sup`,
/// `\operatorname{ess}\lim_{x\to a}f(x)`, `\operatorname{ess}\det A`): they keep Perl's product (57cj.9 review; SYNC row
/// "Math-parse residuals of the 57cj train"). `ess` is never a function of a value, only a qualifier (57cj.11 review);
/// `\arg` is one, the complex argument, so before any other operator it takes it: `\arg\det U` argument@(det U) (the
/// strong-CP phase `\bar\theta=\theta-\arg\det M_q`), `\arg\lim_{z\to0}f(z)` (57cj.10 review). Before a minimum or a
/// maximum, OPFUNCTIONs and no limit-type operator, both words apply (`\arg\min_x f` argument@(min_x f),
/// `\operatorname{ess}\max_x f` ess@(max_x f), as Perl): the one arg-min or ess-max operator is unmodelled (SYNC).
fn qualifies_the_limit_operator(head: &XM, bigop: &XM) -> bool {
  let XM::Lexeme(lex, _) = script_nucleus(head) else {
    return false;
  };
  // (`\arg` means `argument`; `\operatorname{arg}` spells it, 57cj.12 review)
  match lex.split(':').nth(1) {
    Some("argument" | "arg") => {
      head_meaning(bigop).is_some_and(|meaning| matches!(meaning, "infimum" | "supremum"))
    },
    Some("ess") => true,
    _ => false,
  }
}

/// The meaning a head's lexeme or token names (`LIMITOP:supremum:3` supremum), through its scripts and applications.
fn head_meaning(xm: &XM) -> Option<&str> {
  match xm {
    XM::Lexeme(lex, _) => lex.split(':').nth(1),
    XM::Token(props, _) => props.meaning.as_deref(),
    XM::Apply(Operator(op), ..) => head_meaning(script_base(xm).unwrap_or(op)),
    XM::Dual(_, presentation, ..) => head_meaning(presentation),
    _ => None,
  }
}

/// `before`, which `ends_in_a_function_head`, taking `bigop`: a head or an operator's nest applies to it
/// (`((\nabla_x)@(\log))@(\det A)`), a function's bare application takes it in its argument (`\log@(\log@(\det A))`).
fn take_a_limit_operator(before: XM, bigop: XM) -> XM {
  match before {
    // (an OPFUNCTION's or trig function's head: an operator's nest is none, it takes the operator curried)
    XM::Apply(op, Args(mut args), props, meta)
      if is_bare_function_head(&op.0) && args.len() == 1 =>
    {
      let inner = args
        .pop()
        .flatten()
        .map(|inner| take_a_limit_operator(inner, bigop));
      XM::Apply(op, Args(vec![inner]), props, meta)
    },
    before => XM::Apply(
      before.into(),
      Args(vec![Some(bigop)]),
      XProps::default(),
      Meta::default(),
    ),
  }
}

/// A limit-type operator's application (LIMITOP: `\det A`, `\sup_t u`, `\lim u`, `\dim V`), or a derivative's of one.
fn is_a_limit_operator_application(xm: &XM) -> bool {
  if is_differential(xm) {
    let operand = through_differentials(xm);
    return !std::ptr::eq(operand, xm) && is_a_limit_operator_application(operand);
  }
  matches!(xm, XM::Apply(..) | XM::Dual(..))
    && script_base(xm).is_none()
    && head_category(xm) == Some("LIMITOP")
}

/// A summation-like big operator, bare, scripted or applied: ∑, ∏, ∫, ⋃, sup, lim, det (BIGOP,
/// SUMOP, INTOP, LIMITOP — Perl's `bigop`, MathGrammar:717, less DIFFOP).
fn is_summation_like(xm: &XM) -> bool {
  matches!(
    head_category(xm),
    Some("BIGOP" | "SUMOP" | "INTOP" | "LIMITOP")
  )
}

/// Does `xm` end in an expectation that takes a big operator after it: a bare or scripted 𝔼
/// (`\mathbb{E}\sum`), an expectation's bare application (`\mathbb{E}\frac1n\sum`), or a function's
/// or operator's bare application whose argument does (`\max_\pi\mathbb{E}_\tau\sum`)? A group
/// application is closed: `\mathbb{E}[X]`, `\mathbb{E}_x[X]^2`.
fn ends_in_an_expectation(xm: &XM) -> bool {
  is_expectation_operator(xm)
    || bare_application(xm)
      .is_some_and(|(head, arg)| is_expectation_operator(head) || ends_in_an_expectation(arg))
}

/// An OPFUNCTION's or operator's application to a bare argument (`is_bare_operator_application`),
/// as (head, argument).
fn bare_application(xm: &XM) -> Option<(&XM, &XM)> {
  match xm {
    XM::Apply(Operator(head), Args(args), _, meta)
      if meta.fenced.is_none() && is_bare_operator_application(xm) =>
    {
      match args.as_slice() {
        [Some(arg)] => Some((&**head, arg)),
        _ => None,
      }
    },
    _ => None,
  }
}

/// `xm`, which `ends_in_an_expectation`, with its deepest expectation taking `bigop`: a bare one is
/// applied to it, an application takes it after its argument — `𝔼@(1/n)` 𝔼@((1/n)·∑…),
/// `max_π@(𝔼_τ)` max_π@(𝔼_τ@(∑…)).
fn take_the_big_operator(xm: XM, bigop: XM) -> XM {
  if is_expectation_operator(&xm) {
    return XM::Apply(
      xm.into(),
      Args(vec![Some(bigop)]),
      XProps::default(),
      Meta::default(),
    );
  }
  match xm {
    XM::Apply(head, Args(mut args), props, meta) => {
      let arg = match args.pop().flatten() {
        Some(arg) if ends_in_an_expectation(&arg) => take_the_big_operator(arg, bigop),
        // (a function ending the argument takes a limit-type operator: `\mathbb{E}\log\det\Sigma` 𝔼@(log@(det Σ)))
        Some(arg) => function_takes_a_limit_operator(invisible_product(arg, bigop)),
        None => bigop,
      };
      XM::Apply(head, Args(vec![Some(arg)]), props, meta)
    },
    xm => invisible_product(xm, bigop),
  }
}

/// `left` times `right`: one product, with `left`'s factors when it is an open one
/// (`open_product_factors`).
fn invisible_product(left: XM, right: XM) -> XM {
  let mut factors = open_product_factors(left);
  factors.push(Some(right));
  XM::Apply(
    invisible_times().into(),
    Args(factors),
    XProps::default(),
    Meta::default(),
  )
}

/// The `tight_term factor` product: invisible times, unless the grammar reads the factor as an
/// application of what precedes it, where the product would be a second derivation multiplying the
/// trees chain by chain (57bl review: five chains of three, 32 → 1,024 trees; ten `\nabla(a)(b)`
/// terms 1,024 trees, 1.21 GB). Refused, each mirroring the rule it shadows:
/// - a group in parentheses or brackets after a letter that follows an application
///   (`application_before_a_letter`, `letter_after_an_application_apply`; user ruling 2026-09-29):
///   `f(x)g(y)` f@(x)·g@(y) — unless the letter recurs in its group (`letter_recurs_in_its_group`,
///   57bz): `f(x)x(x+1)` f@(x)·x·(x+1);
/// - a delimited group after an operator's application to a delimited group, alone or ending a
///   product (`D(a)(b)`, `c\nabla(a)(b)`; `operator_application_apply` through `op_application`,
///   which `tight_term`, an OPFUNCTION and a closed nest take alike): (D@(a))@(b), whatever the
///   delimiters, as Perl's `addEasyArgs` takes any balanced group (`\nabla(a)\{b\}`), except a Dirac
///   bracket, which Perl multiplies (`\nabla(u)\langle a|b\rangle`). Not after a bar group
///   (`\nabla|u|(v)` has no curried derivation, 57bn).
pub fn factor_product(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  if let [Some(left), Some(right)] = args.as_slice() {
    // A postfixed group is its group here, under scripts too (`f(n)g(n)!` as `f(n)g(n)`,
    // `f(x)g(y)!^2`).
    let right = postfixed_operand(right).unwrap_or(right);
    if is_paren_or_bracket_group(right)
      && letter_after_an_application(left, &ctxt)
      && !letter_recurs_in_its_group(product_end(left, true), right, &ctxt)
    {
      return Err("factor_product: the letter after an application takes this group".into());
    }
    let last = product_end(left, true);
    if is_delimited_group(right)
      && is_operator_group_application(last)
      && operator_group_argument_is_delimited(last)
    {
      return Err("factor_product: the operator's application takes this group".into());
    }
  }
  apply_invisible_times(rule_id, args, pragmas, ctxt)
}

/// `application_before_a_letter speculative_item`: the letter after an application takes a group in
/// parentheses or brackets only, as the fenced-letters pragma reads a letter at the start of a
/// product (`is_dual_fenced_rhs`, pragmatics.rs); a brace, bar, floor or angle group multiplies there
/// and here — `P(A)P\{X>0\}` P@(A)·P·set, `U(t)H|\psi\rangle` U@(t)·H·ket, as Perl (57bl review).
pub fn letter_after_an_application_apply(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  // A postfixed letter's application (`letter_postfixed`) reads as the application it holds:
  // `f(n)g(n)!` f@(n)·(g@(n))! (57bw).
  if !args
    .get(1)
    .and_then(Option::as_ref)
    .map(|item| postfixed_operand(item).unwrap_or(item))
    .is_some_and(is_letter_application_to_a_paren_or_bracket_group)
  {
    return Err("letter_after_an_application_apply: only a paren or bracket group".into());
  }
  apply_invisible_times(rule_id, args, pragmas, ctxt)
}

/// `application_before_a_letter = speculative_item`: a chain starts with a letter's application to
/// a paren or bracket group only — `H|n\rangle f(n)` is H·ket·f·n, `P\{A\}P(B)` P·set·P·B, as the
/// same letter alone and Perl (57bn review: the letter after it had forced H@(ket)).
pub fn letter_application_to_a_group(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  _: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => item);
  if item
    .as_ref()
    .is_some_and(is_letter_application_to_a_paren_or_bracket_group)
  {
    Ok(item)
  } else {
    Err("letter_application_to_a_group: only a paren or bracket group".into())
  }
}

/// A letter's application (`speculative_prefix_apply`, the group kept whole) to a paren or bracket
/// group.
fn is_letter_application_to_a_paren_or_bracket_group(xm: &XM) -> bool {
  matches!(xm, XM::Apply(_, Args(items), ..)
    if matches!(items.as_slice(), [Some(group)] if is_paren_or_bracket_group(group)))
}

/// A delimiter's role and text, from a lexeme (`ROLE:text:index`) or a token.
fn delimiter_role_text(xm: &XM) -> Option<(&str, &str)> {
  match xm {
    XM::Lexeme(lex, _) => {
      let (role, rest) = lex.split_once(':')?;
      let (text, _index) = rest.rsplit_once(':')?;
      Some((role, text))
    },
    XM::Token(props, _) => Some((props.role.as_deref()?, props.content.as_deref()?)),
    _ => None,
  }
}

/// A fenced group written between parentheses or between brackets — what the fenced-letters pragma
/// reads as a letter's argument — no fenced modifier `(>0)`, which `speculative_prefix_apply` refuses.
fn is_paren_or_bracket_group(xm: &XM) -> bool {
  if is_fenced_modifier_dual(xm) {
    return false;
  }
  let XM::Dual(_, presentation, ..) = xm else {
    return false;
  };
  let XM::Wrap(items, ..) = &**presentation else {
    return false;
  };
  matches!(
    (
      items.first().and_then(delimiter_role_text),
      items.last().and_then(delimiter_role_text)
    ),
    (Some(("OPEN", "(")), Some(("CLOSE", ")"))) | (Some(("OPEN", "[")), Some(("CLOSE", "]")))
  )
}

/// Are these a group's written delimiters, an opening and a closing one (`group_factor`: parens,
/// brackets, braces, angle brackets, generic delimiters), not an absent `\right.`? A bar pair's
/// delimiters read OPEN/CLOSE too (`morph_vertbar`): on the right `is_function_group` refuses it
/// (`bar_fence`); on the left `prefix_apply` does not lift a bar group, so no `Wrap` argument shows.
fn are_group_delimiters(items: &[XM]) -> bool {
  matches!(
    (
      items.first().and_then(delimiter_role_text),
      items.last().and_then(delimiter_role_text)
    ),
    (
      Some(("OPEN" | "OTHER_OPEN", _)),
      Some(("CLOSE" | "OTHER_CLOSE", _))
    )
  )
}

/// A group `group_factor` derives: a fenced group between written delimiters, no fenced modifier,
/// and no Dirac bracket (a bar between its delimiters, `\langle a|b\rangle`).
fn is_delimited_group(xm: &XM) -> bool {
  is_function_group(xm)
    && !is_unbalanced_fence(xm)
    && matches!(xm, XM::Dual(_, presentation, ..)
    if matches!(&**presentation, XM::Wrap(items, ..)
      if are_group_delimiters(items)
        && !items[1..items.len() - 1].iter().any(|item| {
          delimiter_role_text(item).is_some_and(|(role, _)| role.ends_with("VERTBAR") || role == "MIDDLE")
        })))
}

/// An operator's application whose argument group has written delimiters (`operator_group_application`
/// takes a `group_factor`), lifted over them: presentation `Apply(op, [Wrap(open … close)])`.
fn operator_group_argument_is_delimited(xm: &XM) -> bool {
  matches!(xm, XM::Dual(_, presentation, ..)
    if matches!(&**presentation, XM::Apply(_, Args(args), ..)
      if matches!(args.as_slice(), [Some(XM::Wrap(items, ..))] if are_group_delimiters(items))))
}

/// Is `xm` a product ending in a bare unknown letter right after an application that
/// `application_before_a_letter` ends in: a letter's own (`speculative_item`), or a function's, an
/// OPFUNCTION's, a trig function's or an operator's application to a group (`delimited_application`,
/// `operator_group_application`; scripted heads as those rules take them)?
fn letter_after_an_application(xm: &XM, ctxt: &ActionContext) -> bool {
  let XM::Apply(Operator(op), Args(factors), ..) = xm else {
    return false;
  };
  if !is_product_operator(op) {
    return false;
  }
  let [before @ .., Some(letter)] = factors.as_slice() else {
    return false;
  };
  if !matches!(letter, XM::Lexeme(lex, _) if lex.starts_with("UNKNOWN:") || lex.starts_with("XDIFFUNK:"))
  {
    return false;
  }
  // An ellipsis is transparent to the chain (`ellipsis_after_an_application`): the letter follows
  // the factor before the ellipses, or opens the chain when they start the product.
  let mut before = before;
  let mut skipped = false;
  while let [rest @ .., Some(last)] = before
    && is_ellipsis(last, ctxt)
  {
    before = rest;
    skipped = true;
  }
  match before.last() {
    None => skipped,
    Some(Some(before)) => is_application_before_a_letter(before),
    Some(None) => false,
  }
}

/// `application_before_a_letter = elideop | ellipsis_id`: an ellipsis opens a chain (`\cdots g(n)`,
/// `\phi(\cdots\phi(W_1x))`).
pub fn ellipsis_opens_an_application_chain(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => item);
  if item.as_ref().is_some_and(|item| is_ellipsis(item, &ctxt)) {
    Ok(item)
  } else {
    Err("ellipsis_opens_an_application_chain: not an ellipsis".into())
  }
}

/// `application_before_a_letter (elideop | ellipsis_id)`: the chain goes on across an ellipsis
/// (`g(1)g(2)\cdots g(n)`, 2605.23467, 2605.10016, 2605.05078); the grammar offers no other ID
/// (`f(x)\infty g(y)` stays a product).
pub fn ellipsis_after_an_application(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  if !args
    .get(1)
    .and_then(Option::as_ref)
    .is_some_and(|item| is_ellipsis(item, &ctxt))
  {
    return Err("ellipsis_after_an_application: not an ellipsis".into());
  }
  apply_invisible_times(rule_id, args, pragmas, ctxt)
}

fn is_application_before_a_letter(xm: &XM) -> bool {
  match xm {
    // A differential operator's application to a group or to an application before a letter
    // (`diffop_group_application`): `\partial_{11}l(F(x),Y)f(x)` (2605.00581).
    XM::Apply(Operator(head), Args(args), _, meta)
      if meta.differential && is_bare_differential_operator(head) =>
    {
      matches!(args.as_slice(), [Some(operand)]
        if is_application_before_a_letter(operand) || is_delimited_group(operand))
    },
    // A letter's application to a group (`speculative_prefix_apply` keeps the group whole).
    XM::Apply(Operator(head), Args(args), ..) => {
      matches!(head.as_ref(), XM::Lexeme(lex, _) if lex.starts_with("UNKNOWN:") || lex.starts_with("XDIFFUNK:"))
        && matches!(args.as_slice(), [Some(group)] if is_paren_or_bracket_group(group))
    },
    // A head's application to a group, lifted over the delimiters (`prefix_apply`).
    XM::Dual(_, presentation, ..) => {
      let XM::Apply(Operator(head), Args(args), ..) = &**presentation else {
        return false;
      };
      if !matches!(args.as_slice(), [Some(XM::Wrap(..))]) {
        return false;
      }
      let head = head.as_ref();
      match script_base(head) {
        Some(base) => {
          matches!(
            operator_category(base),
            Some("OPFUNCTION" | "TRIGFUNCTION" | "OPERATOR")
          )
        },
        None => {
          matches!(head, XM::Lexeme(..) | XM::Token(..))
            && matches!(
              operator_category(head),
              Some("FUNCTION" | "OPFUNCTION" | "TRIGFUNCTION" | "OPERATOR")
            )
        },
      }
    },
    _ => false,
  }
}

/// `D(a)(b)`: a bare or scripted operator's application to one group applies to the next group
/// (Perl `nestOperators`' OPEN branch takes one Expression, MathGrammar:669-671, then `addOpFunArgs`
/// the next group); an application to a list is `addEasyArgs`', which ends the Factor —
/// `\nabla(u,w)(v)` nabla@(u, w)·v, as Perl (57bl review).
pub fn operator_application_apply(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  if !args
    .first()
    .and_then(Option::as_ref)
    .is_some_and(is_operator_group_application)
  {
    return Err("operator_application_apply: an operator's application to a list ends it".into());
  }
  // A fenced modifier annotates what precedes it (`\nabla(u)(>0)` annotated@(∇@(u), absent > 0),
  // as 57bk and Perl; 57bn review), as `speculative_prefix_apply` refuses it too.
  if args
    .get(1)
    .and_then(Option::as_ref)
    .is_some_and(is_fenced_modifier_dual)
  {
    return Err("operator_application_apply: a fenced modifier is no argument".into());
  }
  prefix_apply(rule_id, args, pragmas, ctxt)
}

/// A bare or scripted operator's application to one group, lifted over its delimiters: the head
/// that applies to the next group too (`D(a)(b)`, 57bl).
pub(crate) fn is_operator_group_application(xm: &XM) -> bool {
  matches!(xm, XM::Dual(content, presentation, ..)
    if matches!(&**content, XM::Apply(_, Args(items), ..) if items.len() == 1)
      && matches!(&**presentation, XM::Apply(Operator(op), ..) if is_bare_or_scripted_operator(op)))
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
  // Balanced-Dirac-delimiter refutation (`splits_a_dirac_bracket`): a bra, a label and a ket are one
  // bracket, `qm_bracket`'s, never three factors.
  if let (Some(l), Some(r)) = (&left, &right)
    && splits_a_dirac_bracket(l, r)
  {
    // Hard-reject via Err (not Ok(None)): Err prunes this parse tree so the
    // `qm_bracket` alternative survives; Ok(None) would instead yield a None
    // value that propagates up to a parent action (e.g. list_apply's
    // `right.unwrap()`) and panics.
    return Err("apply_invisible_times: refute Dirac bra·…·ket product (use qm_bracket)".into());
  }
  if let Some(ref l) = left {
    let operator = product_end(l, true);
    // (A differential operator's number takes the factors juxtaposed after it — a soft preference, the
    // pragma `DifferentiatedNumbersTakeTheirFactors` over `differentiated_number_sites`: refused here,
    // the product killed the last parse where the monomial cannot hold what follows, 57cj.3 review.)
    // … and its ellipsis the differential operator after it (`elided_diffop_application`, 57cj review):
    // `\partial_i\ldots\partial_j u` is ∂_i(…·∂_j u), not ∂_i(…)·∂_j u.
    if differentiates_an_ellipsis(operator, &ctxt)
      && right.as_ref().is_some_and(|r| {
        let first = product_end(r, false);
        is_differential(first) || is_bare_differential_operator(first)
      })
    {
      return Err(
        "apply_invisible_times: the differential operator's ellipsis takes the right".into(),
      );
    }
    if is_operator_head(operator) {
      // Perl's OPERATOR is a Factor (MathGrammar:312-313), so one ending a product applies as one
      // starting it does: what it takes cannot follow it in a product (`operator_takes`) —
      // `\nabla u` is ∇@(u), `\mu\nabla^2 u` μ·(∇²)@(u), `\nabla_x^2 u` ((∇_x)²)@(u) — and anything
      // else can: a big operator's application (`\nabla_x\log\det(A)` is (∇_x)@(log)·det(A),
      // 2605.03984, 2605.24401, 2605.25592, 2605.14289), an operator after a closed nest
      // (`\nabla\log\nabla^2 u` is ∇@(log)·(∇²)@(u)). Golden
      // tests/parse/operator_application.tex#operator_nests_over_an_operator.
      if right.as_ref().is_some_and(|r| operator_takes(operator, r)) {
        return Err("apply_invisible_times: the operator on the left takes the right".into());
      }
    } else {
      // OPFUNCTION/TRIGFUNCTION/FUNCTION tokens, scripted or not (`\log_e x`), absorb the next
      // argument via prefix_apply, NOT via invisible times — unless it is a function too: Perl's
      // `fgh` with all FUNCTION is f·g·h.
      // An OPFUNCTION or TRIGFUNCTION that ends a product applies as one starting it (it takes a
      // bare argument): `a\log b c` is a·log@(b c), not a·log·b·c.
      let role_of = |xm: &XM| match xm {
        XM::Token(..) | XM::Lexeme(..) => operator_role(xm, ctxt.nodes),
        _ => script_base(xm).and_then(|base| operator_role(base, ctxt.nodes)),
      };
      // Only when it would take what follows (a bare argument, `addOpFunArgs`); before anything
      // else — a big operator's application, an operator's — it takes nothing and the product goes
      // on (`moreFactors`): `\tfrac12\log\det(\Sigma)`, `\tau\log\sum_i e^{x_i}`.
      let role = role_of(l).or_else(|| {
        right
          .as_ref()
          .filter(|r| {
            let first = product_end(r, false);
            // … or a group, which `addEasyArgs` takes (:571-576): `\lambda^2\log(|A|)^2` is
            // λ²·(log@(|A|))², not λ²·log·(|A|)².
            is_bare_item(first)
              || is_function_group(script_nucleus(first)) && !is_unbalanced_fence(first)
          })
          .and_then(|_| role_of(operator))
          .filter(|r| matches!(r.as_str(), "OPFUNCTION" | "TRIGFUNCTION"))
      });
      let is_function_role =
        |role: Option<&str>| matches!(role, Some("OPFUNCTION" | "TRIGFUNCTION" | "FUNCTION"));
      // … nor an operator or its application, which is no function's argument (`aBarearg`):
      // `\log\nabla^2` is log·∇², `\log\nabla f` log·∇@(f), `\log\nabla(a-b)` log·∇@(a − b), and
      // `\min_h\operatorname*{R}(h,P)` as Perl (57bf lifted the last two; train A/B 2605.13395,
      // 2605.19415, 2605.28109).
      if is_function_role(role.as_deref())
        && !right.as_ref().is_some_and(|r| {
          is_function_role(operator_role(r, ctxt.nodes).as_deref())
            || is_unbalanced_fence(product_end(r, false))
            || is_operator_head(r)
            || is_operator_head(product_end(r, false))
            || is_operator_application(product_end(r, false))
        })
      {
        return Err(
          "apply_invisible_times: left is OPFUNCTION/TRIGFUNCTION/FUNCTION, prefer prefix_apply"
            .into(),
        );
      }
    }
  }
  // Perl's greedy `barearg` (MathGrammar:321-337): an operator applied to a bare argument takes
  // the bare arguments after it — `\nabla u v` is ∇@(u v), not ∇@(u)·v.
  if let (Some(l), Some(r)) = (&left, &right)
    && leaves_a_bare_argument(l, r, true, &ctxt)
  {
    return Err("apply_invisible_times: the operator on the left takes this bare argument".into());
  }
  // Early-action prune for the fenced-modifier shape on the RHS:
  // `x (>0)` / `x (\in C)` — the legitimate parse is
  // `annotated_fenced_modifier`, NOT `x * (>0)` (implicit-times)
  // and NOT `x@(>0)` (function-app, handled in `prefix_apply_applyop`).
  if let Some(ref r) = right
    && is_fenced_modifier_dual(r)
  {
    return Err(
      "apply_invisible_times: right is a fenced modifier expression — \
         prefer annotated_fenced_modifier"
        .into(),
    );
  }
  // Bigop application results should not participate in invisible-times on their right.
  // When ∫_0^∞ x^2 dx is parsed, both `∫_0^∞(x^2 dx)` (absorption) and
  // `∫_0^∞(x^2) * dx` (flat) exist. Prune the flat parse by rejecting
  // invisible-times where the left is Apply(bigop, ...).
  // Perl: addIntOpArgs/addOpArgs absorbs the full integrand; we match by pruning.
  if let Some(ref l) = left
    && is_bigop_or_scripted_bigop(l, ctxt.nodes)
  {
    return Err("apply_invisible_times: left is bigop/scripted bigop, prefer absorption".into());
  }
  // Note: bare OPFUNCTION absorption (diffd@(x) vs diffd*x) is handled by
  // the FunctionsPreferWiderAbsorption pragmatic, which compares competing
  // trees and rejects the narrow parse.
  // Perl: scripted function application — f^2(a), f'(a), g_n(x).
  // When left is a scripted Apply whose base has FUNCTION/OPFUNCTION/TRIGFUNCTION role,
  // and right is a fenced XMDual (from parenthesized fencing), produce function
  // application with XMDual wrapping instead of invisible times.
  if let Some(ref l) = left
    && let Some(ref r) = right
  {
    let is_scripted_function = is_scripted_function_head(l, ctxt.nodes);
    let is_fenced_dual = matches!(r, XM::Dual(c, p, _, _)
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

  // Perl: trigBarearg greedily absorbs ALL following bare factors: \sin xyz → sin(x*y*z).
  // Reject invisible_times(trig_app(args), bare_factor) — the factor should be absorbed
  // into the trig argument via trig_arg rule, not multiplied outside.
  if let (Some(l), Some(r)) = (&left, &right)
    && leaves_a_trig_bare_argument(l, r, &ctxt)
  {
    return Err(
      "apply_invisible_times: trig function should absorb bare factor via trig_arg".into(),
    );
  }
  // Perl: MaybeFunction — mark UNKNOWN tokens as possibleFunction when MATHPARSER_SPECULATE is set
  // and the right side is a delimited group (parenthesized)
  maybe_mark_possible_function(&mut left, &right, ctxt.nodes);

  // A large MULOP's application followed by a run of ellipses, then an item, in a bare argument's chain: the run was no
  // trailing one, so it and the item join the operand — `\log x\otimes y\cdots z` log@(x⊗(y·⋯·z)), as `x\otimes y\cdots z`
  // x⊗(y·⋯·z) at a term's level (Q11, #396; 57cj.20.Q11 review). A trailing run stays outside (user ruling 15).
  if let (Some(XM::Apply(Operator(times_op), Args(factors), times_props, times_meta)), Some(r)) =
    (&left, &right)
    && times_meta.fenced.is_none()
    && times_props.id.is_none()
    && is_invisible_times_op(times_op)
    && let [Some(application), run @ ..] = factors.as_slice()
    && !run.is_empty()
    && run.iter().all(|factor| {
      factor
        .as_ref()
        .is_some_and(|factor| is_ellipsis(factor, &ctxt))
    })
    && !is_ellipsis(r, &ctxt)
    && !is_an_integral_differential_factor(r, &ctxt)
    && !is_differential(r)
    && let XM::Apply(Operator(l_op), Args(l_args), l_props, l_meta) = application
    && l_meta.fenced.is_none()
    && l_props.id.is_none()
    && l_args.len() >= 2
    && is_a_large_mulop(l_op, &ctxt)
  {
    let mut l_args = l_args.clone();
    let last = l_args.pop().flatten();
    let mut operand: Vec<Option<XM>> = match last {
      Some(XM::Apply(last_op, Args(items), last_props, last_meta))
        if last_meta.fenced.is_none()
          && last_props.id.is_none()
          && is_invisible_times_operator(&last_op.0, &ctxt) =>
      {
        items
      },
      other => vec![other],
    };
    operand.extend(run.iter().cloned());
    operand.push(right.clone());
    l_args.push(Some(XM::Apply(
      Operator(times_op.clone()),
      Args(operand),
      XProps::default(),
      Meta::default(),
    )));
    return Ok(Some(XM::Apply(
      Operator(l_op.clone()),
      Args(l_args),
      l_props.clone(),
      l_meta.clone(),
    )));
  }
  // left-to-right associative -- if "left" is already a "times", tuck "right" in:
  if let Some(XM::Apply(ref op, ref mut left_args, _, ref _m)) = left
    && let XM::Token(xop, _xmeta) = &*op.0
  {
    match xop.meaning {
      Some(ref name) if name == "times" => {
        // No external operator to absorb here — `apply_invisible_times`
        // synthesizes its own `times` token, so there's no second
        // operator with an xml:id to redirect. Just absorb the
        // right-hand factor into the existing chain.
        left_args.0.push(right);
        return Ok(left);
      },
      _ => {},
    }
  }
  // Mixed number detection: NUMBER followed by FRACOP → invisible plus
  // Perl: 2\frac{3}{4} = 2 + 3/4; 123\frac{12}{34} = 123 + 12/34 (all-integer)
  // But 123.456\frac{12}{34} = 123.456 × (12/34) (decimal prefix → not mixed)
  let l_num = is_number(&left);
  let l_integer = l_num && is_integer_number(&left);
  let mut r_frac = is_fracop(&right);
  // Also check via nodes: if right is a Lexeme pointing to a DOM node with FRACOP inside
  if l_num
    && !r_frac
    && let Some(XM::Lexeme(ref _lex, ref meta)) = right
  {
    // Use curry_level to find the node — it encodes the node position
    if let Some(ref cv) = meta.curry_level {
      let cv_str = cv.to_string();
      // Extract index from ":N" format — node index is N-1 (lexeme counter is 1-based)
      if let Some(idx_str) = cv_str.strip_prefix(':')
        && let Ok(lex_idx) = idx_str.parse::<usize>()
      {
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
  // A large MULOP's application on the left — in a bare argument's chain, `\sin x\otimes y z`, where the grammar joins
  // the next item to the chain so far — takes the juxtaposed item into its last operand (Q11, divergence #396):
  // sin@(x⊗(y z)), as `x\otimes y z` x⊗(y z) at a term's level. Not a differential (an integral's, which closes the
  // integrand: `\int\log f\otimes g\,dx` keeps log(f⊗g)·dx) nor an ellipsis (a trailing one leaves, user ruling 15; a
  // run between two items joins, above). Residual: a mixed number joins as a product here, `\log x\otimes 2\frac34`
  // log(x⊗(2·¾)), where `x\otimes 2\frac34` is x⊗(2+¾) (57cj.20.Q11 review NIT 6).
  if !is_mixed_number
    && let (Some(l), Some(r)) = (&left, &right)
    && !is_an_integral_differential_factor(r, &ctxt)
    && !is_differential(r)
    && !is_ellipsis(r, &ctxt)
    && let XM::Apply(Operator(l_op), Args(l_args), l_props, l_meta) = l
    && l_meta.fenced.is_none()
    && l_props.id.is_none()
    && l_args.len() >= 2
    && is_a_large_mulop(l_op, &ctxt)
  {
    let mut l_args = l_args.clone();
    let last = l_args.pop().flatten();
    let joined = match last {
      Some(XM::Apply(last_op, Args(mut factors), last_props, last_meta))
        if last_meta.fenced.is_none()
          && last_props.id.is_none()
          && is_invisible_times_operator(&last_op.0, &ctxt) =>
      {
        factors.push(right);
        XM::Apply(last_op, Args(factors), last_props, last_meta)
      },
      other => XM::Apply(
        op.into(),
        Args(vec![other, right]),
        XProps::default(),
        Meta::default(),
      ),
    };
    l_args.push(Some(joined));
    return Ok(Some(XM::Apply(
      Operator(l_op.clone()),
      Args(l_args),
      l_props.clone(),
      l_meta.clone(),
    )));
  }

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
    font: Some(Rc::new(font)),
    ..Default::default()
  }
}

pub fn new_list(mut pieces: Vec<XM>, ctxt: ActionContext) -> Result<Option<XM>, Box<dyn Error>> {
  // drop placeholder token for missing trailing punct, if any
  if pieces.len() > 1 {
    let last_meaning_opt = pieces.last().unwrap().get_token_meaning(ctxt.nodes)?;
    if let Some(last_meaning) = last_meaning_opt
      && last_meaning == "absent"
    {
      pieces.pop();
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
    XM::Lexeme(lex, meta) => match lookup_lex_node(&lex, nodes) {
      Ok(node) => {
        let mut props = XProps::from(node);
        props.role = Some(Cow::Borrowed(role));
        if !is_delimiter
          && let Some(ref c) = props.content.clone()
          && c == "|"
        {
          props.content = Some(Cow::Borrowed("\u{2223}"));
        }
        XM::Token(props, meta)
      },
      _ => XM::Lexeme(lex, meta),
    },
    XM::Token(mut props, meta) => {
      props.role = Some(Cow::Borrowed(role));
      if !is_delimiter
        && let Some(ref c) = props.content.clone()
        && c == "|"
      {
        props.content = Some(Cow::Borrowed("\u{2223}"));
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
    Some(XM::Token(props, _)) if props.role.as_deref() == Some("UNKNOWN") => {
      props.possible_function = Some(Cow::Borrowed("yes"));
    },
    &mut Some(XM::Lexeme(ref lex, _)) if lex.starts_with("UNKNOWN:") => {
      // Lexemes are "ROLE:content:id" references to XML nodes.
      // Set the attribute directly on the underlying XML node.
      if let Some(id_str) = lex.split(':').next_back()
        && let Ok(id) = id_str.parse::<usize>()
        && id > 0
        && id <= nodes.len()
      {
        let mut node = nodes[id - 1].clone();
        let _ = node.set_attribute("possibleFunction", "yes");
      }
    },
    Some(XM::Apply(_, args, ..)) => {
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
    Some(XM::Lexeme(_, m)) => Some(m),
    Some(XM::Apply(_, _, _, m)) => Some(m),
    _ => None,
  };
  if let Some(meta) = meta {
    if let Some(ref cv) = meta.curry_level {
      let cv_str = cv.to_string();
      if let Some(idx_str) = cv_str.strip_prefix(':')
        && let Ok(lex_idx) = idx_str.parse::<usize>()
      {
        let idx = if lex_idx > 0 { lex_idx - 1 } else { 0 };
        if idx < nodes.len() && nodes[idx].get_name() == "XMApp" {
          return Some(&nodes[idx]);
        }
      }
    }
    // Try via Lexeme content: "ROLE:meaning:N" → index N-1
    if let Some(XM::Lexeme(lex, _)) = xm
      && let Some(idx_str) = lex.rsplit(':').next()
      && let Ok(lex_idx) = idx_str.parse::<usize>()
    {
      let idx = if lex_idx > 0 { lex_idx - 1 } else { 0 };
      if idx < nodes.len() && nodes[idx].get_name() == "XMApp" {
        return Some(&nodes[idx]);
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
    font: Some(Rc::new(font::FONT_TEXT_DEFAULT.specialize("\u{2064}"))),
    ..XProps::default()
  }
}

/// An invisible-times product whose last factor is a bare ellipsis (`b\cdots`) that elides the operation before it
/// (`infix_apply_and_elide`) — not one after a head or its bare application, which the ellipsis reaches
/// (`reaches_a_following_ellipsis`: `a+\sin x\cdots` a + sin@(x)·⋯, 57cj.13 review), nor the last of a run of them,
/// which elides no operation (`ends_an_elided_run`: `a+x\cdots\cdots+c` a + x·⋯·⋯ + c, as Perl).
fn ends_in_a_bare_ellipsis(xm: &XM, ctxt: &ActionContext) -> bool {
  matches!(xm, XM::Apply(Operator(op), Args(args), _, meta)
    if meta.fenced.is_none()
      && matches!(&**op, XM::Token(props, _) if props.content.as_deref() == Some("\u{2062}"))
      && matches!(args.as_slice(), [.., Some(before), Some(last)]
        if operator_category(last) == Some("ELIDEOP")
          && !ends_an_elided_run(before, ctxt)
          && !reaches_a_following_ellipsis(product_end(before, true))))
}

/// Does `xm` end in an ellipsis, an ELIDEOP or an ID, so that an ELIDEOP after it continues a run of ellipses, any mix,
/// rather than eliding the operation before the run? The run is one unit, a product of its ellipses (as `trig_ellipses`
/// reads it in a trig argument): `a+x\cdots\cdots+c` is a + x·⋯·⋯ + c, `a+\cdots\cdots\cdots+c` a + ⋯·⋯·⋯ + c and
/// `a+x\ldots\cdots+c` a + x·…·⋯ + c, as Perl — refused as a bare ellipsis (`ends_in_a_bare_ellipsis`) and as the
/// elided addition (`infix_apply_and_elide`: its twin a + x·⋯ + ⋯) the first two had no parse, and the elided addition
/// took the last run apart, a + x·… + ⋯ (57cj.16 ellipsis grid).
fn ends_an_elided_run(xm: &XM, ctxt: &ActionContext) -> bool {
  is_ellipsis(product_end(xm, true), ctxt)
}

fn invisible_times() -> XProps {
  XProps {
    meaning: Some(Cow::Borrowed("times")),
    role: Some(Cow::Borrowed("MULOP")),
    content: Some(Cow::Borrowed("\u{2062}")),
    font: Some(Rc::new(font::FONT_TEXT_DEFAULT.specialize("\u{2062}"))),
    ..XProps::default()
  }
}

fn invisible_comma() -> XProps {
  XProps {
    role: Some(Cow::Borrowed("PUNCT")),
    content: Some(Cow::Borrowed("\u{2063}")),
    font: Some(Rc::new(font::FONT_TEXT_DEFAULT.specialize("\u{2063}"))),
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
    // A differential is a finished factor (`Meta::differential`): `\int_0^1 dx\,f` is
    // ∫(d@(x)·f), its `d` no big operator absorbing `f`, and a differential operator's application
    // (`\partial_x u\,v` (∂_x u)·v); a big operator's application only through its operand:
    // `\partial_t\int_\Omega u\,dx` is ∂_t(∫(u dx)), not (∂_t∫u)·dx (divergence #374).
    XM::Apply(_, args, _, meta) if meta.differential => args
      .0
      .first()
      .and_then(Option::as_ref)
      .is_some_and(|operand| is_bigop_or_scripted_bigop(operand, nodes)),
    XM::Apply(op, args, ..) => {
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
      ) && let Some(Some(base)) = args.0.first()
      {
        return is_bigop_or_scripted_bigop(base, nodes);
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
    XM::Apply(op, args, ..) => {
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
        if let Some(Some(base)) = args.0.first() {
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
  matches!(
    operator_role(xm, nodes).as_deref(),
    Some("FUNCTION") | Some("OPFUNCTION") | Some("TRIGFUNCTION")
  )
}

/// The category an item is headed by, as Perl's operand patterns match it: a token's (a lexeme's
/// lexer category, a token's role); a decorated operator's role; a scripted item's base's; an
/// application's operator's; a function application's (an `XMDual`) presentation's.
fn head_category(xm: &XM) -> Option<&str> {
  match xm {
    XM::Lexeme(..) | XM::Token(..) => operator_category(xm),
    XM::Apply(_, _, props, _) if props.role.is_some() => props.role.as_deref(),
    XM::Apply(Operator(op), args, ..) => {
      let category = head_category(op)?;
      if category.ends_with("SCRIPTOP") {
        args
          .0
          .first()
          .and_then(Option::as_ref)
          .and_then(head_category)
      } else {
        Some(category)
      }
    },
    XM::Dual(_, presentation, ..) => head_category(presentation),
    _ => None,
  }
}

/// Is `xm` a scripted item, `Apply(SCRIPTOP, [base, script])`? Its base, if so.
fn script_base(xm: &XM) -> Option<&XM> {
  match xm {
    XM::Apply(Operator(op), args, ..)
      if operator_category(op).is_some_and(|c| c.ends_with("SCRIPTOP")) =>
    {
      args.0.first().and_then(Option::as_ref)
    },
    _ => None,
  }
}

/// The base under every script of `xm` — `(y_j)` in `(y_j)_{j=1}^{n}` — `xm` itself when unscripted.
pub(crate) fn script_nucleus(xm: &XM) -> &XM {
  match script_base(xm) {
    Some(base) => script_nucleus(base),
    None => xm,
  }
}

/// Perl `aBarearg` (MathGrammar:323-331), an operand an operator or OPFUNCTION takes without
/// parentheses: a function (bare, scripted or applied), an atom, identifier, unknown or number
/// (scripted or not), an unknown applied to its group (Perl's `doubtArgs` leaves the `(`; divergence
/// #18 applies it), or an absolute value `|…|`; scripted or postfixed (Perl's `addScripts` takes
/// POSTFIX, MathGrammar:423). Not a parenthesized group, an operator or big operator.
fn is_bare_item(xm: &XM) -> bool {
  if let Some(base) = script_base(xm).or_else(|| postfix_base(xm)) {
    return is_bare_item(base);
  }
  match xm {
    XM::Dual(_, presentation, _, meta) if matches!(**presentation, XM::Wrap(..)) => {
      meta.bar_fence || is_bare_abs(presentation)
    },
    XM::Lexeme(..) | XM::Token(..) => matches!(
      operator_category(xm),
      Some(
        "UNKNOWN"
          | "XDIFFUNK"
          | "ID"
          | "XDIFFID"
          | "ATOM"
          | "NUMBER"
          | "ARRAY"
          | "FUNCTION"
          | "OPFUNCTION"
          | "TRIGFUNCTION"
          // Perl's `\cdots` is an ID (divergence #3 keeps ELIDEOP): an ellipsis in a bare argument,
          // `\max_{m}|a||b|\cdots|c|` (57bx, 2605.23673).
          | "ELIDEOP"
      )
    ),
    // A partial derivative, a finished factor (57cj review: `\nabla\partial_x u` ∇@(∂_x u)).
    XM::Apply(..) if is_partial_derivative(xm) => true,
    // An unknown applied to its group is Perl's `doubtArgs` as divergence #18 reads it.
    XM::Apply(..) | XM::Dual(..) => matches!(
      head_category(xm),
      Some("FUNCTION" | "OPFUNCTION" | "TRIGFUNCTION" | "UNKNOWN" | "XDIFFUNK")
    ),
    _ => false,
  }
}

/// A group `bare_abs` builds (Perl `VERTBAR absExpression VERTBAR`, MathGrammar:329-330), read by
/// its delimiters where the fence carries no `Meta::bar_fence` mark: VERTBARs — `|…|`, `\|…\|`,
/// `\left|…\right|`, `\left\|…\right\|`, morphed to OPEN/CLOSE by the fence (`morph_vertbar`) —
/// not an OPEN/CLOSE of its own such as amsmath's `\lvert`/`\rvert` (amsmath.sty.ltxml:1150-1153).
fn is_bare_abs(wrap: &XM) -> bool {
  let XM::Wrap(items, ..) = wrap else {
    return false;
  };
  let is_bar = |item: Option<&XM>| match item {
    Some(bar @ XM::Lexeme(..)) => matches!(
      operator_category(bar),
      Some("VERTBAR" | "LEFT_STRETCHY_VERTBAR" | "RIGHT_STRETCHY_VERTBAR")
    ),
    // A `‖` is a bar as `\|` or `\left\|` (`name="||"`); the norm merged from two `|`s,
    // `||v||`, is one by its mark (divergence #353).
    Some(XM::Token(props, _)) => match props.content.as_deref() {
      Some("|") => !matches!(props.name.as_deref(), Some("lvert" | "rvert")),
      Some("‖") => props.name.as_deref() == Some("||"),
      _ => false,
    },
    _ => false,
  };
  is_bar(items.first()) && is_bar(items.last())
}

/// Is `op` what joins the factors of a product (Perl `moreFactors`, `moreBareargs`, MathGrammar:
/// 252-265, :333-337): a MulOp — a MULOP or BINOP, bare or decorated — or the invisible times?
fn is_product_operator(op: &XM) -> bool { matches!(operator_category(op), Some("MULOP" | "BINOP")) }

/// The first (`last` false) or last factor of a product, `xm` itself when it is none.
fn product_end(xm: &XM, last: bool) -> &XM {
  match xm {
    XM::Apply(Operator(op), args, ..) if args.0.len() >= 2 && is_product_operator(op) => {
      let end = if last { args.0.last() } else { args.0.first() };
      match end {
        Some(Some(factor)) => product_end(factor, last),
        _ => xm,
      }
    },
    _ => xm,
  }
}

/// Perl `barearg` (MathGrammar:321): one `aBarearg`, or a product of them.
fn is_bare_argument(xm: &XM) -> bool {
  match xm {
    XM::Apply(Operator(op), args, ..) if args.0.len() >= 2 && is_product_operator(op) => args
      .0
      .iter()
      .all(|factor| factor.as_ref().is_some_and(is_bare_argument)),
    _ => is_bare_item(xm),
  }
}

/// Perl `OPERATOR addScripts nestOperators` (MathGrammar:312-313, :663-671): an OPERATOR, scripted
/// or not, or one nested over a function or another operator — what applies to an argument.
fn is_operator_head(xm: &XM) -> bool {
  if let Some(base) = script_base(xm) {
    return is_operator_head(base);
  }
  match xm {
    XM::Lexeme(..) | XM::Token(..) => operator_category(xm) == Some("OPERATOR"),
    XM::Apply(Operator(op), args, ..) => is_nested_operator(op, args),
    _ => false,
  }
}

/// Is `op` applied to `args` an operator nested over a function, scripted or not, or an operator
/// (Perl `nestOperators`, MathGrammar:663-671; `compound_operator`)? Only an open nest nests: after
/// a function it is closed.
fn is_nested_operator(op: &XM, args: &Args) -> bool {
  nest_is_open(op)
    && matches!(args.0.as_slice(), [Some(nested)]
      if is_operator_head(nested) || is_function_head(nested))
}

/// Is `xm` a head that takes the arguments between delimiters — a function or an operator, bare,
/// scripted or nested over a function or an operator (`prefix_apply`'s lift, 57bf):
/// `\nabla\log(p,q)` `(nabla@(logarithm))@(p, q)` as Perl (`OPERATOR addScripts nestOperators
/// addOpFunArgs`, MathGrammar:312-313; 57bf review).
fn takes_delimited_arguments(xm: &XM) -> bool {
  match script_base(xm) {
    Some(base) => takes_delimited_arguments(base),
    None => match xm {
      XM::Lexeme(..) | XM::Token(..) => matches!(
        operator_category(xm),
        Some("FUNCTION" | "OPFUNCTION" | "TRIGFUNCTION" | "OPERATOR")
      ),
      XM::Apply(Operator(op), args, ..) => is_nested_operator(op, args),
      // … or a bare or scripted operator's application to one group, which applies to the next group
      // too: `D(a)(b)` (D@(a))@(b), Perl's ApplyDelimited Dual (57bl; 2605.01526
      // `\nabla(F\circ\gamma)(w)`).
      XM::Dual(..) => is_operator_group_application(xm),
      _ => false,
    },
  }
}

/// A bare or scripted OPERATOR, not a nest (`D`, `\nabla_x`), whose application to a group applies to
/// the next group (57bl).
pub(crate) fn is_bare_or_scripted_operator(xm: &XM) -> bool {
  match script_base(xm) {
    Some(base) => is_bare_or_scripted_operator(base),
    None => {
      matches!(xm, XM::Lexeme(..) | XM::Token(..)) && operator_category(xm) == Some("OPERATOR")
    },
  }
}

/// A function, with its scripts (Perl `FUNCTION addScripts`, and likewise OPFUNCTION and
/// TRIGFUNCTION), not applied.
fn is_function_head(xm: &XM) -> bool {
  match script_base(xm) {
    Some(base) => is_function_head(base),
    None => {
      matches!(xm, XM::Lexeme(..) | XM::Token(..))
        && matches!(
          operator_category(xm),
          Some("FUNCTION" | "OPFUNCTION" | "TRIGFUNCTION")
        )
    },
  }
}

/// Perl `addOpFunArgs : APPLYOP(?) barearg` (MathGrammar:553-558): an operator or an OPFUNCTION,
/// scripted or not, applied to a bare argument.
fn is_bare_operator_application(xm: &XM) -> bool {
  matches!(xm, XM::Apply(Operator(op), args, ..)
    if (is_operator_head(op) && !is_nested_operator(op, args) || is_opfunction_head(op))
      && matches!(args.0.as_slice(), [Some(arg)]
        if is_bare_argument(arg) && !(nest_is_open(op) && is_closed_expectation_application(arg))))
}

/// An expectation's or probability's application that is no bare application — to a group, with
/// its scripts: `𝔼[X]`, `𝔼_x(f)`, `𝔼[X]^2`. An operator takes it whole (`open_op_head expectation_application`),
/// and it ends the operator's argument as a group does: `\nabla\mathbb{E}[X]Y` is ∇@(𝔼@(X))·Y,
/// `\alpha_t\nabla^\top_{x_t}\mathbb{E}[x|x_t]\Sigma_t^{-1}` α_t·∇@(𝔼@(x | x_t))·Σ_t⁻¹ (2605.20593).
fn is_closed_expectation_application(xm: &XM) -> bool {
  let application = match script_nucleus(xm) {
    // a group application: the presentation holds the lexeme
    XM::Dual(_, presentation, ..) => presentation.as_ref(),
    nucleus => nucleus,
  };
  matches!(application, XM::Apply(Operator(op), ..) if is_expectation_operator(op))
    && !is_bare_operator_application(application)
}

/// An operator's application, as the grammar builds it or as `prefix_apply` lifts it over the
/// delimiters it takes its arguments between (a Dual presenting the application, 57bf).
fn is_operator_application(xm: &XM) -> bool {
  let application = match xm {
    XM::Dual(_, presentation, ..) => presentation,
    _ => xm,
  };
  // … or of an operator's application to the next group, `\log\nabla(u)(v)` log·(∇@(u))@(v) as Perl
  // (57bl review).
  matches!(application, XM::Apply(Operator(op), ..)
    if is_operator_head(op) || is_operator_group_application(op))
}

/// An OPFUNCTION, scripted or not, not applied (Perl `preScripted['OPFUNCTION']`).
fn is_opfunction_head(xm: &XM) -> bool {
  match script_base(xm) {
    Some(base) => is_opfunction_head(base),
    None => {
      matches!(xm, XM::Lexeme(..) | XM::Token(..)) && operator_category(xm) == Some("OPFUNCTION")
    },
  }
}

/// Perl's `barearg` is greedy (MathGrammar:321-337): an operator or OPFUNCTION applied to a bare
/// argument takes every bare argument after it, so a product `left · right` leaving one outside
/// the application that ends `left` is not a parse — `\nabla u v` is ∇@(u v), `a\nabla u\cdot v`
/// a·∇@(u·v), `\log x y` log@(x y), `\max_i a_i\cdot b_i` max_i@(a_i·b_i) — nor one leaving outside a
/// later OPFUNCTION's application that a bound head's argument takes (`bound_head_takes`, 57cg):
/// `\max_i a_i\log b_i` is max_i@(a_i·log@(b_i)).
fn leaves_a_bare_argument(left: &XM, right: &XM, juxtaposed: bool, ctxt: &ActionContext) -> bool {
  // … the one factor a differential operator takes too: `\partial\log x\cdot y` is ∂(log(x·y)) (#374)
  let application = through_differentials(product_end(left, true));
  // An ellipsis after a bare application stays inside it before a continuation item (`op_bare_elided`,
  // 57cb): `\log x\cdots y` is log@(x·⋯·y), not log@(x)·⋯·y; `\ldots` alike — and a run of them, whatever the
  // macros (`op_bare_elided` goes on over an ELIDEOP, an ID is a bare item): `\log x\cdots\cdots y` log@(x·⋯·⋯·y),
  // `\log\ldots\ldots\cdots x` log@(…·…·⋯·x), as Perl (57cj.14 review; was log@(x)·⋯·⋯·y).
  // The run may open `right` too, wherever the product splits it: a MulOp's right operand is a juxtaposed product
  // (`term mulop tight_term`), so `\log x\cdot\cdots y` derived (log@(x)·⋯)·y beside log@((x·⋯)·y), both surviving —
  // `\sin x\cdot\cdots y`, `\sin\ldots\cdot\cdots x`, `\nabla u\cdot\cdots v`, `\max_i a_i\times\cdots b`, `\sin x\cdot\cdots\ldots`
  // alike, 2^n readings at n sites (57cj.17 review; latent, no corpus witness). So the question is asked at every
  // junction the juxtaposed product would have: the split itself, and each one inside the right's opening run up to its
  // first other factor — the run there the left's trailing ellipses and the right's opening ones before it.
  let right_factors = product_factors(right);
  let opening = right_factors
    .iter()
    .take_while(|factor| is_ellipsis(factor, ctxt))
    .count();
  let mut factors = product_factors(left);
  let ends_in_an_ellipsis = is_ellipsis(application, ctxt);
  if ends_in_an_ellipsis && a_run_stays_inside(&factors, product_end(right, false), ctxt) {
    return true;
  }
  for junction in 1..=opening.min(right_factors.len().saturating_sub(1)) {
    factors.push(right_factors[junction - 1]);
    if a_run_stays_inside(&factors, right_factors[junction], ctxt) {
      return true;
    }
  }
  if ends_in_an_ellipsis {
    return false;
  }
  // After a bare function head that ends the argument (`\log\mathbb E_y`), what that head would take
  // as its first item — a function's application too — is its argument, not a factor after:
  // `\log\mathbb E_y\exp(x)` is log@(𝔼_y@(exp(x))) (57cb).
  let first = product_end(right, false);
  if bound_head_takes(application, first, ctxt) {
    return true;
  }
  let takes = if is_bare_function_head(last_bare_leaf(application)) {
    is_bare_item(first)
  } else {
    is_bare_continuation(first, ctxt)
  };
  is_bare_operator_application(application)
    && (takes
      // What the bare argument's last item would take right after it, not across a MulOp
      // (`\log p\cdot(R-B)` is log@(p)·(R−B)).
      || juxtaposed && takes_the_group(last_bare_leaf(application), right, ctxt))
}

/// Does an ellipsis run ending `factors` (a product's, in order) stay inside the bare application before it, so that
/// `item` after the run goes on that application's argument rather than multiplying it (`leaves_a_bare_argument`)? An
/// OPFUNCTION's or operator's bare application takes a continuation item (`op_bare_elided`), an OPFUNCTION's chain
/// ending a trig function's argument one other than a trig function's application (`trig_op_bare_elided`), and a trig
/// function's own bare application an item it chains (`trig_elided`) — where nothing ends the argument.
fn a_run_stays_inside(factors: &[&XM], item: &XM, ctxt: &ActionContext) -> bool {
  let run = factors
    .iter()
    .rev()
    .take_while(|factor| is_ellipsis(factor, ctxt))
    .count();
  if factors.len() <= run {
    return false;
  }
  // … through the one factor a differential operator takes, as `leaves_a_bare_argument` looks (#374):
  // `\partial\log x\ldots y` is ∂(log(x·…·y)), `\partial\nabla u\cdots v` ∂(∇(u·⋯·v)), as Perl and as
  // `\partial\log x\cdot y` ∂(log(x·y)), not ∂(log x)·…·y (57cj.18 review; latent, no corpus witness)
  let before = through_differentials(product_end(factors[factors.len() - run - 1], true));
  is_bare_continuation(item, ctxt)
    && (is_bare_operator_application(before)
      // … an OPFUNCTION's application ending a trig function's argument too, whose chain goes on over the run
      // (`trig_op_bare_elided`) to an item other than a trig function's application (`op_bare_plain_next`):
      // `\sin\log x\cdots y` is sin@(log@(x·⋯·y)), as Perl, not sin@(log@(x))·⋯·y — the two trees left the
      // reading to the forest's order, which a second site flipped (57cj.16 ellipsis grid)
      || trig_argument_ending_in_an_opfunction_application(before, ctxt).is_some_and(|argument| {
        !is_trig_application(item)
          // (… where nothing ends the trig argument, `ends_trig_argument`: a space, `\sin\log x\,\cdots\,y`
          // keeps sin@(log@(x))·⋯·y; a `d`; a symbol of another type, `\sin\log x\cdots\mathbf y`)
          && !factors[factors.len() - run - 1..]
            .iter()
            .any(|factor| ends_with_space(factor, ctxt.nodes))
          && !ends_trig_argument(&product_factors(argument), item, ctxt)
      }))
    // … and a trig function's own bare application before the run, whose argument goes on over it (`trig_elided`)
    // to an item it chains (`is_trig_bare_item`; not a trig function's application, `\cos\theta_1\cdots\cos\theta_n`):
    // `\sin x\cdots y` is sin@(x·⋯·y), as `\sin x\ldots y` sin@(x·…·y) and Perl, not sin@(x)·⋯·y (57cj.16 review) —
    // where nothing ends the argument, the questions `trig_argument_elision` asks of the run and the item
    || trig_argument_before_a_run(before).is_some_and(|argument| {
      let mut run_factors = product_factors(argument);
      run_factors.extend_from_slice(&factors[factors.len() - run..]);
      is_trig_bare_item(item)
        // (not after an OPFUNCTION's or operator's application ending the argument, whose chain the run goes on —
        // the branch above — or leaves, `\sin\log x\cdots\dots` sin@(log@(x))·⋯·…: `trig_argument_elision`)
        && !ends_in_its_own_chain(argument, ctxt.nodes)
        && !factors[factors.len() - run - 1..]
          .iter()
          .any(|factor| ends_with_space(factor, ctxt.nodes))
        && !ends_trig_argument(&run_factors, item, ctxt)
    })
}

/// The argument of a trig function's bare application that ends in an OPFUNCTION's bare application, through
/// compositions (`\sin\log x`, `\sin\cos\log x`, the innermost trig function's) and the ellipses the chain took already
/// (`\cos\log_2 x\ldots`): the chain an ellipsis run after it goes on in (`trig_op_bare_elided`).
fn trig_argument_ending_in_an_opfunction_application<'a>(
  xm: &'a XM,
  ctxt: &ActionContext,
) -> Option<&'a XM> {
  let XM::Apply(Operator(op), Args(args), _, meta) = xm else {
    return None;
  };
  let [Some(arg)] = args.as_slice() else {
    return None;
  };
  if meta.fenced.is_some()
    || !is_bare_function_head(op)
    || head_category(op) != Some("TRIGFUNCTION")
  {
    return None;
  }
  let last = product_factors(arg)
    .into_iter()
    .rev()
    .find(|factor| !is_ellipsis(factor, ctxt))
    .map(|factor| product_end(factor, true))?;
  if is_bare_operator_application(last) && is_opfunction_application(last) {
    Some(arg)
  } else {
    trig_argument_ending_in_an_opfunction_application(last, ctxt)
  }
}

/// A trig function's application, bare or not (`\cos y`, `\sin^2(x)`): no item of an OPFUNCTION's chain inside a trig
/// argument (`op_bare_plain_next`; juxtaposed trig functions are separate factors).
fn is_trig_application(xm: &XM) -> bool {
  matches!(xm, XM::Apply(..) | XM::Dual(..))
    && !is_function_head(xm)
    && head_category(xm) == Some("TRIGFUNCTION")
}

/// Does a bound head at the end of `application` — it, or an application its bare argument ends in —
/// take `item`, the OPFUNCTION's application after it (`bound_application`, user ruling 2026-09-30): a
/// head whose subscript binds a variable `item` mentions free? Only where the grammar lets the
/// argument's last item extend, so the refused product always has its extended twin: through an
/// OPFUNCTION's, a nest's or an operator's bare argument (`op_bare_item`, `bound_item`, `compound_operator
/// applied_func`), where an OPFUNCTION's application stands as a `bound_item`: `\operatorname*{argmin}_w
/// a_w\max_i b_{iw}\log c_i` is argmin_w@(a_w·max_i@(b_iw·log c_i)) (57cg review). An operator's
/// argument a function's application starts ends the walk: `operator_bare_apply` refuses to extend it
/// (its open-nest branch, 57cd).
fn bound_head_takes(application: &XM, item: &XM, ctxt: &ActionContext) -> bool {
  if !is_opfunction_application(item) {
    return false;
  }
  let mut application = application;
  while let XM::Apply(Operator(head), Args(args), ..) = application
    && let [Some(argument)] = args.as_slice()
    && is_bare_operator_application(application)
  {
    if is_bare_or_scripted_operator(head) {
      let first = product_end(argument, false);
      if matches!(
        head_category(postfixed_operand(first).unwrap_or(first)),
        Some("FUNCTION" | "OPFUNCTION" | "TRIGFUNCTION")
      ) {
        return false;
      }
    }
    if mentions_a_bound_variable(head, item, ctxt) {
      return true;
    }
    application = product_end(argument, true);
  }
  false
}

/// What `bound_item` derives (grammar/builder.rs): an OPFUNCTION's application, bare, to a group, or a
/// group application with its scripts (`\log(n)^2`) — not a bare head (`\log_2`), nor a postfixed
/// application.
fn is_opfunction_application(xm: &XM) -> bool {
  matches!(xm, XM::Apply(..) | XM::Dual(..))
    && !is_function_head(xm)
    && head_category(xm) == Some("OPFUNCTION")
}

/// Does `item` mention free a variable `head`'s subscript binds?
fn mentions_a_bound_variable(head: &XM, item: &XM, ctxt: &ActionContext) -> bool {
  let variables = bound_variables(head, ctxt);
  !variables.is_empty() && mentions_free(item, &variables, ctxt)
}

/// A letter a head's subscript binds, or an occurrence of one: its text and its font class
/// (`token_font_mark`) — the same letter in another font is another variable, as
/// `letter_recurs_in_its_group` reads it (`\max_{\boldsymbol\gamma\in H}` binds a bold γ, 2605.26653).
type Variable = (String, Option<&'static str>);

/// The OPFUNCTIONs whose scripts are limits, by their lexemes — the grammar's `limits_opfunction`
/// tokens (grammar/builder.rs): `\max`, `\min`, `\gcd`, `\Pr` (plain.tex:1073-1074, :1083-1084;
/// math_common.pool.ltxml:742, :759-764, `scriptpos => \&doScriptpos`, TeX_Math.pool.ltxml:350) — and
/// the named `argmin`/`argmax` and an expectation, whose subscript binds its variable as theirs does
/// (`\operatorname{argmin}_w`, `\mathbb{E}_{y\sim p}`; user ruling 2026-09-30).
const LIMITS_OPFUNCTIONS: [&str; 7] = [
  "OPFUNCTION:maximum:",
  "OPFUNCTION:minimum:",
  "OPFUNCTION:gcd:",
  "OPFUNCTION:Pr:",
  "OPFUNCTION:argmin:",
  "OPFUNCTION:argmax:",
  // (either spelling: `EXPECTATION:𝔼:` and `EXPECTATION:𝔼.letter:`, parser.rs `type_expectation_lexemes`)
  "EXPECTATION:\u{1D53C}",
];

/// Does a head bind its subscript: its scripts are limits — TeX's `\mathop` without `\nolimits`, a
/// `limits_opfunction` (`LIMITS_OPFUNCTIONS`), or an operator LaTeXML gives a `scriptpos`, amsopn's
/// starred `\operatorname*` and `\DeclareMathOperator*` (amsopn.sty:43-45 `\qopname\newmcodes@ m`;
/// amsopn.sty.ltxml:23-29, :50-53)? A nolimits function's script is a base or a parameter
/// (`\log_a b\cdot\log_b a`, `\exp_p`, `\operatorname{rank}_K`: plain.tex:1054, :1082), an operator's
/// its variable of differentiation (`\nabla_x`, no `scriptpos`): neither binds.
fn binds_its_subscript(nucleus: &XM, ctxt: &ActionContext) -> bool {
  let XM::Lexeme(lex, _) = nucleus else {
    return false;
  };
  LIMITS_OPFUNCTIONS
    .iter()
    .any(|prefix| lex.starts_with(prefix))
    || operator_category(nucleus) == Some("OPERATOR")
      && lookup_lex_node(lex, ctxt.nodes).is_ok_and(|node| {
        realize_xmnode(node, ctxt.document)
          .get_attribute("scriptpos")
          .is_some_and(|position| !position.is_empty())
      })
}

/// The variables a head binds (`binds_its_subscript`): its subscripts' (`\max_{i}^{n}`: i). A subscript
/// binds a letter (`\max_i`), an inequality chain's interior operands (`\min_{1\le i\le n}` i), another
/// relation's first operand that holds one (`\min_{\mu\in\mathbb R}` μ, `\max_{S\ni M}` S, `\min_{|\mu|\le 2B}` μ,
/// `\max_{k\notin\{i,j\}}` k), each item of
/// a list (`\max_{x\in X,y\in Y}` x and y, `\max_{i,j}`), a scripted letter by its base (`\max_{x'}`,
/// `\min_{x_1}`: x).
fn bound_variables(head: &XM, ctxt: &ActionContext) -> Vec<Variable> {
  let mut variables = Vec::new();
  if binds_its_subscript(script_nucleus(head), ctxt) {
    subscript_variables(head, ctxt, true, &mut variables);
  }
  variables
}

/// The variables an application's operator binds anew inside an item: a bound head's
/// (`bound_variables`) or a big operator's (`\sum_{j\in S}`) — plain letters only: a scripted binder
/// binds a variable of its own, not its base letter (`\mathbb{E}_{y'}` hides no y:
/// `\mathbb{E}_{y\sim p}\tau\log\mathbb{E}_{y'}\exp(y\cdot y')` mentions y, 2605.02116).
fn binder_variables(op: &XM, ctxt: &ActionContext) -> Vec<Variable> {
  let mut variables = Vec::new();
  let nucleus = script_nucleus(op);
  if script_base(op).is_some()
    && (binds_its_subscript(nucleus, ctxt)
      || matches!(
        operator_category(nucleus),
        Some("SUMOP" | "INTOP" | "BIGOP" | "LIMITOP")
      ))
  {
    subscript_variables(op, ctxt, false, &mut variables);
  }
  variables
}

/// The variables of every subscript on a scripted head; a scripted letter by its base only
/// `with_scripted`.
fn subscript_variables(
  head: &XM,
  ctxt: &ActionContext,
  with_scripted: bool,
  variables: &mut Vec<Variable>,
) {
  if let XM::Apply(Operator(op), Args(args), ..) = head
    && let [Some(base), Some(script)] = args.as_slice()
    && operator_category(op).is_some_and(|category| category.ends_with("SCRIPTOP"))
  {
    if operator_category(op) == Some("SUBSCRIPTOP") {
      script_variables(script, ctxt, with_scripted, variables);
    }
    subscript_variables(base, ctxt, with_scripted, variables);
  }
}

/// A subscript's variables: its lexeme's node — a letter, or the formula a pre-parsed script holds
/// (`ATOM:∈μR`) — or, a script the formula's own parse read, every letter in it.
fn script_variables(
  script: &XM,
  ctxt: &ActionContext,
  with_scripted: bool,
  variables: &mut Vec<Variable>,
) {
  match script {
    XM::Lexeme(lex, _) => {
      if let Ok(node) = lookup_lex_node(lex, ctxt.nodes) {
        node_bound_variables(node, ctxt.document, with_scripted, variables);
      }
    },
    XM::Apply(Operator(op), Args(args), ..) => {
      script_variables(op, ctxt, with_scripted, variables);
      for arg in args.iter().flatten() {
        script_variables(arg, ctxt, with_scripted, variables);
      }
    },
    XM::Dual(_, presentation, ..) => script_variables(presentation, ctxt, with_scripted, variables),
    XM::Wrap(items, ..) | XM::Arg(items) | XM::Choices(items) => {
      for item in items {
        script_variables(item, ctxt, with_scripted, variables);
      }
    },
    XM::Token(..) | XM::Ref(..) => {},
  }
}

/// `bound_variables` of a subscript node: an inequality chain's interior operands, another relation's
/// first operand that holds a letter, each item of a list, else every letter.
fn node_bound_variables(
  node: &XMLNode,
  document: &Document,
  with_scripted: bool,
  variables: &mut Vec<Variable>,
) {
  let node = realize_xmnode(node, document);
  let children = node.get_child_elements();
  let is_relation = |node: &XMLNode| {
    let node = realize_xmnode(node, document);
    matches!(
      node.get_attribute("role").as_deref(),
      Some("RELOP" | "ARROW")
    ) || node.get_attribute("meaning").as_deref() == Some("multirelation")
  };
  let is_list = |node: &XMLNode| {
    matches!(
      realize_xmnode(node, document)
        .get_attribute("meaning")
        .as_deref(),
      Some("list" | "formulae")
    )
  };
  match (node.get_name().as_str(), children.split_first()) {
    ("XMApp", Some((op, operands))) if is_relation(op) => {
      let inequalities = operands
        .iter()
        .filter(|&operand| is_relation(operand))
        .all(|relation| {
          matches!(
            realize_xmnode(relation, document)
              .get_attribute("meaning")
              .as_deref(),
            Some("less-than" | "less-than-or-equals" | "greater-than" | "greater-than-or-equals")
          )
        });
      let operands: Vec<&XMLNode> = operands
        .iter()
        .filter(|&operand| !is_relation(operand))
        .collect();
      // A chain of inequalities binds what it bounds on both sides, its interior operands:
      // `\max_{a\le x\le b}` x, `\max_{1\le i<j\le n}` i and j (57cg review); any other relation its first
      // operand with a letter, `\max_{i\in S}` i, `\max_{i\ne j}` i, `\max_{x\in A\subset B}` x.
      if inequalities && let [_, interior @ .., _] = operands.as_slice() {
        let before = variables.len();
        for operand in interior {
          node_letters(operand, document, with_scripted, variables);
        }
        if variables.len() > before {
          return;
        }
      }
      for operand in operands {
        let before = variables.len();
        node_letters(operand, document, with_scripted, variables);
        if variables.len() > before {
          break;
        }
      }
    },
    ("XMApp", Some((op, operands))) if is_list(op) => {
      for operand in operands {
        node_bound_variables(operand, document, with_scripted, variables);
      }
    },
    // a list of relations presents its items; its content lists them (`formulae@(x∈X, y∈Y)`)
    ("XMDual", Some((content, _))) => {
      node_bound_variables(content, document, with_scripted, variables)
    },
    _ => node_letters(&node, document, with_scripted, variables),
  }
}

/// Every letter of a node, a scripted item by its base `with_scripted` (`x_i`, `x'`: x; `|\mu|` μ;
/// `(i,j)` i and j).
fn node_letters(
  node: &XMLNode,
  document: &Document,
  with_scripted: bool,
  letters: &mut Vec<Variable>,
) {
  let node = realize_xmnode(node, document);
  let children = node.get_child_elements();
  match (node.get_name().as_str(), children.as_slice()) {
    ("XMTok", _) => letters.extend(node_letter(&node, document)),
    ("XMApp", [op, base, _]) if is_script_node(op, document) => {
      if with_scripted {
        node_letters(base, document, with_scripted, letters)
      }
    },
    ("XMDual", [_, presentation]) => node_letters(presentation, document, with_scripted, letters),
    (_, children) => {
      for child in children {
        node_letters(child, document, with_scripted, letters);
      }
    },
  }
}

/// A token node's letter: one alphabetic character of an unknown or identifier (`\mu`,
/// `\boldsymbol\gamma`, `\mathcal D`) — not a number, an operator, or a name (`max`).
fn node_letter(node: &XMLNode, document: &Document) -> Option<Variable> {
  if node.get_name() != "XMTok"
    || !matches!(
      node.get_attribute("role").as_deref(),
      None | Some("UNKNOWN" | "ID")
    )
  {
    return None;
  }
  let text = node.get_content();
  let letter = {
    let mut chars = text.chars();
    chars.next().is_some_and(char::is_alphabetic) && chars.next().is_none()
  };
  letter.then(|| (text, token_font_mark(node, document)))
}

/// A sub- or superscript operator node.
fn is_script_node(op: &XMLNode, document: &Document) -> bool {
  matches!(
    realize_xmnode(op, document)
      .get_attribute("role")
      .as_deref(),
    Some("SUBSCRIPTOP" | "SUPERSCRIPTOP")
  )
}

/// Does `xm` mention one of `variables` free — the letter in its font anywhere in it, a script's too
/// (`\log b_i` mentions i), but not where a bound head or a big operator in it binds the letter anew:
/// `\max_{z\in A}|f(z)|\max_{z\in A}|g(z)|`, the second max's z is its own (2605.24231), while
/// `\max_{j\le i}` mentions i. Every choice of an ambiguous item must.
fn mentions_free(xm: &XM, variables: &[Variable], ctxt: &ActionContext) -> bool {
  match xm {
    XM::Lexeme(lex, _) => lookup_lex_node(lex, ctxt.nodes)
      .is_ok_and(|node| node_mentions_free(node, variables, ctxt.document)),
    XM::Apply(Operator(op), Args(args), ..) => {
      let rebound = binder_variables(op, ctxt);
      let free: Vec<Variable> = variables
        .iter()
        .filter(|variable| !rebound.contains(variable))
        .cloned()
        .collect();
      !free.is_empty()
        && (mentions_free(op, &free, ctxt)
          || args
            .iter()
            .flatten()
            .any(|arg| mentions_free(arg, &free, ctxt)))
    },
    XM::Dual(_, presentation, ..) => mentions_free(presentation, variables, ctxt),
    XM::Wrap(items, ..) | XM::Arg(items) => items
      .iter()
      .any(|item| mentions_free(item, variables, ctxt)),
    XM::Choices(items) => {
      !items.is_empty()
        && items
          .iter()
          .all(|item| mentions_free(item, variables, ctxt))
    },
    XM::Token(..) | XM::Ref(..) => false,
  }
}

/// `mentions_free` in a node a lexeme names: a letter, or the parsed formula a pre-parsed ATOM holds
/// (a fraction, a script).
fn node_mentions_free(node: &XMLNode, variables: &[Variable], document: &Document) -> bool {
  let node = realize_xmnode(node, document);
  let children = node.get_child_elements();
  match (node.get_name().as_str(), children.as_slice()) {
    ("XMTok", _) => node_letter(&node, document).is_some_and(|letter| variables.contains(&letter)),
    ("XMDual", [_, presentation]) => node_mentions_free(presentation, variables, document),
    ("XMHint", _) => false,
    ("XMApp", [op, ..]) => {
      let rebound = node_binder_variables(op, document);
      let free: Vec<Variable> = variables
        .iter()
        .filter(|variable| !rebound.contains(variable))
        .cloned()
        .collect();
      !free.is_empty()
        && children
          .iter()
          .any(|child| node_mentions_free(child, &free, document))
    },
    (_, children) => children
      .iter()
      .any(|child| node_mentions_free(child, variables, document)),
  }
}

/// `binder_variables` of an operator node: a scripted big operator's or limits head's subscripts'.
fn node_binder_variables(op: &XMLNode, document: &Document) -> Vec<Variable> {
  let mut variables = Vec::new();
  let mut nucleus = realize_xmnode(op, document).into_owned();
  while nucleus.get_name() == "XMApp"
    && let [script_op, base, _] = nucleus.get_child_elements().as_slice()
    && is_script_node(script_op, document)
  {
    nucleus = realize_xmnode(base, document).into_owned();
  }
  let binds = match nucleus.get_attribute("role").as_deref() {
    Some("SUMOP" | "INTOP" | "BIGOP" | "LIMITOP") => true,
    Some("OPFUNCTION") => {
      matches!(
        nucleus.get_attribute("meaning").as_deref(),
        Some("maximum" | "minimum" | "gcd")
      ) || nucleus.get_content() == "Pr"
    },
    Some("OPERATOR") => nucleus
      .get_attribute("scriptpos")
      .is_some_and(|position| !position.is_empty()),
    _ => false,
  };
  if binds {
    node_subscript_variables(op, document, &mut variables);
  }
  variables
}

/// The variables of every subscript on a scripted node.
fn node_subscript_variables(node: &XMLNode, document: &Document, variables: &mut Vec<Variable>) {
  let node = realize_xmnode(node, document);
  if node.get_name() == "XMApp"
    && let [script_op, base, script] = node.get_child_elements().as_slice()
    && is_script_node(script_op, document)
  {
    if realize_xmnode(script_op, document)
      .get_attribute("role")
      .as_deref()
      == Some("SUBSCRIPTOP")
    {
      node_bound_variables(script, document, false, variables);
    }
    node_subscript_variables(base, document, variables);
  }
}

/// What a bare argument takes after its first item (`op_bare_next`): a bare item, but no OPFUNCTION —
/// bare, scripted, applied or postfixed — and no ellipsis, which stays inside only between two items
/// (57cb, user ruling 2026-09-29): `\log x\log y` is log@(x)·log@(y), `\nabla u\cdots` ∇@(u)·⋯,
/// `\nabla u\ldots` ∇@(u)·….
fn is_bare_continuation(xm: &XM, ctxt: &ActionContext) -> bool {
  let nucleus = script_nucleus(postfixed_operand(xm).unwrap_or(xm));
  is_bare_item(xm)
    // a partial derivative is a bare argument's first item only (`op_bare_item`, not `op_bare_next`):
    // `\nabla u\partial_x v` is ∇@(u)·∂_x v, as Perl (57cj review)
    && !is_partial_derivative(xm)
    && !matches!(head_category(nucleus), Some("OPFUNCTION" | "ELIDEOP"))
    && !is_ellipsis(nucleus, ctxt)
}

/// A big operator, a limit-type or a differential operator standing alone, bare or scripted: `\sum`, `\sum_n`,
/// `\int_0^1`, `\bigcup`, `\lim_{n\to\infty}`, `\det`, `\sup`, `\partial_x` (SUMOP, INTOP, BIGOP, LIMITOP, DIFFOP — the
/// categories `is_bigop_or_scripted_bigop` reads, here of a head with no operand yet).
fn is_bare_big_operator(xm: &XM) -> bool {
  let nucleus = script_nucleus(xm);
  matches!(nucleus, XM::Lexeme(..) | XM::Token(..))
    && matches!(
      operator_category(nucleus),
      Some("SUMOP" | "INTOP" | "BIGOP" | "LIMITOP" | "DIFFOP")
    )
}

/// What an ellipsis right after it continues or multiplies, never the last operand of an elided operation
/// (`infix_apply_and_elide`): a head with no argument of its own — a function or trig function (`is_bare_function_head`),
/// an operator (`is_operator_head`), a big, limit-type or differential operator (`is_bare_big_operator`), bare or
/// scripted — whose argument the ellipsis opens, or such a head's bare (unfenced) application, whose argument it goes on
/// or which it multiplies (`\det A\cdots` det@(A·⋯), `\sin x\cdots` sin@(x)·⋯).
fn reaches_a_following_ellipsis(xm: &XM) -> bool {
  let is_a_head =
    |xm: &XM| is_bare_function_head(xm) || is_operator_head(xm) || is_bare_big_operator(xm);
  is_a_head(xm)
    || matches!(xm, XM::Apply(Operator(head), Args(args), _, meta)
      if meta.fenced.is_none() && args.len() == 1 && is_a_head(head))
}

/// A function head with no argument of its own, bare or scripted: `\log`, `\mathbb E_y`, `\sin^2`.
fn is_bare_function_head(xm: &XM) -> bool {
  let nucleus = script_nucleus(xm);
  matches!(nucleus, XM::Lexeme(..) | XM::Token(..))
    && matches!(
      operator_category(nucleus),
      Some("OPFUNCTION" | "TRIGFUNCTION")
    )
}

/// The last item of a bare application's argument, through trailing bare applications:
/// `\log p\,\log q` ends in `q`.
fn last_bare_leaf(application: &XM) -> &XM {
  match application {
    XM::Apply(_, args, ..) => match args.0.as_slice() {
      [Some(arg)] => {
        let last = product_end(arg, true);
        if is_bare_operator_application(last) {
          last_bare_leaf(last)
        } else {
          last
        }
      },
      _ => application,
    },
    _ => application,
  }
}

/// Does `item`, ending a bare argument, take the group `right` begins with? An unknown takes its
/// group by divergence #18 (OXIDIZED_DESIGN_MATH; `speculative_item`: `a\log f(x)` is
/// a·log@(f@(x))) unless it recurs in the group (`letter_recurs_in_its_group`: `\sin x(x+1)`
/// sin@(x)·(x+1)); a bare function by Perl's `addEasyArgs` (MathGrammar:571-576), the group's
/// scripts too (divergence #351: `\log\exp(x)^2` is log@((exp@(x))²)).
fn takes_the_group(item: &XM, right: &XM, ctxt: &ActionContext) -> bool {
  let first = product_end(right, false);
  // A postfixed group is its group (`\log f(x)!` log@((f@(x))!), not log@(f)·(x)!; 57bw review).
  let first = postfixed_operand(first).unwrap_or(first);
  let unknown = matches!(item, XM::Lexeme(..) | XM::Token(..))
    && matches!(operator_category(item), Some("UNKNOWN" | "XDIFFUNK"));
  unknown && is_applicable_group(first) && !letter_recurs_in_its_group(item, first, ctxt)
    || is_function_head(item) && is_function_group(script_nucleus(first))
}

/// Does the bare letter `head` occur as a plain value in its own group — `x(x+1)`, `n(n-1)`,
/// `\lambda(\lambda I-A)`, `y(1-\sigma(y))`? Perl reads every unknown before a group as a product
/// (`doubtArgs`, MathGrammar:515-528); #18 applies it, except here (57bz; 2605.28300, 2605.02279,
/// 2605.04013, 2605.30479). A letter only (`\#(\#A)`, `\Box(\Box\phi)` stay applied; 2605.12296,
/// 2605.13710), unmarked or bold (a blackboard, script, fraktur or upright head is a named function:
/// `\mathbb E[\mathbb E[X]]`, `{\cal V}(|{\cal V}|=K)`, 2605.01849); not before an argument list,
/// the sign of a function Perl's `forbidArgs` flags under MaybeFunctions (MathGrammar:524-525;
/// default Perl multiplies every unknown) — `E(Y|X,E)`, `T(\sigma,P,T)`, `I(a;I\mid q)`; 2605.11385,
/// 2605.11761, 2605.01520. An occurrence is the same letter in the same font, not a head (`f(f(x))`),
/// not scripted or in a script (`x(1+x^2)`, `p(y_p)`), not under an accent, and not in function
/// position in a product: a letter before a group of its own (`\sigma(W\sigma(Wx))`), a differential
/// `d` before a factor (`d(x\,dy)` stays d@(x·d·y), 2605.21794); every choice of an ambiguous item
/// must hold it.
fn letter_recurs_in_its_group(head: &XM, group: &XM, ctxt: &ActionContext) -> bool {
  let XM::Lexeme(lex, _) = head else {
    return false;
  };
  if !(lex.starts_with("UNKNOWN:") || lex.starts_with("XDIFFUNK:")) || holds_arguments(group, ctxt)
  {
    return false;
  }
  let Ok(node) = lookup_lex_node(lex, ctxt.nodes) else {
    return false;
  };
  let mark = token_font_mark(&realize_xmnode(node, ctxt.document), ctxt.document);
  if !matches!(mark, None | Some("bold")) {
    return false;
  }
  let Ok(text) = realized_value(head, ctxt) else {
    return false;
  };
  if text.is_empty() || !text.chars().all(is_plain_math_letter) {
    return false;
  }
  let body = match group {
    XM::Dual(_, presentation, ..) => presentation.as_ref(),
    _ => group,
  };
  occurs_as_value(body, &text, mark, ctxt)
}

/// A group of arguments: parentheses or brackets around items separated by commas, semicolons or a
/// bar (`fence`'s lists and conditionals present `[open, item, separator, item, …, close]`), or
/// around one item that is such a list, relations included (Perl's `Argument`, MathGrammar:581-587).
/// Not an angle pair (`U\langle\mathcal{G}U,U\rangle` is U times an inner product, 2605.25149) nor a
/// colon pair (`\nabla x(G^{-1}\nabla x:\nabla e)`, 2605.21445).
fn holds_arguments(group: &XM, ctxt: &ActionContext) -> bool {
  let XM::Dual(_, presentation, ..) = group else {
    return false;
  };
  let XM::Wrap(items, ..) = presentation.as_ref() else {
    return false;
  };
  let text = |xm: &XM| {
    realized_value(xm, ctxt)
      .map(Cow::into_owned)
      .unwrap_or_default()
  };
  let listed = |xm: &XM| {
    let content = match xm {
      XM::Dual(content, ..) => content.as_ref(),
      _ => xm,
    };
    matches!(content, XM::Apply(Operator(op), ..)
    if matches!(op.as_ref(), XM::Token(props, _)
      if matches!(
        props.meaning.as_deref(),
        Some("list" | "vector" | "formulae" | "conditional")
      )))
  };
  match items.as_slice() {
    [open, _, separator, _, .., _close] => {
      matches!(text(open).as_str(), "(" | "[")
        && matches!(
          text(separator).as_str(),
          "," | ";" | "|" | "\u{2223}" | "mid"
        )
    },
    [open, only, _] => matches!(text(open).as_str(), "(" | "[") && listed(only),
    _ => false,
  }
}

/// A letter as the default math font sets it: Latin or Greek, or a Mathematical Alphanumeric
/// Symbol in bold, italic or bold italic — not a script, fraktur, double-struck, sans-serif or
/// monospace glyph (`{\cal V}` sets 𝒱 with no family of its own).
fn is_plain_math_letter(ch: char) -> bool {
  ch.is_ascii_alphabetic()
    || ('\u{0391}'..='\u{03C9}').contains(&ch) && ch.is_alphabetic()
    || matches!(
      ch,
      'ϑ' | 'ϕ' | 'ϖ' | 'ϱ' | 'ϵ' | 'ϰ' | 'ℓ' | 'ℎ' | 'ı' | 'ȷ'
    )
    || ('\u{1D400}'..='\u{1D49B}').contains(&ch)
    || ('\u{1D6A4}'..='\u{1D6A5}').contains(&ch)
    || ('\u{1D6A8}'..='\u{1D755}').contains(&ch)
}

/// `letter_recurs_in_its_group`'s walk.
fn occurs_as_value(xm: &XM, text: &str, mark: Option<&str>, ctxt: &ActionContext) -> bool {
  match xm {
    XM::Lexeme(lex, _) => {
      (lex.starts_with("UNKNOWN:") || lex.starts_with("XDIFFUNK:"))
        && realized_value(xm, ctxt).is_ok_and(|v| v == text)
        && lookup_lex_node(lex, ctxt.nodes).is_ok_and(|node| {
          token_font_mark(&realize_xmnode(node, ctxt.document), ctxt.document) == mark
        })
    },
    XM::Apply(Operator(op), Args(args), ..) => {
      if matches!(
        operator_category(op),
        Some("SUBSCRIPTOP" | "SUPERSCRIPTOP" | "OVERACCENT" | "UNDERACCENT")
      ) {
        return false;
      }
      let product = is_invisible_times_op(op);
      let args: Vec<&XM> = args.iter().flatten().collect();
      args.iter().enumerate().any(|(k, arg)| {
        // In a product a letter before a group of its own (`\sigma(W\sigma(Wx))`) and a differential
        // `d` before a factor (`d(x\,dy)`) are in function position; beside a visible operator a
        // letter is a value (`x(x-(y+z))`, `d(d-1)`).
        let function_position = product
          && args.get(k + 1).is_some_and(|next| {
            matches!(arg, XM::Lexeme(..)) && is_applicable_group(next) || is_differential_d(arg)
          });
        !function_position && occurs_as_value(arg, text, mark, ctxt)
      })
    },
    XM::Dual(_, presentation, ..) => occurs_as_value(presentation, text, mark, ctxt),
    XM::Wrap(items, ..) | XM::Arg(items) => items
      .iter()
      .any(|item| occurs_as_value(item, text, mark, ctxt)),
    XM::Choices(items) => {
      !items.is_empty()
        && items
          .iter()
          .all(|item| occurs_as_value(item, text, mark, ctxt))
    },
    _ => false,
  }
}

/// A fence whose close does not balance its open — `(0,1]`, `[a,b)`, which Perl's `factorOpenExpr`
/// builds (any CLOSE, MathGrammar:473-481): no function's argument, as `addEasyArgs` needs a
/// `balancedClose` (:571-576), so a function before it multiplies it — `\log(0,1]` is log·(0,1]
/// (golden tests/parse/opfunction_arguments.tex#function_before_an_unbalanced_interval_multiplies).
fn is_unbalanced_fence(xm: &XM) -> bool {
  matches!(xm, XM::Dual(.., meta) if meta.unbalanced_fence)
}

/// A group a function applies to (Perl `addEasyArgs`, MathGrammar:571-576: a balanced `OPEN …
/// CLOSE`, `\lvert…\rvert` included) — not a bar fence (`Meta::bar_fence`), an `aBarearg` that
/// starts the bare argument, nor a fenced modifier `(>0)`. By the fence's own marks, so a `split`
/// row, whose bars read through XMRefs, reads as the inline formula.
fn is_function_group(xm: &XM) -> bool {
  matches!(xm, XM::Dual(_, presentation, _, meta)
    if matches!(**presentation, XM::Wrap(..)) && !meta.bar_fence)
    && !is_fenced_modifier_dual(xm)
}

/// A fenced group an unknown applies to — what `speculative_prefix_apply` accepts: not bars
/// (`|x|`, `\lvert x\rvert`), which multiply, nor a fenced modifier `(>0)`, which annotates.
fn is_applicable_group(xm: &XM) -> bool {
  matches!(xm, XM::Dual(_, presentation, ..) if matches!(**presentation, XM::Wrap(..)))
    && !is_vertbar_fenced_dual(xm)
    && !is_fenced_modifier_dual(xm)
}

/// Does Perl's operator `head` take `right` rather than multiply it (MathGrammar:312-313): while
/// its nest is open a function or operator, scripted or applied (`nestOperators`, :663-671); a
/// parenthesized group (`nestOperators`' OPEN, `addEasyArgs`, :571-576); a bare argument
/// (`addOpFunArgs`, :553-558)?
fn operator_takes(head: &XM, right: &XM) -> bool {
  is_bare_item(product_end(right, false))
    || matches!(right, XM::Dual(_, presentation, ..) if matches!(**presentation, XM::Wrap(..)))
    || nest_is_open(head)
      && matches!(
        head_category(right),
        Some("OPERATOR" | "FUNCTION" | "OPFUNCTION" | "TRIGFUNCTION")
      )
}

/// Extract the role of an XM operator.
fn get_operator_role(op: &Operator, nodes: &[libxml::tree::Node]) -> Option<String> {
  operator_role(&op.0, nodes)
}

/// Extract the role from a lexeme ID by looking up the DOM node (through an XMRef).
fn get_lexeme_role(lex_id: &str, nodes: &[libxml::tree::Node]) -> Option<String> {
  let node = lookup_lex_node(lex_id, nodes).ok()?;
  crate::data::resolve_xmref(node)
    .unwrap_or_else(|| node.clone())
    .get_attribute("role")
}

/// An item's role as Perl reads it, `p_getAttribute(realizeXMNode($x), 'role')`: a token's
/// own; a lexeme's node's, read through an XMRef, as a gathered/split row's content branch holds
/// them (MathParser.pm:135-150); and a decorated operator's, the role `decorate_operator` gave it
/// (Perl DecorateOperator, MathParser.pm:1649-1654).
fn operator_role(xm: &XM, nodes: &[libxml::tree::Node]) -> Option<String> {
  match xm {
    XM::Token(props, _) | XM::Apply(_, _, props, _) | XM::Ref(props) => {
      props.role.as_deref().map(String::from)
    },
    XM::Lexeme(lex_id, _) => get_lexeme_role(lex_id, nodes),
    _ => None,
  }
}

/// Is `xm` a relation or arrow operator (Perl's `relop` pseudo-terminal, MathGrammar:704-712), bare
/// or decorated with scripts (`decorate_operator`)? A lexeme's role is its lexer category, read
/// through an XMRef when it was lexed.
fn is_relational_op(xm: &XM) -> bool {
  operator_category(xm).is_some_and(|r| r.contains("RELOP") || r.contains("ARROW"))
}

/// An operator's grammatical category without the lexeme table: a lexeme's lexer category (read
/// through an XMRef when it was lexed), a token's role, a decorated operator's role.
pub(crate) fn operator_category(xm: &XM) -> Option<&str> {
  match xm {
    // an expectation or probability is an OPFUNCTION of its own lexeme category, for the grammar
    // (`plain_opfunction`, builder.rs)
    XM::Lexeme(lex, _) => lex.split(':').next().map(lexeme_category),
    XM::Token(props, _) | XM::Apply(_, _, props, _) => props.role.as_deref(),
    _ => None,
  }
}

/// A lexeme's grammatical category: its first `:`-segment, an EXPECTATION's `OPFUNCTION`, an ATOM_NUMBER's
/// (a fraction of numbers, util.rs) `ATOM`.
pub(crate) fn lexeme_category(category: &str) -> &str {
  match category {
    "EXPECTATION" => "OPFUNCTION",
    "ATOM_NUMBER" => "ATOM",
    _ => category,
  }
}

/// Perl `moreRelations` (MathGrammar Formula): a relation after a relation continues one
/// `multirelation` — `a < b < c`, `x\sim_p y\sim z`, `y < 2 <` — whatever the relations: bare,
/// a two-part `<=`, or decorated. `left` is the formula so far, `op` the next relation and
/// `operand` what follows it; with no relation on the left, the relation is `op(left, operand)`.
fn chain_relation(left: Option<XM>, op: Option<XM>, operand: Option<XM>) -> XM {
  match left {
    Some(XM::Apply(left_op, mut left_args, props, meta)) if is_multirelation(&left_op.0) => {
      left_args.0.push(op);
      left_args.0.push(operand);
      XM::Apply(left_op, left_args, props, meta)
    },
    Some(XM::Apply(left_op, left_args, ..))
      if is_relational_op(&left_op.0) && left_args.0.len() == 2 =>
    {
      let multirel_tok = XProps {
        meaning: Some(Cow::Borrowed("multirelation")),
        ..XProps::default()
      };
      let mut left_args = left_args.0.into_iter();
      let (left_1, left_2) = (left_args.next().flatten(), left_args.next().flatten());
      XM::Apply(
        multirel_tok.into(),
        Args(vec![left_1, Some(*left_op.0), left_2, op, operand]),
        XProps::default(),
        Meta::default(),
      )
    },
    left => XM::Apply(
      op.into(),
      Args(vec![left, operand]),
      XProps::default(),
      Meta::default(),
    ),
  }
}

/// Is `op` the `multirelation` a chain of relations flattens into (MathParser.pm, moreRelations)?
fn is_multirelation(op: &XM) -> bool {
  matches!(op, XM::Token(props, _) if props.meaning.as_deref() == Some("multirelation"))
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
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => left, vertbar, right);
  // Perl's conditional is `Term | ExpressionsNoBars` (MathGrammar:261-268), no relation on either
  // side; ours takes a relation (`P(A|B=b)`), not one missing an operand, which only a bar
  // splitting the formula makes — `\bigg|g\big|_{t=1}-h\bigg|\le C` is no (|g|)_{t=1}−h | (absent ≤ C),
  // `\le C|a|\,|b|_L` no (absent ≤ C·|a|) | … (2605.26054, 2605.02499, 2605.12082; repro
  // math-parse/conditional_bar_takes_no_bare_relation).
  if [&left, &right]
    .into_iter()
    .flatten()
    .any(|side| relation_lacks_operand(side, true) || relation_lacks_operand(side, false))
  {
    return Err("vertbar_modifier: a relation beside the bar lacks its operand".into());
  }
  // … nor the statements after it: a comma list of relations holding an ellipsis or crossing a
  // `\quad` (`y_i|\theta_i\sim P,\quad i=1,\ldots,n` is no conditional@(y_i, formulae@(…)); 57bv A/B:
  // 2605.05396, 2605.13128, 2605.19519, 2605.03152; `P(A|B=b,C=c)` stays a condition).
  if right.as_ref().is_some_and(|r| is_statement_list(r, &ctxt)) {
    return Err("vertbar_modifier: the statements after a condition are no condition".into());
  }
  // … nor a relation that is no event: it relates the whole conditional (`conditional_formula`,
  // 57cc, user ruling 2026-09-29) — `y|x\sim N(0,1)` is conditional@(y, x) ∼ N@(0, 1) (2605.03594,
  // 2605.05396); and a bar after such a relation is the right side's conditional, `x|y\sim z|w`
  // (x|y) ∼ (z|w), no conditional@(conditional@(x, y) ∼ z, w).
  if right
    .as_ref()
    .is_some_and(|r| condition_holds_a_non_event(r, &ctxt))
  {
    return Err("vertbar_modifier: a relation that is no event relates the conditional".into());
  }
  if left.as_ref().is_some_and(relates_a_conditional) {
    return Err(
      "vertbar_modifier: a relation over a conditional takes a conditional on its right".into(),
    );
  }
  // … and its condition reads no bar of its own (`ExpressionsNoBars` sets `$forbidVertBar`,
  // :267-268): `|f(x)|_0^1|+|\nabla a|_L|\nabla b|_L` is no (|f(x)|)_0^1 | (+|∇a|_L | eval(∇b, L)).
  if right
    .as_ref()
    .is_some_and(|r| holds_bar_reading(r, &["evaluated-at", "conditional"]))
  {
    return Err("vertbar_modifier: the condition reads a bar of its own".into());
  }
  // The bar is the source token, re-roled: Perl's `Annotate($item[2], role => 'MODIFIEROP',
  // meaning => 'conditional')` (MathGrammar:263) — `\mid` keeps `∣` and its name, `\big|` its size
  // and padding, where a token built here rendered each as a plain `|` (57bc; golden
  // tests/parse/bar_pairs.tex, WISDOM #94). In a split or gathered row's content branch the bar
  // is an `XMRef`, and `Annotate` keeps it one, `<XMRef idref role meaning/>`, as the differential
  // does (`diffop_apply`).
  let source = match vertbar {
    Some(XM::Lexeme(lex, _)) => match lookup_lex_node(&lex, ctxt.nodes) {
      Ok(node) if node.get_name() == "XMRef" => {
        let bar = XM::Ref(XProps {
          id: node.get_attribute("idref").map(Cow::Owned),
          xmkey: node.get_attribute("_xmkey").map(Cow::Owned),
          role: Some(Cow::Borrowed("MODIFIEROP")),
          meaning: Some(Cow::Borrowed("conditional")),
          ..XProps::default()
        });
        return Ok(Some(XM::Apply(
          bar.into(),
          Args(vec![left, right]),
          XProps::default(),
          Meta::default(),
        )));
      },
      Ok(node) => Some(XProps::from(node)),
      Err(_) => None,
    },
    Some(XM::Token(props, _)) => Some(props),
    _ => None,
  };
  let modop = match source {
    Some(bar) => XProps {
      meaning: Some(Cow::Borrowed("conditional")),
      role: Some(Cow::Borrowed("MODIFIEROP")),
      ..bar
    },
    // No source token to annotate: Perl MorphVertbar's unfonted `|`.
    None => XProps {
      meaning: Some(Cow::Borrowed("conditional")),
      role: Some(Cow::Borrowed("MODIFIEROP")),
      stretchy: Some(Cow::Borrowed("false")),
      content: Some(Cow::Borrowed("|")),
      font: Some(Rc::new(font::FONT_TEXT_DEFAULT.specialize("|"))),
      ..XProps::default()
    },
  };
  Ok(Some(XM::Apply(
    modop.into(),
    Args(vec![left, right]),
    XProps::default(),
    Meta::default(),
  )))
}

/// `expectation_letter` (M3): an expectation's or probability's lexeme read as the letter the lexer
/// gave its token (`util::letter_lexeme`, as `parser::type_expectation_lexemes` found it), marked
/// (`Meta::expectation_letter`) for the soft prune that keeps it only where no reading takes it as
/// an operator (`ExpectationLettersAreFallbacks`).
pub fn expectation_as_letter(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => typed);
  let Some(XM::Lexeme(lex, _)) = typed else {
    return Err("expectation_as_letter: no expectation lexeme".into());
  };
  let node = lookup_lex_node(&lex, ctxt.nodes)
    .map_err(|_| "expectation_as_letter: the lexeme names no node")?;
  let index = lex
    .rsplit(':')
    .next()
    .and_then(|index| index.parse::<usize>().ok())
    .ok_or("expectation_as_letter: the lexeme has no index")?;
  let letter = crate::util::letter_lexeme(node, index);
  let XM::Lexeme(name, mut meta) =
    XM::Lexeme(Rc::from(letter.as_str()), Meta::default()).specialize(Meta::default(), pragmas)?
  else {
    unreachable!()
  };
  meta.expectation_letter = true;
  Ok(Some(XM::Lexeme(name, meta)))
}

/// How many expectations `xm` reads as letters (`expectation_as_letter`, M3): the rank of the soft prune
/// `ExpectationLettersAreFallbacks` at the root and, per glade, in the ASF traverser.
pub(crate) fn expectation_letter_count(xm: &XM) -> usize {
  match xm {
    XM::Lexeme(_, meta) => usize::from(meta.expectation_letter),
    XM::Token(..) | XM::Ref(_) => 0,
    XM::Apply(Operator(op), args, ..) => {
      expectation_letter_count(op)
        + args
          .trees()
          .iter()
          .map(|arg| expectation_letter_count(arg))
          .sum::<usize>()
    },
    XM::Dual(content, presentation, ..) => {
      expectation_letter_count(content) + expectation_letter_count(presentation)
    },
    XM::Wrap(items, ..) | XM::Arg(items) | XM::Choices(items) => {
      items.iter().map(expectation_letter_count).sum()
    },
  }
}

/// An expectation or probability operator (`\mathbb{E}`, `\mathbb{P}`, 𝔼, ℙ), bare or scripted: its
/// lexeme is spelled by its glyph (`parser::type_expectation_lexemes`).
fn is_expectation_operator(xm: &XM) -> bool {
  matches!(script_nucleus(xm), XM::Lexeme(lex, _) if lex.starts_with("EXPECTATION:"))
}

/// `open_op_head expectation_before_a_big_operator bigop_operand`: an operator takes an expectation's
/// application to the big operator after it, `\nabla_\theta\mathbb{E}_x\sum_i f_i` (∇_θ)@(𝔼_x@(∑…)) (57cf;
/// 2605.09853): the expectation applied to the big operator, the operator to that.
pub fn operator_takes_an_expectation_s_big_operator(
  rule_id: i32,
  mut args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  if let (Some(big_operator), Some(expectation)) = (args.pop(), args.pop()) {
    let applied = prefix_apply(
      rule_id,
      vec![expectation, big_operator],
      pragmas,
      ActionContext {
        nodes:    ctxt.nodes,
        document: ctxt.document,
      },
    )?;
    args.push(applied);
  }
  prefix_apply(rule_id, args, pragmas, ctxt)
}

/// A relation that states an event of a condition, what is given: an undecorated equality,
/// inequality, order, membership or inclusion (57cc, user ruling 2026-09-29, divergence #377) —
/// `Y\mid X=x` (2605.03233), `T-s\mid T>s` (2605.16066), `A_0|A_t\in S`. Any other relation (`\sim`, `\approx`, `\propto`, `\equiv`, an
/// arrow, a decorated relation) relates the whole conditional.
fn is_event_relation(op: &XM, ctxt: &ActionContext) -> bool {
  script_base(op).is_none()
    && realized_meaning(op, ctxt).is_some_and(|meaning| {
      matches!(
        meaning.as_str(),
        "equals"
          | "not-equals"
          | "less-than"
          | "greater-than"
          | "less-than-or-equals"
          | "greater-than-or-equals"
          | "much-less-than"
          | "much-greater-than"
          | "element-of"
          | "not-element-of"
          | "contains"
          | "not-contains"
      ) || meaning.contains("subset")
        || meaning.contains("superset")
    })
}

/// Does a condition hold a relation that is no event — itself, a chain of relations, or an item
/// of its comma list (read from the presentation, as `is_statement_list` reads it)?
fn condition_holds_a_non_event(xm: &XM, ctxt: &ActionContext) -> bool {
  match xm {
    XM::Apply(Operator(op), args, ..) if is_multirelation(op) => args
      .0
      .iter()
      .flatten()
      .skip(1)
      .step_by(2)
      .any(|relation| !is_event_relation(relation, ctxt)),
    XM::Apply(Operator(op), args, ..) if is_relational_op(op) && args.0.len() >= 2 => {
      !is_event_relation(op, ctxt)
    },
    XM::Dual(content, pres, ..) => match (&**content, &**pres) {
      (XM::Apply(op, ..), XM::Wrap(items, ..))
        if matches!(&*op.0, XM::Token(props, _)
          if matches!(props.meaning.as_deref(), Some("list" | "formulae"))) =>
      {
        items
          .iter()
          .any(|item| condition_holds_a_non_event(item, ctxt))
      },
      _ => false,
    },
    _ => false,
  }
}

/// `expression vertbar formula_list`: a condition list that is not all relations (`y_i\mid\mu,\sigma^2`,
/// `Y\mid Z,X=x`, `y|x_1=a_1,\ldots,x_n=a_n`); a list of relations only is `relation_pairs`' (`U\mid A=a,B=b`
/// conditional@(U, formulae@(A = a, B = b)), 2605.18724), one derivation (57cc).
pub fn conditional_over_a_list(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  if let [_, _, Some(XM::Dual(_, pres, ..))] = args.as_slice()
    && let XM::Wrap(items, ..) = &**pres
    && items.iter().step_by(2).all(is_relational_item)
  {
    return Err("conditional_over_a_list: a list of relations is relation_pairs'".into());
  }
  vertbar_modifier(rule_id, args, pragmas, ctxt)
}

/// A conditional, `conditional@(x, y)`.
fn is_conditional(xm: &XM) -> bool {
  matches!(xm, XM::Apply(Operator(op), ..)
    if matches!(&**op, XM::Token(props, _) if props.meaning.as_deref() == Some("conditional")))
}

/// A relation (or chain) whose first operand is a factor-level conditional: `conditional@(x, y) ∼ z`.
fn relates_a_conditional(xm: &XM) -> bool {
  match xm {
    XM::Apply(Operator(op), args, ..) if is_relational_op(op) || is_multirelation(op) => args
      .0
      .first()
      .is_some_and(|first| first.as_ref().is_some_and(is_conditional)),
    _ => false,
  }
}

/// `conditional_head relop expression`: a relation after a factor-level conditional relates the
/// whole conditional unless it is an event, which the statement-level bar keeps in the condition
/// (`Y\mid X=x` is conditional@(Y, X = x)); one derivation each (57cc).
pub fn conditional_relation(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  // (between two conditionals there is no condition to keep it: `a|b=c|d`, 2605.29001's shape)
  if let [_, Some(relation), right] = args.as_slice()
    && is_event_relation(relation, &ctxt)
    && !right.as_ref().is_some_and(is_conditional)
  {
    return Err("conditional_relation: an event stays in the condition".into());
  }
  infix_relation(rule_id, args, pragmas, ctxt)
}

/// Does the relation `xm` lack its first (`first`) or last operand — `absent ≤ C`, `δ ≍ absent`?
fn relation_lacks_operand(xm: &XM, first: bool) -> bool {
  match xm {
    XM::Apply(Operator(op), args, ..) if is_multirelation(op) || is_relational_op(op) => {
      let operand = if first { args.0.first() } else { args.0.last() };
      matches!(operand, Some(Some(operand)) if is_absent(operand)
        || relation_lacks_operand(operand, first))
    },
    _ => false,
  }
}

/// The `absent` operand a relation missing one gets (`absent()`).
fn is_absent(xm: &XM) -> bool {
  matches!(xm, XM::Token(props, _) if props.meaning.as_deref() == Some("absent"))
}

/// VERTBAR conditional with a comma-LIST left-hand side: `a,b | c` →
/// `conditional(list@(a,b), c)`. The grammar rule is the explicit
/// `statements punct statement vertbar statements` (NOT a generalized
/// `statements vertbar statements`, which over-applies to abs-value `|a|`
/// and explodes the parse forest — see EXPECTED_ID_XMREF_DESIGN 2026-06-26p/q).
/// Restricting to a literal comma-before-bar shape means it fires ONLY on
/// genuine list-LHS conditionals, leaving abs-value/norm parsing untouched.
/// Composes `list_apply` (build the list LHS) + `vertbar_modifier` (condition it).
/// Root fix for the Class-B `\Pr(s_A,s_B|\Omega)` dangling-XMRef witness.
pub fn vertbar_modifier_listlhs(
  rule_id: i32,
  mut args: Vec<Option<XM>>,
  prag: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  // args: [left_statements, punct, statement, vertbar, right_statements]
  if args.len() != 5 {
    return Err("vertbar_modifier_listlhs: expected 5 args".into());
  }
  let right = args.pop().unwrap();
  let vertbar = args.pop().unwrap();
  let mid = args.pop().unwrap();
  let sep = args.pop().unwrap();
  let left = args.pop().unwrap();
  // Perl binds `|` to the LAST item before the bar (Factor-level, tight), NOT the
  // whole comma-list: `a,b | c` → `list@(a, conditional@(b, c))` (NOT
  // `conditional@(list@(a,b), c)`). So condition `mid | right` FIRST, then append
  // that conditional as the final item of the list `left`. (`x | y,z` — no comma
  // before the bar — is handled by the single-LHS `statement vertbar statements`
  // rule, giving `conditional@(x, list@(y,z))`, also matching Perl.)
  let cond = {
    let inner = ActionContext {
      nodes:    ctxt.nodes,
      document: &mut *ctxt.document,
    };
    vertbar_modifier(rule_id, vec![mid, vertbar, right], prag, inner)?
  };
  // Append the conditional as the list's last item via `list_apply_core` (NOT
  // `list_apply`, whose Rule-4 would reject the bare conditional item — here it
  // is legitimately the last item of a bar-after-comma list). Guard the unwraps:
  // if either operand is absent, drop this combo (pruned).
  let (sep, cond) = match (sep, cond) {
    (Some(s), Some(c)) => (s, c),
    _ => return Err("vertbar_modifier_listlhs: missing separator or conditional".into()),
  };
  list_apply_core(left, sep, cond, ctxt)
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
  // Consecutive relations: relop2 is relop1's right operand, the chain Perl's `moreRelations`.
  Ok(Some(chain_relation(left, relop1, relop2)))
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
  Ok(Some(chain_relation(left, relop, Some(absent()))))
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
  let o = realized_value(&open, &ctxt)?;
  if o == "{" {
    let op = XProps {
      meaning: Some(Cow::Borrowed("cases")),
      ..XProps::default()
    };
    let refs = create_xmrefs(&mut [&mut arg], ctxt)?;
    let content = XM::Apply(
      Operator::from(op),
      Args(refs.into_iter().map(Some).collect()),
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
  let c = realized_value(&close, &ctxt)?;
  if c == "}" {
    let op = XProps {
      meaning: Some(Cow::Borrowed("cases")),
      ..XProps::default()
    };
    let refs = create_xmrefs(&mut [&mut arg], ctxt)?;
    let content = XM::Apply(
      Operator::from(op),
      Args(refs.into_iter().map(Some).collect()),
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
        XM::Lexeme(l, _) => Some(&**l),
        XM::Wrap(items, ..) if !items.is_empty() => {
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
  if let XM::Wrap(items, ..) = script
    && items.len() == 2
  {
    return Some(items[1].clone());
  }
  // Old format: bare Lexeme — look up from DOM
  if let XM::Lexeme(lex, _) = script {
    let node = lookup_lex_node(lex, nodes).ok()?;
    let children = node.get_child_elements();
    if let Some(first_child) = children.first() {
      for (i, n) in nodes.iter().enumerate() {
        if n == first_child {
          return Some(XM::Lexeme(
            Rc::from(format!("{}", i + 1).as_str()),
            Meta::default(),
          ));
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
  args_xm: Vec<Option<XM>>,
  stuff: Vec<XM>,
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  // A list label holding a relation is Perl's `Formulae` (`ketExpression : Formulae`, MathGrammar:401-404;
  // `NewFormulae`, MathParser.pm:1439-1448), as a fenced one is (`relation_list_as_formulae`):
  // `|F=1,m_F=1\rangle` is ket@(formulae@(F = 1, m _ F = 1)), `|n,l\rangle` keeps ket@(list@(n, l)) (57bt;
  // 2605.07372). The presentation's copy of the label is renamed with the content's.
  let mut args_xm: Vec<Option<XM>> = args_xm
    .into_iter()
    .map(|arg| arg.map(relation_list_as_formulae))
    .collect();
  let mut stuff: Vec<XM> = stuff.into_iter().map(relation_list_as_formulae).collect();
  let op = XProps {
    meaning: Some(Cow::Borrowed(meaning)),
    ..XProps::default()
  };
  let mut arg_refs: Vec<&mut XM> = args_xm.iter_mut().filter_map(|a| a.as_mut()).collect();
  let refs: Vec<Option<XM>> = create_xmrefs(arg_refs.as_mut_slice(), ctxt)?
    .into_iter()
    .map(Some)
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

/// A Dirac label — a bra's, ket's, inner product's or operator product's part — reads no bar outside
/// a nested group: Perl's `ketExpression` sets `$forbidVertBar` (MathGrammar:300-302, :401-403), which
/// turns off the ket, bar-pair, evaluation-bar and conditional readings inside it (:258-261, :293,
/// :298, :301), and its `maybeBra` reads a braket before a bra (:373-379), so a label holds no ket and
/// no open bra: `\langle a|b\rangle|c\rangle` inner-product@(a, b)·ket@(c), the sided ket of
/// `\sum_b\left\langle b\right|\psi\left|b\right\rangle_{A}|b\rangle` outside the operator product
/// (57bp; 2605.02840, 2605.28949). A bar pair in a middle stays (#356), and a nested group's bars are
/// its own (#368).
const DIRAC_LABEL_BARS: &str =
  "qm: a Dirac label reads no ket, open bra, evaluation bar or conditional";

/// A label `DIRAC_LABEL_BARS` forbids. A list label's items are labels each: a bare list has no
/// delimiters of its own, so it pairs no bars of its own — a ket, an open bra, an evaluation bar or a
/// conditional in an item is the label's (57bt; a list presents as a Wrap, which `holds_ket` and its
/// kin take for a nested group).
fn is_forbidden_dirac_label(label: &XM) -> bool {
  if let Some(mut items) = bare_list_items(label) {
    return items.any(is_forbidden_dirac_label);
  }
  holds_ket(label)
    || holds_open_bra(label)
    || holds_bar_reading(label, &["evaluated-at", "conditional"])
    || holds_plain_angle_relation(label)
}

/// Does `xm` hold a plain `<` or `>` relation? Perl's `ketExpression` forbids one (`$forbidLRAngle`,
/// MathGrammar:402, :708), as the signs that delimit a bra or ket: `\langle p_\alpha\mid\alpha<\gamma\rangle`
/// (2605.09161), an ordinal-indexed sequence, is no inner product (57ch review).
fn holds_plain_angle_relation(xm: &XM) -> bool {
  match xm {
    XM::Lexeme(lex, _) => {
      lex.starts_with("RELOP:less-than:") || lex.starts_with("RELOP:greater-than:")
    },
    XM::Token(props, _) | XM::Ref(props) => {
      props.role.as_deref() == Some("RELOP")
        && matches!(props.meaning.as_deref(), Some("less-than" | "greater-than"))
    },
    XM::Apply(Operator(op), args, ..) => {
      holds_plain_angle_relation(op) || args.0.iter().flatten().any(holds_plain_angle_relation)
    },
    XM::Dual(content, presentation, ..) => {
      holds_plain_angle_relation(content) || holds_plain_angle_relation(presentation)
    },
    XM::Wrap(items, ..) | XM::Arg(items) | XM::Choices(items) => {
      items.iter().any(holds_plain_angle_relation)
    },
  }
}

/// The items of a bare list — a `list` or `formulae` Dual that presents its items alone, with no
/// delimiters of its own (`presents_its_items_alone`) — when `xm` is one.
fn bare_list_items(xm: &XM) -> Option<impl Iterator<Item = &XM>> {
  let XM::Dual(content, presentation, ..) = xm else {
    return None;
  };
  let XM::Apply(Operator(op), args, ..) = &**content else {
    return None;
  };
  let XM::Wrap(items, ..) = &**presentation else {
    return None;
  };
  let listed = matches!(&**op, XM::Token(props, _)
    if matches!(props.meaning.as_deref(), Some("list" | "formulae")));
  (listed && presents_its_items_alone(presentation, args.0.len())).then(|| items.iter().step_by(2))
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
  if expr.as_ref().is_some_and(is_forbidden_dirac_label) {
    return Err(DIRAC_LABEL_BARS.into());
  }
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
  unp!(args => bar, expr, _close);
  if bar
    .as_ref()
    .zip(expr.as_ref())
    .is_some_and(|(bar, expr)| left_bar_pairs_an_evaluation_bar(bar, expr))
  {
    return Err("qm_ket: a `\\left|` opening pairs the item's evaluation bar".into());
  }
  if expr.as_ref().is_some_and(is_forbidden_dirac_label) {
    return Err(DIRAC_LABEL_BARS.into());
  }
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
  unp!(args => _open, left, bar, right, _close);
  if bar
    .as_ref()
    .zip(right.as_ref())
    .is_some_and(|(bar, right)| left_bar_pairs_an_evaluation_bar(bar, right))
  {
    return Err("qm_braket: a `\\left|` divider pairs the item's evaluation bar".into());
  }
  if [&left, &right]
    .into_iter()
    .flatten()
    .any(is_forbidden_dirac_label)
  {
    return Err(DIRAC_LABEL_BARS.into());
  }
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
  unp!(args => _open, left, _bar1, mid, bar2, right, _close);
  if bar2
    .as_ref()
    .zip(right.as_ref())
    .is_some_and(|(bar, right)| left_bar_pairs_an_evaluation_bar(bar, right))
  {
    return Err("qm_bracket: a `\\left|` opening pairs the item's evaluation bar".into());
  }
  if [&left, &mid, &right]
    .into_iter()
    .flatten()
    .any(is_forbidden_dirac_label)
  {
    return Err(DIRAC_LABEL_BARS.into());
  }
  qm_fenced(
    "quantum-operator-product",
    vec![left, mid, right],
    stuff,
    ctxt,
  )
}

/// Plain `<` and `>` are relations — `\mathrel`, mathcode "313C and "313E (plain.tex:99, :101) — which
/// an author also types for angle brackets: `<f,g>=1`, `S_{ij}=<a,b>`, `\lambda\cdot<a,b>`, `e_{<u,i>}`,
/// `\exp(<a,b>)`, `<a|H|b>` (divergence OXIDIZED_DESIGN_MATH #7). A pair of them is no angles where both
/// stand as relations at once: an operand ends right before the `<` and one starts right after the
/// `>`, or the `<` continues an inequality (`continues_an_inequality`). `c_1<c_D,c_2>c_D` is
/// formulae@(c_1 < c_D, c_2 > c_D), `0<x<a,\;t>0` formulae@(0 < x < a, t > 0), `0<s<1,\ t>-1`
/// formulae@(0 < s < 1, t > −1), `\mathbb{P}(m<S<m+\delta\mid S>m)` P@(conditional@(m < S < m + δ, S > m)),
/// as Perl reads the relations (57bt; 2605.04340, 2605.12157, 2605.19552, 2605.13710, 2605.17504; ~33 of
/// the 53 angle fences of the 57bp7 output, in 18 papers, read relations as a fence). `\lambda<x,y>`,
/// `0<<a,b>>1` and `\mu_{23}=<2|\vec\mu|3>` stay angles, and so does a pair a relation follows: `2<x,y>=z`
/// is 2·⟨x, y⟩ = z (the Rust golden `math/ambiguous_relations`; Perl joins `>` `=` into one relation,
/// `TwoPartRelop`, MathGrammar:711, and reads `formulae@(2 < x, y >= z)`).
fn angle_signs_are_relations(open: &XM, close: &XM, ctxt: &ActionContext) -> bool {
  match (
    lexeme_position(open, "RELOP:less-than:"),
    lexeme_position(close, "RELOP:greater-than:"),
  ) {
    (Some(open), Some(close)) => {
      continues_an_inequality(open, ctxt.nodes)
        || operand_ends_before(open, ctxt.nodes) && operand_starts_after(close, ctxt.nodes)
    },
    _ => false,
  }
}

/// Does the plain `<` at `at` continue an inequality — an operand ends right before it, and a `<`-like
/// relation (`<`, `\le`, `\ll`) stands right before that operand? `m<S<m+\delta\mid S>m` (2605.17504),
/// `0<s<1,\ t>-1`, `d_1<\cdots<d_n,\ k_i,d_i>0`: the `<` is a relation, whatever follows its `>`. An
/// operand here is one letter, number or ellipsis, scripted or not.
fn continues_an_inequality(at: usize, nodes: &[XMLNode]) -> bool {
  operand_start_before(at, nodes)
    .and_then(|start| start.checked_sub(1))
    .and_then(|before| nodes.get(before))
    .is_some_and(|node| {
      crate::data::get_grammatical_role(node) == "RELOP"
        && crate::data::get_token_meaning(node).starts_with("less-than")
    })
}

/// The position of the one-item operand that ends right before the node at `at`: a letter, number,
/// atom or ellipsis — a scripted one by its base, before its first script's start marker — or a group,
/// with the head it is applied to (`s(X)`, `\mathbb{P}_{X\sim p}\big(m<s(X)<m+\delta\mid s(X)>m\big)`, 2605.17504).
fn operand_start_before(at: usize, nodes: &[XMLNode]) -> Option<usize> {
  let before = at.checked_sub(1)?;
  let node = nodes.get(before)?;
  let role = crate::data::get_grammatical_role(node);
  if is_script_role(&role) {
    let start = nodes[..before].iter().rposition(|marker| marker == node)?;
    return operand_start_before(start, nodes);
  }
  if role == "CLOSE" {
    let mut depth = 0usize;
    let open = (0..=before).rev().find(|&i| {
      match crate::data::get_grammatical_role(&nodes[i]).as_str() {
        "CLOSE" => depth += 1,
        "OPEN" => depth = depth.saturating_sub(1),
        _ => {},
      }
      depth == 0
    })?;
    return Some(operand_start_before(open, nodes).unwrap_or(open));
  }
  matches!(
    role.as_str(),
    "UNKNOWN" | "ID" | "NUMBER" | "ATOM" | "ELIDEOP"
  )
  .then_some(before)
}

/// The 0-based position in the parse's nodes of the lexeme `xm`, when its name starts with `prefix`
/// (`ROLE:text:index`, the index 1-based, `lookup_lex_node`).
fn lexeme_position(xm: &XM, prefix: &str) -> Option<usize> {
  let XM::Lexeme(lex, _) = xm else {
    return None;
  };
  if !lex.starts_with(prefix) {
    return None;
  }
  lex
    .rsplit(':')
    .next()?
    .parse::<usize>()
    .ok()?
    .checked_sub(1)
}

/// A script's start and end markers, which carry the script's node (util.rs).
fn is_script_role(role: &str) -> bool {
  matches!(
    role,
    "POSTSUBSCRIPT" | "POSTSUPERSCRIPT" | "FLOATSUBSCRIPT" | "FLOATSUPERSCRIPT"
  )
}

/// Does an operand end right before the node at `at` — one a relation there relates on its left? A
/// script's end marker stands for its base, scripted or not (`c_1<`, `x_i^2<`); a big operator's
/// scripts, a script's start marker (`e_{<u,i>}`), a function or operator head, another relation, a
/// punctuation mark or the formula's start end none.
fn operand_ends_before(at: usize, nodes: &[XMLNode]) -> bool {
  let Some(before) = at.checked_sub(1) else {
    return false;
  };
  let Some(node) = nodes.get(before) else {
    return false;
  };
  let role = crate::data::get_grammatical_role(node);
  if is_script_role(&role) {
    // An end marker: its start marker, earlier, carries the same node; none before a start marker.
    return nodes[..before]
      .iter()
      .rposition(|marker| marker == node)
      .is_some_and(|start| operand_ends_before(start, nodes));
  }
  matches!(
    role.as_str(),
    "UNKNOWN"
      | "ID"
      | "NUMBER"
      | "ATOM"
      | "ARRAY"
      | "CLOSE"
      | "POSTFIX"
      | "SUPOP"
      | "ELIDEOP"
      | "VERTBAR"
      | "FUNCTION"
  )
}

/// Does an operand start right after the node at `at` — one a relation there relates on its right?
/// A script marker (`e_{<u,i>}`'s end), a sign, a relation, a punctuation
/// mark or the formula's end starts none: `\lambda<x,y>+\mu<u,v>` keeps its angles. Nor does a
/// detached node — the null delimiter a retry supplies, which sits last in `nodes` whatever its place
/// in the stream (`balance_null_delimiters`, parser.rs), or a wide space's stand-in mark (`filter_hints`).
fn operand_starts_after(at: usize, nodes: &[XMLNode]) -> bool {
  nodes.get(at + 1).is_some_and(|node| {
    node.get_parent().is_some()
      && matches!(
        crate::data::get_grammatical_role(node).as_str(),
        "UNKNOWN"
          | "ID"
          | "NUMBER"
          | "ATOM"
          | "ARRAY"
          | "OPEN"
          | "VERTBAR"
          | "ELIDEOP"
          | "FUNCTION"
          | "OPFUNCTION"
          | "TRIGFUNCTION"
          | "OPERATOR"
          | "BIGOP"
          | "SUMOP"
          | "INTOP"
          | "LIMITOP"
          | "DIFFOP"
      )
  })
}

/// Refutes a reading of plain `<` … `>` as angles where they stand as relations
/// (`angle_signs_are_relations`); `args` open with the `<` and close with the `>`.
fn refute_related_angle_signs(
  args: &[Option<XM>],
  ctxt: &ActionContext,
) -> Result<(), Box<dyn Error>> {
  if let (Some(Some(open)), Some(Some(close))) = (args.first(), args.last())
    && angle_signs_are_relations(open, close, ctxt)
  {
    return Err("plain `<` and `>` between operands are relations, no angles".into());
  }
  Ok(())
}

/// Refutes a plain `<a|b>` or `<a|f|b>` Dirac reading whose `<` continues an inequality
/// (`continues_an_inequality`): `\mathbb{P}(m<S<m+\delta\mid S>m)` (2605.17504). A factor on either
/// side keeps the bracket, as Perl's: `c_m^*<m|H|n>c_n` c_m^*·⟨m|H|n⟩·c_n (57ch review).
fn refute_an_inequality_bracket(
  args: &[Option<XM>],
  ctxt: &ActionContext,
) -> Result<(), Box<dyn Error>> {
  if let Some(Some(open)) = args.first()
    && lexeme_position(open, "RELOP:less-than:")
      .is_some_and(|at| continues_an_inequality(at, ctxt.nodes))
  {
    return Err("a `<` that continues an inequality is a relation, no bracket".into());
  }
  Ok(())
}

/// `langle_rel term_list rangle_rel`: `<x,y>` an angle fence (`fenced`), where its signs are no
/// relations (`angle_signs_are_relations`, 57bt).
pub fn ascii_angle_fenced(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  refute_related_angle_signs(&args, &ctxt)?;
  fenced(rule_id, args, pragmas, ctxt)
}

/// `<a|b>` → inner-product@(a, b) (`qm_braket`), where its `<` continues no inequality
/// (`refute_an_inequality_bracket`, 57bt).
pub fn ascii_qm_braket(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  refute_an_inequality_bracket(&args, &ctxt)?;
  qm_braket(rule_id, args, pragmas, ctxt)
}

/// `<a|f|b>` → quantum-operator-product@(a, f, b) (`qm_bracket`), where its `<` continues no
/// inequality (`refute_an_inequality_bracket`, 57bt).
pub fn ascii_qm_bracket(
  rule_id: i32,
  args: Vec<Option<XM>>,
  pragmas: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  refute_an_inequality_bracket(&args, &ctxt)?;
  qm_bracket(rule_id, args, pragmas, ctxt)
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
  let evaluation_bar_inside = holds_evaluation_bar(&arg);
  // Merge each pair of | into ‖
  let open = merge_vertbar_pair(open1_opt.unwrap(), "OPEN", ctxt.nodes);
  let close = merge_vertbar_pair(_close1_opt.unwrap(), "CLOSE", ctxt.nodes);
  let op = XProps {
    meaning: Some(Cow::Borrowed("norm")),
    ..XProps::default()
  };
  interpret_delimited(op.into(), vec![open, arg, close], ctxt)
    // Two `|`s merged: Perl's `SINGLEVERTBAR SINGLEVERTBAR` norm is a Factor, no `aBarearg`
    // (`\log||x||^2` log·‖x‖²); a bar fence like `\|x\|` here (divergence #353).
    .and_then(|fence| mark_bar_fence(fence, false, evaluation_bar_inside))
    .map(Some)
}

/// Norm with the *double-bar* token: `\|x\|` and `\Vert x\Vert` each lex to a
/// single `VERTBAR:||` token (the doubled bar ‖), so they form
/// `||₁ expression ||₂` (3 args) — distinct from the four-single-bar `||x||`
/// form handled by [`norm_fenced`]. Without this rule the standard norm
/// notation `\|x\|` / `\Vert x\Vert` (ubiquitous in analysis / ML: `\|x\|_2`,
/// operator norms) failed to parse → `ltx_math_unparsed`, whereas Perl parses
/// it to `norm@(x)`. `merge_vertbar_pair` morphs each bar to an OPEN/CLOSE ‖
/// delimiter (it doesn't actually require a pair — it normalizes one XM).
pub fn double_norm_fenced(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  // Args: ||1 expression ||2
  unp!(args => open_opt, arg_opt, close_opt);
  let arg = arg_opt.unwrap();
  let open = merge_vertbar_pair(open_opt.unwrap(), "OPEN", ctxt.nodes);
  let close = merge_vertbar_pair(close_opt.unwrap(), "CLOSE", ctxt.nodes);
  let op = XProps {
    meaning: Some(Cow::Borrowed("norm")),
    ..XProps::default()
  };
  // `\|` is a VERTBAR (`doublevertbar`): the pair is a bar fence, an `aBarearg`.
  interpret_delimited(op.into(), vec![open, arg, close], ctxt)
    .and_then(|fence| mark_bar_fence(fence, false, false))
    .map(Some)
}

/// True iff the XM points to a token whose underlying DOM node carries
/// a negative `rpadding` attribute — the prefiltered signal that an
/// `<XMHint width="-X.Xpt"/>` (negative `\kern`) immediately followed
/// this token. Used by `stretchy_triple_norm_fenced` to confirm the kerned
/// `\vertiii` idiom (divergence #354). Task #263.
fn has_negative_rpadding(xm: &XM, nodes: &[XMLNode]) -> bool {
  let node_opt = match xm {
    XM::Lexeme(lex, _) => lookup_lex_node(lex, nodes).ok(),
    _ => None,
  };
  let Some(node) = node_opt else {
    return false;
  };
  match node.get_attribute("rpadding") {
    Some(s) => crate::util::get_xmhint_spacing(&s) < 0.0,
    None => false,
  }
}

/// A kerned stack's merged glyph (`merge_vertbar_pair`, `merge_vertbar_triple`) takes the padding
/// after the stack's `last` bar: the kerns between its bars are drawn by the glyph now, not spacing
/// around it (57am review round 7: every `⦀` carried the `-1.1pt` between the first two bars).
fn padded_as(mut merged: XM, last: &XM, nodes: &[XMLNode]) -> XM {
  if let XM::Token(props, _) = &mut merged {
    props.rpadding = match last {
      XM::Lexeme(lex, _) => lookup_lex_node(lex, nodes)
        .ok()
        .and_then(|node| node.get_attribute("rpadding"))
        .map(Cow::Owned),
      XM::Token(last_props, _) => last_props.rpadding.clone(),
      _ => None,
    };
  }
  merged
}

/// Apply `merge_vertbar_pair` semantics but emit U+2980 (⦀ TRIPLE
/// VERTICAL BAR DELIMITER) instead of U+2016 (‖) — used by
/// `stretchy_triple_norm_fenced` for the `\vertiii`/`|||·|||` idiom.
fn merge_vertbar_triple(xm: XM, role: &'static str, nodes: &[XMLNode]) -> XM {
  let mut props = match xm {
    XM::Lexeme(ref lex, _) => match lookup_lex_node(lex, nodes) {
      Ok(node) => XProps::from(node),
      _ => XProps::default(),
    },
    XM::Token(ref p, _) => p.clone(),
    _ => XProps::default(),
  };
  props.role = Some(Cow::Borrowed(role));
  props.content = Some(Cow::Borrowed("\u{2980}")); // ⦀
  props.stretchy = None;
  XM::Token(props, Meta::default())
}

/// Two `\left|` bars around two `\right|` bars → norm, Perl's `SINGLEVERTBAR SINGLEVERTBAR
/// absExpression SINGLEVERTBAR SINGLEVERTBAR` (MathGrammar:294): `\left|\left|x\right|\right|` is
/// norm@(x), kerned (`\vertii`, `\left|\kern-…\left|·\right|\kern-…\right|`) or not. The lexer
/// sides the bars (LEFT_/RIGHT_STRETCHY_VERTBAR), so two separate fences `\left|x\right|\left|y\right|`
/// never match the rule, and `Meta::abs_depth` ranks the norm (one level) above |(|x|)| (two); the
/// kern check this action once made (task #263) read the unkerned stack as |(|x|)|, which Perl does
/// not (57am review round 8). Witness arXiv:2211.13044 §S4.Ex17.
pub fn stretchy_norm_fenced(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(args => open1_opt, open2_opt, arg_opt, close1_opt, close2_opt);
  let open1 = open1_opt.ok_or("stretchy_norm_fenced: missing open1")?;
  let open2 = open2_opt.ok_or("stretchy_norm_fenced: missing open2")?;
  let close1 = close1_opt.ok_or("stretchy_norm_fenced: missing close1")?;
  let close2 = close2_opt.ok_or("stretchy_norm_fenced: missing close2")?;
  let arg = arg_opt.ok_or("stretchy_norm_fenced: missing arg")?;
  let open = padded_as(
    merge_vertbar_pair(open1, "OPEN", ctxt.nodes),
    &open2,
    ctxt.nodes,
  );
  let close = padded_as(
    merge_vertbar_pair(close1, "CLOSE", ctxt.nodes),
    &close2,
    ctxt.nodes,
  );
  let op = XProps {
    meaning: Some(Cow::Borrowed("norm")),
    ..XProps::default()
  };
  interpret_delimited(op.into(), vec![open, arg, close], ctxt)
    .and_then(|fence| mark_bar_fence(fence, false, false))
    .map(Some)
}

/// Triple-bar `\left|\kern\left|\kern\left|·\right|\kern\right|\kern\right|`
/// → operator-norm. Requires the kern signal on the first TWO bars of each
/// side (open1 + open2 kerned to their successors; symmetric on the right).
/// Goes beyond Perl LaTeXML (Perl produces ‖|x|‖ — partial recognition —
/// for the same input). Task #263.
pub fn stretchy_triple_norm_fenced(
  _rule_id: i32,
  mut args: Vec<Option<XM>>,
  _: &[ValidationPragmatics],
  ctxt: ActionContext,
) -> Result<Option<XM>, Box<dyn Error>> {
  unp!(
    args =>
    open1_opt, open2_opt, open3_opt,
    arg_opt,
    close1_opt, close2_opt, close3_opt
  );
  let open1 = open1_opt.ok_or("stretchy_triple_norm_fenced: missing open1")?;
  let open2 = open2_opt.ok_or("stretchy_triple_norm_fenced: missing open2")?;
  let close1 = close1_opt.ok_or("stretchy_triple_norm_fenced: missing close1")?;
  let close2 = close2_opt.ok_or("stretchy_triple_norm_fenced: missing close2")?;
  let open3 = open3_opt.ok_or("stretchy_triple_norm_fenced: missing open3")?;
  let close3 = close3_opt.ok_or("stretchy_triple_norm_fenced: missing close3")?;
  let arg = arg_opt.ok_or("stretchy_triple_norm_fenced: missing arg")?;
  // Two kern signals on each side (between bars 1-2 and bars 2-3).
  if !has_negative_rpadding(&open1, ctxt.nodes)
    || !has_negative_rpadding(&open2, ctxt.nodes)
    || !has_negative_rpadding(&close1, ctxt.nodes)
    || !has_negative_rpadding(&close2, ctxt.nodes)
  {
    return Err("stretchy_triple_norm_fenced: bars not triple-kern-stacked".into());
  }
  let open = padded_as(
    merge_vertbar_triple(open1, "OPEN", ctxt.nodes),
    &open3,
    ctxt.nodes,
  );
  let close = padded_as(
    merge_vertbar_triple(close1, "CLOSE", ctxt.nodes),
    &close3,
    ctxt.nodes,
  );
  let op = XProps {
    meaning: Some(Cow::Borrowed("operator-norm")),
    ..XProps::default()
  };
  interpret_delimited(op.into(), vec![open, arg, close], ctxt)
    .and_then(|fence| mark_bar_fence(fence, false, false))
    .map(Some)
}

/// Merge two single `|` tokens into `‖` (U+2016) with the given role.
/// Perl CatSymbols: concatenates two delimiters into a combined symbol.
fn merge_vertbar_pair(xm: XM, role: &'static str, nodes: &[XMLNode]) -> XM {
  let mut props = match xm {
    XM::Lexeme(ref lex, _) => match lookup_lex_node(lex, nodes) {
      Ok(node) => XProps::from(node),
      _ => XProps::default(),
    },
    XM::Token(ref p, _) => p.clone(),
    _ => XProps::default(),
  };
  props.role = Some(Cow::Borrowed(role));
  props.content = Some(Cow::Borrowed("\u{2016}")); // ‖
  props.stretchy = None; // remove stretchy=false from individual |
  XM::Token(props, Meta::default())
}
