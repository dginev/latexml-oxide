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
use crate::pragmatics::ValidationPragmatics;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operator(pub Box<Tree>);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args(pub Vec<Option<Tree>>);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct XMTok {
  pub role: Option<Cow<'static, str>>,
  pub meaning: Option<Cow<'static, str>>,
  pub content: Option<Cow<'static, str>>,
  pub name: Option<Cow<'static, str>>,
  pub scriptpos: Option<Cow<'static, str>>,
  pub font: Option<Font>,
}
impl Display for XMTok {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // stub with Debug for now
    writeln!(f, "{self:?}")
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tree {
  Lexeme(String, Meta),
  Token(Box<XMTok>, Meta), // does this need Meta?
  Apply(Operator, Args, Meta),
  // Dual(Tree, Tree, Meta), // TODO
  Choices(Vec<Tree>),
}
impl From<XMTok> for Tree {
  fn from(t: XMTok) -> Self { Tree::Token(Box::new(t), Meta::default()) }
}

impl Display for Operator {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.fmt_indented(&Vec::new(), f) }
}
impl From<XMTok> for Operator {
  fn from(t: XMTok) -> Self { Operator(Box::new(Tree::Token(Box::new(t), Meta::default()))) }
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
    if let Tree::Lexeme(_, _) = &*self.0 {
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
          .unwrap_or(&Tree::Lexeme(String::from("missing_argument"), Meta::default()))
          .fmt_indented(level, f)?;
      } else {
        let mut last_level: Vec<bool> = level.to_vec();
        if !last_level.is_empty() {
          last_level.pop();
          last_level.push(false);
        }
        maybe_arg
          .as_ref()
          .unwrap_or(&Tree::Lexeme(String::from("missing_argument"), Meta::default()))
          .fmt_indented(&last_level, f)?;
      };
    }
    Ok(())
  }
  /// Obtain defined subtrees as a slice, e.g. for consistency validation
  pub fn trees(&self) -> Vec<&Tree> { self.0.iter().filter_map(|x| x.as_ref()).collect() }
  /// Obtain defined subtrees as a mutable slice, e.g. for consistency validation
  pub fn trees_mut(&mut self) -> Vec<&mut Tree> { self.0.iter_mut().filter_map(|x| x.as_mut()).collect() }

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

