use libxml::tree::Node;
use rtx_core::common::font::Font;
use rtx_core::common::xml::element_nodes;
use rtx_core::document::Document;
use rtx_core::Info;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::fmt::Display;

use super::curry::{CurryConstraint, CurryConstraints, CurryTerm};
use super::metadata::Meta;
use super::ActionContext;
use crate::parser::realize_xmnode;
use crate::pragmatics::ValidationPragmatics;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operator(pub Box<XM>);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args(pub Vec<Option<XM>>);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// The allowed properties on any newly created XMath nodes
/// during grammatical processing
pub struct XProps {
  /// text content of the node
  pub content: Option<Cow<'static, str>>,
  /// grammatical role used during math parsing
  pub role: Option<Cow<'static, str>>,
  /// conceptual meaning of a construct, used in disambiguation and Content output
  pub meaning: Option<Cow<'static, str>>,
  /// similar to `meaning`, but more fixed, usually associated with constants
  pub name: Option<Cow<'static, str>>,
  /// script position w.r.t to baseline
  pub scriptpos: Option<Cow<'static, str>>,
  /// a unique identifier, in the `xml:id` sense
  pub id: Option<Cow<'static, str>>,
  /// a pointer to a different node, usually for `XMRef`
  pub idref: Option<Cow<'static, str>>,
  /// an optional subtree-specific Font
  pub font: Option<Font>,
  /// usually associated with the internal `_font` attribute references
  pub fontref: Option<Cow<'static, str>>,
}
impl Display for XProps {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // stub with Debug for now
    writeln!(f, "{self:?}")
  }
}

impl XProps {
  pub fn to_xmath(&self, node: &mut Node) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(ref content) = self.content {
      node.set_content(content)?;
    }
    if let Some(ref role) = self.role {
      node.set_attribute("role", role)?;
    }
    if let Some(ref meaning) = self.meaning {
      node.set_attribute("meaning", meaning)?;
    }
    if let Some(ref name) = self.name {
      node.set_attribute("name", name)?;
    }
    if let Some(ref scriptpos) = self.scriptpos {
      node.set_attribute("scriptpos", scriptpos)?;
    }
    if let Some(ref id) = self.id {
      node.set_attribute("xml:id", id)?; // TODO: double-check
    }
    if let Some(ref idref) = self.idref {
      node.set_attribute("idref", idref)?;
    }
    if let Some(ref fontref) = self.fontref {
      node.set_attribute("_font", fontref)?;
    }
    if let Some(ref font) = self.font {
      // TODO: how do we absorb the font attributes here? relative to current?
      if let Some(size) = font.size {
        node.set_attribute("fontsize", &size.to_string())?;
      }
    }
    Ok(())
  }
}

