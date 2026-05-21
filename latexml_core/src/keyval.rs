//! Key-Value Definitions
//!
//! Provides an interface to define and access KeyVal definition.
//! Used in conjunction with `KeyVals` to
//!  fully implement KeyVal pairs.

use std::borrow::Cow;
use std::rc::Rc;

use crate::binding::def::dialect::{def_conditional, def_macro};
use crate::common::def_parser::parse_parameters;
use crate::common::error::*;
use crate::common::store::Stored;
use crate::definition::argument::ArgWrap;
use crate::definition::conditional::ConditionalOptions;
use crate::definition::{ExpansionBody, ExpansionClosure};
use crate::mouth::tokenize;
use crate::parameter::Parameter;
use crate::state;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;

#[derive(Debug, Clone, PartialEq)]
pub struct KeyVal {
  // which KeyVals are we parsing and how do we behave?
  prefix: String,
  key:    String,
  keyset: String,
}

impl Default for KeyVal {
  fn default() -> Self {
    KeyVal {
      prefix: "KV".to_string(),
      keyset: String::new(),
      key:    String::new(),
    }
  }
}

impl KeyVal {
  pub fn new(prefix: Option<String>, keyset: String, key: String) -> Self {
    let prefix = prefix.unwrap_or_else(|| "KV".to_string());
    KeyVal { prefix, key, keyset }
  }

  pub fn get_header(&self) -> String { s!("{}@{}@{}", self.prefix, self.keyset, self.key) }

  //======================================================================
  // Property access
  //======================================================================

  pub fn get_prop(&self, key: &str) -> Option<Stored> {
    state::lookup_value(&s!("KEYVAL@{}@{}", key, self.get_header()))
  }
  pub fn get_default(&self) -> Option<Stored> { self.get_prop("default") }
  pub fn get_type(&self) -> Option<Rc<Parameter>> {
    // Read directly via with_value — avoids the Stored::clone that
    // get_prop's lookup_value pays just so we can pattern-match on
    // the Parameter variant and Rc::clone its body. Hot path during
    // keyval parsing.
    state::with_value(&s!("KEYVAL@type@{}", self.get_header()), |v| match v {
      Some(Stored::Parameter(p)) => Some(Rc::clone(p)),
      _ => None,
    })
  }
}

// semi-internals
pub(crate) fn keyval_qname(prefix: &str, keyset: &str, key: &str) -> String {
  let prefix = if prefix.is_empty() { "KV" } else { prefix };
  s!("{prefix}@{keyset}@{key}")
}

pub(crate) fn keyval_get(qname: &str, prop: &str) -> Option<Stored> {
  state::lookup_value(&s!("KEYVAL@{prop}@{qname}"))
}

/// Using local assignments in the State to set keyvals, but that necessitates a
/// cast of `ArgWrap`` to `State`` on set and `State` to `ArgWrap` on get.
/// Certain values don't really work well with that, e.g. Stored::Bool ...
/// The original Perl made no type casts, as there weren't any concrete types.
pub(crate) fn keyval_set(qname: &str, prop: &str, value: Stored) {
  state::assign_value(&s!("KEYVAL@{prop}@{qname}"), value, None);
}

/// check if a key-value pair is defined
pub fn has_keyval(prefix: &str, keyset: &str, key: &str) -> bool {
  let qname = keyval_qname(prefix, keyset, key);
  state::with_value(&s!("KEYVAL@defined@{}", qname), |v| v.is_some())
    || state::has_meaning(&T_CS!(s!("\\{qname}")))
}

/// disable a given key-val
pub fn disable_keyval(prefix: &str, keyset: &str, key: &str) -> Result<()> {
  let qname = keyval_qname(prefix, keyset, key);
  keyval_set(&qname, "disabled", true.into());
  // disable the key
  define_ordinary(
    &qname,
    Some(ExpansionBody::Tokens(tokenize(&s!(
      "\\PackageWarning{{keyval}}{{`{key}' has been disabled. }}"
    )))),
  )
}

//======================================================================
// Key Definition
//======================================================================
#[derive(Debug, Default, Clone)]
/// Configuration fields for declaring a new KeyVal pattern
pub struct KeyvalConfig<'a> {
  pub prefix:      &'a str,
  pub keyset:      &'a str,
  pub key:         &'a str,
  pub vtype:       &'a str,
  pub default:     Option<&'a str>,
  pub kind:        Option<&'a str>,
  pub code:        Option<ExpansionBody>,
  pub macroprefix: Option<&'a str>,
  pub mismatch:    Option<ExpansionBody>,
  pub normalize:   Option<bool>,
  pub bin:         Option<Tokens>,
  pub choices:     Vec<&'static str>,
}

