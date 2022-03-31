#[macro_use]
pub mod expandable;
pub mod conditional;
pub mod constructor;
pub mod math_primitive;
pub mod primitive;
pub mod register;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::common::error::*;
use crate::common::object::Object;
use crate::common::store::Stored;

use self::register::{RegisterType, RegisterValue};
use crate::document::Document;
use crate::gullet::Gullet;
use crate::mouth;
use crate::parameter::Parameters;
use crate::state::State;
use crate::stomach::Stomach;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::Digested;

pub type ExpansionClosure = Arc<dyn Fn(&mut Gullet, Vec<Tokens>, &mut State) -> Result<Tokens>>;
pub type ConditionalClosure = Arc<dyn Fn(&mut Gullet, Vec<Tokens>, &mut State) -> Result<bool>>;
pub type PrimitiveFn = dyn Fn(&mut Stomach, Vec<Tokens>, &mut State) -> Result<Vec<Digested>>;
pub type PrimitiveClosure = Arc<PrimitiveFn>;
pub type BeforeDigestClosure = Arc<dyn Fn(&mut Stomach, &mut State) -> Result<Vec<Digested>>>;
pub type PropertiesClosure = Arc<dyn Fn(&mut Stomach, &Vec<Option<Digested>>, &mut State) -> Result<HashMap<String, Stored>>>;
pub type DigestionClosure = Arc<dyn Fn(&mut Stomach, &mut Whatsit, &mut State) -> Result<Vec<Digested>>>;
pub type ReplacementClosure = Arc<dyn Fn(&mut Document, &Vec<Option<Digested>>, &HashMap<String, Stored>, &mut State) -> Result<()>>;
pub type ConstructionClosure = Arc<dyn Fn(&mut Document, &Whatsit, &mut State) -> Result<()>>;
pub type DigestedReversionClosure = Arc<dyn Fn(&Whatsit, &Vec<Option<Digested>>) -> Result<Tokens>>;

pub type SizingClosure = Arc<dyn Fn(&Whatsit) -> (i32, i32, i32)>;

#[derive(Clone)]
pub enum ExpansionBody {
  Closure(ExpansionClosure),
  Tokens(Tokens),
}

#[derive(Clone)]
pub enum Reversion {
  Closure(DigestedReversionClosure),
  Tokens(Tokens),
}

impl From<&str> for Reversion {
  fn from(t: &str) -> Reversion { Reversion::Tokens(mouth::tokenize_internal(t, None)) }
}
impl From<Tokens> for Reversion {
  fn from(ts: Tokens) -> Reversion { Reversion::Tokens(ts) }
}

impl From<Token> for Option<ExpansionBody> {
  fn from(t: Token) -> Option<ExpansionBody> { Tokens!(t).into() }
}

impl From<Tokens> for ExpansionBody {
  fn from(t: Tokens) -> ExpansionBody { ExpansionBody::Tokens(t) }
}

impl From<Tokens> for Option<ExpansionBody> {
  fn from(t: Tokens) -> Option<ExpansionBody> { Some(t.into()) }
}

impl From<&str> for ExpansionBody {
  fn from(s: &str) -> ExpansionBody { mouth::tokenize_internal(s, None).into() }
}

impl From<String> for ExpansionBody {
  fn from(s: String) -> ExpansionBody { s.as_str().into() }
}

pub trait Definition: Object {
  fn invoke(&self, gullet: &mut Gullet, once_only: bool, state: &mut State) -> Result<Tokens>;
  fn invoke_primitive(&self, gullet: &mut Stomach, caller: Arc<dyn Definition>, state: &mut State) -> Result<Vec<Digested>>;

  /// We can almost always return the CS by reference, except in a Register's RefCell, where we are
  /// forced to clone
  fn get_cs(&self) -> Cow<Token>;
  fn get_cs_name(&self) -> Cow<str>;
  fn get_cs_or_alias(&self) -> Cow<Token> {
    match self.get_alias() {
      Some(alias) => Cow::Owned(T_CS!(alias)),
      None => self.get_cs(),
    }
  }
  fn get_alias(&self) -> Option<&String>;
  fn is_protected(&self) -> bool { false }
  fn is_register(&self) -> bool { false }
  fn is_prefix(&self) -> bool { false }
  fn is_readonly(&self) -> bool { false }

