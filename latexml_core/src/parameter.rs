use once_cell::sync::Lazy;
#[cfg(feature = "codegen")]
use proc_macro2::TokenStream;
#[cfg(feature = "codegen")]
use quote::{ToTokens, quote};
use regex::Regex;
use std::fmt;
use std::rc::Rc;

use crate::Digested;
use crate::common::arena::{self, SymStr};
use crate::common::error::*;
use crate::common::object::Object;
use crate::definition::argument::ArgWrap;
use crate::definition::constructor::Constructor;
use crate::definition::{BeforeDigestClosure, Definition, DigestionClosure};
use crate::gullet;
use crate::mouth::Mouth;
use crate::pin;
use crate::state::*;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;

pub type ReaderFn = dyn Fn(Option<&Parameters>, &[Tokens]) -> Result<ArgWrap>;
pub type ReaderPredigestFn = dyn Fn(ArgWrap, &[Tokens]) -> Result<Option<Digested>>;
pub type ReaderPredigestClosure = Rc<ReaderPredigestFn>;
pub type ReaderClosure = Rc<ReaderFn>;

// Rust Note:
// the reversion functions initially had "&mut Gullet" as a parameter.
// This turned out to be infeasible if we are to maintain the latexml code flow
// as we have calls into reversions from arbitrary binding closures, at ALL phases.
// Compromise: use the gated Stomach in state::whenever you need gullet in reversion, as in
// let mut stomach = state::stomach.borrow_mut();
//
//
pub type ReversionClosure =
  Rc<dyn Fn(Vec<Token>, Option<&Parameters>, &[Tokens]) -> Result<Tokens>>;

/// A reversion closure that operates on the original Digested argument,
/// enabling access to structured data (e.g., KeyVals) for custom reversion formatting.
/// Perl equivalent: the `reversion` option on DefParameterType, which receives the raw value.
pub type DigestedReversionClosure = Rc<dyn Fn(&Digested) -> Result<Tokens>>;

static LAST_WCHAR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\w$").unwrap());
static FIRST_WCHAR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\w").unwrap());

#[derive(Clone)]
pub struct Parameter {
  pub novalue:            bool,
  pub semiverbatim:       Option<Vec<char>>,
  pub optional:           bool,
  pub name:               SymStr,
  pub spec:               SymStr,
  pub extra:              Vec<Tokens>,
  pub inner:              Option<Parameters>,
  pub reader:             ReaderClosure,
  pub predigest:          Option<ReaderPredigestClosure>,
  pub reversion:          Option<ReversionClosure>,
  /// Reversion closure that operates on the original Digested argument.
  /// Takes precedence over `reversion` when the argument is a digested value.
  /// Perl equivalent: `reversion` option on DefParameterType with `undigested => 1`.
  pub digested_reversion: Option<DigestedReversionClosure>,
  pub before_digest:      Vec<BeforeDigestClosure>,
  pub after_digest:       Vec<DigestionClosure>,
}
impl Default for Parameter {
  fn default() -> Self {
    Parameter {
      novalue:            false,
      semiverbatim:       None,
      optional:           false,
      name:               arena::pin_static("parameter_default"),
      spec:               pin!(""),
      extra:              Vec::new(),
      inner:              None,
      reader:             Rc::new(|_args, _extra| {
        Warn!(
          "Parameter",
          "mock_reader",
          "Please define a real reader, this is a mock fallback!"
        );
        Ok(ArgWrap::None)
      }),
      predigest:          None,
      reversion:          None,
      digested_reversion: None,
      before_digest:      Vec::new(),
      after_digest:       Vec::new(),
    }
  }
}
impl fmt::Debug for Parameter {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(
      f,
      "Parameter(\n\t name:{:?}, novalue:{:?}, semiverbatim:{:?},",
      self.name, self.novalue, self.semiverbatim,
    )?;
    writeln!(f, "\t optional:{:?}, spec:{:?}", self.optional, self.spec)?;
    writeln!(f, "\t inner: {:?}", self.inner)?;
    writeln!(
      f,
      "\t extra: {:?}\n\t reversion: {:?}, before_digest: {:?}, after_digest: {:?} )",
      self.extra,
      self.reversion.is_some(),
      self.before_digest.len(),
      self.after_digest.len()
    )
  }
}
impl fmt::Display for Parameter {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    arena::with(self.name, |name| write!(f, "{name}"))
  }
}

