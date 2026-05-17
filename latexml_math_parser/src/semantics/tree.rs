use latexml_core::Info;
use latexml_core::common::font::Font;
use latexml_core::common::xml::element_nodes;
use latexml_core::document::Document;
use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::fmt::Display;

use super::ActionContext;
use super::curry::{CurryConstraint, CurryConstraints, CurryTerm};
use super::metadata::Meta;
use crate::parser::{p_get_value, realize_xmnode};
use crate::pragmatics::ValidationPragmatics;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operator(pub Box<XM>);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args(pub Vec<Option<XM>>);

#[derive(Debug, Clone, Default)]
/// The allowed properties on any newly created XMath nodes
/// during grammatical processing
pub struct XProps {
  /// text content of the node
  pub content:           Option<Cow<'static, str>>,
  /// grammatical role used during math parsing
  pub role:              Option<Cow<'static, str>>,
  /// conceptual meaning of a construct, used in disambiguation and Content output
  pub meaning:           Option<Cow<'static, str>>,
  /// similar to `meaning`, but more fixed, usually associated with constants
  pub name:              Option<Cow<'static, str>>,
  /// script position w.r.t to baseline
  pub scriptpos:         Option<Cow<'static, str>>,
  /// a unique identifier, in the `xml:id` sense
  pub id:                Option<Cow<'static, str>>,
  /// a pointer to a different node, usually for `XMRef`
  pub idref:             Option<Cow<'static, str>>,
  /// an intermediate key to be fully realized as an id at a later time
  pub xmkey:             Option<Cow<'static, str>>,
  /// an optional subtree-specific Font
  pub font:              Option<Font>,
  /// usually associated with the internal `_font` attribute references
  pub fontref:           Option<Cow<'static, str>>,
  /// stretchy attribute for delimiters (e.g. "false" to suppress MathML stretching)
  pub stretchy:          Option<Cow<'static, str>>,
  /// marker for UNKNOWN tokens that may be used as functions (set by MATHPARSER_SPECULATE)
  pub possible_function: Option<Cow<'static, str>>,
  /// math style (display, text, script, scriptscript) — preserved from constructor
  pub mathstyle:         Option<Cow<'static, str>>,
  /// fraction line thickness (e.g. "0pt" for binomial)
  pub thickness:         Option<Cow<'static, str>>,
  /// declaration id (from \lxDeclare)
  pub decl_id:           Option<Cow<'static, str>>,
  /// lpadding/rpadding from alignment spacing
  pub lpadding:          Option<Cow<'static, str>>,
  pub rpadding:          Option<Cow<'static, str>>,
}
/// Custom PartialEq: ignores `xmkey`, `id`, `idref`, and `scriptpos` which are
/// bookkeeping/layout fields. Structurally identical parse trees that differ only
/// in internal reference keys or script-position labels (pre1 vs pre2 vs post1)
/// should be considered equal for deduplication purposes.
/// `scriptpos` is excluded because different grammar paths produce different
/// pre/post level assignments for the same mathematical expression. E.g.,
/// `{}^4{}_{12}C^{5+}` can produce 27 structurally distinct trees that differ
/// only in scriptpos values — all represent the same expression.
impl PartialEq for XProps {
  fn eq(&self, other: &Self) -> bool {
    self.content == other.content
      && self.role == other.role
      && self.meaning == other.meaning
      && self.name == other.name
      // Skip: scriptpos — layout hint, not semantic distinction
      // Skip: id, idref, xmkey — bookkeeping for Dual/Ref resolution
      && self.font == other.font
      && self.fontref == other.fontref
      && self.stretchy == other.stretchy
      && self.possible_function == other.possible_function
      && self.mathstyle == other.mathstyle
      && self.thickness == other.thickness
      && self.decl_id == other.decl_id
      && self.lpadding == other.lpadding
      && self.rpadding == other.rpadding
  }
}
impl Eq for XProps {}

impl Display for XProps {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // stub with Debug for now
    writeln!(f, "{self:?}")
  }
}

type XAttributes = (
  Option<Cow<'static, str>>,
  Option<Font>,
  Option<HashMap<String, String>>,
);
impl XProps {
  /// Consumes the `XProps` and returns the (content, font, attributes) in an arrangement suitable
  /// for using the `Document` methods
  pub fn into_attributes(mut self) -> XAttributes {
    let mut attrs = HashMap::default();
    if let Some(role) = self.role.take() {
      attrs.insert(String::from("role"), role.into_owned());
    }
    if let Some(meaning) = self.meaning.take() {
      attrs.insert(String::from("meaning"), meaning.into_owned());
    }
    if let Some(name) = self.name.take() {
      attrs.insert(String::from("name"), name.into_owned());
    }
    if let Some(scriptpos) = self.scriptpos.take() {
      attrs.insert(String::from("scriptpos"), scriptpos.into_owned());
    }
    if let Some(id) = self.id.take() {
      attrs.insert(String::from("xml:id"), id.into_owned()); // TODO: double-che.into_owned()ck
    }
    if let Some(idref) = self.idref.take() {
      attrs.insert(String::from("idref"), idref.into_owned());
    }
    if let Some(xmkey) = self.xmkey.take() {
      attrs.insert(String::from("_xmkey"), xmkey.into_owned());
    }
    if let Some(fontref) = self.fontref.take() {
      attrs.insert(String::from("_font"), fontref.into_owned());
    }
    if let Some(stretchy) = self.stretchy.take() {
      attrs.insert(String::from("stretchy"), stretchy.into_owned());
    }
    if let Some(pf) = self.possible_function.take() {
      attrs.insert(String::from("possibleFunction"), pf.into_owned());
    }
    if let Some(ms) = self.mathstyle.take() {
      attrs.insert(String::from("mathstyle"), ms.into_owned());
    }
    if let Some(th) = self.thickness.take() {
      attrs.insert(String::from("thickness"), th.into_owned());
    }
    if let Some(di) = self.decl_id.take() {
      attrs.insert(String::from("decl_id"), di.into_owned());
    }
    if let Some(lp) = self.lpadding.take() {
      attrs.insert(String::from("lpadding"), lp.into_owned());
    }
    if let Some(rp) = self.rpadding.take() {
      attrs.insert(String::from("rpadding"), rp.into_owned());
    }
    let attrs_opt = if attrs.is_empty() { None } else { Some(attrs) };
    (self.content.take(), self.font.take(), attrs_opt)
  }
}

/// The math parsing process can manipulate a variety of trees,
/// finally serialized via the XMath schema of LaTeXML.
///
/// The main structural variants are associated with
/// a "parsing state::, via an attached `Meta` object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XM {
  Lexeme(String, Meta),
  Token(XProps, Meta), // does this need Meta?
  Apply(Operator, Args, XProps, Meta),
  Dual(Box<XM>, Box<XM>, XProps, Meta),
  Ref(XProps),
  Wrap(Vec<XM>, XProps, Meta),
  Arg(Vec<XM>),
  Choices(Vec<XM>),
}
impl From<XProps> for XM {
  fn from(t: XProps) -> Self { XM::Token(t, Meta::default()) }
}

impl Display for Operator {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.fmt_indented(&Vec::new(), f) }
}
impl From<XProps> for Operator {
  fn from(t: XProps) -> Self { Operator(Box::new(XM::Token(t, Meta::default()))) }
}
impl From<XM> for Operator {
  fn from(xm: XM) -> Self { Operator(Box::new(xm)) }
}
impl Operator {
  /// obtain a reference to this operator's metadata
  pub fn get_meta(&self) -> &Meta { (*self.0).get_meta() }
  /// obtain a reference to this operator's metadata
  pub fn get_meta_mut(&mut self) -> &mut Meta { (*self.0).get_meta_mut() }