/// Register a keyval qname in the global registry for enumeration by \xkvview.
fn register_keyval(qname: &str) {
  use crate::common::arena;
  let registry_key = "KEYVAL@registry";
  // Borrow the registry via `with_value` to skip the outer Stored::clone
  // (lookup_value clones the enum; we only need the inner Vec<SymStr>).
  let mut registry: Vec<crate::common::arena::SymStr> =
    state::with_value(registry_key, |v| match v {
      Some(Stored::Strings(v)) => v.to_vec(),
      _ => Vec::new(),
    });
  let sym = arena::pin(qname);
  // avoid duplicates (re-definitions)
  if !registry.contains(&sym) {
    registry.push(sym);
  }
  state::assign_value(registry_key, Stored::Strings(registry.into()), None);
}

/// Metadata for a registered keyval, used by \xkvview.
#[derive(Debug, Clone)]
pub struct KeyvalMeta {
  pub key:     String,
  pub prefix:  String,
  pub keyset:  String,
  pub kind:    String,
  pub default: String,
}

/// Enumerate all registered keyvals with their metadata (for \xkvview).
pub fn enumerate_keyvals() -> Vec<KeyvalMeta> {
  use crate::common::arena;
  let registry_key = "KEYVAL@registry";
  let registry = state::with_value(registry_key, |v| match v {
    Some(Stored::Strings(v)) => v.to_vec(),
    _ => Vec::new(),
  });
  if registry.is_empty() {
    return Vec::new();
  }
  let mut result = Vec::new();
  for sym in registry {
    // Resolve the interned qname once via a closure — the five
    // keyval_get calls below all take `&str`, so we hand each the same
    // resolved borrow rather than allocating a per-key String.
    let entry = arena::with(sym, |qname| {
      let key = keyval_get(qname, "key_name")
        .map(|s| s.to_string())
        .unwrap_or_default();
      let prefix = keyval_get(qname, "keyval_prefix")
        .map(|s| s.to_string())
        .unwrap_or_else(|| "KV".to_string());
      let keyset = keyval_get(qname, "keyset")
        .map(|s| s.to_string())
        .unwrap_or_default();
      let kind = keyval_get(qname, "kind")
        .map(|s| s.to_string())
        .unwrap_or_else(|| "ordinary".to_string());
      let default = keyval_get(qname, "default")
        .map(|s| s.to_string())
        .unwrap_or_else(|| "[none]".to_string());
      KeyvalMeta {
        key,
        prefix,
        keyset,
        kind,
        default,
      }
    });
    result.push(entry);
  }
  result
}

/// (Re-)defines this Key of kind 'kind'.
///
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
pub fn define(options: KeyvalConfig) -> Result<()> {
  let KeyvalConfig {
    prefix,
    keyset,
    key,
    vtype,
    default,
    kind,
    code,
    macroprefix,
    mismatch,
    normalize,
    bin,
    choices,
  } = options;

  let qname = keyval_qname(prefix, keyset, key);

  // define that the key exists and is not disabled
  keyval_set(&qname, "exists", true.into());
  keyval_set(&qname, "disabled", false.into());
  // store metadata for introspection (used by \xkvview)
  // only register when xkvview tracking is enabled
  if state::lookup_bool("XKVVIEW_TRACKING") {
    keyval_set(
      &qname,
      "kind",
      Stored::Tokens(tokenize(kind.unwrap_or("ordinary"))),
    );
    keyval_set(&qname, "keyval_prefix", Stored::Tokens(tokenize(prefix)));
    keyval_set(&qname, "keyset", Stored::Tokens(tokenize(keyset)));
    keyval_set(&qname, "key_name", Stored::Tokens(tokenize(key)));
    register_keyval(&qname);
  }
  // set the type
  let vtype = if vtype.is_empty() { "{}" } else { vtype };
  let paramlist_opt = parse_parameters(
    vtype,
    &T_OTHER!(s!("KeyVal {key} in set {keyset} with prefix {prefix}")),
    true,
  )?;
  match paramlist_opt {
    None => {
      Warn!(
        "unexpected",
        "keyval",
        s!(
          "No parameters in keyval {key} (in set {keyset} with prefix {prefix}) taking only first"
        )
      );
    },
    Some(paramlist) => {
      if paramlist.get_num_args() != 1 {
        Warn!(
          "unexpected",
          "keyval",
          s!(
            "Too many parameters in keyval {key} (in set {keyset} with prefix {prefix})\
          taking only first"
          )
        );
      }
      keyval_set(&qname, "type", paramlist.take_parameters().remove(0).into());
    },
  };
  // set the default
  // Question: Why was $default converted ToString ???
  if let Some(default_str) = default {
    let default_tks = tokenize(default_str);
    keyval_set(&qname, "default", Stored::Tokens(default_tks.clone()));
    def_macro(
      T_CS!(s!("\\{qname}@default")),
      None,
      ExpansionBody::Tokens(Tokens!(
        T_CS!(s!("\\{qname}")),
        T_BEGIN!(),
        default_tks,
        T_END!()
      )),
      None,
    )?;
  }

  // figure out the kind of key-val parameter we are defining
  let kind = kind.unwrap_or("ordinary");
  match kind {
    "ordinary" => define_ordinary(&qname, code)?,
    "command" => {
      // Perl #2777 (2026-03-27): macroprefix falls back to "cmd"+qname
      // when undefined OR empty. The truthy check that existed pre-fix
      // already treated empty-string as falsy; we match that semantics
      // explicitly for Option<&str>.
      let macroname = match macroprefix {
        Some(mpfx) if !mpfx.is_empty() => s!("{mpfx}{key}"),
        _ => s!("cmd{qname}"),
      };
      define_command(&qname, code, &macroname)?;
    },
    "choice" => define_choice(
      &qname,
      code,
      mismatch,
      choices,
      normalize.unwrap_or(false),
      bin,
    )?,
    "boolean" => define_boolean(
      &qname,
      code,
      mismatch,
      &if let Some(mpfx) = macroprefix {
        Cow::Owned(s!("{mpfx}{key}"))
      } else {
        Cow::Borrowed(&qname)
      },
    )?,
    _ => Warn!(
      "unknown",
      "undef",
      s!(
        "Unknown KeyVals kind {kind} should be one of 'ordinary', 'command', 'choice', 'boolean'. "
      )
    ),
  };
  Ok(())
}

