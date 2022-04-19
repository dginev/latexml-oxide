use libxml::tree::Node as XMLNode;
use marpa::lexer::token::Token;
use marpa::stack::*;
use marpa::thin::Value;
use marpa::tree_builder::*;
use rtx_core::common::font::{self, Font};
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

pub use self::tree::{Args, Operator, Tree, XMTok};
use crate::pragmatics::ValidationPragmatics;
use rtx_core::raw_map;

mod curry;
mod from;
mod metadata;
mod tree;

use metadata::Meta;

pub type ActionClosure = Arc<dyn Fn(i32, Vec<Option<Tree>>, &[ValidationPragmatics], &[XMLNode]) -> Result<Option<Tree>, Box<dyn Error>>>;

#[derive(Default)]
pub struct Actions {
  dispatch: HashMap<i32, ActionClosure>,
}

impl Actions {
  pub fn register(&mut self, id: i32, closure: ActionClosure) { self.dispatch.insert(id, closure); }
  pub fn action_on(
    &self,
    id: i32,
    mut args: Vec<Option<Tree>>,
    pragmas: &[ValidationPragmatics],
    nodes: &[XMLNode],
  ) -> Result<Option<Tree>, Box<dyn Error>> {
    if let Some(action) = self.dispatch.get(&id) {
      action(id, args, pragmas, nodes)
    } else {
      match args.len() {
        0 => Ok(None),
        1 => Ok(args.remove(0)),
        more => {
          eprintln!("Only returning first of {:?} elements at rule id {:?} content: {:?}", more, id, args);
          Ok(args.remove(0))
        },
      }
    }
  }

  pub fn get_tree(&self, b: TreeBuilder, v: Value, pragmas: &[ValidationPragmatics], nodes: &[XMLNode]) -> Result<Option<Tree>, Box<dyn Error>> {
    let handle = proc_value(b, v);
    self.translate_node(&handle, pragmas, nodes)
  }

  pub fn translate_node<T: Token>(&self, n: &Handle<T>, pragmas: &[ValidationPragmatics], nodes: &[XMLNode]) -> Result<Option<Tree>, Box<dyn Error>> {
    match *n.borrow() {
      Node::Tree(ref rule, ref children) => {
        let mut translated_children = Vec::new();
        for child in children.iter() {
          translated_children.push(self.translate_node(child, pragmas, nodes)?);
        }
        self.action_on(*rule, translated_children, pragmas, nodes)
      },
      Node::Rule(ref rule, ref children) => {
        let mut translated_children = Vec::new();
        for child in children.iter() {
          translated_children.push(self.translate_node(child, pragmas, nodes)?);
        }
        self.action_on(*rule, translated_children, pragmas, nodes)
      },
      Node::Token(_ty, ref val) => {
        let token_str = ::std::str::from_utf8(val).unwrap_or("malformed-utf8");
        Ok(Some(
          Tree::Lexeme(token_str.to_owned(), Meta::default()).specialize(Meta::default(), pragmas)?,
        ))
      },
      Node::Leaf(ref tok) => Ok(Some(Tree::Lexeme(tok.to_string(), Meta::default()))),
      Node::Null(_) => {
        // e.g.* argument failed nothing, just skip.
        Ok(None)
        // Tree::Lexeme("null".into())
      },
    }
  }
}

/// standard infix application of an operator
pub fn infix_apply(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics], _: &[XMLNode]) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => arg1, infixop, arg2);
  let apply_tree = Tree::Apply(infixop.into(), Args(vec![arg1, arg2]), Meta::default());
  Ok(Some(apply_tree))
}

/// application with trailing elision, as in `x \cdot y \cdot\cdot\cdot`
pub fn infix_apply_and_elide(
  rule_id: i32,
  mut args: Vec<Option<Tree>>,
  p: &[ValidationPragmatics],
  nodes: &[XMLNode],
) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => arg1, infixop, arg2, elision);
  // check if "left" is already an application of infix op, in which case we can do n-ary apply.
  if let Some(Tree::Apply(new_op, mut new_args, meta)) = infix_apply_nary(rule_id, vec![arg1, infixop, arg2], p, nodes)? {
    new_args.0.push(elision);
    Ok(Some(Tree::Apply(new_op, new_args, meta)))
  } else {
    Ok(None)
  }
}