impl PartialEq for Parameter {
  fn eq(&self, other: &Parameter) -> bool { self.name == other.name }
}
impl Object for Parameter {
  fn stringify(&self) -> String { arena::to_string(self.spec) }
}

static OPTIONAL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^Optional(.+)$").unwrap());
static SKIP_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^Skip(.+)$").unwrap());

impl Parameter {
  pub fn new<T: AsRef<str>>(name: T, spec: T, extra: Option<Vec<Tokens>>) -> Result<Self> {
    Parameter {
      name: arena::pin(name),
      spec: arena::pin(spec),
      extra: extra.unwrap_or_default(),
      ..Parameter::default()
    }
    .init()
  }
  pub fn init(mut self) -> Result<Self> {
    // Create a parameter reading object for a specific type.
    // If either a declared entry or a function Read<Type> accessible from LaTeXML::Package::Pool
    // is defined.
    let mut descriptor: Option<Rc<Parameter>> = with_mapping_sym(
      arena::pin_static("PARAMETER_TYPES"),
      self.name,
      |looked_up_mapping| {
        if let Some(Stored::Parameter(d_lookup)) = looked_up_mapping {
          Some(Rc::clone(d_lookup))
        } else {
          None
        }
      },
    );
    if descriptor.is_none() {
      // TODO: see discussion on line 168
      let basetype_opt = arena::with(self.name, |name| {
        OPTIONAL_REGEX
          .captures(name)
          .map(|captures| captures.get(1).map_or("", |m| m.as_str()).to_string())
      })
      .map(arena::pin);
      if let Some(basetype) = basetype_opt {
        descriptor = with_mapping_sym(
          arena::pin_static("PARAMETER_TYPES"),
          basetype,
          |basetype_param_opt| match basetype_param_opt {
            Some(Stored::Parameter(d_lookup)) => Ok(Some(d_lookup.clone())),
            _ => match Parameter::check_reader_function(&arena::with(self.name, |name| {
              s!("Read{name}")
            })) {
              Some(reader) => Ok(Some(Rc::new(Parameter {
                reader,
                optional: true,
                ..Parameter::default()
              }))),
              None => match Parameter::check_reader_function(&arena::with(basetype, |type_str| {
                s!("Read{type_str}")
              })) {
                Some(reader) => Ok(Some(Rc::new(Parameter {
                  reader,
                  optional: true,
                  novalue: true,
                  ..Parameter::default()
                }))),
                None => fatal!(
                  Parameter,
                  Init,
                  s!("Can't initialize parameter {:?}, unknown?", self.name)
                ),
              },
            },
          },
        )?;
        self.optional = true;
      } else {
        // TODO: This looks like a code smell. Do we need a new arena method?
        // We start with a ticket, do a non-allocation operation on the underlying &str,
        // Then want to ping the newly acquired &str slice in the arena. Clearly that is only
        // possible *AFTER* the original &str is released, but how do we avoid allocating?
        // Is this a use for unsafe{} or is there a more idiomatic way?
        // Maybe, with_mut... which allows an inner pin?
        let basetype_opt = arena::with(self.name, |name| {
          SKIP_REGEX
            .captures(name)
            .map(|captures| captures.get(1).map_or("", |m| m.as_str()).to_string())
        })
        .map(arena::pin);
        if let Some(basetype) = basetype_opt {
          descriptor = with_mapping_sym(
            arena::pin_static("PARAMETER_TYPES"),
            basetype,
            |basetype_param_opt| match basetype_param_opt {
              Some(Stored::Parameter(d_lookup)) => Some(d_lookup.clone()),
              _ => match arena::with(self.name, |name| Parameter::check_reader_function(name)) {
                Some(reader) => Some(Rc::new(Parameter {
                  reader,
                  optional: true,
                  novalue: true,
                  ..Parameter::default()
                })),
                None => Parameter::check_reader_function(&arena::with(basetype, |type_str| {
                  s!("Read{type_str}")
                }))
                .map(|reader| {
                  Rc::new(Parameter {
                    reader,
                    optional: true,
                    novalue: true,
                    ..Parameter::default()
                  })
                }),
              },
            },
          );
          if let Some(ref _desc) = descriptor {
            self.novalue = true;
            self.optional = true;
          }
        } else {
          descriptor =
            Parameter::check_reader_function(&arena::with(self.name, |name| s!("Read{name}")))
              .map(|reader| Rc::new(Parameter { reader, ..Parameter::default() }));
        }
      }
    }
    match descriptor {
      Some(descriptor) => {
        // descriptor needs to get integrated into Self
        //  except `spec` and `name` which are always preserved!
        self.reader = descriptor.reader.clone(); // What else?
        if descriptor.novalue {
          self.novalue = true;
        }
        self.semiverbatim.clone_from(&descriptor.semiverbatim);
        // Also doing optional setting on the fly, so don't override unless true
        // self.optional = descriptor.optional;
        if descriptor.optional {
          self.optional = true;
        }
        self.reversion.clone_from(&descriptor.reversion);
        self
          .digested_reversion
          .clone_from(&descriptor.digested_reversion);
        self.before_digest.clone_from(&descriptor.before_digest);
        self.after_digest.clone_from(&descriptor.after_digest);
        self.predigest.clone_from(&descriptor.predigest);
      },
      None => fatal!(
        Parameter,
        Unknown,
        arena::with2(self.name, self.spec, |name, spec| s!(
          "Unrecognized parameter type with name {:?}, spec {:?}",
          name,
          spec
        ))
      ),
    }
    // Last but not least, initialize any "inner" parameters
    self.inner = self.inner.map(|inner_ps| match inner_ps.clone().init() {
      Ok(ps) => ps,
      Err(e) => {
        log::warn!("inner parameter init failed: {e}");
        inner_ps
      },
    });
    Ok(self)
  }