/// Helper function to define state, neccesary for an ordinary key.
fn define_ordinary(qname: &str, code_expansion: Option<ExpansionBody>) -> Result<()> {
  let qname_cs = T_CS!(s!("\\{qname}"));
  let plain_params = parse_parameters("{}", &qname_cs, true)?;
  def_macro(qname_cs, plain_params, code_expansion, None)
}

/// Helper function to define state, neccesary for a command key.
fn define_command(qname: &str, code: Option<ExpansionBody>, macroname: &str) -> Result<()> {
  let qname_cs = T_CS!(s!("\\{qname}"));
  let plain_params = parse_parameters("{}", &qname_cs, true)?;
  let plainp = plain_params.clone();
  let orig = s!("\\ltxml@orig@{qname}");
  let macroname_cs = s!("\\{macroname}");
  let closure: ExpansionClosure = Rc::new(move |value: Vec<ArgWrap>| {
    def_macro(T_CS!(&orig), plainp.clone(), code.clone(), None)?;
    let value_tks: Vec<Token> = value
      .into_iter()
      .flat_map(|v| {
        v.owned_tokens()
          .map(|inner| inner.unlist())
          .unwrap_or_default()
      })
      .collect();
    // $value !?!??! Is it a number 1--9 ???)
    Ok(Tokens!(
      T_CS!("\\def"),
      T_CS!(&macroname_cs),
      T_BEGIN!(),
      value_tks.clone(),
      T_END!(),
      T_CS!(&orig),
      T_BEGIN!(),
      T_PARAM!(),
      value_tks,
      T_END!()
    ))
  });
  def_macro(
    qname_cs,
    plain_params,
    ExpansionBody::Closure(closure),
    None,
  )
}

/// Helper function to define state, neccesary for an choice key.
fn define_choice(
  qname: &str,
  code_opt: Option<ExpansionBody>,
  mismatch_opt: Option<ExpansionBody>,
  choices: Vec<&'static str>,
  normalize: bool,
  bin: Option<Tokens>,
) -> Result<()> {
  let (varmacro_opt, idxmacro_opt) = if let Some(bin_tks) = bin {
    let mut bin_iter = bin_tks.unlist().into_iter();
    (bin_iter.next(), bin_iter.next())
  } else {
    (None, None)
  };
  let qname_cs = T_CS!(s!("\\{qname}"));
  let orig = T_CS!(s!("\\ltxml@orig@{qname}"));
  let plain_params = parse_parameters("{}", &qname_cs, true)?;
  let plain_params_main = plain_params.clone();
  let closure: ExpansionClosure = Rc::new(move |mut values| {
    // Store the normalized value (if applicable)
    let value = values.remove(0).owned_tokens().unwrap_or_default();
    let mut nvalue = value.to_string();
    if normalize {
      nvalue = nvalue.to_lowercase();
    }
    if let Some(varmacro) = varmacro_opt {
      def_macro(
        varmacro,
        None,
        ExpansionBody::Tokens(Tokens::new(Explode!(nvalue))),
        None,
      )?;
    }
    // iterate over the possible choices and store them
    let mut valid = false;
    for (index, choice_str) in choices.iter().enumerate() {
      if (normalize && (choice_str.to_lowercase() == nvalue)) || *choice_str == nvalue {
        valid = true;
        if let Some(idxmacro) = idxmacro_opt {
          def_macro(
            idxmacro,
            None,
            ExpansionBody::Tokens(Tokens::new(Explode!(index))),
            None,
          )?;
        }
      }
    }
    // find a name for the original macro to store in
    let mut tokens = Vec::new();
    // if we have chosen a valid index, run $code
    if valid {
      if let Some(ref code) = code_opt {
        def_macro(orig, plain_params.clone(), code.clone(), None)?;
        tokens.push(orig);
        tokens.push(T_BEGIN!());
        tokens.extend(value.unlist());
        tokens.push(T_END!());
      }
    } else if let Some(ref mismatch) = mismatch_opt {
      // else run `mismatch
      def_macro(orig, plain_params.clone(), mismatch.clone(), None)?;
      tokens.push(orig);
      tokens.push(T_BEGIN!());
      tokens.extend(value.unlist());
      tokens.push(T_END!());
    }
    Ok(Tokens::new(tokens))
  });
  def_macro(
    qname_cs,
    plain_params_main,
    ExpansionBody::Closure(closure),
    None,
  )
}