  /// borrow the constraints and pass them to the outer caller
  pub fn get_constraints(&self) -> Vec<&CurryConstraint> {
    // while we're at it, operators shouldn't have a curry_level set at this stage. Should they?!
    let meta = self.0.get_meta();
    meta.curry_constraints.iter().collect()
  }
  /// extract the constraints and pass them to the outer caller
  pub fn drain_constraints(&mut self) -> Vec<CurryConstraint> {
    // while we're at it, operators shouldn't have a curry_level set at this stage. Should they?!
    let meta = self.0.get_meta_mut();
    meta.curry_level = None;
    meta.curry_constraints.drain().collect()
  }

  pub fn unconstrain_recursive(&mut self) { self.0.unconstrain_recursive(); }

  fn fmt_indented(&self, level: &[bool], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let XM::Lexeme(..) = &*self.0 {
      self.0.fmt_indented(level, f)
    } else {
      let indent = if level.is_empty() {
        // special case, if @ starts the print, add a level for clarity
        // TODO: Is there a better general treatment here?
        String::from("   ")
      } else {
        aux_generate_indent(level, false)
      };
      writeln!(f, "{indent}@-op┐")?;
      let mut rhs_level: Vec<bool> = level.to_vec();
      rhs_level.push(true);
      rhs_level.push(false);
      self.0.fmt_indented(&rhs_level, f)
    }
  }
}
impl Display for Args {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.fmt_indented(&Vec::new(), f) }
}

impl Args {
  fn fmt_indented(&self, level: &[bool], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut peekable = self.0.iter().peekable();
    while let Some(maybe_arg) = peekable.next() {
      if peekable.peek().is_some() {
        maybe_arg
          .as_ref()
          .unwrap_or(&XM::Lexeme(
            String::from("missing_argument"),
            Meta::default(),
          ))
          .fmt_indented(level, f)?;
      } else {
        let mut last_level: Vec<bool> = level.to_vec();
        if !last_level.is_empty() {
          last_level.pop();
          last_level.push(false);
        }
        maybe_arg
          .as_ref()
          .unwrap_or(&XM::Lexeme(
            String::from("missing_argument"),
            Meta::default(),
          ))
          .fmt_indented(&last_level, f)?;
      };
    }
    Ok(())
  }
  /// Obtain defined subtrees as a slice, e.g. for consistency validation
  pub fn trees(&self) -> Vec<&XM> { self.0.iter().filter_map(|x| x.as_ref()).collect() }
  /// Obtain defined subtrees as a mutable slice, e.g. for consistency validation
  pub fn trees_mut(&mut self) -> Vec<&mut XM> {
    self.0.iter_mut().filter_map(|x| x.as_mut()).collect()
  }

  /// borrow the constraints and pass them to the outer caller
  pub fn get_constraints(&self) -> Vec<&CurryConstraint> {
    self
      .trees()
      .into_iter()
      .flat_map(|t| t.get_meta().curry_constraints.iter())
      .collect()
  }
  /// extract the constraints and pass them to the outer caller
  pub fn drain_constraints(&mut self) -> Vec<CurryConstraint> {
    self
      .trees_mut()
      .into_iter()
      .flat_map(|t| t.get_meta_mut().curry_constraints.drain())
      .collect()
  }
  pub fn unconstrain_recursive(&mut self) {
    for tree in self.0.iter_mut().flatten() {
      tree.unconstrain_recursive();
    }
  }
}

impl From<Vec<XM>> for Args {
  fn from(items: Vec<XM>) -> Args { Args(items.into_iter().map(Some).collect()) }
}

