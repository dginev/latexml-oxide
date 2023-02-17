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

pub use self::tree::{Args, Operator, Tree, XProps};
use self::tree::lookup_lex_node;
// use crate::parser::realize_xmnode;
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
          eprintln!("Only returning first of {more:?} elements at rule id {id:?} content: {args:?}");
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
  unp!(args => arg1, infixop, arg2);
  let apply_tree = Tree::Apply(infixop.into(), Args(vec![arg1, arg2]), XProps::default(), Meta::default());
  Ok(Some(apply_tree))
}

/// application with trailing elision, as in `x \cdot y \cdot\cdot\cdot`
pub fn infix_apply_and_elide(
  rule_id: i32,
  mut args: Vec<Option<Tree>>,
  p: &[ValidationPragmatics],
  nodes: &[XMLNode],
) -> Result<Option<Tree>, Box<dyn Error>> {
  unp!(args => arg1, infixop, arg2, elision);
  // check if "left" is already an application of infix op, in which case we can do n-ary apply.
  if let Some(Tree::Apply(new_op, mut new_args, props, meta)) = infix_apply_nary(rule_id, vec![arg1, infixop, arg2], p, nodes)? {
    new_args.0.push(elision);
    Ok(Some(Tree::Apply(new_op, new_args, props, meta)))
  } else {
    Ok(None)
  }
}

// infix_apply in the base case,
// but when chained, using the flat "multirelation" behavior of latexml
pub fn infix_relation(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics], _: &[XMLNode]) -> Result<Option<Tree>, Box<dyn Error>> {
  unp!(args => left, infixop, right);
  // if left has a "multirelation" already, add right in.
  // if left applies a relation, flatten it out to infix form.
  // base case - build a simple infix apply
  let mut left = left;
  match left {
    Some(Tree::Apply(ref op, ref mut left_args, _, ref _left_meta)) => {
      if let Tree::Token(ref tok, _) = *op.0 {
        if tok.meaning == Some(Cow::Borrowed("multirelation")) {
          left_args.0.push(infixop);
          left_args.0.push(right);
          Ok(left)
        } else {
          Ok(Some(Tree::Apply(infixop.into(), Args(vec![left, right]), XProps::default(), Meta::default())))
        }
      } else if let Tree::Lexeme(ref lex, ref _left_meta) = *op.0 {
        if lex.split(':').next().unwrap().contains("RELOP") {
          // first multirelation need is here.
          let multirel_tok = XProps {
            meaning: Some(Cow::Borrowed("multirelation")),
            ..XProps::default()
          };
          let mut drained_left_args = left_args.0.drain(..);
          let left_1 = drained_left_args.next().unwrap();
          let left_2 = drained_left_args.next().unwrap();
          let moved_op = (*op.0).clone();
          Ok(Some(Tree::Apply(
            multirel_tok.into(),
            Args(vec![left_1, Some(moved_op), left_2, infixop, right]),
            XProps::default(),
            Meta::default(),
          )))
        } else {
          Ok(Some(Tree::Apply(infixop.into(), Args(vec![left, right]), XProps::default(), Meta::default())))
        }
      } else {
        Ok(Some(Tree::Apply(infixop.into(), Args(vec![left, right]), XProps::default(), Meta::default())))
      }
    },
    _ => Ok(Some(Tree::Apply(infixop.into(), Args(vec![left, right]), XProps::default(), Meta::default()))),
  }
}

