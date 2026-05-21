#[macro_use]
pub mod expandable;
pub mod argument;
pub mod conditional;
pub mod constructor;
pub mod math_primitive;
pub mod primitive;
pub mod register;

use libxml::tree::Node;
use std::borrow::Cow;
use std::fmt;
use std::rc::Rc;

use crate::common::arena::{self, SymHashMap, SymStr};
use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::font::Font;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::definition::conditional::ConditionalType;

use self::argument::ArgWrap;
use self::register::{RegisterType, RegisterValue};

use crate::Digested;
use crate::document::Document;
use crate::gullet::Gullet;
use crate::mouth;
use crate::parameter::Parameters;
use crate::state::{Scope, expire_state_unlocked, local_state_unlocked};
use crate::token::Token;
use crate::tokens::{NO_TOKENS, Tokens};
use crate::whatsit::Whatsit;

pub type ExpansionClosure = Rc<dyn Fn(Vec<ArgWrap>) -> Result<Tokens>>;
pub type ConditionalClosure = Rc<dyn Fn(Vec<ArgWrap>) -> Result<bool>>;
pub type PrimitiveFn = dyn Fn(Vec<ArgWrap>) -> Result<Vec<Digested>>;
pub type PrimitiveClosure = Rc<PrimitiveFn>;
pub type BeforeDigestClosure = Rc<dyn Fn() -> Result<Vec<Digested>>>;
pub type PropertiesClosure = Rc<dyn Fn(&Vec<Option<Digested>>) -> Result<SymHashMap<Stored>>>;
pub type DigestionClosure = Rc<dyn Fn(&mut Whatsit) -> Result<Vec<Digested>>>;
pub type ReplacementClosure =
  Rc<dyn Fn(&mut Document, &Vec<Option<Digested>>, &SymHashMap<Stored>) -> Result<()>>;
pub type ConstructionClosure = Rc<dyn Fn(&mut Document, &Whatsit) -> Result<()>>;
pub type DigestedReversionClosure = Rc<dyn Fn(&Whatsit, &Vec<Option<Digested>>) -> Result<Tokens>>;
pub type SizingClosure = Rc<dyn Fn(&Whatsit) -> Result<(Dimension, Dimension, Dimension)>>;
pub type FontClosure = Rc<dyn Fn(Option<&Whatsit>) -> Result<Font>>;

#[derive(Clone)]
pub enum ExpansionBody {
  Closure(ExpansionClosure),
  Tokens(Tokens),
}

impl ExpansionBody {
  /// A convenience method that pushes a token into the internal tokens of
  /// an existing `ExpansionBody::Tokens`, re-wrapping the outer structs
  pub fn push(&mut self, t: Token) {
    match self {
      ExpansionBody::Tokens(tks) => tks.unlist_mut().push(t),
      ExpansionBody::Closure(_) => {
        // Can't push a token into a closure-based expansion — silently ignore
      },
    }
  }
}

impl std::fmt::Debug for ExpansionBody {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ExpansionBody::Closure(code) => write!(f, "CODE({:p})", Rc::as_ptr(code)),
      ExpansionBody::Tokens(ts) => write!(f, "{ts:?}"),
    }
  }
}

impl Default for ExpansionBody {
  fn default() -> Self { ExpansionBody::Tokens(NO_TOKENS) }
}

impl PartialEq for ExpansionBody {
  fn eq(&self, other: &ExpansionBody) -> bool {
    match self {
      ExpansionBody::Closure(self_closure) => match other {
        ExpansionBody::Closure(other_closure) => Rc::ptr_eq(self_closure, other_closure),
        ExpansionBody::Tokens(other_tokens) => {
          // sometimes the \meaning game forces us into the same CODE(0x...) pointer footprint
          // appearing as Tokens and as the original Closure. if we do this carefully, we
          // can get the two .to_string() variants to match...
          format!("CODE({:p})", Rc::as_ptr(self_closure)) == other_tokens.to_string()
        },
      },
      ExpansionBody::Tokens(self_tks) => match other {
        ExpansionBody::Tokens(other_tks) => self_tks == other_tks,
        ExpansionBody::Closure(other_closure) => {
          format!("CODE({:p})", Rc::as_ptr(other_closure)) == self_tks.to_string()
        },
      },
    }
  }
}

#[derive(Clone)]
pub enum PrimitiveBody {
  Closure(PrimitiveClosure),
  String(SymStr),
}
impl From<char> for PrimitiveBody {
  fn from(c: char) -> Self { PrimitiveBody::String(arena::pin_char(c)) }
}