impl XM {
  pub fn get_meta(&self) -> &Meta {
    match self {
      XM::Lexeme(_, ref meta) => meta,
      XM::Token(_, ref meta) => meta,
      XM::Apply(_, _, _, ref meta) => meta,
      XM::Dual(_, _, _, ref meta) => meta,
      XM::Wrap(_, _, ref meta) => meta,
      XM::Choices(cs) => cs[0].get_meta(), // Should we return a none type instead?
      XM::Ref(_) | XM::Arg(_) => todo!(),
    }
  }
  pub fn get_meta_mut(&mut self) -> &mut Meta {
    match self {
      XM::Lexeme(_, ref mut meta) => meta,
      XM::Token(_, ref mut meta) => meta,
      XM::Apply(_, _, _, ref mut meta) => meta,
      XM::Dual(_, _, _, ref mut meta) => meta,
      XM::Wrap(_, _, ref mut meta) => meta,
      XM::Choices(cs) => cs[0].get_meta_mut(), // Should we return a none type instead?
      XM::Ref(_) | XM::Arg(_) => todo!(),
    }
  }
  pub fn get_inner_meta(&self) -> Vec<&Meta> {
    match self {
      XM::Lexeme(_atom, meta) => vec![meta],
      XM::Apply(op, args, ..) => vec![op.get_meta()]
        .into_iter()
        .chain(
          args
            .0
            .iter()
            .filter(|arg| arg.is_some())
            .map(|arg| arg.as_ref().unwrap().get_meta()),
        )
        .collect(),
      XM::Dual(content, presentation, ..) => vec![content.get_meta(), presentation.get_meta()],
      _ => Vec::new(),
    }
  }
  pub fn get_value(&self, nodes: &[Node]) -> Result<Cow<'_, str>, Box<dyn Error>> {
    Ok(match self {
      XM::Lexeme(lex, _) => Cow::Owned(p_get_value(lookup_lex_node(lex, nodes)?)),
      XM::Token(props, _) => match props.content {
        None => props.name.clone().unwrap_or(Cow::Borrowed("")),
        Some(ref v) => {
          if v.is_empty() {
            props.name.clone().unwrap_or(Cow::Borrowed(""))
          } else {
            v.clone()
          }
        },
      },
      XM::Apply(op, args, ..) => Cow::Owned(format!(
        "{}{}",
        op.0.get_value(nodes)?,
        args
          .trees()
          .iter()
          .map(|t| t.get_value(nodes).expect("inner"))
          .collect::<Vec<_>>()
          .join("")
      )),
      // Choices/Arg don't carry a serialized value — return empty for safety;
      // callers treat Ref similarly (see the XM::Ref arm below).
      XM::Choices(_) | XM::Arg(_) => Cow::Borrowed(""),
      XM::Dual(content, pres, ..) => Cow::Owned(format!(
        "{}{}",
        content.get_value(nodes).expect("inner"),
        pres.get_value(nodes).expect("inner")
      )),
      XM::Wrap(args, ..) => Cow::Owned(
        args
          .iter()
          .map(|a| a.get_value(nodes).expect("oof"))
          .collect::<Vec<_>>()
          .join(""),
      ),
      XM::Ref(_) => Cow::Borrowed(""),
    })
  }

  /// Specialize a tree with the given Meta object,
  /// also verifying the tree's inner consistency.
  ///
  /// Whenever a contradiction/inconsistency is detected, we return an error
  /// This method should always be called on tree construction, as it also manages the various curry
  /// constraints, keeping the resolution local / fast.
  pub fn specialize(
    self,
    mut into: Meta,
    pragmas: &[ValidationPragmatics],
  ) -> Result<Self, Box<dyn Error>> {
    match self {
      XM::Lexeme(name, meta) => {
        let new_meta = meta.with_curry_atom(into, &name)?;
        Ok(XM::Lexeme(name, new_meta))
      },
      XM::Token(t, meta) => {
        // Specialization of bare Token variants isn't exercised by current
        // grammar rules. Return the tree unchanged so a future rule that
        // invokes this path fails at a higher layer (validation) rather
        // than panicking here.
        Ok(XM::Token(t, meta))
      },
      XM::Ref(_) => Ok(self),
      XM::Apply(mut op, mut args, props, meta) => {
        // First, if we have a specialize directive, execute it:
        match into.specialize {
          Some(ref directive)
            if directive == "embellish"
            // Atoms with embellishments should get their curry levels renamed
            // to avoid conflicts with the same atoms *without* the embellishments
            // as often this technique is used to generate new unique names.
            && args.0.len() <= 2 =>
          {
            if let Some(XM::Lexeme(_, arg_meta)) = &mut args.0[0] {
              if let Some(CurryTerm::Var(ref mut curry_var)) = arg_meta.curry_level {
                let mut base_op = op.0.base_operator_name();
                // fish out a local name to use as an embellishment
                if let Some(last_colon_idx) = base_op.rfind(':') {
                  base_op.replace_range(..=last_colon_idx, "");
                } else if let Some(last_dot_idx) = base_op.rfind('.') {
                  base_op.replace_range(..=last_dot_idx, "");
                }
                if base_op.is_empty() {
                  base_op = String::from("embellished");
                }
                curry_var.push('-');
                curry_var.push_str(&base_op);
              }
              into.curry_level.clone_from(&arg_meta.curry_level)
            }
          },
          _ => {},
        }
        // Next, we validate the constraints.
        let initial_constraint_count = meta.curry_constraints.len();
        let mut new_meta = meta;
        // SPECIAL CASE (of course): if we are using an "unconstrained" directive, no need to do any
        // of this
        if into.specialize.as_deref() == Some("unconstrained") {
          op.unconstrain_recursive();
          args.unconstrain_recursive();
          new_meta = new_meta.with(into)?;
          new_meta.curry_constraints = CurryConstraints::new();
          new_meta.specialize = None;
        } else {
          new_meta = new_meta.with(into)?;
          // COPY constraints from inner subtrees to the new apply root
          for constraint in op.get_constraints().into_iter() {
            new_meta.curry_constraints.insert(constraint.clone());
          }
          for constraint in args.get_constraints().into_iter() {
            new_meta.curry_constraints.insert(constraint.clone());
          }
          // and require the current curry level to be >= 1 if compound
          // (otherwise, it's already built in)
          if let Some(ref expr) = new_meta.curry_level {
            match expr {
              CurryTerm::Add(..) | CurryTerm::Sub(..) => {
                new_meta.curry_constraints.insert(CurryConstraint((
                  expr.clone(),
                  Ordering::Greater,
                  CurryTerm::Literal(0),
                )));
              },
              _ => {},
            }
          }
        }
        if initial_constraint_count < new_meta.curry_constraints.len() {
          // whenever we add a constraint, re-valdiate the expression, and prune it if needed
          // println!("Tree: \n{}", XM::Apply(op.clone(), args.clone(), new_meta.clone()));
          new_meta.validate()?;
        }
        let new_tree = XM::Apply(op, args, props, new_meta);
        for pragma in pragmas {
          // expert pragmatics get to validate each new tree,
          // in order to prune wrong interpretations as early as possible
          pragma.validate(&new_tree)?;
        }
        Ok(new_tree)
      },
      // Dual/Wrap/Arg variants aren't specialized by current grammar rules
      // — return unchanged rather than panic. Future grammar extensions can
      // add real specialization rules if needed.
      dual @ XM::Dual(..) => Ok(dual),
      wrap @ XM::Wrap(..) => Ok(wrap),
      arg @ XM::Arg(_) => Ok(arg),
      XM::Choices(_) => Err("can not specialize choices".into()),
    }
  }

  /// Prunes choices based on a validation pass leveraging a choice of pragmatics
  /// if the pruning arrives at no viable trees at all, the original tree is returned,
  /// hence the "soft" function name.
  /// These are executed at the end of the program, so need to be invoked recursively on each
  /// subtree
  pub fn soft_prune_choices(self, pragmatics: ValidationPragmatics) -> Self {
    match self {
      XM::Choices(trees) => {
        let (consistent_trees, inconsistent_trees): (Vec<XM>, Vec<XM>) = trees
          .into_iter()
          .partition(|tree| pragmatics.validate_recursive(tree).is_ok());
        match consistent_trees.len() {
          0 => XM::Choices(inconsistent_trees),
          1 => consistent_trees.into_iter().next().unwrap(),
          _more => XM::Choices(consistent_trees),
        }
      },
      other => other,
    }
  }

  /// Count the number of `absent` markers in this tree.
  ///
  /// `absent` appears as a sentinel `XM::Token` (with
  /// `meaning="absent"`) when a parse has to fill in a missing
  /// operand position — e.g. `< x >` parsed as the relational
  /// chain `absent < x > absent`. Fewer `absent` markers signals
  /// a parse that satisfies the grammar without needing fillers.
  pub fn count_absent(&self) -> usize {
    match self {
      XM::Token(props, _) => {
        if props.meaning.as_deref() == Some("absent") {
          1
        } else {
          0
        }
      },
      XM::Apply(op, args, ..) => op.0.count_absent() + args.trees().iter().map(|a| a.count_absent()).sum::<usize>(),
      XM::Dual(c, p, ..) => c.count_absent() + p.count_absent(),
      XM::Wrap(items, ..) => items.iter().map(|i| i.count_absent()).sum(),
      XM::Choices(trees) => trees.iter().map(|t| t.count_absent()).sum(),
      XM::Arg(items) => items.iter().map(|i| i.count_absent()).sum(),
      XM::Lexeme(_, _) | XM::Ref(_) => 0,
    }
  }

  /// Semantic-node count used by parse-ranking pragmas to prefer
  /// compact interpretations over verbose ones. Not a general
  /// "tree size" function — it deliberately ignores presentation
  /// duplication so structurally-equivalent parses get a fair
  /// comparison.
  ///
  /// Smaller wins: `norm@(a)` (2 nodes) beats
  /// `absolute-value@(absolute-value@(a))` (3 nodes);
  /// `differential-d@(x)` (2 nodes) beats `d*x` (3 nodes).
  ///
  /// **Counting conventions**:
  /// * Each `XM::Apply` contributes ONE node — the operator is part
  ///   of the Apply's identity, not a separate child. So `f@(x)`
  ///   is 2 nodes (the Apply + x), not 3.
  /// * `XM::Dual(content, presentation)` counts **only the content
  ///   tree** — the presentation branch is a parallel rendering of
  ///   the same semantics and contributes the same count, so
  ///   double-counting would inflate purely-cosmetic siblings.
  /// * `XM::Ref(props)` is resolved to its target via the
  ///   presentation-branch index built at the Dual boundary —
  ///   so a Ref pointing to a deep sub-tree contributes its full
  ///   target's node count, not just 1. This keeps the ranking
  ///   honest when one parse uses a single Ref to a complex node
  ///   versus another that lays out multiple Refs to leaves.
  pub fn count_nodes_for_parse_ranking(&self) -> usize {
    let mut index: HashMap<String, &XM> = HashMap::default();
    self.build_ref_index(&mut index);
    self.count_nodes_with_index(&index, &mut Vec::new())
  }

  /// Walk this tree and register every `(id, &XM)` pair that a
  /// `Ref` might lookup. Refs key on `props.id` or `props.xmkey`.
  fn build_ref_index<'a>(&'a self, out: &mut HashMap<String, &'a XM>) {
    let take = |p: &'a XProps| -> Option<String> {
      p.id.as_ref().map(|c| c.to_string()).or_else(|| p.xmkey.as_ref().map(|c| c.to_string()))
    };
    match self {
      XM::Token(props, _) => {
        if let Some(k) = take(props) {
          out.entry(k).or_insert(self);
        }
      },
      XM::Apply(_op, args, props, _) => {
        if let Some(k) = take(props) {
          out.entry(k).or_insert(self);
        }
        for arg in args.trees() {
          arg.build_ref_index(out);
        }
      },
      XM::Dual(c, p, props, _) => {
        if let Some(k) = take(props) {
          out.entry(k).or_insert(self);
        }
        c.build_ref_index(out);
        p.build_ref_index(out);
      },
      XM::Wrap(items, props, _) => {
        if let Some(k) = take(props) {
          out.entry(k).or_insert(self);
        }
        for it in items {
          it.build_ref_index(out);
        }
      },
      XM::Arg(items) | XM::Choices(items) => {
        for it in items {
          it.build_ref_index(out);
        }
      },
      XM::Lexeme(_, _) | XM::Ref(_) => {},
    }
  }

  /// Counting worker with the Ref index. `visited` is a stack of
  /// idref/xmkey strings currently being resolved — guards against
  /// cyclic references (defensive; idref graphs in valid XMath
  /// are acyclic).
  fn count_nodes_with_index(&self, index: &HashMap<String, &XM>, visited: &mut Vec<String>) -> usize {
    match self {
      XM::Token(_, _) | XM::Lexeme(_, _) => 1,
      XM::Apply(_op, args, ..) => 1 + args.trees().iter().map(|a| a.count_nodes_with_index(index, visited)).sum::<usize>(),
      XM::Dual(c, _p, ..) => c.count_nodes_with_index(index, visited),
      XM::Wrap(items, ..) => 1 + items.iter().map(|i| i.count_nodes_with_index(index, visited)).sum::<usize>(),
      XM::Choices(trees) => trees.iter().map(|t| t.count_nodes_with_index(index, visited)).sum(),
      XM::Arg(items) => 1 + items.iter().map(|i| i.count_nodes_with_index(index, visited)).sum::<usize>(),
      XM::Ref(props) => {
        let key = props.idref.as_ref().map(|c| c.to_string()).or_else(|| props.xmkey.as_ref().map(|c| c.to_string()));
        match key {
          Some(k) if !visited.contains(&k) => match index.get(&k) {
            Some(target) => {
              visited.push(k);
              let n = target.count_nodes_with_index(index, visited);
              visited.pop();
              n
            },
            None => 1,
          },
          // Already visiting this ref (cycle) or no key at all —
          // count as 1 so we don't double-count or infinite-loop.
          _ => 1,
        }
      },
    }
  }

  /// Multi-tree pragma: keep only the surviving trees with the
  /// fewest `absent` markers. If a single tree wins, unwrap from
  /// `XM::Choices`. If many tie at the minimum, leave them all in
  /// `XM::Choices` for downstream pragmas. If the forest is empty
  /// or unambiguous, pass through unchanged.
  ///
  /// Rationale: P1 from the 2026-05-17 tiebreaking research notes
  /// — parses that use the `absent` filler are structurally weaker
  /// than parses that don't.
  pub fn prefer_fewer_absent(self) -> Self {
    match self {
      XM::Choices(trees) if trees.len() > 1 => {
        let min = trees.iter().map(|t| t.count_absent()).min().unwrap_or(0);
        let kept: Vec<XM> = trees.into_iter().filter(|t| t.count_absent() == min).collect();
        match kept.len() {
          0 => XM::Choices(Vec::new()),
          1 => kept.into_iter().next().unwrap(),
          _ => XM::Choices(kept),
        }
      },
      other => other,
    }
  }

  /// Multi-tree pragma: keep only the surviving trees with the
  /// smallest node count. Helps select compact semantic
  /// interpretations over deeply nested literal ones — e.g.
  /// `norm@(x)` over `absolute-value@(absolute-value@(x))`.
  ///
  /// Rationale: P2 from the 2026-05-17 tiebreaking research notes.
  pub fn prefer_smaller_tree(self) -> Self {
    match self {
      XM::Choices(trees) if trees.len() > 1 => {
        let min = trees.iter().map(|t| t.count_nodes_for_parse_ranking()).min().unwrap_or(0);
        let kept: Vec<XM> = trees.into_iter().filter(|t| t.count_nodes_for_parse_ranking() == min).collect();
        match kept.len() {
          0 => XM::Choices(Vec::new()),
          1 => kept.into_iter().next().unwrap(),
          _ => XM::Choices(kept),
        }
      },
      other => other,
    }
  }

  /// Does the root of this parse tree match `Dual(Apply(op_meaning,
  /// [_, _]), …)` — i.e. a Dual-wrapped 2-argument Apply with the
  /// given operator meaning? Used by the math-root operator-preference
  /// pragma.
  fn root_dual_apply_meaning_is(&self, expected: &str, expected_arg_count: usize) -> bool {
    match self {
      XM::Dual(content, _, _, _) => content.root_apply_meaning_is(expected, expected_arg_count),
      _ => false,
    }
  }

  /// Inner helper: matches `Apply` with an operator whose meaning
  /// equals `expected` and whose Args has exactly `expected_arg_count`
  /// trees.
  fn root_apply_meaning_is(&self, expected: &str, expected_arg_count: usize) -> bool {
    if let XM::Apply(Operator(op), args, ..) = self {
      if args.trees().len() != expected_arg_count {
        return false;
      }
      match &**op {
        XM::Token(props, _) => props.meaning.as_deref() == Some(expected),
        XM::Lexeme(name, _) => name == expected || name.starts_with(&format!("{expected}:")),
        _ => false,
      }
    } else {
      false
    }
  }

  /// Does the root of this parse match `Dual(?, Apply(op_meaning,
  /// [...]))` where the op meaning starts with `delimited-`?
  fn root_dual_apply_is_delimited_wrapper(&self) -> bool {
    let XM::Dual(content, _, _, _) = self else {
      return false;
    };
    let XM::Apply(Operator(op), _, _, _) = &**content else {
      return false;
    };
    match &**op {
      XM::Token(props, _) => props.meaning.as_deref().is_some_and(|m| m.starts_with("delimited-")),
      XM::Lexeme(name, _) => name.starts_with("delimited-"),
      _ => false,
    }
  }

  /// Does the root of this parse match `Dual(?, Apply(op_meaning,
  /// [...]))` where the op meaning is one of the named-interval
  /// operators (open/closed/half-open intervals)?
  fn root_dual_apply_is_named_interval(&self) -> bool {
    static NAMED_INTERVALS: &[&str] =
      &["open-interval", "closed-interval", "open-closed-interval", "closed-open-interval"];
    let XM::Dual(content, _, _, _) = self else {
      return false;
    };
    if let XM::Apply(Operator(op), args, ..) = &**content {
      if args.trees().len() != 2 {
        return false;
      }
      let meaning = match &**op {
        XM::Token(props, _) => props.meaning.as_deref(),
        XM::Lexeme(name, _) => Some(name.as_str()),
        _ => None,
      };
      meaning.is_some_and(|m| NAMED_INTERVALS.contains(&m))
    } else {
      false
    }
  }

  /// Does the root Dual's **presentation** Wrap contain exactly one
  /// non-delimiter child that is itself a `Dual` whose content
  /// shares the same operator meaning as the outer? This is the
  /// shape that produces `set@(set@(…))` / `vector@(vector@(…))`
  /// when rendered — the outer content's Ref resolves to the inner
  /// Dual instead of to flat items.
  fn root_dual_has_redundant_inner_wrap(&self) -> bool {
    let XM::Dual(content, presentation, _, _) = self else {
      return false;
    };
    let outer_meaning = match &**content {
      XM::Apply(Operator(op), _, ..) => match &**op {
        XM::Token(props, _) => props.meaning.as_deref().map(String::from),
        XM::Lexeme(name, _) => Some(name.clone()),
        _ => None,
      },
      _ => return false,
    };
    let Some(outer_m) = outer_meaning else { return false };
    let XM::Wrap(items, _, _) = &**presentation else {
      return false;
    };
    let is_delim = |x: &XM| match x {
      XM::Token(p, _) => matches!(p.role.as_deref(), Some("OPEN") | Some("CLOSE")),
      XM::Lexeme(n, _) => n.starts_with("OPEN:") || n.starts_with("CLOSE:"),
      _ => false,
    };
    let inner: Vec<&XM> = items.iter().filter(|x| !is_delim(x)).collect();
    if inner.len() != 1 {
      return false;
    }
    let XM::Dual(inner_content, ..) = inner[0] else {
      return false;
    };
    // Inner Dual's content should be an Apply whose meaning matches
    // outer (or is a closely-related "list"/"vector"/"formulae"
    // meaning — common when `interpret_delimited` lifts the list
    // wrapper above an inner Dual).
    if let XM::Apply(Operator(inner_op), _, ..) = &**inner_content {
      let inner_meaning = match &**inner_op {
        XM::Token(props, _) => props.meaning.as_deref(),
        XM::Lexeme(name, _) => Some(name.as_str()),
        _ => None,
      };
      // Outer-set wrapping inner-{set/list/vector/formulae} is the
      // shape we want to prune. The legacy never picks this.
      let inner_str = inner_meaning.unwrap_or("");
      if outer_m == inner_str
        || (matches!(outer_m.as_str(), "set" | "vector")
          && matches!(inner_str, "set" | "vector" | "list" | "formulae"))
      {
        return true;
      }
    }
    false
  }

  /// Does the root match `Dual(?, Apply(op_meaning, [...]))` where
  /// the Apply's first child (after following any `XM::Ref` through
  /// the presentation branch's idref index) is ALSO an `Apply` with
  /// the same `op_meaning`? Used to detect redundant
  /// `set@(set@(...))` and similar self-wrapping shapes.
  fn root_dual_is_redundant_self_wrap(&self) -> bool {
    let XM::Dual(content, _, _, _) = self else {
      return false;
    };
    let outer_meaning = match &**content {
      XM::Apply(Operator(op), args, ..) if args.trees().len() == 1 => {
        let m = match &**op {
          XM::Token(props, _) => props.meaning.as_deref().map(String::from),
          XM::Lexeme(name, _) => Some(name.clone()),
          _ => None,
        };
        m
      },
      _ => return false,
    };
    let Some(outer_m) = outer_meaning else {
      return false;
    };
    // Resolve the inner: dereference Apply's single child. If it's
    // a Ref, follow it through the idref index built across the
    // whole Dual.
    let mut index: HashMap<String, &XM> = HashMap::default();
    self.build_ref_index(&mut index);
    if let XM::Apply(_, args, _, _) = &**content {
      let first_arg = args.trees().first().copied();
      let resolved = match first_arg {
        Some(XM::Ref(props)) => {
          let key = props
            .idref
            .as_ref()
            .map(|c| c.to_string())
            .or_else(|| props.xmkey.as_ref().map(|c| c.to_string()));
          key.and_then(|k| index.get(&k).copied())
        },
        other => other,
      };
      if let Some(XM::Apply(Operator(inner_op), _, _, _)) = resolved {
        let inner_meaning = match &**inner_op {
          XM::Token(props, _) => props.meaning.as_deref(),
          XM::Lexeme(name, _) => Some(name.as_str()),
          _ => None,
        };
        return inner_meaning == Some(outer_m.as_str());
      }
    }
    false
  }

  /// Multi-tree pragma: prune parses whose math root has a
  /// "self-wrapping" Apply — `Apply(op, [Apply(op, ...)])` — when
  /// the forest also contains a non-self-wrapping alternative.
  ///
  /// Triggering shapes: `set@(set@(a, b, c))`, `vector@(vector@(...))`,
  /// etc. These arise from grammar ambiguity where both a direct
  /// rule (`fenced` producing the inner Apply) and an outer
  /// wrapping path (which then takes the inner Apply as a single
  /// argument) match. The legacy never selects the wrapping form
  /// — it's always the direct form. This pragma encodes that
  /// preference.
  pub fn prefer_non_self_wrapping_root(self) -> Self {
    match self {
      XM::Choices(trees) if trees.len() > 1 => {
        let is_redundant = |t: &XM| t.root_dual_is_redundant_self_wrap() || t.root_dual_has_redundant_inner_wrap();
        let has_non_wrapping = trees.iter().any(|t| !is_redundant(t));
        if !has_non_wrapping {
          return XM::Choices(trees);
        }
        let kept: Vec<XM> = trees.into_iter().filter(|t| !is_redundant(t)).collect();
        match kept.len() {
          0 => XM::Choices(Vec::new()),
          1 => kept.into_iter().next().unwrap(),
          _ => XM::Choices(kept),
        }
      },
      other => other,
    }
  }

  /// Multi-tree pragma: when the forest contains parses whose root
  /// is a named 2-arg interval (`open-interval`, `closed-interval`,
  /// or half-open variants) AND parses whose root is either
  /// `vector@(2)` or `delimited-XY@(...)` wrapping the same span,
  /// drop the non-interval parses.
  ///
  /// Rationale: for `(a, b)`, `[a, b]`, `(a, b]`, `[a, b)` — the
  /// math-parser grammar admits both:
  ///   - `interval_term → open-interval@(_, _)` / `closed-interval@(_, _)` (the named-interval interpretation)
  ///   - `fenced_factor → vector@(2)` or `delimited-XY@(...)` wrapper (the generic-bracket interpretation)
  /// Math convention reads these as intervals. Tree-iteration order in
  /// legacy picks the interval; under ASF the Cartesian-product
  /// order goes the other way.
  ///
  /// Scope is **deliberately narrow**: only applied at the root of
  /// the parse forest. Vectors / wrappers inside function arguments
  /// (like `f(a, b)` parsed as `Apply(f, [vector(a, b)])`) are
  /// unaffected because they're nested under an `Apply`, not at
  /// the root. 3+ element parens-fenced lists also unaffected —
  /// only `interval_term`'s 2-element shape matches.
  pub fn prefer_named_interval_at_root(self) -> Self {
    match self {
      XM::Choices(trees) if trees.len() > 1 => {
        let has_interval = trees.iter().any(|t| t.root_dual_apply_is_named_interval());
        if !has_interval {
          return XM::Choices(trees);
        }
        let kept: Vec<XM> = trees
          .into_iter()
          .filter(|t| {
            // Keep the named-interval parses; drop generic
            // `vector@(2)` and `delimited-XX@(...)` alternatives at
            // the root.
            t.root_dual_apply_is_named_interval()
              || !(t.root_dual_apply_meaning_is("vector", 2) || t.root_dual_apply_is_delimited_wrapper())
          })
          .collect();
        match kept.len() {
          0 => XM::Choices(Vec::new()),
          1 => kept.into_iter().next().unwrap(),
          _ => XM::Choices(kept),
        }
      },
      other => other,
    }
  }

  /// given a tree, return the base operator name, if any
  /// Simple text summary for debug logging (no DOM access needed)
  pub fn text_summary(&self) -> String {
    match self {
      XM::Lexeme(name, meta) => {
        let fenced = meta.fenced.as_deref().unwrap_or("");
        if fenced.is_empty() {
          name.to_string()
        } else {
          format!("{name}[{fenced}]")
        }
      },
      XM::Token(props, _) => {
        let role = props.role.as_deref().unwrap_or("?");
        let meaning = props.meaning.as_deref().unwrap_or("");
        if meaning.is_empty() {
          role.to_string()
        } else {
          format!("{role}:{meaning}")
        }
      },
      XM::Apply(op, args, ..) => {
        let op_str = op.0.text_summary();
        let args_str: Vec<String> = args.trees().iter().map(|a| a.text_summary()).collect();
        format!("{}@({})", op_str, args_str.join(", "))
      },
      XM::Dual(c, p, ..) => format!("Dual({}, {})", c.text_summary(), p.text_summary()),
      XM::Wrap(items, ..) => {
        let items_str: Vec<String> = items.iter().map(|i| i.text_summary()).collect();
        format!("Wrap({})", items_str.join(", "))
      },
      XM::Choices(ref trees) => format!("Choices({})", trees.len()),
      XM::Ref(idx) => format!("Ref({})", idx),
      XM::Arg(items) => {
        let items_str: Vec<String> = items.iter().map(|i| i.text_summary()).collect();
        format!("Arg({})", items_str.join(", "))
      },
    }
  }

  pub fn base_operator_name(&self) -> String {
    match self {
      XM::Lexeme(ref name, _) => name.to_string(),
      XM::Apply(ref op, ref args, ..) => {
        match &*op.0 {
          XM::Lexeme(ref name, _) if name == "unknown.subscript" => {
            let arg_base = args.0.first().unwrap().as_ref().unwrap().clone();
            format!("sub__{}", arg_base.base_operator_name())
          },
          XM::Lexeme(ref name, _) if name == "unknown.superscript" => {
            // TODO: Too much datastructure boilerplate with the unwrap incantation
            //       might be better to create some getter methods to explain the intent better
            //       this is meant to do "give me a clone of the first argument to this XM::Apply"
            //       which happens to be a base of a sub or super-script.
            let arg_base = args.0.first().unwrap().as_ref().unwrap();
            arg_base.base_operator_name()
          },
          XM::Lexeme(other, _) => other.to_string(),
          XM::Apply(sub_other, ..) => format!("reduced__{}", sub_other.0.base_operator_name()),
          _ => String::new(),
        }
      },
      _ => String::new(),
    }
  }

  pub fn get_baseline(&self) -> &Self {
    match self {
      XM::Lexeme(..) => self,
      XM::Token(..) => self,
      XM::Ref(_) => self,
      XM::Apply(ref op, ref args, ..) => {
        if let XM::Lexeme(name, _) = &*op.0 {
          if name == "unknown.subscript" || name == "unknown.superscript" {
            args.trees().first().unwrap().get_baseline()
          } else {
            self
          }
        } else {
          self
        }
      },
      // Dual/Wrap/Arg: fall back to treating self as the baseline. These
      // variants aren't typically exercised in get_baseline contexts today,
      // but returning self is safe — callers use the baseline to attach
      // scripts to, and these shapes don't break that attachment.
      XM::Dual(..) | XM::Wrap(..) | XM::Arg(_) => self,
      XM::Choices(args) => args.first().unwrap().get_baseline(),
    }
  }

  /// extract the constraints and pass them to the outer caller
  pub fn drain_constraints(&mut self) -> Vec<CurryConstraint> {
    // while we're at it, operators shouldn't have a curry_level set at this stage. Should they?!
    let meta = self.get_meta_mut();
    meta.curry_level = None;
    meta.curry_constraints.drain().collect()
  }
  /// Recursively remove constraints
  pub fn unconstrain_recursive(&mut self) {
    match self {
      XM::Lexeme(_, meta) => {
        meta.curry_constraints.drain();
      },
      XM::Token(_, meta) => {
        meta.curry_constraints.drain();
      },
      XM::Apply(Operator(op), args, _, meta) => {
        meta.curry_constraints.drain();
        op.unconstrain_recursive();
        args.unconstrain_recursive();
      },
      XM::Dual(content, pres, _props, meta) => {
        meta.curry_constraints.drain();
        content.unconstrain_recursive();
        pres.unconstrain_recursive();
      },
      XM::Wrap(content, _props, meta) => {
        meta.curry_constraints.drain();
        for c in content.iter_mut() {
          c.unconstrain_recursive();
        }
      },
      XM::Choices(args) | XM::Arg(args) => {
        for tree in args {
          tree.unconstrain_recursive();
        }
      },
      XM::Ref(_) => {},
    };
  }

  /// level-indented formatter akin to the std::fmt Display trait
  pub fn fmt_indented(&self, level: &[bool], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let indent = aux_generate_indent(level, false);
    match self {
      XM::Lexeme(name, meta) => writeln!(f, "{indent}{name} {meta}"),
      XM::Ref(idref) => writeln!(f, "{indent}Ref[{idref}]"),
      XM::Token(t, meta) => writeln!(f, "{indent}{t} {meta}"),
      XM::Apply(op, args, _, meta) => {
        if !meta.syntax_trace.is_empty() {
          let indent_base = aux_generate_indent(level, true);
          writeln!(f, "{indent_base}\n{indent_base}{meta}")?;
        }
        let mut arg_level: Vec<bool> = level.to_vec();
        arg_level.push(true);
        op.fmt_indented(level, f)?;
        args.fmt_indented(&arg_level, f)
      },
      XM::Dual(content, pres, ..) => {
        writeln!(f, "\n{indent}Dual")?;
        let mut arg_level: Vec<bool> = level.to_vec();
        arg_level.push(true);
        let mut peekable = vec![content, pres].into_iter().peekable();
        while let Some(arg) = peekable.next() {
          if peekable.peek().is_none() {
            arg_level.pop();
            arg_level.push(false);
            arg.fmt_indented(&arg_level, f)?
          } else {
            arg.fmt_indented(&arg_level, f)?
          }
        }
        writeln!(f)
      },
      XM::Wrap(content, ..) => {
        writeln!(f, "\n{indent}Wrap")?;
        let mut arg_level: Vec<bool> = level.to_vec();
        arg_level.push(true);
        let mut peekable = content.iter().peekable();
        while let Some(arg) = peekable.next() {
          if peekable.peek().is_none() {
            arg_level.pop();
            arg_level.push(false);
            arg.fmt_indented(&arg_level, f)?
          } else {
            arg.fmt_indented(&arg_level, f)?
          }
        }
        writeln!(f)
      },
      XM::Arg(args) => {
        writeln!(f, "\n{indent}Arg")?;
        let mut arg_level: Vec<bool> = level.to_vec();
        arg_level.push(true);
        let mut peekable = args.iter().peekable();
        while let Some(arg) = peekable.next() {
          if peekable.peek().is_none() {
            arg_level.pop();
            arg_level.push(false);
            arg.fmt_indented(&arg_level, f)?
          } else {
            arg.fmt_indented(&arg_level, f)?
          }
        }
        writeln!(f)
      },
      XM::Choices(args) => {
        writeln!(f, "\n{indent}Choices")?;
        let mut arg_level: Vec<bool> = level.to_vec();
        arg_level.push(true);
        let mut peekable = args.iter().peekable();
        while let Some(arg) = peekable.next() {
          if peekable.peek().is_none() {
            arg_level.pop();
            arg_level.push(false);
            arg.fmt_indented(&arg_level, f)?
          } else {
            arg.fmt_indented(&arg_level, f)?
          }
        }
        writeln!(f)
      },
    }
  }

  /// Rebuild a marpa-derived parse tree into an XMath XML tree
  pub fn into_xmath(
    self,
    owner: &mut Node,
    nodes: &mut [Node],
    document: &mut Document,
  ) -> Result<Node, Box<dyn Error + Send + Sync>> {
    // Grow the call stack on demand if we are running low. Heavily
    // nested math trees (XMApp(op, [XMApp(...)]) chains) recurse
    // through this function at depth proportional to expression
    // nesting; on Rust's 8 MB main-thread default stack that overflows
    // on grammar-ambiguous papers (sandbox 0711.4787, 0903.3289,
    // hep-th0101151, math0505371, math9204211, hep-ph9210253,
    // hep-ph9512208, astro-ph0612758 — 8 papers, SIGABRT with
    // `thread 'main' has overflowed its stack`). stacker::maybe_grow
    // allocates a new stack chunk on a separate thread when the
    // current frame's remaining space drops below the red zone.
    stacker::maybe_grow(64 * 1024, 4 * 1024 * 1024, || {
      self.into_xmath_inner(owner, nodes, document)
    })
  }

  fn into_xmath_inner(
    self,
    owner: &mut Node,
    nodes: &mut [Node],
    document: &mut Document,
  ) -> Result<Node, Box<dyn Error + Send + Sync>> {
    // Drain nested XM::Choices iteratively before the match. Each
    // Choices wrapper is a tail call into the first remaining choice
    // — preserving the per-layer Info log so the discard count
    // is visible identically to the prior recursive form.
    let mut tree = self;
    loop {
      match tree {
        XM::Choices(mut choices) => {
          Info!(
            "math_parser",
            "choices",
            format!(
              "to_xmath handler discarded {} parse choices.",
              choices.len() - 1
            )
          );
          tree = choices.remove(0);
        },
        other => {
          tree = other;
          break;
        },
      }
    }
    match tree {
      XM::Lexeme(content, _meta) => {
        let id = content
          .split(':')
          .next_back()
          .unwrap()
          .parse::<usize>()
          .unwrap()
          - 1;
        let atom_node = &mut nodes[id];
        atom_node.unbind();
        Ok(atom_node.clone())
      },
      XM::Token(props, _meta) => {
        // Transition the {font} property to the "_font" attribute.
        let has_explicit_font = props.font.is_some();
        let (content_opt, font, attrs) = props.into_attributes();
        let mut xmtok = document.open_element_at(owner, "ltx:XMTok", attrs, font)?;
        if let Some(ref content) = content_opt {
          if !content.is_empty() {
            xmtok.set_content(content)?;
          }
        }
        // Perl: Font->specialize($content) — for parser-created tokens without
        // explicit font, the ambient _font is specialized based on content.
        // Operators get default font (no italic), letters get italic.
        if !has_explicit_font {
          if let Some(font_hash) = xmtok.get_attribute("_font") {
            let content = content_opt.as_deref().unwrap_or("");
            if content.is_empty() {
              // Empty content: no font needed
              let _ = xmtok.remove_attribute("_font");
            } else if let Some(font) = document.decode_font(&font_hash) {
              let specialized = font.specialize(content);
              // Re-encode: store the specialized font and update _font hash
              document.set_node_font(&mut xmtok, &specialized)?;
            }
          }
        }
        document.close_element_at(&mut xmtok)?;
        Ok(xmtok)
      },
      XM::Apply(op, args, props, _meta) => {
        // let mut apply_node = Node::new("XMApp", None, document.get_document()).unwrap();
        // props.into_xmath(&mut apply_node,document)?;
        let (_, font, attrs) = props.into_attributes();
        let mut apply_node = document.open_element_at(owner, "ltx:XMApp", attrs, font)?;
        let mut op_node = op.0.into_xmath(&mut apply_node, nodes, document)?;

        add_child_guard_xmarg(&mut apply_node, &mut op_node)?;
        for arg in args.0.into_iter().flatten() {
          let mut arg_node = arg.into_xmath(&mut apply_node, nodes, document)?;
          add_child_guard_xmarg(&mut apply_node, &mut arg_node)?;
        }
        document.close_element_at(&mut apply_node)?;
        Ok(apply_node)
      },
      XM::Dual(content, pres, props, _meta) => {
        let (_, font, attrs) = props.into_attributes();
        let mut dual_node = document.open_element_at(owner, "ltx:XMDual", attrs, font)?;
        // Content branch first, then presentation (Perl convention)
        let mut content_node = content.into_xmath(&mut dual_node, nodes, document)?;
        add_child_guard_xmarg(&mut dual_node, &mut content_node)?;
        let mut pres_node = pres.into_xmath(&mut dual_node, nodes, document)?;
        add_child_guard_xmarg(&mut dual_node, &mut pres_node)?;
        document.close_element_at(&mut dual_node)?;
        Ok(dual_node)
      },
      XM::Wrap(content, props, _meta) => {
        let (_, font, attrs) = props.into_attributes();
        let mut wrap_node = document.open_element_at(owner, "ltx:XMWrap", attrs, font)?;
        for c in content.into_iter() {
          let mut content_node = c.into_xmath(&mut wrap_node, nodes, document)?;
          add_child_guard_xmarg(&mut wrap_node, &mut content_node)?;
        }
        document.close_element_at(&mut wrap_node)?;
        Ok(wrap_node)
      },
      XM::Ref(refprops) => {
        let mut ref_node = Node::new("XMRef", None, document.get_document()).unwrap();
        if let Some(id) = refprops.id {
          document.set_attribute(&mut ref_node, "idref", &id)?;
        }
        if let Some(xmkey) = refprops.xmkey {
          // Use _pxmkey for parser-generated keys (pxm prefix) to avoid
          // conflicting with base_xmath's \lx@dual resolver.
          // Regular _xmkey for all other refs.
          let attr_name = if xmkey.starts_with("pxm") {
            "_pxmkey"
          } else {
            "_xmkey"
          };
          document.set_attribute(&mut ref_node, attr_name, &xmkey)?;
        }
        Ok(ref_node)
      },
      XM::Arg(inner_list) => {
        let mut arg_node = Node::new("XMArg", None, document.get_document()).unwrap();
        for inner_item in inner_list {
          let mut inner_node = inner_item.into_xmath(&mut arg_node, nodes, document)?;
          add_child_guard_xmarg(&mut arg_node, &mut inner_node)?;
        }
        Ok(arg_node)
      },
      // XM::Choices was already drained iteratively above the match.
      XM::Choices(_) => unreachable!("Choices drained before match"),
    }
  }

  pub fn get_token_meaning(&self, nodes: &[Node]) -> Result<Option<Cow<'_, str>>, Box<dyn Error>> {
    let props = match self {
      XM::Token(props, _) => props,
      XM::Lexeme(lex, _) => {
        return match get_token_meaning(lookup_lex_node(lex, nodes)?) {
          Some(v) => Ok(Some(Cow::Owned(v))),
          None => Ok(None),
        };
      },
      XM::Apply(ref op, ..) => {
        // Compound operator (e.g. composed_bigop): get meaning from the operator
        return op.0.get_token_meaning(nodes);
      },
      _ => return Ok(None),
    };
    Ok(match props.meaning {
      Some(ref v) if !v.is_empty() => Some(Cow::Borrowed(v)),
      _ => match props.name {
        Some(ref v) if !v.is_empty() => Some(Cow::Borrowed(v)),
        _ => match props.content {
          Some(ref v) if !v.is_empty() => Some(Cow::Borrowed(v)),
          _ => match props.role {
            Some(ref v) if !v.is_empty() => Some(Cow::Borrowed(v)),
            _ => None,
          },
        },
      },
    })
  }

  pub fn realize_xmnode(&self, ctxt: &ActionContext) -> Result<Option<Node>, Box<dyn Error>> {
    match self {
      XM::Lexeme(lex, _) => {
        let lex_node = lookup_lex_node(lex, ctxt.nodes)?;
        Ok(Some(realize_xmnode(lex_node, ctxt.document).into_owned()))
      },
      XM::Ref(ref refprops) => {
        if let Some(node) = ctxt.document.lookup_id(refprops.id.as_ref().unwrap()) {
          Ok(Some(node.clone()))
        } else {
          // Perl: Error("expected", 'id', undef, "Cannot find a node with xml:id=...").
          // For now return None so upstream continues without a realized node;
          // a hard failure would abort the entire math parse.
          Ok(None)
        }
      },
      _ => Ok(None), // error?
    }
  }
}

