use lazy_static::lazy_static;
use proc_macro2::TokenStream; // use proc_macro2::{Ident, Punct, Spacing, Span, TokenStream};
use quote::{quote, ToTokens};
use regex::Regex;
use std::borrow::Cow;
use std::fmt;
use std::sync::Arc; // TokenStreamExt

use crate::common::error::*;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::definition::argument::ArgWrap;
use crate::definition::constructor::Constructor;
use crate::definition::{BeforeDigestClosure, Definition, DigestionClosure};
use crate::gullet::Gullet;
use crate::mouth::Mouth;
use crate::state::State;
use crate::stomach::Stomach;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::{Digested, Locator};

pub type ReaderFn = dyn Fn(&mut Gullet, Option<&Parameters>, &[Tokens], &mut State) -> Result<ArgWrap>;
pub type ReaderPredigestFn = dyn Fn(&mut Stomach, ArgWrap, &mut State) -> Result<Option<Digested>>;
pub type ReaderPredigestClosure = Arc<ReaderPredigestFn>;
pub type ReaderClosure = Arc<ReaderFn>;

// Rust Note:
// the reversion functions initially had "&mut Gullet" as a parameter.
// This turned out to be infeasible if we are to maintain the latexml code flow
// as we have calls into reversions from arbitrary binding closures, at ALL phases.
// Compromise: use the gated Stomach in state whenever you need gullet in reversion, as in
// let mut stomach = state.stomach.borrow_mut();
// let mut gullet = stomach.get_gullet_mut();
//
pub type ReversionClosure = Arc<dyn Fn(Vec<Token>, Option<&Parameters>, &[Tokens], &State) -> Result<Tokens>>;

lazy_static! {
  static ref LAST_WCHAR_RE: Regex = Regex::new(r"\w$").unwrap();
  static ref FIRST_WCHAR_RE: Regex = Regex::new(r"^\w").unwrap();
}

#[derive(Clone)]
pub struct Parameter {
  pub novalue: bool,
  pub semiverbatim: Option<Vec<char>>,
  pub optional: bool,
  pub pack_parameters: bool,
  pub name: Cow<'static, str>,
  pub spec: Cow<'static, str>,
  pub extra: Vec<Tokens>,
  pub inner: Option<Parameters>,
  pub reader: ReaderClosure,
  pub reader_predigest: Option<ReaderPredigestClosure>,
  pub reversion: Option<ReversionClosure>,
  pub before_digest: Vec<BeforeDigestClosure>,
  pub after_digest: Vec<DigestionClosure>,
}
impl Default for Parameter {
  fn default() -> Self {
    Parameter {
      novalue: false,
      semiverbatim: None,
      optional: false,
      pack_parameters: false,
      name: Cow::Borrowed("parameter_default"),
      spec: Cow::Borrowed(""),
      extra: Vec::new(),
      inner: None,
      reader: Arc::new(|_gullet, _args, _extra, _state| {
        Warn!(
          "Parameter",
          "mock_reader",
          None,
          None,
          "Please define a real reader, this is a mock fallback!"
        );
        Ok(ArgWrap::OptionTokens(None))
      }),
      reader_predigest: None,
      reversion: None,
      before_digest: Vec::new(),
      after_digest: Vec::new(),
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
    writeln!(
      f,
      "\t optional:{:?}, spec:{:?}\n\t inner: {:?}\n\t extra: {:?}\n\t reversion: {:?}, before_digest: {:?}, after_digest: {:?} )",
      self.optional,
      self.spec,
      self.inner,
      self.extra,
      self.reversion.is_some(),
      self.before_digest.len(),
      self.after_digest.len()
    )
  }
}
impl fmt::Display for Parameter {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.name) }
}

impl PartialEq for Parameter {
  fn eq(&self, other: &Parameter) -> bool { self.name == other.name }
}
impl Object for Parameter {
  fn stringify(&self) -> String { self.spec.to_string() }
  fn get_locator(&self) -> Option<Cow<Locator>> { unimplemented!() }
}

lazy_static! {
  static ref OPTIONAL_REGEX: Regex = Regex::new(r"^Optional(.+)$").unwrap();
  static ref SKIP_REGEX: Regex = Regex::new(r"^Skip(.+)$").unwrap();
}