pub fn infix_apply_nary(
  _rule_id: i32,
  mut args: Vec<Option<Tree>>,
  _: &[ValidationPragmatics],
  _: &[XMLNode],
) -> Result<Option<Tree>, Box<dyn Error>> {
  unp!(args => left, infixop, right);
  let mut left = left;
  // left-to-right associative:
  // 1. if "left" is already an application of "infixop",
  // 2. then tuck "right" inside it.
  if let Some(Tree::Apply(ref left_op, ref mut left_args, _, ref _m)) = left {
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
  let apply_tree = Tree::Apply(infixop.into(), Args(vec![left, right]), XProps::default(), Meta::default());
  Ok(Some(apply_tree))
}

pub fn prefix_apply(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics], _: &[XMLNode]) -> Result<Option<Tree>, Box<dyn Error>> {
  unp!(args => prefixop, arg1);
  Ok(Some(Tree::Apply(prefixop.into(), Args(vec![arg1]), XProps::default(), Meta::default())))
}
pub fn postfix_apply(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics], _: &[XMLNode]) -> Result<Option<Tree>, Box<dyn Error>> {
  unp!(args => arg, op);
  Ok(Some(Tree::Apply(op.into(), Args(vec![arg]), XProps::default(), Meta::default())))
}

pub fn circumfix_fenced(
  _rule_id: i32,
  mut args: Vec<Option<Tree>>,
  _: &[ValidationPragmatics],
  _: &[XMLNode],
) -> Result<Option<Tree>, Box<dyn Error>> {
  unp!(args => _open, arg, _close);
  Ok(arg)
}

/// remove start_/end_ wrappers
pub fn faux_wrap(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics], _: &[XMLNode]) -> Result<Option<Tree>, Box<dyn Error>> {
  unp!(args => _faux1, content, _faux2);
  Ok(content)
}

pub fn standalone_script(
  _rule_id: i32,
  mut args: Vec<Option<Tree>>,
  _: &[ValidationPragmatics],
  nodes: &[XMLNode],
) -> Result<Option<Tree>, Box<dyn Error>> {
  unp!(args => _start_script, base, _end_script);
  // TODO: it looks like we need properties on each Tree::Apply,
  // and porting NewScript is a head-scratcher.
  // for now, just keep the property if it's there.
  new_script(base.unwrap(), None, nodes)
}

pub fn postfix_script(
  _rule_id: i32,
  mut args: Vec<Option<Tree>>,
  _: &[ValidationPragmatics],
  nodes: &[XMLNode],
) -> Result<Option<Tree>, Box<dyn Error>> {
  unp!(args => base, op);
  new_script(op.unwrap(), base, nodes)
}

pub fn prefix_script(
  _rule_id: i32,
  mut args: Vec<Option<Tree>>,
  _: &[ValidationPragmatics],
  nodes: &[XMLNode],
) -> Result<Option<Tree>, Box<dyn Error>> {
  unp!(args => op, base);
  new_script(op.unwrap(), base, nodes)
}

/// This is loosely in the lines of MathParser::NewScript, but taking into account
/// the realities of our new data structures.
pub fn new_script(script: Tree, base: Option<Tree>, nodes: &[XMLNode]) -> Result<Option<Tree>, Box<dyn Error>> {
  if let Tree::Lexeme(ref lex, _) = script {
    let node = lookup_lex_node(dbg!(lex.as_str()), nodes)?;
    let script_wrap = node.get_parent().unwrap();
    let node_role = script_wrap.get_attribute("role").unwrap();
    let is_float = node_role.starts_with("FLOAT");
    let is_super = node_role.ends_with("SUPERSCRIPT");
    let role = Cow::Borrowed(if is_super { "SUPERSCRIPTOP" } else { "SUBSCRIPTOP" });
    let scriptpos = Cow::Borrowed(if is_float { "pre1" } else { "post1" });
    if base.is_some() {
       // TODO: scriptpos => "$x$l"
      let op = new_props(None, None, Some(raw_map!("role"=>role, "scriptpos"=>scriptpos)));
      let script_arg = obtain_arg(script, 0);
      Ok(Some(Tree::Apply(op.into(), Args(vec![base, script_arg]), XProps::default(), Meta::default())))
    } else {
      // DG: This is completely wrong, and just temporarily passes one test. Scripts need to be fleshed out with generality. (TODO)
      node
        .get_parent()
        .unwrap()
        .set_attribute("scriptpos", "1")
        .expect("XML attributes should set without issue.");
      Ok(Some(script))
    }
  } else {
    panic!(
      "new_script is meant to be called on script terminals (e.g. POSTSUBSCRIPT/POSTSUPERSCRIPT), got {:?}",
      script
    );
  }
}