  /// Obtain the reader of a given parameter name, if available
  pub fn check_reader_function(name: &str) -> Option<ReaderClosure> {
    // TODO: This function doesn't have a direct Rust equivalent, since the metaprogramming isn't
    // possible But what is the exact purpose of seeking through the pool namespace? Wouldn't
    // any parameter be already assigned in the state::
    with_mapping("PARAMETER_TYPES", name, |param_opt| {
      if let Some(Stored::Parameter(param)) = param_opt {
        Some(param.reader.clone())
      } else {
        None
      }
    })
  }

  pub fn setup_catcodes(&self) {
    if self.semiverbatim.is_some() {
      begin_semiverbatim(self.semiverbatim.as_deref());
    }
  }

  pub fn revert_catcodes(&self) -> Result<()> {
    if self.semiverbatim.is_some() {
      end_semiverbatim()?;
    }
    Ok(())
  }

  pub fn read(&self, fordefn: Option<&dyn Definition>) -> Result<ArgWrap> {
    // For semiverbatim, I had messed with catcodes, but there are cases
    // (eg. \caption(...\label{badchars}}) where you really need to
    // cleanup after the fact!
    // Hmmm, seem to still need it...
    self.setup_catcodes();

    let closure = &self.reader;
    let value_from_reader: ArgWrap = closure(self.inner.as_ref(), &self.extra)?;
    // Direct enum destructure: was `is_tokens() then owned_tokens()`
    // which matched twice (once for the is_tokens check, again for
    // the owned_tokens dispatch over all ArgWrap variants). This
    // function fires on every parameter read of every macro call —
    // ~2M times on si.tex per callgrind.
    let value_arg = match value_from_reader {
      ArgWrap::Tokens(mut value) => {
        if let Some(ref semi_chars) = self.semiverbatim {
          value = value.neutralize(semi_chars);
        }
        ArgWrap::Tokens(value)
      },
      other => other,
    };
    self.revert_catcodes()?;

    // Single arena borrow to compute both name-prefix checks that
    // this function needs — was two separate `arena::with` calls
    // (each a RefCell borrow + interner resolve), now a single
    // closure that returns the pair.
    let (is_optional_match, is_until) = arena::with(self.name, |name| {
      (name.starts_with("OptionalMatch"), name.starts_with("Until"))
    });

    // Perl: experiment: skip spaces after a successful OptionalMatch read
    if !value_arg.is_none() && self.optional && is_optional_match {
      gullet::skip_spaces()?;
    }

    let checked_value =
      if !self.optional && !self.novalue && (value_arg.is_none() && self.predigest.is_none()) {
        // Deyan: Special exception, which may motivate switching the reader type to Option<Tokens>
        // in the long-run        Until *may* have a value, but it also may *not*, both OK.
        // So... except it from the error message here
        if !is_until {
          let fordefn_str = fordefn.map(|fdefn| fdefn.stringify()).unwrap_or_default();
          Error!(
            "expected",
            self,
            s!("Missing argument {} for {}", self.stringify(), fordefn_str)
          );
          ArgWrap::Tokens(Tokens!(T_OTHER!("missing")))
        } else {
          value_arg
        }
      } else {
        value_arg
      };
    Ok(checked_value)
  }