// infix_apply in the base case,
// but when chained, using the flat "multirelation" behavior of latexml
pub fn infix_relation(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics], _: &[XMLNode]) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => left, infixop, right);
  // if left has a "multirelation" already, add right in.
  // if left applies a relation, flatten it out to infix form.
  // base case - build a simple infix apply
  let mut left = left;
  match left {
    Some(Tree::Apply(ref op, ref mut left_args, ref _left_meta)) => {
      if let Tree::Token(ref tok, _) = *op.0 {
        if tok.meaning == Some(Cow::Borrowed("multirelation")) {
          left_args.0.push(infixop);
          left_args.0.push(right);
          Ok(left)
        } else {
          Ok(Some(Tree::Apply(infixop.into(), Args(vec![left, right]), Meta::default())))
        }
      } else if let Tree::Lexeme(ref lex, ref _left_meta) = *op.0 {
        if lex.split(':').next().unwrap().contains("RELOP") {
          // first multirelation need is here.
          let multirel_tok = XMTok {
            meaning: Some(Cow::Borrowed("multirelation")),
            ..XMTok::default()
          };
          let mut drained_left_args = left_args.0.drain(..);
          let left_1 = drained_left_args.next().unwrap();
          let left_2 = drained_left_args.next().unwrap();
          let moved_op = (*op.0).clone();
          Ok(Some(Tree::Apply(
            multirel_tok.into(),
            Args(vec![left_1, Some(moved_op), left_2, infixop, right]),
            Meta::default(),
          )))
        } else {
          Ok(Some(Tree::Apply(infixop.into(), Args(vec![left, right]), Meta::default())))
        }
      } else {
        Ok(Some(Tree::Apply(infixop.into(), Args(vec![left, right]), Meta::default())))
      }
    },
    _ => Ok(Some(Tree::Apply(infixop.into(), Args(vec![left, right]), Meta::default()))),
  }
}

pub fn infix_apply_nary(
  _rule_id: i32,
  mut args: Vec<Option<Tree>>,
  _: &[ValidationPragmatics],
  _: &[XMLNode],
) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => left, infixop, right);
  let mut left = left;
  // left-to-right associative:
  // 1. if "left" is already an application of "infixop",
  // 2. then tuck "right" inside it.
  if let Some(Tree::Apply(ref left_op, ref mut left_args, ref _m)) = left {
    if let Tree::Lexeme(left_op_lex, _xmeta) = &*left_op.0 {
      if let Some(Tree::Lexeme(ref infix_op_lex, _)) = infixop {
        let left_op_pieces: Vec<_> = left_op_lex.split(':').collect();
        let infix_op_pieces: Vec<_> = infix_op_lex.split(':').collect();
        if left_op_pieces.len() == 3
          && infix_op_pieces.len() == 3
          && left_op_pieces[0] == infix_op_pieces[0]
          && left_op_pieces[1] == infix_op_pieces[1]
        {
          left_args.0.push(right);
          return Ok(left);
        }
      }
    }
  }
  // base case: new apply tree
  let apply_tree = Tree::Apply(infixop.into(), Args(vec![left, right]), Meta::default());
  Ok(Some(apply_tree))
}

pub fn prefix_apply(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics], _: &[XMLNode]) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => prefixop, arg1);
  Ok(Some(Tree::Apply(prefixop.into(), Args(vec![arg1]), Meta::default())))
}
pub fn postfix_apply(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics], _: &[XMLNode]) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => arg, op);
  Ok(Some(Tree::Apply(op.into(), Args(vec![arg]), Meta::default())))
}

pub fn circumfix_fenced(
  _rule_id: i32,
  mut args: Vec<Option<Tree>>,
  _: &[ValidationPragmatics],
  _: &[XMLNode],
) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => _open, arg, _close);
  Ok(arg)
}

/// remove start_/end_ wrappers
pub fn faux_wrap(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics], _: &[XMLNode]) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => _faux1, content, _faux2);
  Ok(content)
}

pub fn postfix_script(
  _rule_id: i32,
  mut args: Vec<Option<Tree>>,
  _: &[ValidationPragmatics],
  nodes: &[XMLNode],
) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => base, op);
  new_script(op.unwrap(), base, nodes)
}