impl Parameter {
  pub fn new(name: Cow<'static, str>, spec: Cow<'static, str>, state: &mut State) -> Result<Self> {
    Parameter {
      name,
      spec,
      ..Parameter::default()
    }
    .init(state)
  }
  pub fn init(mut self, state: &mut State) -> Result<Self> {
    // Create a parameter reading object for a specific type.
    // If either a declared entry or a function Read<Type> accessible from LaTeXML::Package::Pool
    // is defined.
    let looked_up_mapping = state.lookup_mapping("PARAMETER_TYPES", &self.name);
    let descriptor: Option<Arc<Parameter>>;
    if let Some(Stored::Parameter(d_lookup)) = looked_up_mapping {
      descriptor = Some(Arc::clone(d_lookup));
    } else if let Some(captures) = OPTIONAL_REGEX.captures(&self.name) {
      let basetype = captures.get(1).map_or("", |m| m.as_str());
      descriptor = match state.lookup_mapping("PARAMETER_TYPES", basetype) {
        Some(Stored::Parameter(d_lookup)) => Some(d_lookup.clone()),
        _ => match Parameter::check_reader_function(&s!("Read{}", &self.name), state) {
          Some(reader) => Some(Arc::new(Parameter {
            reader,
            optional: true,
            ..Parameter::default()
          })),
          None => match Parameter::check_reader_function(&s!("Read{}", basetype), state) {
            Some(reader) => Some(Arc::new(Parameter {
              reader,
              optional: true,
              novalue: true,
              ..Parameter::default()
            })),
            None => fatal!(Parameter, Init, s!("Can't initialize parameter {:?}, unknown?", self.name)),
          },
        },
      };
      self.optional = true;
    } else if let Some(captures) = SKIP_REGEX.captures(&self.name) {
      let basetype = captures.get(1).map_or("", |m| m.as_str());
      descriptor = match state.lookup_mapping("PARAMETER_TYPES", basetype) {
        Some(Stored::Parameter(d_lookup)) => Some(d_lookup.clone()),
        _ => match Parameter::check_reader_function(&self.name, state) {
          Some(reader) => Some(Arc::new(Parameter {
            reader,
            optional: true,
            novalue: true,
            ..Parameter::default()
          })),
          None => Parameter::check_reader_function(&s!("Read{}", basetype), state).map(|reader| {
            Arc::new(Parameter {
              reader,
              optional: true,
              novalue: true,
              ..Parameter::default()
            })
          }),
        },
      };
      if let Some(ref _desc) = descriptor {
        self.novalue = true;
        self.optional = true;
      }
    } else {
      descriptor = Parameter::check_reader_function(&s!("Read{}", &self.name), state).map(|reader| {
        Arc::new(Parameter {
          reader,
          ..Parameter::default()
        })
      });
    }
    match descriptor {
      Some(descriptor) => {
        // descriptor needs to get integrated into Self
        //  except `spec` and `name` which are always preserved!
        self.reader = descriptor.reader.clone(); // What else?
        if descriptor.novalue {
          self.novalue = true;
        }
        self.semiverbatim = descriptor.semiverbatim.clone();
        // Also doing optional setting on the fly, so don't override unless true
        // self.optional = descriptor.optional;
        if descriptor.optional {
          self.optional = true;
        }
        self.reversion = descriptor.reversion.clone();
        self.before_digest = descriptor.before_digest.clone();
        self.after_digest = descriptor.after_digest.clone();
        self.reader_predigest = descriptor.reader_predigest.clone();
        self.pack_parameters = descriptor.pack_parameters;
      },
      None => fatal!(
        Parameter,
        Unknown,
        s!("Unrecognized parameter type with name {:?}, spec {:?}", self.name, self.spec)
      ),
    }
    // Last but not least, initialize any "inner" parameters
    self.inner = self.inner.map(|inner_ps| inner_ps.init(state)
        .expect("inner param init shouldn't fail?"));
    Ok(self)
  }

  /// Obtain the reader of a given parameter name, if available
  pub fn check_reader_function(name: &str, state: &State) -> Option<ReaderClosure> {
    // TODO: This function doesn't have a direct Rust equivalent, since the metaprogramming isn't possible
    // But what is the exact purpose of seeking through the pool namespace? Wouldn't any parameter be already assigned in the state?
    if let Some(Stored::Parameter(param)) = state.lookup_mapping("PARAMETER_TYPES", name) {
      Some(param.reader.clone())
    } else {
      None
    }
  }

  pub fn setup_catcodes(&self, state: &mut State) {
    if self.semiverbatim.is_some() {
      state.begin_semiverbatim(self.semiverbatim.as_deref());
    }
  }