impl Display for XM {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.fmt_indented(&Vec::new(), f) }
}

/// Unwrap any leftover XMArg guards from the markup.
/// This is done earlier in LaTeXML-classic, during the semantics phase.
/// With marpa, we can postpone reparenting to the very end, when the tree is requested.
fn add_child_guard_xmarg(
  receiver: &mut Node,
  incoming: &mut Node,
) -> Result<(), Box<dyn Error + Send + Sync>> {
  if incoming.get_name() == "XMArg" {
    let mut to_reparent = element_nodes(incoming);
    for incoming_child in to_reparent.iter_mut() {
      incoming_child.unlink();
    }
    for mut incoming_child in to_reparent {
      receiver.add_child(&mut incoming_child)?;
    }
  } else {
    receiver.add_child(incoming)?;
  }
  Ok(())
}

pub fn get_token_meaning(in_node: &Node) -> Option<String> {
  // let node = realize_xmnode(in_node, document);
  let node = in_node;
  match node.get_attribute("meaning") {
    Some(v) if !v.is_empty() => Some(v),
    _ => match node.get_attribute("name") {
      Some(v) if !v.is_empty() => Some(v),
      _ => {
        let content = node.get_content();
        if !content.is_empty() {
          Some(content)
        } else {
          match node.get_attribute("role") {
            Some(v) if !v.is_empty() => Some(v),
            _ => None,
          }
        }
      },
    },
  }
}
/// Looks up the node associated with a given lexeme,
/// via the node index held in the third colon-separated lexeme piece.
pub(crate) fn lookup_lex_node<'a>(
  lex: &'a str,
  nodes: &'a [Node],
) -> Result<&'a Node, Box<dyn Error>> {
  // Lex strings are produced internally by the grammar (`{name}:{value}:{idx}`
  // shape), so the suffix parses cleanly under normal operation. If we ever
  // hit a malformed lex — e.g. because a custom rule emitted a short form —
  // return an Err rather than panicking, matching the degrade-not-panic
  // policy used elsewhere on user-reachable paths.
  let idx_str = lex.split(':').next_back().unwrap_or("");
  let node_idx = idx_str
    .parse::<usize>()
    .map_err(|e| format!("malformed lex {lex:?}: {e}"))?
    .checked_sub(1)
    .ok_or_else(|| format!("lex idx 0 (expected 1-based) in {lex:?}"))?;
  nodes.get(node_idx).ok_or_else(|| {
    format!(
      "lex idx {node_idx} out of range (nodes.len={}) in {lex:?}",
      nodes.len()
    )
    .into()
  })
}