pub fn prefix_script(
  _rule_id: i32,
  mut args: Vec<Option<Tree>>,
  _: &[ValidationPragmatics],
  nodes: &[XMLNode],
) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => op, base);
  new_script(op.unwrap(), base, nodes)
}

/// This is loosely in the lines of MathParser::NewScript, but taking into account
/// the realities of our new data structures.
pub fn new_script(script: Tree, base: Option<Tree>, nodes: &[XMLNode]) -> Result<Option<Tree>, Box<dyn Error>> {
  if let Tree::Lexeme(ref lex, _) = script {
    let node = lookup_lex_node(lex.as_str(), nodes)?;
    let node_role = node.get_attribute("role").unwrap();
    let is_float = node_role.starts_with("FLOAT");
    let is_super = node_role.ends_with("SUPERSCRIPT");
    let role = Cow::Borrowed(if is_super { "SUPERSCRIPTOP" } else { "SUBSCRIPTOP" });
    let scriptpos = Cow::Borrowed(if is_float { "pre1" } else { "post1" });
    let op = new_token(None, None, raw_map!("role"=>role, "scriptpos"=>scriptpos)); // TODO: scriptpos => "$x$l"
    let script_arg = obtain_arg(script, 0);
    Ok(Some(Tree::Apply(op.into(), Args(vec![base, script_arg]), Meta::default())))
  } else {
    panic!("new_script is meant to be called on script terminals (e.g. POSTSUBSCRIPT/POSTSUPERSCRIPT)");
  }
}

/// Looks up the node associated with a given lexeme,
/// via the node index held in the third colon-separated lexeme piece.
pub fn lookup_lex_node<'a, 'b>(lex: &'a str, nodes: &'b [XMLNode]) -> Result<&'b XMLNode, Box<dyn Error>> {
  let node_idx = lex.split(':').last().unwrap().parse::<usize>()?;
  Ok(nodes.get(node_idx).unwrap())
}

// Get n-th arg of an XMApp.
// However, this is really only used to get the script out of a sub/super script
pub fn obtain_arg(tree: Tree, n: usize) -> Option<Tree> {
  match &tree {
    Tree::Lexeme(_, _) => Some(tree),
    Tree::Apply(_, ref args, _) => match args.0.get(n) {
      Some(t) => t.clone(),
      None => None,
    },
    _ => unimplemented!(),
  }
}

pub fn invisible_times(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics], _: &[XMLNode]) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => left, right);
  let mut left = left;
  // left-to-right associative -- if "left" is already a "times", tuck "right" in:
  if let Some(Tree::Apply(ref op, ref mut left_args, ref _m)) = left {
    if let Tree::Token(xop, _xmeta) = &*op.0 {
      match xop.meaning {
        Some(ref name) if name == "times" => {
          left_args.0.push(right);
          return Ok(left);
        },
        _ => {},
      }
    }
  }
  // otherwise create a new one:
  let times = XMTok {
    meaning: Some(Cow::Borrowed("times")),
    role: Some(Cow::Borrowed("MULOP")),
    content: Some(Cow::Borrowed("\u{2062}")),
    name: None,
    scriptpos: None,
    font: None,
  };
  Ok(Some(Tree::Apply(times.into(), Args(vec![left, right]), Meta::default())))
}

pub fn new_token(
  meaning: Option<Cow<'static, str>>,
  content: Option<Cow<'static, str>>,
  mut props: HashMap<&'static str, Cow<'static, str>>,
) -> XMTok {
  let role = props.remove("role");
  let name = props.remove("name");
  let scriptpos = props.remove("scriptpos");
  // TODO:
  let font = match props.remove("font") {
    Some(_fnt) => unimplemented!(),
    None => {
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
  XMTok {
    meaning,
    content,
    role,
    name,
    scriptpos,
    font: Some(font),
  }
}

// Some handy shorthands.
// pub fn absent() { new_token(Some(Cow::Borrowed("absent")), None, HashMap::default()); }

// sub InvisibleComma {
// return New(undef, "\x{2063}", role => 'PUNCT', font =>
// LaTeXML::Common::Font->new()); }