#[derive(Clone)]
pub enum Reversion {
  Closure(DigestedReversionClosure),
  Tokens(Tokens),
}

impl PartialEq for Reversion {
  fn eq(&self, other: &Reversion) -> bool {
    match self {
      Reversion::Tokens(t) => match other {
        Reversion::Tokens(t2) => t == t2,
        _ => false,
      },
      // never compare pointers - i.e. never equal
      _ => false,
    }
  }
}

impl From<&str> for Reversion {
  fn from(t: &str) -> Reversion {
    Reversion::Tokens(mouth::tokenize_internal(t).pack_parameters().unwrap())
  }
}
impl From<Tokens> for Reversion {
  fn from(ts: Tokens) -> Reversion { Reversion::Tokens(ts) }
}

impl From<Token> for ExpansionBody {
  fn from(t: Token) -> ExpansionBody { ExpansionBody::Tokens(Tokens!(t)) }
}

impl From<Token> for Option<ExpansionBody> {
  fn from(t: Token) -> Option<ExpansionBody> { Some(ExpansionBody::Tokens(Tokens!(t))) }
}

impl From<Tokens> for ExpansionBody {
  fn from(t: Tokens) -> ExpansionBody { ExpansionBody::Tokens(t) }
}

impl From<Tokens> for Option<ExpansionBody> {
  fn from(t: Tokens) -> Option<ExpansionBody> { if t.is_empty() { None } else { Some(t.into()) } }
}

impl From<&str> for ExpansionBody {
  fn from(s: &str) -> ExpansionBody { mouth::tokenize_internal(s).into() }
}

impl From<String> for ExpansionBody {
  fn from(s: String) -> ExpansionBody { s.as_str().into() }
}

impl From<ArgWrap> for ExpansionBody {
  fn from(t: ArgWrap) -> ExpansionBody {
    ExpansionBody::Tokens(t.owned_tokens().unwrap_or_default())
  }
}
impl From<ArgWrap> for Option<ExpansionBody> {
  fn from(t: ArgWrap) -> Option<ExpansionBody> {
    match t.owned_tokens() {
      Some(tks) if !tks.is_empty() => Some(ExpansionBody::Tokens(tks)),
      _ => None,
    }
  }
}

#[derive(Clone)]
pub enum FontDirective {
  Closure(FontClosure),
  Asset(Rc<Font>),
}

impl From<Font> for FontDirective {
  fn from(f: Font) -> Self { FontDirective::Asset(Rc::new(f)) }
}
impl From<FontClosure> for FontDirective {
  fn from(fc: FontClosure) -> Self { FontDirective::Closure(fc) }
}
impl FontDirective {
  pub fn get_font(&self, whatsit: Option<&Whatsit>) -> Result<Rc<Font>> {
    match self {
      FontDirective::Closure(fc) => Ok(Rc::new((fc)(whatsit)?)),
      FontDirective::Asset(ref font) => Ok(Rc::clone(font)),
    }
  }
  pub fn get_asset(&self) -> Option<Rc<Font>> {
    if let FontDirective::Asset(font) = self {
      Some(Rc::clone(font))
    } else {
      None
    }
  }
}
impl fmt::Debug for FontDirective {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      FontDirective::Closure(_) => write!(f, "<FontClosure>"),
      FontDirective::Asset(font) => write!(f, "{:?}", *font),
    }
  }
}
impl PartialEq for FontDirective {
  fn eq(&self, other: &FontDirective) -> bool {
    match self {
      FontDirective::Closure(_) => false, // we can't compare them for now?
      FontDirective::Asset(asset1) => match other {
        FontDirective::Closure(_) => false,
        FontDirective::Asset(asset2) => *asset1 == *asset2,
      },
    }
  }
}

pub trait Definition: Object {
  fn invoke(&self, once_only: bool) -> Result<Tokens>;
  fn invoke_primitive(&self) -> Result<Vec<Digested>>;