  pub fn digest(
    &self,
    mut value_arg: ArgWrap,
    _fordefn: Option<&Constructor>,
  ) -> Result<Option<Digested>> {
    // Perl Parameter.pm lines 122,139-141: capture MODE, check after digest
    let mode = crate::state::lookup_string_from_sym(crate::pin!("MODE"));
    // If semiverbatim, Expand (before digest), so tokens can be neutralized; BLECH!!!!
    if self.semiverbatim.is_some() {
      self.setup_catcodes();
      if value_arg.is_tokens() {
        if let Some(value) = value_arg.owned_tokens() {
          let neutralized = gullet::reading_from_mouth(Mouth::default(), move || {
            gullet::unread(value);
            let mut tokens = Vec::new();
            loop {
              match gullet::get_pending_comment() {
                Some(token) => tokens.push(token),
                None => match gullet::read_x_token(Some(true), false, None) {
                  Ok(token_opt) => match token_opt {
                    Some(token) => tokens.push(token),
                    None => break,
                  },
                  Err(x) => return Err(x),
                },
              }
            }
            Ok(Tokens::new(tokens).neutralize(&[]))
          })?;
          value_arg = ArgWrap::Tokens(neutralized);
        } else {
          value_arg = ArgWrap::default();
        }
      }
    }

    for pre in self.before_digest.iter() {
      // Done for effect only.
      pre()?; // maybe pass extras?
    }
    let digested_value = if let Some(ref closure) = &self.predigest {
      closure(value_arg, &self.extra)?
    } else {
      // Note: we have an open question for the type interface.
      //  What happens when a wrapped "None" value,
      // (such as the missing value of an Optional [] argument)
      // gets digested?
      //
      // currently a `Digested::default` gets returned, which has an empty TBox and also gets
      // returned for e.g. empty mandatory Plain arguments {}.
      // But we need *different* values, as the explicit "\foo[]" is an override to empty, while
      // "\foo" will use the default value for the Optional.
      if self.optional && value_arg.is_none() {
        None
      } else {
        Some(value_arg.be_digested()?)
      }
    };
    for post in self.after_digest.iter() {
      // Done for effect only.
      let mut w = Whatsit::default();
      post(&mut w)?; // maybe pass extras?
    }

    self.revert_catcodes()?;

    // Perl Parameter.pm lines 139-141: avoid mode change leaking out of parameter digestion
    let newmode = crate::state::lookup_string_from_sym(crate::pin!("MODE"));
    if mode != newmode && mode != "horizontal" {
      crate::stomach::leave_horizontal_internal();
    }

    Ok(digested_value)
  }

