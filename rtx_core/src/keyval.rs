//! Key-Value Definitions
//!
//! Provides an interface to define and access KeyVal definition.
//! Used in conjunction with `KeyVals` to
//!  fully implement KeyVal pairs.

use std::rc::Rc;

use crate::binding::def::dialect::{def_macro, def_conditional};
use crate::common::error::*;
use crate::common::arena;
use crate::common::def_parser::parse_parameters;
// use crate::common::font::Font;
// use crate::common::locator::Locator;
use crate::common::store::Stored;
use crate::definition::argument::ArgWrap;
use crate::definition::conditional::ConditionalOptions;
use crate::definition::{ExpansionBody, ExpansionClosure};
use crate::parameter::Parameter;
use crate::gullet::Gullet;
// use crate::definition::expandable::Expandable;
// use crate::definition::Definition;
// use crate::document::Document;
// use crate::list::List;
use crate::state::State;
use crate::mouth::tokenize;
use crate::token::{Catcode,Token};
use crate::tokens::Tokens;

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
/// Configuration fields for declaring a new KeyVal pattern
pub struct KeyvalConfig<'a> {
  pub prefix: &'a str,
  pub keyset: &'a str,
  pub key: &'a str,
  pub vtype: &'a str,
  pub default: Option<&'a str>,
  pub kind: Option<&'a str>,
  pub code: Option<ExpansionBody>,
  pub macroprefix: Option<&'a str>,
  pub mismatch: Option<ExpansionBody>,
  pub normalize:Option<bool>,
  pub bin: Option<Tokens>,
  pub choices : Vec<&'static str>,
}

/// (Re-)defines this Key of kind 'kind'.
///Defines a keyword `key` used in keyval arguments for the set `keyset` and,
///and if the option `code` is given, defines appropriate macros
///when used with the `keyval` package (or extensions thereof).
///
///If `type` is given, it defines the type of value that must be supplied,
///such as `Dimension`.  If `default` is given, that value will be used
///when `key` is used without an equals and explicit value in a keyvals argument.
///
///A `scope` option can be given, which can be used to defined the key-value pair
///globally instead of in the current scope.
///
///Several more `option`s can be given. These implement the behaviour of the
///xkeyval package.
///
///The `prefix` parameter can be used to configure a custom prefix for
///the macros to be defined. The `kind` parameter can be used to configure special types of xkeyval
///pairs.
///
///The 'ordinary' kind behaves like a normal keyval parameter.
///
///The 'command' kind defines a command key, that when run stores the value of the
///key in a special macro, which can be further specefied by the `macroprefix`
///option.
///
///The 'choice' kind defines a choice key, which takes additional options
///`choices` (to specify which choices are valid values), `mismatch` (to be run
///if an invalid choice is made) and `bin` (see xkeyval documentation for
///details).
///
///The 'boolean' kind defines a special choice key that takes possible values true and
///false, and defines a new Conditional according to the assumed value. The name of
///this conditional can be specified with the `macroprefix` option.
///
///The kind parameter only takes effect when `code` is given, otherwise only
///meta-data is stored.
pub fn define(options: KeyvalConfig, gullet: &mut Gullet, state:&mut State) -> Result<()> {
  let prefix = options.prefix;
  let keyset = options.keyset;
  let key = options.key;
  let vtype = options.vtype;
  let default_opt = options.default;
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
    "ordinary" => define_ordinary(&qname, options.code, state)?,
    "command" => {
      let macroname = if let Some(mpfx) = options.macroprefix {
        s!("{mpfx}{key}")
      } else { s!("cmd{qname}")};
      define_command(&qname, options.code, &macroname, state)?;
    },
    "choice" =>
      define_choice(&qname, options.code, options.mismatch,
      options.choices, options.normalize.unwrap_or(false), options.bin, state)?,
    "boolean" => {
      let macroname = if let Some(mpfx) = options.macroprefix {
        s!("{mpfx}{key}")
      } else { qname.clone() };
      define_boolean(&qname, options.code, options.mismatch, &macroname, gullet, state)?
    },
    _ => Warn!("unknown", "undef", None, state, s!("Unknown KeyVals kind {kind} should be one of\
     'ordinary', 'command', 'choice', 'boolean'. "))
  };
  Ok(())
}

/// Helper function to define state neccesary for an ordinary key.
fn define_ordinary(qname:&str, code_expansion: Option<ExpansionBody>, state: &mut State) -> Result<()> {
  let qname_cs = T_CS!(s!("\\{qname}"));
  let plain_params = parse_parameters("{}",&qname_cs, Some(state))?;
  def_macro(qname_cs, plain_params, code_expansion, None, state);
  Ok(())
}