/// The math parsing process can manipulate a variety of trees,
/// finally serialized via the XMath schema of LaTeXML.
///
/// The main structural variants are associated with
/// a "parsing state", via an attached `Meta` object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XM {
  Lexeme(String, Meta),
  Token(XProps, Meta), // does this need Meta?
  Apply(Operator, Args, XProps, Meta),
  Dual(Box<XM>, Box<XM>, XProps, Meta),
  Ref(String),
  Wrap(Vec<XM>, XProps, Meta),
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
    let mut meta = self.0.get_meta_mut();
    meta.curry_level = None;
    meta.curry_constraints.drain().collect()
  }

  pub fn unconstrain_recursive(&mut self) { self.0.unconstrain_recursive(); }

  fn fmt_indented(&self, level: &[bool], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let XM::Lexeme(_, _) = &*self.0 {
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
          .unwrap_or(&XM::Lexeme(String::from("missing_argument"), Meta::default()))
          .fmt_indented(level, f)?;
      } else {
        let mut last_level: Vec<bool> = level.to_vec();
        if !last_level.is_empty() {
          last_level.pop();
          last_level.push(false);
        }
        maybe_arg
          .as_ref()
          .unwrap_or(&XM::Lexeme(String::from("missing_argument"), Meta::default()))
          .fmt_indented(&last_level, f)?;
      };
    }
    Ok(())
  }
  /// Obtain defined subtrees as a slice, e.g. for consistency validation
  pub fn trees(&self) -> Vec<&XM> { self.0.iter().filter_map(|x| x.as_ref()).collect() }
  /// Obtain defined subtrees as a mutable slice, e.g. for consistency validation
  pub fn trees_mut(&mut self) -> Vec<&mut XM> { self.0.iter_mut().filter_map(|x| x.as_mut()).collect() }

  /// borrow the constraints and pass them to the outer caller
  pub fn get_constraints(&self) -> Vec<&CurryConstraint> { self.trees().into_iter().flat_map(|t| t.get_meta().curry_constraints.iter()).collect() }
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
      XM::Ref(_) => unimplemented!(),
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
      XM::Ref(_) => unimplemented!(),
    }
  }
  pub fn get_inner_meta(&self) -> Vec<&Meta> {
    match self {
      XM::Lexeme(_atom, meta) => vec![meta],
      XM::Apply(op, args, _, _) => vec![op.get_meta()]
        .into_iter()
        .chain(args.0.iter().filter(|arg| arg.is_some()).map(|arg| arg.as_ref().unwrap().get_meta()))
        .collect(),
      XM::Dual(content, presentation, _, _) => vec![content.get_meta(), presentation.get_meta()],
      _ => Vec::new(),
    }
  }
  /// Specialize a tree with the given Meta object,
  /// also verifying the tree's inner consistency.
  ///
  /// Whenever a contradiction/inconsistency is detected, we return an error
  /// This method should always be called on tree construction, as it also manages the various curry
  /// constraints, keeping the resolution local / fast.
  pub fn specialize(self, mut into: Meta, pragmas: &[ValidationPragmatics]) -> Result<Self, Box<dyn Error>> {
    match self {
      XM::Lexeme(name, meta) => {
        let new_meta = meta.with_curry_atom(into, &name)?;
        Ok(XM::Lexeme(name, new_meta))
      },
      XM::Token(_t, _meta) => {
        unimplemented!()
      },
      XM::Ref(_) => Ok(self),
      XM::Apply(mut op, mut args, props, meta) => {
        // First, if we have a specialize directive, execute it:
        match into.specialize {
          Some(ref directive) if directive == "embellish" => {
            // Atoms with embellishments should get their curry levels renamed
            // to avoid conflicts with the same atoms *without* the embellishments
            // as often this technique is used to generate new unique names.
            if args.0.len() <= 2 {
              if let Some(XM::Lexeme(_, arg_meta)) = &mut args.0[0] {
                if let Some(CurryTerm::Var(ref mut curry_var)) = arg_meta.curry_level {
                  let mut base_op = op.0.base_operator_name();
                  // fish out a local name to use as an embellishment
                  if base_op.contains(':') {
                    base_op = base_op.split(':').last().unwrap().to_owned();
                  } else if base_op.contains('.') {
                    base_op = base_op.split('.').last().unwrap().to_owned();
                  }
                  if base_op.is_empty() {
                    base_op = String::from("embellished");
                  }
                  curry_var.push('-');
                  curry_var.push_str(&base_op);
                }
                into.curry_level = arg_meta.curry_level.clone();
              }
            }
          },
          _ => {},
        }
        // Next, we validate the constraints.
        let initial_constraint_count = meta.curry_constraints.len();
        let mut new_meta = meta;
        // SPECIAL CASE (of course): if we are using an "unconstrained" directive, no need to do any of this
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
              CurryTerm::Add(_, _) | CurryTerm::Sub(_, _) => {
                new_meta
                  .curry_constraints
                  .insert(CurryConstraint((expr.clone(), Ordering::Greater, CurryTerm::Literal(0))));
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
      XM::Dual(_, _, _, _) => unimplemented!(),
      XM::Wrap(_, _, _) => unimplemented!(),
      XM::Choices(_) => Err("can not specialize choices".into()),
    }
  }

  /// Prunes choices based on a validation pass leveraging a choice of pragmatics
  /// if the pruning arrives at no viable trees at all, the original tree is returned,
  /// hence the "soft" function name.
  /// These are executed at the end of the program, so need to be invoked recursively on each subtree
  pub fn soft_prune_choices(self, pragmatics: ValidationPragmatics) -> Self {
    match self {
      XM::Choices(trees) => {
        let (consistent_trees, inconsistent_trees): (Vec<XM>, Vec<XM>) =
          trees.into_iter().partition(|tree| pragmatics.validate_recursive(tree).is_ok());
        match consistent_trees.len() {
          0 => XM::Choices(inconsistent_trees),
          1 => consistent_trees.into_iter().next().unwrap(),
          _more => XM::Choices(consistent_trees),
        }
      },
      other => other,
    }
  }

  /// given a tree, return the base operator name, if any
  pub fn base_operator_name(&self) -> String {
    match self {
      XM::Lexeme(ref name, _) => name.to_string(),
      XM::Apply(ref op, ref args, _, _) => {
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
          XM::Apply(sub_other, _, _, _) => format!("reduced__{}", sub_other.0.base_operator_name()),
          _ => String::new(),
        }
      },
      _ => String::new(),
    }
  }

  pub fn get_baseline(&self) -> &Self {
    match self {
      XM::Lexeme(_, _) => self,
      XM::Token(_, _) => self,
      XM::Ref(_) => self,
      XM::Apply(ref op, ref args, _, _) => {
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
      XM::Dual(_, _, _, _) => unimplemented!(),
      XM::Wrap(_inner, _, _) => unimplemented!(),
      XM::Choices(args) => args.first().unwrap().get_baseline(),
    }
  }

  /// extract the constraints and pass them to the outer caller
  pub fn drain_constraints(&mut self) -> Vec<CurryConstraint> {
    // while we're at it, operators shouldn't have a curry_level set at this stage. Should they?!
    let mut meta = self.get_meta_mut();
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
      XM::Choices(args) => {
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
      XM::Dual(content, pres, _, _) => {
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
      XM::Wrap(content, _, _) => {
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
  pub fn to_xmath(&self, nodes: &mut [Node], document: &mut Document) -> Result<Node, Box<dyn Error + Send + Sync>> {
    match self {
      XM::Lexeme(content, _meta) => {
        let id = content.split(':').last().unwrap().parse::<usize>().unwrap() - 1;
        let atom_node = &mut nodes[id];
        atom_node.unbind();
        Ok(atom_node.clone())
      },
      XM::Token(props, _meta) => {
        let mut xmtok = Node::new("XMTok", None, document.get_document()).unwrap();
        props.to_xmath(&mut xmtok)?;
        Ok(xmtok)
      },
      XM::Apply(op, args, props, _meta) => {
        let mut apply_node = Node::new("XMApp", None, document.get_document()).unwrap();
        props.to_xmath(&mut apply_node)?;
        let mut op_node = op.0.to_xmath(nodes, document)?;
        self.to_xmath_add_child(&mut apply_node, &mut op_node)?;

        for arg in args.0.iter().flatten() {
          let mut arg_node = arg.to_xmath(nodes, document)?;
          self.to_xmath_add_child(&mut apply_node, &mut arg_node)?;
        }
        Ok(apply_node)
      },
      XM::Dual(content, pres, props, _meta) => {
        let mut dual_node = Node::new("XMDual", None, document.get_document()).unwrap();
        props.to_xmath(&mut dual_node)?;

        let mut content_node = content.to_xmath(nodes, document)?;
        self.to_xmath_add_child(&mut dual_node, &mut content_node)?;
        let mut pres_node = pres.to_xmath(nodes, document)?;
        self.to_xmath_add_child(&mut dual_node, &mut pres_node)?;
        Ok(dual_node)
      },
      XM::Wrap(content, props, _meta) => {
        let mut wrap_node = Node::new("XMWrap", None, document.get_document()).unwrap();
        props.to_xmath(&mut wrap_node)?;

        for c in content.iter() {
          let mut content_node = c.to_xmath(nodes, document)?;
          self.to_xmath_add_child(&mut wrap_node, &mut content_node)?;
        }
        Ok(wrap_node)
      },
      XM::Ref(idref) => {
        let mut ref_node = Node::new("XMRef", None, document.get_document()).unwrap();
        document.set_attribute(&mut ref_node, "idref", idref)?;
        Ok(ref_node)
      },
      XM::Choices(choices) => {
        Info!("to_xmath handler discarded {} parse choices.", choices.len() - 1);
        choices[0].to_xmath(nodes, document)
      },
    }
  }

  /// Unwrap any leftover XMArg guards from the markup.
  /// This is done earlier in LaTeXML-classic, during the semantics phase.
  /// With marpa, we can postpone reparenting to the very end, when the tree is requested.
  pub fn to_xmath_add_child(&self, receiver: &mut Node, incoming: &mut Node) -> Result<(), Box<dyn Error + Send + Sync>> {
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

  pub fn get_token_meaning(&self, nodes: &[Node]) -> Result<Option<Cow<str>>, Box<dyn Error>> {
    let props = match self {
      XM::Token(props, _) => props,
      XM::Lexeme(lex, _) => {
        return match get_token_meaning(lookup_lex_node(lex, nodes)?) {
          Some(v) => Ok(Some(Cow::Owned(v))),
          None => Ok(None),
        }
      },
      other => {
        dbg!(other);
        unimplemented!()
      },
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
        Ok(Some(realize_xmnode(lex_node, ctxt.document, ctxt.state).into_owned()))
      },
      XM::Ref(ref idref) => {
        if let Some(node) = ctxt.document.lookup_id(idref) {
          Ok(Some(node.clone()))
        } else {
          // TODO
          //   Error("expected", 'id', undef, "Cannot find a node with xml:id='$idref'",
          //   ($LaTeXML::MathParser::IDREFS{$idref}
          //     ? "Previously bound to " . ToString($LaTeXML::MathParser::IDREFS{$idref})
          //     : ()));
          // return ['ltx:ERROR', {}, "Missing XMRef idref=$idref"]; } }
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
pub(crate) fn lookup_lex_node<'a>(lex: &'a str, nodes: &'a [Node]) -> Result<&'a Node, Box<dyn Error>> {
  let node_idx = lex.split(':').last().unwrap().parse::<usize>()? - 1;
  let node = nodes
    .get(node_idx)
    .expect("lex node lookup is grammar-internal and should always have an accurate index.");
  Ok(node)
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
      "XMRef" => XM::Token(XProps::from(n), Meta::default()),
      "XMDual" => XM::Token(XProps::from(n), Meta::default()),
      "XMWrap" => XM::Token(XProps::from(n), Meta::default()),
      "XMArg" => XM::Token(XProps::from(n), Meta::default()),
      _ => unimplemented!(),
    }
  }
}

impl From<&Node> for XProps {
  fn from(node: &Node) -> Self {
    let mut attrs = node.get_attributes();
    let str1 = node.get_content();
    let content = if str1.is_empty() { None } else { Some(Cow::Owned(str1)) };
    let role = attrs.remove("role").map(Cow::Owned);
    let name = attrs.remove("name").map(Cow::Owned);
    let meaning = attrs.remove("meaning").map(Cow::Owned);
    let scriptpos = attrs.remove("scriptpos").map(Cow::Owned);
    let id = attrs.remove("id").map(Cow::Owned); // xml:id ?
    let idref = attrs.remove("idref").map(Cow::Owned);
    let fontref = attrs.remove("_font").map(Cow::Owned);

    XProps {
      content,
      role,
      name,
      meaning,
      scriptpos,
      id,
      idref,
      fontref,
      font: None,
    }
  }
}