  pub fn revert(&self, value_opt: Option<Tokens>) -> Result<Option<Tokens>> {
    if let Some(ref reverter) = self.reversion {
      if let Some(value) = value_opt {
        Ok(Some((reverter)(
          value.unlist(),
          self.inner.as_ref(),
          &self.extra,
        )?))
      } else {
        Ok(None)
      }
    } else if let Some(value) = value_opt {
      Ok(Some(Tokens::new(value.revert())))
    } else {
      Ok(None)
    }
  }

  /// This is needed by structured parameter types like KeyVals
  /// where the argument may already have been tokenized before the KeyVals
  /// (and the parameter types for the keys) had a chance to properly parse.
  // Yuck!
  pub fn reparse(&self, tokens: Tokens) -> Result<ArgWrap> {
    // Needs neutralization, since the keyvals may have been tokenized already???
    // perhaps a better test would involve whether $tokens is, in fact, Tokens?
    if self.name == pin!("Plain") || self.predigest.is_some() {
      // Gack!
      Ok(ArgWrap::Tokens(tokens))
    } else if self.semiverbatim.is_some() {
      // Needs neutralization
      // but maybe specific to catcodes
      Ok(ArgWrap::Tokens(
        tokens.neutralize(self.semiverbatim.as_ref().unwrap().as_slice()),
      ))
    } else {
      gullet::reading_from_mouth(Mouth::default(), || {
        // start with empty mouth
        let mut tokens = tokens.unlist();
        if !tokens.is_empty() // Strip outer braces from dimensions & friends
          && arena::with(self.name,|name|
              matches!(name, "Number"|"Dimension"|"Glue"|"MuDimension"|"MuGlue"))
          && tokens.first().map(|t| t.get_catcode() == Catcode::BEGIN)
              .unwrap_or(false)
          && tokens.last().map(|t| t.get_catcode() == Catcode::END).unwrap_or(false)
        {
          tokens.remove(0);
          tokens.pop();
        }
        gullet::unread_vec(tokens); // but put back tokens to be read
        let value = self.read(None)?;
        gullet::skip_spaces()?;
        Ok(value)
      })
    }
  }
}

#[derive(Clone, Debug, Default)]
pub struct Parameters(Vec<Parameter>);

impl PartialEq for Parameters {
  fn eq(&self, other: &Parameters) -> bool { self.0 == other.0 }
}
impl Object for Parameters {
  fn stringify(&self) -> String {
    let mut result = String::new();
    for parameter in self.0.iter() {
      let s = parameter.stringify();
      let lead_letter = match s.chars().next() {
        Some(c) => c.is_alphanumeric(),
        None => false,
      };
      let trail_letter = match result.chars().last() {
        Some(c) => c.is_alphanumeric(),
        None => false,
      };
      if lead_letter && trail_letter {
        result.push(' ');
      }
      result.push_str(&s);
    }
    result
  }
}