  /// We can almost always return the CS by reference, except in a Register's RefCell, where we are
  /// forced to clone
  fn get_cs(&self) -> Cow<'_, Token>;
  fn get_cs_name(&self) -> Cow<'_, str>;
  fn get_cs_or_alias(&self) -> Cow<'_, Token> {
    match self.get_alias() {
      Some(alias) => Cow::Owned(T_CS!(alias)),
      None => self.get_cs(),
    }
  }
  fn get_sizer(&self) -> Option<SizingClosure> { None }
  fn get_alias(&self) -> Option<&String>;
  fn is_protected(&self) -> bool { false }
  fn is_register(&self) -> bool { false }
  fn is_prefix(&self) -> bool { false }
  fn is_readonly(&self) -> bool { false }
  fn get_test(&self) -> Option<&ConditionalClosure> { None }
  fn get_conditional_type(&self) -> Option<ConditionalType> { None }

  fn read_arguments(&self) -> Result<Vec<ArgWrap>>
  where Self: Sized {
    match self.get_parameters() {
      None => Ok(Vec::new()),
      Some(params) => params.read_arguments(Some(self)),
    }
  }
  fn get_parameters(&self) -> Option<&Parameters>;

  // ======================================================================
  // Return the Tokens that would invoke the given definition with arguments.
  fn invocation(&mut self, args: Vec<Option<Tokens>>, _gullet: &mut Gullet) -> Result<Tokens> {
    let mut invocation_result: Vec<Token> = vec![self.get_cs().into_owned()];

    match self.get_parameters() {
      None => {},
      Some(params) => {
        for result_token in params.revert_arguments(args)? {
          invocation_result.push(result_token);
        }
      },
    }
    Ok(Tokens::new(invocation_result))
  }

  fn get_num_args(&self) -> usize { 0 }

  fn do_absorption(&self, _document: &mut Document, _whatsit: &Whatsit) -> Result<Vec<Node>>;
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { None }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { None }
  fn after_digest_body(&self) -> Option<&Vec<DigestionClosure>> { None }
  fn capture_body(&self) -> bool { false }

  fn execute_before_digest(&self) -> Result<Vec<Digested>> {
    local_state_unlocked(true);
    let mut before_digested = Vec::new();
    if let Some(pre_list) = self.before_digest() {
      for pre in pre_list.iter() {
        before_digested.extend(pre()?);
      }
    }
    expire_state_unlocked();
    Ok(before_digested)
  }
  fn execute_after_digest(&self, whatsit: &mut Whatsit) -> Result<Vec<Digested>> {
    local_state_unlocked(true);
    let mut after_digested = Vec::new();
    if let Some(post_list) = self.after_digest() {
      for post in post_list.iter() {
        after_digested.extend(post(whatsit)?);
      }
    }
    expire_state_unlocked();
    Ok(after_digested)
  }

  fn execute_after_digest_body(&self, whatsit: &mut Whatsit) -> Result<Vec<Digested>> {
    local_state_unlocked(true);
    let mut after_body_digested = Vec::new();
    if let Some(post_list) = self.after_digest_body() {
      // info!("Found {:?} after_digest_body closures, capture_body was: {:?}", post_list.len(),
      // self.capture_body());
      for post in post_list {
        let after_body_digest_result = post(whatsit)?;
        after_body_digested.extend(after_body_digest_result);
      }
    }
    expire_state_unlocked();
    Ok(after_body_digested)
  }

  fn value_of(&self, _args: Vec<ArgWrap>) -> Option<RegisterValue> { None }
  /// runs the setter to assign the value for a register
  fn set_value(&self, _value: RegisterValue, _scope: Option<Scope>, _args: Vec<ArgWrap>) {
    log::warn!("set_value called on non-register definition");
  }
  fn register_type(&self) -> Option<RegisterType> { None }
  fn get_reversion_spec(&self) -> Option<Reversion> { None }
  fn get_expansion(&self) -> Option<&ExpansionBody> { None }

  fn stringify_type(&self, deftype: &str) -> String {
    let name = match self.get_alias() {
      Some(alias) => alias.clone(),
      None => self.get_cs().with_cs_name(ToString::to_string),
    };
    if let Some(parameters) = self.get_parameters() {
      s!("{}[{} {}]", deftype, name, parameters.stringify())
    } else {
      s!("{}[{}]", deftype, name)
    }
  }
}

// We need to compare definitions for the internal TeX logic to make sense, but we don't have Perl's
// level of meta-programming, since cloning an `Rc<Definition>` for storage makes it impossible to
// compare with the old `Rc<Definition>`. Hence, we need our own meta-programming "hack", via the
// `stringify` method that is different for each `definition` implementation
// (`Primitive`/`Constructor`/etc) and each control sequence
//
// This could evolve if Rust comes up with a best practice for implementing `PartialEq` on trait
// objects.
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
      ExpansionBody::Tokens(ref t) => write!(f, "{t}"),
      ExpansionBody::Closure(ref code) => {
        write!(f, "ExpansionBody::Closure({:p})", Rc::as_ptr(code))
      }, // what is the right way to serialize this, e.g. for the \meaning macro
    }
  }
}
