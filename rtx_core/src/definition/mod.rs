#[macro_use]
pub mod expandable;
pub mod argument;
pub mod conditional;
pub mod constructor;
pub mod math_primitive;
pub mod primitive;
pub mod register;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use libxml::tree::Node;

use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::font::Font;
use crate::common::object::Object;
use crate::common::store::Stored;

use self::argument::ArgWrap;
use self::register::{RegisterType, RegisterValue};

use crate::document::Document;
use crate::gullet::Gullet;
use crate::mouth;
use crate::parameter::Parameters;
use crate::state::State;
use crate::stomach::Stomach;
use crate::token::{Catcode, Token};
use crate::tokens::{Tokens, NO_TOKENS};
use crate::whatsit::Whatsit;
use crate::Digested;

pub type ExpansionClosure = Arc<dyn Fn(&mut Gullet, Vec<ArgWrap>, &mut State) -> Result<Tokens>>;
pub type ConditionalClosure = Arc<dyn Fn(&mut Gullet, Vec<ArgWrap>, &mut State) -> Result<bool>>;
pub type PrimitiveFn = dyn Fn(&mut Stomach, Vec<ArgWrap>, &mut State) -> Result<Vec<Digested>>;
pub type PrimitiveClosure = Arc<PrimitiveFn>;
pub type BeforeDigestClosure = Arc<dyn Fn(&mut Stomach, &mut State) -> Result<Vec<Digested>>>;
pub type PropertiesClosure = Arc<dyn Fn(&mut Stomach, &Vec<Option<Digested>>, &mut State) -> Result<HashMap<String, Stored>>>;
pub type DigestionClosure = Arc<dyn Fn(&mut Stomach, &mut Whatsit, &mut State) -> Result<Vec<Digested>>>;
pub type ReplacementClosure = Arc<dyn Fn(&mut Document, &Vec<Option<Digested>>, &HashMap<String, Stored>, &mut State) -> Result<()>>;
pub type ConstructionClosure = Arc<dyn Fn(&mut Document, &Whatsit, &mut State) -> Result<()>>;
pub type DigestedReversionClosure = Arc<dyn Fn(&Whatsit, &Vec<Option<Digested>>, &State) -> Result<Tokens>>;
pub type SizingClosure = Arc<dyn Fn(&Whatsit, &mut State) -> Result<(Dimension, Dimension, Dimension)>>;
pub type FontClosure = Arc<dyn Fn(Option<&Whatsit>, &mut State) -> Result<Font>>;

#[derive(Clone)]
pub enum ExpansionBody {
  Closure(ExpansionClosure),
  Tokens(Tokens),
}

impl std::fmt::Debug for ExpansionBody {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ExpansionBody::Closure(code) => write!(f, "CODE({:p})", Arc::as_ptr(code)),
      ExpansionBody::Tokens(ts) => write!(f, "{ts:?}"),
    }
  }
}

impl Default for ExpansionBody {
  fn default() -> Self { ExpansionBody::Tokens(NO_TOKENS) }
}

impl PartialEq for ExpansionBody {
  #[allow(clippy::vtable_address_comparisons)]
  fn eq(&self, other: &ExpansionBody) -> bool {
    match self {
      ExpansionBody::Closure(self_closure) => match other {
        ExpansionBody::Closure(other_closure) => Arc::ptr_eq(self_closure, other_closure),
        ExpansionBody::Tokens(other_tokens) => {
          // sometimes the \meaning game forces us into the same CODE(0x...) pointer footprint appearing as Tokens and as the original Closure.
          // if we do this carefully, we can get the two .to_string() variants to match...
          format!("CODE({:p})",Arc::as_ptr(self_closure)) == other_tokens.to_string()
        }
      },
      ExpansionBody::Tokens(self_tks) => match other {
        ExpansionBody::Tokens(other_tks) => self_tks == other_tks,
        ExpansionBody::Closure(other_closure) => {
          format!("CODE({:p})",Arc::as_ptr(other_closure)) == self_tks.to_string()
        }
      },
    }
  }
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
        _ => false
      },
      // never compare pointers - i.e. never equal
      _ => false
    }
  }
}

impl From<&str> for Reversion {
  fn from(t: &str) -> Reversion { Reversion::Tokens(mouth::tokenize_internal(t).pack_parameters()) }
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
  fn from(t: Tokens) -> Option<ExpansionBody> {
    if t.is_empty() {
      None
    } else {
      Some(t.into())
    }
  }
}

impl From<&str> for ExpansionBody {
  fn from(s: &str) -> ExpansionBody { mouth::tokenize_internal(s).into() }
}

impl From<String> for ExpansionBody {
  fn from(s: String) -> ExpansionBody { s.as_str().into() }
}

impl From<ArgWrap> for ExpansionBody {
  fn from(t: ArgWrap) -> ExpansionBody { ExpansionBody::Tokens(t.owned_tokens().unwrap_or_default()) }
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
  Asset(Arc<Font>),
}

impl From<Font> for FontDirective {
  fn from(f: Font) -> Self { FontDirective::Asset(Arc::new(f)) }
}
impl From<FontClosure> for FontDirective {
  fn from(fc: FontClosure) -> Self { FontDirective::Closure(fc) }
}
impl FontDirective {
  pub fn get_font(&self, whatsit: Option<&Whatsit>, state: &mut State) -> Result<Arc<Font>> {
    match self {
      FontDirective::Closure(fc) => Ok(Arc::new((fc)(whatsit, state)?)),
      FontDirective::Asset(ref font) => Ok(Arc::clone(font)),
    }
  }
  pub fn get_asset(&self) -> Option<Arc<Font>> {
    if let FontDirective::Asset(font) = self {
      Some(Arc::clone(font))
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
  fn get_sizer(&self) -> Option<SizingClosure> { None }
  fn get_alias(&self) -> Option<&String>;
  fn is_protected(&self) -> bool { false }
  fn is_register(&self) -> bool { false }
  fn is_prefix(&self) -> bool { false }
  fn is_readonly(&self) -> bool { false }

  fn read_arguments(&self, gullet: &mut Gullet, state: &mut State) -> Result<Vec<ArgWrap>>
  where Self: Sized {
    match self.get_parameters() {
      None => Ok(Vec::new()),
      Some(params) => params.read_arguments(gullet, Some(self), state),
    }
  }
  fn get_parameters(&self) -> Option<&Parameters>;

  // ======================================================================
  // Return the Tokens that would invoke the given definition with arguments.
  fn invocation(&mut self, args: Vec<Option<Tokens>>, gullet: &mut Gullet, state: &mut State) -> Result<Tokens> {
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

  fn do_absorbtion(&self, _document: &mut Document, _whatsit: &Whatsit, _state: &mut State) -> Result<Vec<Node>>;
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

  fn value_of(&self, args: Vec<ArgWrap>, state: &mut State) -> Option<RegisterValue> { unimplemented!() }
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
      ExpansionBody::Tokens(ref t) => write!(f, "{t}"),
      ExpansionBody::Closure(ref code) => write!(f,"ExpansionBody::Closure({:p})", Arc::as_ptr(code)), // what is the right way to serialize this, e.g. for the \meaning macro
    }
  }
}
