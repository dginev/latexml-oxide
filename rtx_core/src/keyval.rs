// use std::borrow::Cow;
// use rustc_hash::{FxHashMap as HashMap};
// use std::fmt;
use std::rc::Rc;

use crate::binding::def::dialect::def_macro;
use crate::common::error::*;
use crate::common::arena;
use crate::common::def_parser::parse_parameters;
// use crate::common::font::Font;
// use crate::common::locator::Locator;
use crate::common::store::Stored;
use crate::definition::ExpansionBody;
use crate::parameter::Parameter;
// use crate::definition::expandable::Expandable;
// use crate::definition::Definition;
// use crate::document::Document;
// use crate::list::List;
use crate::state::State;
use crate::mouth::tokenize;
use crate::token::{Catcode,Token};
// use crate::tokens::Tokens;

#[derive(Debug, Clone)]
pub struct KeyVal {
  // which KeyVals are we parsing and how do we behave?
  prefix: String,
  key: String,
  keyset: String,
}

impl Default for KeyVal {
  fn default() -> Self {
    KeyVal {
      prefix: "KV".to_string(),
      keyset: String::new(),
      key: String::new(),
    }
  }
}

impl KeyVal {
  pub fn new(prefix: Option<String>, keyset: String, key: String) -> Self {
    let prefix = prefix.unwrap_or_else(|| "KV".to_string());
    KeyVal {
      prefix,
      key,
      keyset,
    }
  }

  pub fn get_header(&self) -> String { s!("{}@{}@{}", self.prefix, self.keyset, self.key) }

  //======================================================================
  // Property access
  //======================================================================

  pub fn get_prop<'a>(&self, key: &str, state: &'a State) -> Option<&'a Stored> {
    state.lookup_value(&s!("KEYVAL@{}@{}", key, self.get_header()))
  }
  pub fn get_default(&self, state: &State) -> Option<Stored> {
    self.get_prop("default", state).map(|v| (*v).clone())
  }
  pub fn get_type<'a>(&'a self, state: &'a State) -> Option<Rc<Parameter>> {
    match self.get_prop("type", state) {
      Some(Stored::Parameter(p)) => Some(Rc::clone(p)),
      _ => None,
    }
  }
}

// useful (only) for rtx_core::keyvals::KeyVals
pub(crate) fn keyval_qname(prefix:&str, keyset:&str, key:&str) -> String {
  let prefix = if prefix.is_empty() { "KV" } else {prefix};
  s!("{prefix}@{keyset}@{key}")
}

pub(crate) fn keyval_get<'a>(qname:&str, prop:&str, state:&'a State) -> Option<&'a Stored> {
  state.lookup_value_sym(&arena::pin(s!("KEYVAL@{prop}@{qname}")))
}

pub(crate) fn keyval_set(qname:&str, prop:&str, value:Stored, state:&mut State) {
  state.assign_value_sym(arena::pin(s!("KEYVAL@{prop}@{qname}")), value, None);
}

//======================================================================
// Key Definition
//======================================================================
#[derive(Debug, Default, Clone)]
pub struct KeyvalConfig<'a> {
  kind: Option<&'a str>,
  code: Option<&'a str>,
  macroprefix: Option<&'a str>,
  mismatch: Option<&'a str>,
  normalize:Option<bool>,
  choices : Option<&'a str>,
  bin: Option<&'a str>,
}

/// (re-)define this key
pub fn define(prefix: &str, keyset:&str, key:&str, vtype:&str, default_opt:Option<&str>, options: KeyvalConfig, state:&mut State) -> Result<()> {
  let qname = keyval_qname(prefix, keyset, key);

  // define that the key exists and is not disabled
  keyval_set(&qname, "exists"  , 1.into(), state);
  keyval_set(&qname, "disabled", 0.into(), state);
  // set the type
  let vtype = if vtype.is_empty() { "{}" } else { vtype };
  let paramlist_opt = parse_parameters(vtype,
    &T_OTHER!(s!("KeyVal {key} in set {keyset} with prefix {prefix}")), Some(state))?;
  match paramlist_opt {
    None => {
      Warn!("unexpected", "keyval", None, state,
        s!("No parameters in keyval {key} (in set {keyset} with prefix {prefix}) taking only first"));
    }
    Some(paramlist) => {
      if paramlist.get_num_args() != 1 {
        Warn!("unexpected", "keyval", None, state,
          "Too many parameters in keyval {key} (in set {keyset} with prefix {prefix}) taking only first"); }
      keyval_set(&qname, vtype, paramlist.take_parameters().remove(0).into(), state);
    }
  };
  // set the default
  if let Some(default) = default_opt {
    let tdefault = tokenize(default, Some(state));
    keyval_set(&qname, "default", Stored::Tokens(Tokens!(tdefault.clone())), state);
    def_macro(T_CS!(s!("\\{qname}@default")), None,
      ExpansionBody::Tokens(Tokens!(
        T_CS!(s!("\\{qname}")), T_BEGIN!(), tdefault, T_END!())),
    None, state);
  }

  // figure out the kind of key-val parameter we are defining
  let kind = options.kind.unwrap_or("ordinary");
  match kind {
    "ordinary" => define_ordinary(&qname, options.code),
    "command" => {
      let macroname = if let Some(mpfx) = options.macroprefix {
        s!("{mpfx}{key}")
      } else { s!("cmd{qname}")};
      define_command(&qname, options.code, &macroname);
    },
    "choice" =>
      define_choice(&qname, options.code, options.mismatch,
      options.choices, options.normalize.unwrap_or(false), options.bin),
    "boolean" => {
      let macroname = if let Some(mpfx) = options.macroprefix {
        s!("{mpfx}{key}")
      } else { qname.clone() };
      define_boolean(&qname, options.code, options.mismatch, &macroname)
    },
    _ => Warn!("unknown", "undef", None, state, s!("Unknown KeyVals kind {kind} should be one of\
     'ordinary', 'command', 'choice', 'boolean'. "))
  };
  Ok(())
}

fn define_ordinary(qname:&str, code: Option<&str>) {}
fn define_command(qname: &str, code: Option<&str>, macroname: &str) {}
fn define_choice(qname:&str, code: Option<&str>, mismatch: Option<&str>, choices: Option<&str>, normalize: bool, bin: Option<&str>) {}
fn define_boolean(qname:&str, code: Option<&str>, mismatch: Option<&str>, macroname: &str) {}