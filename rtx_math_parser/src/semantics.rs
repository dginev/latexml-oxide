use marpa::lexer::token::Token;
use marpa::stack::*;
use marpa::thin::Value;
use marpa::tree_builder::*;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;

pub use self::tree::{Args, Operator, Tree};
use crate::pragmatics::ValidationPragmatics;

mod tree;
mod metadata;
mod curry;
mod from;

use metadata::Meta;

pub type ActionClosure = Rc<
  dyn Fn(i32, Vec<Option<Tree>>, &[ValidationPragmatics]) -> Result<Option<Tree>, Box<dyn Error>>,
>;

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
          Tree::Atom(token_str.to_owned(), Meta::default()).specialize(Meta::default(), pragmas)?,
        ))
      },
      Node::Leaf(ref tok) => Ok(Some(Tree::Atom(tok.to_string(), Meta::default()))),
      Node::Null(_) => {
        // e.g.* argument failed nothing, just skip.
        Ok(None)
        // Tree::Atom("null".into())
      },
    }
  }
}

// constructors
pub fn infix_apply(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics]) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => arg1, infixop, arg2);
  Ok(Some(Tree::Apply(infixop.into(), Args(vec![arg1, arg2]), Meta::default())))
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
// ambiguous and implicit - invisible operations
pub fn invisible_infix_mulop(_rule_id: i32, mut args: Vec<Option<Tree>>, _: &[ValidationPragmatics]) -> Result<Option<Tree>, Box<dyn Error>> {
  unpack!(args => factor, term_argument);
  // Two choices - multiplication or application
  let choices = vec![
    Tree::Apply(factor.clone().into(), term_argument.clone().into(), Meta::default()),
    Tree::Apply("implied_op".into(), Args(vec![factor, term_argument]), Meta::default()),
  ];
  Ok(Some(Tree::Choices(choices.into_iter().collect())))
}