/// Helper function to define state, neccesary for a boolean key.
fn define_boolean(
  qname: &str,
  code_opt: Option<ExpansionBody>,
  mismatch: Option<ExpansionBody>,
  macroname: &str,
) -> Result<()> {
  def_conditional(
    T_CS!(s!("\\if{macroname}")),
    None,
    None,
    ConditionalOptions::default(),
  )?; // We might need to $scope here
  let orig = s!("\\ltxml@@rig@{qname}");
  let orig_cs = T_CS!(orig);
  let plain_params = parse_parameters("{}", &orig_cs, true)?;
  let macroname_true = T_CS!(s!("\\{macroname}true"));
  let macroname_false = T_CS!(s!("\\{macroname}false"));
  let closure: ExpansionClosure = Rc::new(move |mut values: Vec<ArgWrap>| {
    // set the conditional to true/false
    let value = values.remove(0).owned_tokens().unwrap_or_default();
    let value_str = value.to_string().to_lowercase();
    let mut tokens = vec![];
    // Toggle the conditional by invoking \XXXtrue or \XXXfalse
    if value_str == "true" {
      tokens.push(macroname_true);
    } else {
      tokens.push(macroname_false);
    }
    // Store and invoke the original macro if needed
    if let Some(ref code) = code_opt {
      def_macro(orig_cs, plain_params.clone(), code.clone(), None)?;
      tokens.push(orig_cs);
      tokens.push(T_BEGIN!());
      tokens.extend(value.unlist());
      tokens.push(T_END!());
    }
    Ok(Tokens::new(tokens))
  });

  define_choice(
    qname,
    Some(ExpansionBody::Closure(closure)),
    mismatch,
    vec!["true", "false"],
    true,
    None,
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn keyval_default_fields() {
    let kv = KeyVal::default();
    assert_eq!(kv.prefix, "KV");
    assert!(kv.keyset.is_empty());
    assert!(kv.key.is_empty());
  }

  #[test]
  fn keyval_new_custom_prefix() {
    let kv = KeyVal::new(
      Some("custom".to_string()),
      "ks".to_string(),
      "k".to_string(),
    );
    assert_eq!(kv.prefix, "custom");
    assert_eq!(kv.keyset, "ks");
    assert_eq!(kv.key, "k");
  }

  #[test]
  fn keyval_new_default_prefix_on_none() {
    // None prefix → default "KV".
    let kv = KeyVal::new(None, "ks".to_string(), "k".to_string());
    assert_eq!(kv.prefix, "KV");
  }

  #[test]
  fn keyval_get_header_format() {
    let kv = KeyVal::new(
      Some("P".to_string()),
      "set".to_string(),
      "width".to_string(),
    );
    assert_eq!(kv.get_header(), "P@set@width");
  }

  #[test]
  fn keyval_get_header_default_prefix() {
    let kv = KeyVal::new(None, "tabular".to_string(), "vattach".to_string());
    assert_eq!(kv.get_header(), "KV@tabular@vattach");
  }

  #[test]
  fn keyval_equality_by_all_fields() {
    let a = KeyVal::new(Some("P".to_string()), "ks".to_string(), "k".to_string());
    let b = KeyVal::new(Some("P".to_string()), "ks".to_string(), "k".to_string());
    let c = KeyVal::new(Some("P".to_string()), "ks".to_string(), "other".to_string());
    assert_eq!(a, b);
    assert_ne!(a, c);
  }

  #[test]
  fn keyval_qname_normalizes_empty_prefix() {
    // Empty prefix is substituted with "KV".
    assert_eq!(keyval_qname("", "set", "k"), "KV@set@k");
    assert_eq!(keyval_qname("P", "set", "k"), "P@set@k");
  }
}