  fn read_arguments(&self, gullet: &mut Gullet, state: &mut State) -> Result<Vec<Tokens>>
  where Self: Sized {
    match self.get_parameters() {
      None => Ok(Vec::new()),
      Some(params) => params.read_arguments(gullet, self, state),
    }
  }
  fn get_parameters(&self) -> Option<&Parameters>;

  // ======================================================================
  // Return the Tokens that would invoke the given definition with arguments.
  fn invocation(&mut self, args: Vec<Tokens>, gullet: &mut Gullet, state: &mut State) -> Result<Tokens> {
    let mut invocation_result: Vec<Token> = vec![self.get_cs().into_owned()];

    match self.get_parameters() {
      None => {},
      Some(params) => {
        for result_token in params.revert_arguments(args, state)? {
          invocation_result.append(&mut result_token.unlist());
        }
      },
    }
    Ok(Tokens::new(invocation_result))
  }

  fn get_num_args(&self) -> usize { 0 }

  fn do_absorbtion(&self, _document: &mut Document, _whatsit: &Whatsit, _state: &mut State) -> Result<()>;
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { None }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { None }
  fn after_digest_body(&self) -> Option<&Vec<DigestionClosure>> { None }
  fn capture_body(&self) -> bool { false }

  fn execute_before_digest(&self, stomach: &mut Stomach, state: &mut State) -> Result<Vec<Digested>> {
    state.unlocked = true;
    let mut before_digested = Vec::new();
    if let Some(pre_list) = self.before_digest() {
      for pre in pre_list.iter() {
        before_digested.extend(pre(stomach, state)?);
      }
    }
    Ok(before_digested)
  }
  fn execute_after_digest(&self, stomach: &mut Stomach, whatsit: &mut Whatsit, state: &mut State) -> Result<Vec<Digested>> {
    state.unlocked = true;
    let mut after_digested = Vec::new();
    if let Some(post_list) = self.after_digest() {
      for post in post_list.iter() {
        after_digested.extend(post(stomach, whatsit, state)?);
      }
    }
    Ok(after_digested)
  }

  fn execute_after_digest_body(&self, stomach: &mut Stomach, whatsit: &mut Whatsit, state: &mut State) -> Result<Vec<Digested>> {
    state.unlocked = true;
    let mut after_body_digested = Vec::new();
    if let Some(post_list) = self.after_digest_body() {
      // info!("Found {:?} after_digest_body closures, capture_body was: {:?}", post_list.len(),
      // self.capture_body());
      for post in post_list {
        let after_body_digest_result = post(stomach, whatsit, state)?;
        after_body_digested.extend(after_body_digest_result);
      }
    }
    Ok(after_body_digested)
  }

  fn value_of(&self, args: Vec<Token>, state: &State) -> Option<RegisterValue> { unimplemented!() }
  fn register_type(&self) -> Option<RegisterType> { None }
  fn get_reversion_spec(&self) -> Option<Reversion> { unimplemented!() }
  fn get_expansion(&self) -> Option<&ExpansionBody> { None }

  fn stringify_type(&self, deftype: &str) -> String {
    let name = match self.get_alias() {
      Some(alias) => alias.to_string(),
      None => self.get_cs().get_cs_name().to_string(),
    };
    if let Some(parameters) = self.get_parameters() {
      s!("{}[{} {}]", deftype, name, parameters.stringify())
    } else {
      s!("{}[{}]", deftype, name)
    }
  }
}

// We need to compare definitions for the internal TeX logic to make sense, but we don't have Perl's level of meta-programming,
// since cloning an `Arc<Definition>` for storage makes it impossible to compare with the old `Arc<Definition>`.
// Hence, we need our own meta-programming "hack", via the `stringify` method that is different for each
// `definition` implementation (`Primitive`/`Constructor`/etc)
// and each control sequence
//
// This could evolve if Rust comes up with a best practice for implementing `PartialEq` on trait objects.
impl PartialEq for dyn Definition {
  fn eq(&self, other: &dyn Definition) -> bool { self.stringify() == other.stringify() }
}

impl fmt::Display for dyn Definition {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if let Some(params) = self.get_parameters() {
      write!(f, "{} {}", self.get_cs_name(), params)
    } else {
      write!(f, "{}", self.get_cs_name())
    }
  }
}

impl fmt::Display for ExpansionBody {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ExpansionBody::Tokens(ref t) => write!(f, "{}", t),
      ExpansionBody::Closure(_) => unimplemented!(), // what is the right way to serialize this, e.g. for the \meaning macro
    }
  }
}