impl Parameters {
  pub fn new(params: Vec<Parameter>) -> Self { Parameters(params) }
  pub fn get_num_args(&self) -> usize { self.0.iter().filter(|&p| !p.novalue).count() }
  pub fn get_parameters(&self) -> Vec<&Parameter> { self.0.iter().collect() }
  pub fn take_parameters(self) -> Vec<Parameter> { self.0 }
  pub fn revert_arguments(&self, args: Vec<Option<Tokens>>) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    for (parameter, arg) in self.0.iter().zip(args) {
      if !parameter.novalue {
        if let Some(reverted_tks) = parameter.revert(arg)? {
          tokens.extend(reverted_tks.unlist());
        }
      }
    }
    Ok(tokens)
  }

  /// Revert arguments from their digested form, using `digested_reversion` when available.
  /// This allows parameter types (like BoxSpecification) to control reversion formatting
  /// based on the structured digested data rather than token-level reversion.
  /// Perl equivalent: `$parameters->revertArguments($self->getArgs)`
  pub fn revert_digested_arguments(
    &self,
    digested_args: &[Option<Digested>],
  ) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    for (parameter, arg_opt) in self.0.iter().zip(digested_args) {
      if !parameter.novalue {
        let reverted = if let Some(ref digested_rev) = parameter.digested_reversion {
          // Use digested_reversion: operates on the raw Digested value
          match arg_opt {
            Some(arg) => Some(digested_rev(arg)?),
            None => None,
          }
        } else {
          // Fall back to standard reversion: Digested → Tokens → Parameter::revert
          let token_reverted = match arg_opt {
            Some(arg) => Some(arg.revert()?),
            None => None,
          };
          parameter.revert(token_reverted)?
        };
        if let Some(tks) = reverted {
          tokens.extend(tks.unlist());
        }
      }
    }
    Ok(tokens)
  }
  // Try to initialize each associated Parameter
  pub fn init(mut self) -> Result<Self> {
    let mut initialized = Vec::new();
    for param in self.0.drain(..) {
      initialized.push(param.init()?);
    }
    self.0 = initialized;
    Ok(self)
  }

  pub fn read_arguments(&self, fordefn: Option<&dyn Definition>) -> Result<Vec<ArgWrap>> {
    let mut args = Vec::with_capacity(self.0.len());
    for parameter in &self.0 {
      let values = parameter.read(fordefn)?;
      if parameter.predigest.is_some() {
        // TODO: Sometimes we legitimately want to use e.g. Number parameters without the predigest
        // closure... so this shouldn't be an error, not even an info -- but leaving it here
        // if something changes in the future. error!(
        //   target: &s!("parameter:{}", parameter.name),
        //   "parameter with predigest closure was invoked in an expandable context. Parameter
        // digestion won't execute." );
      }
      if !parameter.novalue {
        args.push(values);
      }
    }
    Ok(args)
  }

  pub fn read_arguments_and_digest(&self, fordefn: &Constructor) -> Result<Vec<Option<Digested>>> {
    let mut args = Vec::with_capacity(self.0.len());
    for parameter in &self.0 {
      let value = parameter.read(Some(fordefn))?;
      if !parameter.novalue {
        let digested_value = parameter.digest(value, Some(fordefn))?;
        args.push(digested_value);
      }
    }
    Ok(args)
  }

  pub fn reparse_argument(&self, value: ArgWrap) -> Result<Vec<ArgWrap>> {
    if value.is_none() {
      return Ok(Vec::new());
    }
    let value_tokens = value.revert()?;
    // start with empty mouth
    let reader_mouth = Mouth::new("", None)?;
    gullet::reading_from_mouth(reader_mouth, || {
      gullet::unread(value_tokens); // but put back tokens to be read
      let values = self.read_arguments(None)?;
      gullet::skip_spaces()?;
      Ok(values)
    })
  }

  pub fn as_keysets(&self) -> Vec<String> { self.0.iter().map(|p| p.stringify()).collect() }
}
impl fmt::Display for Parameters {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut content = String::new();
    for parameter in &self.0 {
      let param_content = parameter.to_string();
      if LAST_WCHAR_RE.is_match(&content) && FIRST_WCHAR_RE.is_match(&param_content) {
        content.push(' ');
      }
      content.push_str(&param_content);
    }
    write!(f, "{content}")
  }
}

impl From<Parameters> for Vec<Parameter> {
  fn from(ps: Parameters) -> Vec<Parameter> { ps.0 }
}