fn aux_generate_indent(level: &[bool], is_base: bool) -> String {
  let mut indent = String::new();
  let mut peekable = level.iter().peekable();
  while let Some(is_inked) = peekable.next() {
    indent += if peekable.peek().is_none() {
      if is_base {
        "   │  "
      } else if *is_inked {
        "   ├── "
      } else {
        "   └── "
      }
    } else if *is_inked {
      "   │"
    } else {
      "    "
    }
  }
  indent
}

impl From<&Node> for XM {
  fn from(n: &Node) -> Self {
    match n.get_name().as_str() {
      "XMTok" => XM::Token(XProps::from(n), Meta::default()),
      "XMApp" => {
        let mut children = element_nodes(n);
        let op = children.remove(0);
        let args: Args = children.iter().map(XM::from).collect::<Vec<_>>().into();
        XM::Apply((&op).into(), args, XProps::from(n), Meta::default())
      },
      "XMArg" => {
        let children = element_nodes(n);
        let inner_xm = children.iter().map(XM::from).collect::<Vec<_>>();
        XM::Arg(inner_xm)
      },
      "XMDual" => {
        let mut children = element_nodes(n);
        let content = children.remove(0);
        let presentation = if children.is_empty() {
          XM::Wrap(Vec::new(), XProps::default(), Meta::default())
        } else {
          XM::from(&children.remove(0))
        };
        XM::Dual(
          Box::new(XM::from(&content)),
          Box::new(presentation),
          XProps::from(n),
          Meta::default(),
        )
      },
      "XMRef" => XM::Ref(XProps::from(n)),
      "XMWrap" => {
        let children = element_nodes(n);
        let inner_xm = children.iter().map(XM::from).collect::<Vec<_>>();
        XM::Wrap(inner_xm, XProps::from(n), Meta::default())
      },
      // Fallback for unhandled node types — treat as token preserving attributes
      _other => XM::Token(XProps::from(n), Meta::default()),
    }
  }
}