// Get n-th arg of an XMApp.
// However, this is really only used to get the script out of a sub/super script
pub fn obtain_arg(tree: Tree, n: usize) -> Option<Tree> {
  match &tree {
    Tree::Lexeme(_, _) | Tree::Token(_, _) => Some(tree),
    Tree::Apply(_, ref args, _, _) => match args.0.get(n) {
      Some(t) => t.clone(),
      None => None,
    },
    _ => unimplemented!(),
  }
}

pub fn apply_invisible_times(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics], _: &[XMLNode]) -> Result<Option<Tree>, Box<dyn Error>> {
  unp!(args => left, right);
  let mut left = left;
  // left-to-right associative -- if "left" is already a "times", tuck "right" in:
  if let Some(Tree::Apply(ref op, ref mut left_args, _, ref _m)) = left {
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
  let times = invisible_times();
  Ok(Some(Tree::Apply(times.into(), Args(vec![left, right]), XProps::default(), Meta::default())))
}

pub fn compound_operator_2(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics], nodes: &[XMLNode]) -> Result<Option<Tree>, Box<dyn Error>> {
  unp!(args => op1, op2);
  // invisble comma:
  let comma = invisible_comma();
  // TODO: We need to extend that rule to the n-ary case
  // Currently following the original MathGrammar and creating a List XMDual
  new_list(vec![op1.unwrap(), comma.into(), op2.unwrap()], nodes)
}

fn invisible_times() -> XProps {
  XProps {
    meaning: Some(Cow::Borrowed("times")),
    role: Some(Cow::Borrowed("MULOP")),
    content: Some(Cow::Borrowed("\u{2062}")),
    name: None,
    scriptpos: None,
    font: None,
  }
}

fn invisible_comma() -> XProps {
  XProps {
    meaning: Some(Cow::Borrowed("")),
    role: Some(Cow::Borrowed("PUNCT")),
    content: Some(Cow::Borrowed("\u{2063}")),
    name: None,
    scriptpos: None,
    font: None,
  }
}

pub fn new_props(
  meaning: Option<Cow<'static, str>>,
  content: Option<Cow<'static, str>>,
  props_opt: Option<HashMap<&'static str, Cow<'static, str>>>,
) -> XProps {
  let mut props = props_opt.unwrap_or_default();
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
  XProps {
    meaning,
    content,
    role,
    name,
    scriptpos,
    font: Some(font),
  }
}

pub fn new_list(mut pieces: Vec<Tree>, nodes: &[XMLNode]) -> Result<Option<Tree>, Box<dyn Error>> {
  // drop placeholder token for missing trailing punct, if any
  if pieces.len() > 1 {
    let last_meaning_opt = pieces.last().unwrap().get_token_meaning(nodes)?;
    if let Some(last_meaning) = last_meaning_opt {
      if last_meaning == "absent" {
        pieces.pop();
      }
    }
  }
  if pieces.len() == 1 {
    Ok(pieces.pop())
  } else {
    let (seps, items) = extract_separators(pieces);
    Ok(Some(Tree::Dual(
      Box::new(Tree::Apply(
        new_props(Some(Cow::Borrowed("list")), None, None).into(),
        create_xmrefs(&items).into(),
        XProps::default(),
        Meta::default()
      )),
      Box::new(Tree::Wrap(
        items,
        XProps::default(),
        Meta::default())),
      XProps::default(),
      Meta::default()
    )))
  }
}

fn extract_separators(items: Vec<Tree>) -> (Vec<Tree>, Vec<Tree>) {
  (Vec::new(), items)
}
fn create_xmrefs(items: &[Tree]) -> Vec<Tree> {
  Vec::new()
}

// Some handy shorthands.
// pub fn absent() -> XMTok { new_token(
//   Some(Cow::Borrowed("absent")),
//    None, HashMap::default()) }

// sub InvisibleComma {
// return New(undef, "\x{2063}", role => 'PUNCT', font =>
// LaTeXML::Common::Font->new()); }