// ToTokens impls gated by `codegen` feature — see comment in
// `tokens.rs` for rationale (audit DEP-14, 2026-05-18).
#[cfg(feature = "codegen")]
impl ToTokens for Parameters {
  fn to_tokens(&self, stream: &mut TokenStream) {
    let params = &self.0;
    stream.extend(quote! {
        Parameters::new(<[Parameter]>::into_vec(Box::new([ #(#params),* ])))
    });
  }
}

#[cfg(feature = "codegen")]
impl ToTokens for Parameter {
  fn to_tokens(&self, stream: &mut TokenStream) {
    let name = arena::with(self.name, |name| quote!(arena::pin_static(#name)));
    let spec = arena::with(self.spec, |spec| quote!(arena::pin_static(#spec)));
    let extra = &self.extra;
    let inner = match &self.inner {
      None => quote!(None),
      Some(inner_ps) => quote!(Some(#inner_ps)),
    };
    stream.extend(quote! {
      Parameter {
        name: #name,
        spec: #spec,
        extra: <[Tokens]>::into_vec(Box::new([ #(#extra),* ])),
        inner: #inner,
        ..Parameter::default()
      }
    });
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parameter_default_has_expected_fields() {
    let p = Parameter::default();
    assert!(!p.novalue);
    assert!(p.semiverbatim.is_none());
    assert!(!p.optional);
    assert!(p.inner.is_none());
    assert!(p.extra.is_empty());
    assert!(p.before_digest.is_empty());
    assert!(p.after_digest.is_empty());
    assert!(p.predigest.is_none());
    assert!(p.reversion.is_none());
    assert!(p.digested_reversion.is_none());
    assert_eq!(arena::to_string(p.name), "parameter_default");
    assert_eq!(arena::to_string(p.spec), "");
  }

  #[test]
  fn parameter_display_is_name() {
    let p = Parameter {
      name: arena::pin("Plain"),
      ..Default::default()
    };
    assert_eq!(format!("{p}"), "Plain");
  }

  #[test]
  fn parameter_stringify_is_spec() {
    let p = Parameter {
      spec: arena::pin("{}"),
      ..Default::default()
    };
    assert_eq!(p.stringify(), "{}");
  }

  #[test]
  fn parameter_partial_eq_by_name() {
    // PartialEq compares by name only — Perl parity (closures can't
    // be structurally compared).
    let mut a = Parameter::default();
    let mut b = Parameter::default();
    a.name = arena::pin("x");
    b.name = arena::pin("x");
    assert_eq!(a, b);
    b.name = arena::pin("y");
    assert_ne!(a, b);
  }

  #[test]
  fn parameters_new_and_take() {
    let p = Parameter::default();
    let ps = Parameters::new(vec![p]);
    let taken = ps.take_parameters();
    assert_eq!(taken.len(), 1);
  }

  #[test]
  fn parameters_get_num_args_counts_valued() {
    // novalue=true parameters don't count toward num_args.
    let mut a = Parameter::default();
    let mut b = Parameter::default();
    let mut c = Parameter::default();
    a.novalue = false;
    b.novalue = true;
    c.novalue = false;
    let ps = Parameters::new(vec![a, b, c]);
    assert_eq!(ps.get_num_args(), 2);
  }

  #[test]
  fn parameters_empty() {
    let ps = Parameters::new(vec![]);
    assert_eq!(ps.get_num_args(), 0);
    assert_eq!(ps.get_parameters().len(), 0);
  }

  #[test]
  fn parameters_get_parameters_returns_refs_to_all() {
    // get_parameters returns ALL, including novalue ones (num_args
    // filters; get_parameters doesn't).
    let mut a = Parameter::default();
    let mut b = Parameter::default();
    a.novalue = false;
    b.novalue = true;
    let ps = Parameters::new(vec![a, b]);
    assert_eq!(ps.get_parameters().len(), 2);
  }
}