impl From<&Node> for XProps {
  fn from(node: &Node) -> Self {
    let mut attrs = node.get_attributes();
    let str1 = node.get_content();
    let content = if str1.is_empty() {
      None
    } else {
      Some(Cow::Owned(str1))
    };
    let role = attrs.remove("role").map(Cow::Owned);
    let name = attrs.remove("name").map(Cow::Owned);
    let meaning = attrs.remove("meaning").map(Cow::Owned);
    let scriptpos = attrs.remove("scriptpos").map(Cow::Owned);
    let id = attrs.remove("id").map(Cow::Owned); // xml:id ?
    let idref = attrs.remove("idref").map(Cow::Owned);
    let fontref = attrs.remove("_font").map(Cow::Owned);

    let stretchy = attrs.remove("stretchy").map(Cow::Owned);
    let possible_function = attrs.remove("possibleFunction").map(Cow::Owned);
    let mathstyle = attrs.remove("mathstyle").map(Cow::Owned);
    let thickness = attrs.remove("thickness").map(Cow::Owned);
    let decl_id = attrs.remove("decl_id").map(Cow::Owned);
    let lpadding = attrs.remove("lpadding").map(Cow::Owned);
    let rpadding = attrs.remove("rpadding").map(Cow::Owned);
    XProps {
      content,
      role,
      name,
      meaning,
      scriptpos,
      id,
      idref,
      fontref,
      stretchy,
      possible_function,
      mathstyle,
      thickness,
      decl_id,
      lpadding,
      rpadding,
      ..Default::default()
    }
  }
}