  pub fn revert_catcodes(&self, state: &mut State) -> Result<()> {
    if self.semiverbatim.is_some() {
      state.end_semiverbatim()?;
    }
    Ok(())
  }

  pub fn read(&self, gullet: &mut Gullet, fordefn: Option<&dyn Definition>, state: &mut State) -> Result<ArgWrap> {
    // For semiverbatim, I had messed with catcodes, but there are cases
    // (eg. \caption(...\label{badchars}}) where you really need to
    // cleanup after the fact!
    // Hmmm, seem to still need it...
    self.setup_catcodes(state);

    let closure = &self.reader;
    let value_from_reader: ArgWrap = closure(gullet, self.inner.as_ref(), &self.extra, state)?;
    let value_arg = if value_from_reader.is_tokens() {
      let wants_option = self.optional || value_from_reader.is_option();
      match value_from_reader.owned_tokens() {
        Some(mut value) => {
          if let Some(ref semi_chars) = self.semiverbatim {
            value = value.neutralize(semi_chars, state);
          }
          if self.pack_parameters {
            value = value.pack_parameters();
          }
          if wants_option {
            if value.is_empty() {
              ArgWrap::OptionTokens(None)
            } else {
              ArgWrap::OptionTokens(Some(value))
            }
          } else {
            ArgWrap::Tokens(value)
          }
        },
        None => {
          if wants_option {
            ArgWrap::OptionTokens(None)
          } else {
            ArgWrap::Tokens(Tokens!())
          }
        },
      }
    } else {
      value_from_reader
    };
    self.revert_catcodes(state)?;

    let checked_value = if !self.optional && !self.novalue && (value_arg.is_none() && self.reader_predigest.is_none()) {
      // Deyan: Special exception, which may motivate switching the reader type to Option<Tokens> in the long-run
      //        Until *may* have a value, but it also may *not*, both OK. So... except it from the error message here
      if !self.name.starts_with("Until") {
        let fordefn_str = fordefn.map(|fdefn| fdefn.stringify()).unwrap_or_default();
        Error!(
          "expected",
          self,
          gullet,
          state,
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

  pub fn digest(&self, stomach: &mut Stomach, mut value_arg: ArgWrap, _fordefn: Option<&Constructor>, state: &mut State) -> Result<Option<Digested>> {
    // If semiverbatim, Expand (before digest), so tokens can be neutralized; BLECH!!!!
    if self.semiverbatim.is_some() {
      self.setup_catcodes(state);
      if value_arg.is_tokens() {
        if let Some(value) = value_arg.owned_tokens() {
          let neutralized = stomach.reading_from_mouth(Mouth::default(), state, move |stomach: &mut Stomach, state: &mut State| {
            let gullet = stomach.get_gullet_mut();
            gullet.unread(value);
            let mut tokens = Vec::new();
            loop {
              match gullet.read_x_token(Some(true), true, state) {
                Ok(token_opt) => match token_opt {
                  Some(token) => tokens.push(token),
                  None => break,
                },
                Err(x) => return Err(x),
              }
            }
            let evec = Vec::new();
            Ok(Tokens::new(tokens).neutralize(&evec, state).unlist())
          })?;
          value_arg = ArgWrap::Tokens(Tokens::new(neutralized));
        } else {
          value_arg = ArgWrap::default();
        }
      }
    }

    for pre in self.before_digest.iter() {
      // Done for effect only.
      pre(stomach, state)?; // maybe pass extras?
    }
    let digested_value = if let Some(ref closure) = &self.reader_predigest {
      closure(stomach, value_arg, state)?
    } else {
      // Note: we have an open question for the type interface.
      //  What happens when a wrapped "None" value,
      // (such as the missing value of an Optional [] argument)
      // gets digested?
      //
      // currently a `Digested::default` gets returned, which has an empty TBox and also gets returned
      // for e.g. empty mandatory Plain arguments {}.
      // But we need *different* values, as the explicit "\foo[]" is an override to empty, while "\foo"
      // will use the default value for the Optional.
      if self.optional && value_arg.is_none() {
        None
      } else {
        Some(value_arg.be_digested(stomach, state)?)
      }
    };
    for post in self.after_digest.iter() {
      // Done for effect only.
      let mut w = Whatsit::default();
      post(stomach, &mut w, state)?; // maybe pass extras?
    }

    self.revert_catcodes(state)?;

    Ok(digested_value)
  }

  pub fn revert(&self, value_opt: Option<Tokens>, state: &State) -> Result<Option<Tokens>> {
    if let Some(ref reverter) = self.reversion {
      if let Some(value) = value_opt {
        Ok(Some((reverter)(value.unlist(), self.inner.as_ref(), &self.extra, state)?))
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
  pub fn reparse(&self, _value: Tokens, _gullet: &mut Gullet, _state:&State) -> Result<Tokens> {
    unimplemented!()
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
  fn get_locator(&self) -> Option<Cow<Locator>> { unimplemented!() }
}

impl Parameters {
  pub fn new(params: Vec<Parameter>) -> Self { Parameters(params) }
  pub fn get_num_args(&self) -> usize { self.0.iter().filter(|&p| !p.novalue).count() }
  pub fn get_parameters(&self) -> Vec<&Parameter> { self.0.iter().collect() }
  pub fn revert_arguments(&self, args: Vec<Option<Tokens>>, state: &State) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    for (parameter, arg) in self.0.iter().zip(args.into_iter()) {
      if !parameter.novalue {
        if let Some(reverted_tks) = parameter.revert(arg, state)? {
          tokens.extend(reverted_tks.unlist());
        }
      }
    }
    Ok(tokens)
  }
  // Try to initialize each associated Parameter
  pub fn init(mut self, state: &mut State) -> Result<Self> {
    let mut initialized = Vec::new();
    for param in self.0.drain(..) {
      initialized.push(param.init(state)?);
    }
    self.0 = initialized;
    Ok(self)
  }

  pub fn read_arguments(&self, gullet: &mut Gullet, fordefn: Option<&dyn Definition>, state: &mut State) -> Result<Vec<ArgWrap>> {
    let mut args = Vec::new();
    gullet.setup_scan();
    for parameter in &self.0 {
      let values = parameter.read(gullet, fordefn, state)?;
      if parameter.reader_predigest.is_some() {
        // TODO: Sometimes we legitimately want to use e.g. Number parameters without the predigest closure...
        // so this shouldn't be an error, not even an info -- but leaving it here if something changes in the future.
        // error!(
        //   target: &s!("parameter:{}", parameter.name),
        //   "parameter with predigest closure was invoked in an expandable context. Parameter digestion won't execute."
        // );
      }
      if !parameter.novalue {
        args.push(values);
      }
    }
    Ok(args)
  }

  pub fn read_arguments_and_digest(&self, stomach: &mut Stomach, fordefn: &Constructor, state: &mut State) -> Result<Vec<Option<Digested>>> {
    let mut args = Vec::new();
    stomach.get_gullet_mut().setup_scan();
    for parameter in &self.0 {
      let value = parameter.read(stomach.get_gullet_mut(), Some(fordefn), state)?;
      if !parameter.novalue {
        let digested_value = parameter.digest(stomach, value, Some(fordefn), state)?;
        args.push(digested_value);
      }
    }
    Ok(args)
  }

  pub fn reparse_argument(&self, gullet: &mut Gullet, value: ArgWrap, ostate: &mut State) -> Result<Vec<ArgWrap>> {
    if value.is_none() {
      return Ok(Vec::new())
    }
    let value_tokens = value.revert(ostate)?;
    // start with empty mouth
    let reader_mouth = Mouth::new("", None, ostate)?;
    gullet.reading_from_mouth(reader_mouth, ostate, |gulletx: &mut Gullet, state| {

        gulletx.unread(value_tokens); // but put back tokens to be read
        let values = self.read_arguments(gulletx, None, state)?;
        gulletx.skip_spaces(state);
        Ok(values)
    })
  }
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
  fn from(ps: Parameters) -> Vec<Parameter> {
    ps.0
  }
}

impl ToTokens for Parameters {
  fn to_tokens(&self, stream: &mut TokenStream) {
    let params = &self.0;
    stream.extend(quote! {
        Parameters::new(<[Parameter]>::into_vec(Box::new([ #(#params),* ])))
    });
  }
}

impl ToTokens for Parameter {
  fn to_tokens(&self, stream: &mut TokenStream) {
    let name = match &self.name {
      Cow::Borrowed(v) => quote!(Cow::Borrowed(#v)),
      Cow::Owned(v) => quote!(Cow::Borrowed(#v)),
    };
    let spec = match &self.spec {
      Cow::Borrowed(v) => quote!(Cow::Borrowed(#v)),
      Cow::Owned(v) => quote!(Cow::Borrowed(#v)),
    };
    let extra = &self.extra;
    let inner = match &self.inner {
      None => quote!(None),
      Some(inner_ps) => quote!(Some(#inner_ps))
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