use marpa::lexer::token::Token;
use marpa::stack::*;
use marpa::thin::Value;
use marpa::tree_builder::*;
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;

pub use self::tree::{Args, Operator, Tree, XMTok};
use crate::pragmatics::ValidationPragmatics;

mod curry;
mod from;
mod metadata;
mod tree;

use metadata::Meta;

pub type ActionClosure = Rc<dyn Fn(i32, Vec<Option<Tree>>, &[ValidationPragmatics]) -> Result<Option<Tree>, Box<dyn Error>>>;

#[derive(Default)]
pub struct Actions {
  dispatch: HashMap<i32, ActionClosure>,
}

impl Actions {
  pub fn register(&mut self, id: i32, closure: ActionClosure) { self.dispatch.insert(id, closure); }
  pub fn action_on(&self, id: i32, mut args: Vec<Option<Tree>>, pragmas: &[ValidationPragmatics]) -> Result<Option<Tree>, Box<dyn Error>> {
    if let Some(action) = self.dispatch.get(&id) {
      action(id, args, pragmas)
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

  pub fn get_tree(&self, b: TreeBuilder, v: Value, pragmas: &[ValidationPragmatics]) -> Result<Option<Tree>, Box<dyn Error>> {
    let handle = proc_value(b, v);
    self.translate_node(&handle, pragmas)
  }

  pub fn translate_node<T: Token>(&self, n: &Handle<T>, pragmas: &[ValidationPragmatics]) -> Result<Option<Tree>, Box<dyn Error>> {
    match *n.borrow() {
      Node::Tree(ref rule, ref children) => {
        let mut translated_children = Vec::new();
        for child in children.iter() {
          translated_children.push(self.translate_node(child, pragmas)?);
        }
        self.action_on(*rule, translated_children, pragmas)
      },
      Node::Rule(ref rule, ref children) => {
        let mut translated_children = Vec::new();
        for child in children.iter() {
          translated_children.push(self.translate_node(child, pragmas)?);
        }
        self.action_on(*rule, translated_children, pragmas)
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
pub fn infix_apply(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics]) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => arg1, infixop, arg2);
  let apply_tree = Tree::Apply(infixop.into(), Args(vec![arg1, arg2]), Meta::default());
  Ok(Some(apply_tree))
}

/// application with trailing elision, as in `x \cdot y \cdot\cdot\cdot`
pub fn infix_apply_and_elide(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics]) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => arg1, infixop, arg2, elision);
  let apply_tree = Tree::Apply(infixop.into(), Args(vec![arg1, arg2, elision]), Meta::default());
  Ok(Some(apply_tree))
}

// infix_apply in the base case,
// but when chained, using the flat "multirelation" behavior of latexml
pub fn infix_relation(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics]) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => left, infixop, right);
  // if left has a "multirelation" already, add right in.
  // if left applies a relation, flatten it out to infix form.
  // base case - build a simple infix apply
  let mut left = left;
  match left {
    Some(Tree::Apply(ref op, ref mut left_args,ref _left_meta)) => {
      if let Tree::Token(ref tok, _) = *op.0 {
        if tok.meaning == Some(Cow::Borrowed("multirelation")) {
          left_args.0.push(infixop);
          left_args.0.push(right);
          Ok(left)
        } else {
           Ok(Some(Tree::Apply(infixop.into(), Args(vec![left, right]), Meta::default() )))
        }
      } else if let Tree::Lexeme(ref lex, ref _left_meta) = *op.0 {
        if lex.split(':').next().unwrap().contains("RELOP") {
          // first multirelation need is here.
          let multirel_tok = XMTok { meaning: Some(Cow::Borrowed("multirelation")), ..XMTok::default() };
          let mut drained_left_args = left_args.0.drain(..).into_iter();
          let left_1 = drained_left_args.next().unwrap();
          let left_2 = drained_left_args.next().unwrap();
          let moved_op = (*op.0).clone();
          Ok(Some(Tree::Apply(multirel_tok.into(), Args(vec![left_1, Some(moved_op), left_2, infixop, right]), Meta::default() )))
        } else {
          Ok(Some(Tree::Apply(infixop.into(), Args(vec![left, right]), Meta::default() )))
        }
      } else {
        Ok(Some(Tree::Apply(infixop.into(), Args(vec![left, right]), Meta::default() )))
      }
    }
    _ => Ok(Some(Tree::Apply(infixop.into(), Args(vec![left, right]), Meta::default())))
  }

}

pub fn infix_apply_nary(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics]) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => left, infixop, right);
  let mut left = left;
  // left-to-right associative -- if "left" is already "infixop", tuck "right" in:
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

pub fn prefix_apply(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics]) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => prefixop, arg1);
  Ok(Some(Tree::Apply(prefixop.into(), Args(vec![arg1]), Meta::default())))
}
pub fn postfix_apply(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics]) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => arg, op);
  Ok(Some(Tree::Apply(op.into(), Args(vec![arg]), Meta::default())))
}

pub fn circumfix_fenced(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics]) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => _open, arg, _close);
  Ok(arg)
}

pub fn post_script(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics]) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => base, op, annotation);
  Ok(Some(Tree::Apply(op.into(), Args(vec![base, annotation]), Meta::default())))
}

pub fn invisible_times(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics]) -> Result<Option<Tree>, Box<dyn Error>> {
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
  };
  Ok(Some(Tree::Apply(times.into(), Args(vec![left, right]), Meta::default())))
}