/// Helper function to define state neccesary for a command key.
fn define_command(qname: &str, code: Option<ExpansionBody>, macroname: &str, state: &mut State) -> Result<()> {
  let qname_cs = T_CS!(s!("\\{qname}"));
  let plain_params = parse_parameters("{}",&qname_cs, Some(state))?;
  let plainp = plain_params.clone();
  let orig = s!("\\ltxml@orig@{qname}");
  let macroname_cs = s!("\\{macroname}");
  let closure : ExpansionClosure = Rc::new(
    move |_gullet, value:Vec<ArgWrap>, istate| {
      def_macro(T_CS!(&orig), plainp.clone(), code.clone(), None, istate);
      let value_tks : Vec<Token> = value.into_iter().flat_map(|v| v.owned_tokens().map(|inner| inner.unlist()).unwrap_or_default()).collect();
      // $value !?!??! Is it a number 1--9 ???)
      Ok(Tokens!(
        T_CS!("\\def"), T_CS!(&macroname_cs), T_BEGIN!(), value_tks.clone(), T_END!(),
        T_CS!(&orig), T_BEGIN!(), T_PARAM!(), value_tks, T_END!()))
  });
  def_macro(qname_cs, plain_params, ExpansionBody::Closure(closure), None, state);
  Ok(())
}

/// Helper function to define state neccesary for an choice key.
fn define_choice(qname:&str, code_opt: Option<ExpansionBody>, mismatch_opt: Option<ExpansionBody>, choices: Vec<&'static str>, normalize: bool, bin: Option<Tokens>, state: &mut State) -> Result<()> {
  let (varmacro_opt, idxmacro_opt) = if let Some(bin_tks) = bin {
    let mut bin_iter = bin_tks.unlist().into_iter();
    (bin_iter.next(), bin_iter.next())
  } else { (None, None) };
  let qname_cs = T_CS!(s!("\\{qname}"));
  let orig   = T_CS!(s!("\\ltxml@orig@{qname}"));
  let plain_params = parse_parameters("{}",&qname_cs, Some(state))?;
  let plain_params_main = plain_params.clone();
  let closure : ExpansionClosure = Rc::new(move |_gullet, mut values, istate| {
    // Store the normalized value (if applicable)
    let value = values.remove(0).owned_tokens().unwrap_or_default();
    let mut nvalue = value.to_string();
    if normalize {
      nvalue = nvalue.to_lowercase();
    }
    if let Some(ref varmacro) = varmacro_opt {
      def_macro(varmacro.clone(), None, ExpansionBody::Tokens(Tokens::new(Explode!(nvalue))), None, istate);
    }
    // iterate over the possible choices and store them
    let mut index = 0;
    let mut valid = false;
    for choice_str in choices.iter() {
      if (normalize && ( choice_str.to_lowercase() == nvalue )) || *choice_str == nvalue {
        valid   = true;
        if let Some(ref idxmacro) = idxmacro_opt {
          def_macro(idxmacro.clone(), None, ExpansionBody::Tokens(Tokens::new(Explode!(index))), None, istate);
        }
        index += 1;
      }
    }
    // find a name for the original macro to store in
    let mut tokens = Vec::new();
    // if we have chosen a valid index, run $code
    if valid {
      if let Some(ref code) = code_opt {
        def_macro(orig.clone(), plain_params.clone(), code.clone(), None, istate);
        tokens.push(orig.clone());
        tokens.push(T_BEGIN!());
        tokens.extend(value.unlist());
        tokens.push(T_END!());
      }
    } else if let Some(ref mismatch) = mismatch_opt {
      // else run `mismatch
      def_macro(orig.clone(), plain_params.clone(), mismatch.clone(), None, istate);
      tokens.push(orig.clone());
      tokens.push(T_BEGIN!());
      tokens.extend(value.unlist());
      tokens.push(T_END!());
    }
    Ok(Tokens::new(tokens))
  });
  def_macro(qname_cs, plain_params_main, ExpansionBody::Closure(closure), None, state);
  Ok(())
}

/// Helper function to define state neccesary for a boolean key.
fn define_boolean(qname:&str, code_opt: Option<ExpansionBody>, mismatch: Option<ExpansionBody>, macroname: &str, gullet: &mut Gullet, state: &mut State) -> Result<()> {
  def_conditional(T_CS!(s!("\\if{macroname}")), None, None, ConditionalOptions::default(), gullet, state);    // We might need to $scope here
  let orig = s!("\\ltxml@@rig@{qname}");
  let orig_cs = T_CS!(orig);
  let plain_params = parse_parameters("{}",&orig_cs, Some(state))?;
  let macroname_true = T_CS!(s!("\\{macroname}true"));
  let macroname_false = T_CS!(s!("\\{macroname}false"));
  let closure: ExpansionClosure = Rc::new(move |_gullet, mut values:Vec<ArgWrap>, istate| {
    // set the value to true (if needed)
    let value = values.remove(0).owned_tokens().unwrap_or_default();
    let value_str = value.to_string().to_lowercase();
    if value_str == "true" {
      macroname_true.clone()
    } else {
      macroname_false.clone() };
    let mut tokens = vec![];
    // Store and invoke the original macro if needed
    if let Some(ref code) = code_opt {
      def_macro(orig_cs.clone(), plain_params.clone(), code.clone(), None, istate);
      tokens.push(orig_cs.clone());
      tokens.push(T_BEGIN!());
      tokens.extend(value.unlist());
      tokens.push(T_END!());
    }
    Ok(Tokens::new(tokens))
  });

  define_choice(qname, Some(ExpansionBody::Closure(closure)),
    mismatch, vec!["true", "false"], true, None, state)
}