impl Tree {
  pub fn get_meta(&self) -> &Meta {
    match self {
      Tree::Lexeme(_, ref meta) => meta,
      Tree::Token(_, ref meta) => meta,
      Tree::Apply(_, _, ref meta) => meta,
      Tree::Choices(cs) => cs[0].get_meta(), // Should we return a none type instead?
    }
  }
  pub fn get_meta_mut(&mut self) -> &mut Meta {
    match self {
      Tree::Lexeme(_, ref mut meta) => meta,
      Tree::Token(_, ref mut meta) => meta,
      Tree::Apply(_, _, ref mut meta) => meta,
      Tree::Choices(cs) => cs[0].get_meta_mut(), // Should we return a none type instead?
    }
  }
  pub fn get_inner_meta(&self) -> Vec<&Meta> {
    match self {
      Tree::Lexeme(_atom, meta) => vec![meta],
      Tree::Apply(op, args, _) => vec![op.get_meta()]
        .into_iter()
        .chain(args.0.iter().filter(|arg| arg.is_some()).map(|arg| arg.as_ref().unwrap().get_meta()))
        .collect(),
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
      Tree::Lexeme(name, meta) => {
        let new_meta = meta.with_curry_atom(into, &name)?;
        Ok(Tree::Lexeme(name, new_meta))
      },
      Tree::Token(_t, _meta) => {
        unimplemented!()
      },
      Tree::Apply(mut op, mut args, meta) => {
        // First, if we have a specialize directive, execute it:
        match into.specialize {
          Some(ref directive) if directive == "embellish" => {
            // Atoms with embellishments should get their curry levels renamed
            // to avoid conflicts with the same atoms *without* the embellishments
            // as often this technique is used to generate new unique names.
            if args.0.len() <= 2 {
              if let Some(Tree::Lexeme(_, arg_meta)) = &mut args.0[0] {
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
          // println!("Tree: \n{}", Tree::Apply(op.clone(), args.clone(), new_meta.clone()));
          new_meta.validate()?;
        }
        let new_tree = Tree::Apply(op, args, new_meta);
        for pragma in pragmas {
          // expert pragmatics get to validate each new tree,
          // in order to prune wrong interpretations as early as possible
          pragma.validate(&new_tree)?;
        }
        Ok(new_tree)
      },
      Tree::Choices(_) => Err("can not specialize choices".into()),
    }
  }

  /// Prunes choices based on a validation pass leveraging a choice of pragmatics
  /// if the pruning arrives at no viable trees at all, the original tree is returned,
  /// hence the "soft" function name.
  /// These are executed at the end of the program, so need to be invoked recursively on each subtree
  pub fn soft_prune_choices(self, pragmatics: ValidationPragmatics) -> Self {
    match self {
      Tree::Choices(trees) => {
        let (consistent_trees, inconsistent_trees): (Vec<Tree>, Vec<Tree>) =
          trees.into_iter().partition(|tree| pragmatics.validate_recursive(tree).is_ok());
        match consistent_trees.len() {
          0 => Tree::Choices(inconsistent_trees),
          1 => consistent_trees.into_iter().next().unwrap(),
          _more => Tree::Choices(consistent_trees),
        }
      },
      other => other,
    }
  }

  /// given a tree, return the base operator name, if any
  pub fn base_operator_name(&self) -> String {
    match self {
      Tree::Lexeme(ref name, _) => name.to_string(),
      Tree::Apply(ref op, ref args, _) => {
        match &*op.0 {
          Tree::Lexeme(ref name, _) if name == "unknown.subscript" => {
            let arg_base = args.0.first().unwrap().as_ref().unwrap().clone();
            format!("sub__{}", arg_base.base_operator_name())
          },
          Tree::Lexeme(ref name, _) if name == "unknown.superscript" => {
            // TODO: Too much datastructure boilerplate with the unwrap incantation
            //       might be better to create some getter methods to explain the intent better
            //       this is meant to do "give me a clone of the first argument to this Tree::Apply"
            //       which happens to be a base of a sub or super-script.
            let arg_base = args.0.first().unwrap().as_ref().unwrap();
            arg_base.base_operator_name()
          },
          Tree::Lexeme(other, _) => other.to_string(),
          Tree::Apply(sub_other, _, _) => format!("reduced__{}", sub_other.0.base_operator_name()),
          _ => String::new(),
        }
      },
      _ => String::new(),
    }
  }

  pub fn get_baseline(&self) -> &Self {
    match self {
      Tree::Lexeme(_, _) => self,
      Tree::Token(_, _) => self,
      Tree::Apply(ref op, ref args, _) => {
        if let Tree::Lexeme(name, _) = &*op.0 {
          if name == "unknown.subscript" || name == "unknown.superscript" {
            args.trees().first().unwrap().get_baseline()
          } else {
            self
          }
        } else {
          self
        }
      },
      Tree::Choices(args) => args.first().unwrap().get_baseline(),
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
      Tree::Lexeme(_, meta) => {
        meta.curry_constraints.drain();
      },
      Tree::Token(_, meta) => {
        meta.curry_constraints.drain();
      },
      Tree::Apply(Operator(op), args, meta) => {
        meta.curry_constraints.drain();
        op.unconstrain_recursive();
        args.unconstrain_recursive();
      },
      Tree::Choices(args) => {
        for tree in args {
          tree.unconstrain_recursive();
        }
      },
    };
  }

  /// level-indented formatter akin to the std::fmt Display trait
  pub fn fmt_indented(&self, level: &[bool], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let indent = aux_generate_indent(level, false);
    match self {
      Tree::Apply(op, args, meta) => {
        if !meta.syntax_trace.is_empty() {
          let indent_base = aux_generate_indent(level, true);
          writeln!(f, "{indent_base}\n{indent_base}{meta}")?;
        }
        let mut arg_level: Vec<bool> = level.to_vec();
        arg_level.push(true);
        op.fmt_indented(level, f)?;
        args.fmt_indented(&arg_level, f)
      },
      Tree::Lexeme(name, meta) => writeln!(f, "{indent}{name} {meta}"),
      Tree::Token(t, meta) => writeln!(f, "{indent}{t} {meta}"),
      Tree::Choices(args) => {
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
      Tree::Lexeme(content, _meta) => {
        let atom_node = &mut nodes[content.split(':').last().unwrap().parse::<usize>().unwrap() - 1];
        atom_node.unbind();
        Ok(atom_node.clone())
      },
      Tree::Token(xmtok, _meta) => {
        let mut xmtok_node = Node::new("XMTok", None, document.get_document()).unwrap();
        if let Some(ref meaning) = xmtok.meaning {
          xmtok_node.set_attribute("meaning", meaning)?;
        }
        if let Some(ref name) = xmtok.name {
          xmtok_node.set_attribute("name", name)?;
        }
        if let Some(ref role) = xmtok.role {
          xmtok_node.set_attribute("role", role)?;
        }
        if let Some(ref scriptpos) = xmtok.scriptpos {
          xmtok_node.set_attribute("scriptpos", scriptpos)?;
        }
        if let Some(ref font) = xmtok.font {
          // TODO: how do we absorb the font attributes here? relative to current?
          if let Some(size) = font.size {
            xmtok_node.set_attribute("fontsize", &size.to_string())?;
          }
        }
        if let Some(ref content) = xmtok.content {
          xmtok_node.set_content(content)?;
        }
        Ok(xmtok_node)
      },
      Tree::Apply(op, args, _meta) => {
        // first execute all recursive calls on kids, and only *THEN*
        // create a new apply node, as our libxml wrapper has a weird bug
        // where two new Nodes of the same name are seen as the same.
        let mut apply_node = Node::new("XMApp", None, document.get_document()).unwrap();
        let mut op_node = op.0.to_xmath(nodes, document)?;
        self.to_xmath_add_child(&mut apply_node, &mut op_node)?;

        for arg in args.0.iter().flatten() {
          let mut arg_node = arg.to_xmath(nodes, document)?;
          self.to_xmath_add_child(&mut apply_node, &mut arg_node)?;
        }
        Ok(apply_node)
      },
      Tree::Choices(choices) => {
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
}

impl Display for Tree {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.fmt_indented(&Vec::new(), f) }
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
