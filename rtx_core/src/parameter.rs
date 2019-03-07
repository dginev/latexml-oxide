use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;
use std::fmt;
use std::rc::Rc;

use crate::common::error::*;
use crate::common::store::Stored;
use crate::common::object::Object;
use crate::definition::constructor::Constructor;
use crate::definition::{BeforeDigestClosure, Definition, DigestionClosure};
use crate::gullet::Gullet;
use crate::mouth::Mouth;
use crate::state::State;
use crate::stomach::Stomach;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::Digested;

pub type ReaderFn = Fn(&mut Gullet, Vec<Option<Parameters>>, Vec<ParameterExtra>, &mut State) -> Result<Tokens>;
pub type ReaderPredigestFn = Fn(&mut Stomach, Tokens, &mut State) -> Result<Option<Digested>>;
pub type ReaderPredigestClosure = Rc<ReaderPredigestFn>;
pub type ReaderClosure = Rc<ReaderFn>;
pub type ReversionClosure = Rc<Fn(&mut Gullet, Vec<Token>, Vec<ParameterExtra>, &mut State) -> Result<Tokens>>;

lazy_static! {
  static ref LAST_WCHAR_RE: Regex = Regex::new(r"\w$").unwrap();
  static ref FIRST_WCHAR_RE: Regex = Regex::new(r"^\w").unwrap();
}

#[derive(Clone, Debug)]
pub enum ParameterExtra {
  Token(Token),
  ParametersOption(Option<Parameters>),
}
impl From<Token> for ParameterExtra {
  fn from(t: Token) -> ParameterExtra { ParameterExtra::Token(t) }
}
impl From<Option<Parameters>> for ParameterExtra {
  fn from(opt: Option<Parameters>) -> ParameterExtra { ParameterExtra::ParametersOption(opt) }
}
impl From<ParameterExtra> for Token {
  fn from(param: ParameterExtra) -> Token {
    if let ParameterExtra::Token(t) = param {
      t
    } else {
      T_OTHER!("")
    }
  }
}
impl From<ParameterExtra> for Option<Parameters> {
  fn from(param: ParameterExtra) -> Option<Parameters> {
    if let ParameterExtra::ParametersOption(ps) = param {
      ps
    } else {
      None
    }
  }
}

impl From<Tokens> for Vec<ParameterExtra> {
  fn from(tks: Tokens) -> Vec<ParameterExtra> { tks.unlist().into_iter().map(Into::into).collect() }
}

#[derive(Clone)]
pub struct Parameter {
  pub novalue: bool,
  pub semiverbatim: bool,
  pub optional: bool,
  pub name: String,
  pub spec: String,
  pub extra: Vec<ParameterExtra>,
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
      semiverbatim: false,
      optional: false,
      name: s!("parameter_default"),
      spec: String::new(),
      extra: Vec::new(),
      reader: Rc::new(|_gullet, _args, _extra, _state| {
        Warn!("Parameter","mock_reader", None, None, "Please define a real reader, this is a mock fallback!");
        Ok(Tokens!())
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
      "\t optional:{:?}, spec:{:?}\n\t extra: {:?}\n\t reversion: {:?}, before_digest: {:?}, after_digest: {:?} )",
      self.optional,
      self.spec,
      self.extra,
      self.reversion.is_some(),
      self.before_digest.len(),
      self.after_digest.len()
    )
  }
}
impl fmt::Display for Parameter {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.name)
  }
}

impl PartialEq for Parameter {
  fn eq(&self, other: &Parameter) -> bool { self.name == other.name }
}
impl Object for Parameter {}

lazy_static! {
  static ref OPTIONAL_REGEX: Regex = Regex::new(r"^Optional(.+)$").unwrap();
  static ref SKIP_REGEX: Regex = Regex::new(r"^Skip(.+)$").unwrap();
}

impl Parameter {
  pub fn new(name: &str, spec: &str, state: &mut State) -> Result<Self> {
    Parameter {
      name: name.to_string(),
      spec: spec.to_string(),
      ..Parameter::default()
    }
    .init(state)
  }
  pub fn init(mut self, state: &mut State) -> Result<Self> {
    // Create a parameter reading object for a specific type.
    // If either a declared entry or a function Read<Type> accessible from LaTeXML::Package::Pool
    // is defined.
    let looked_up_mapping = state.lookup_mapping("PARAMETER_TYPES", &self.name);
    let mut descriptor: Option<Parameter>;
    if let Some(&Stored::Parameter(ref d_lookup)) = looked_up_mapping {
      descriptor = Some((*d_lookup).clone());
    } else if let Some(captures) = OPTIONAL_REGEX.captures(&self.name) {
      let basetype = captures.get(1).map_or("", |m| m.as_str());
      descriptor = match state.lookup_mapping("PARAMETER_TYPES", basetype) {
        Some(&Stored::Parameter(ref d_lookup)) => Some(d_lookup.clone()),
        _ => match Parameter::check_reader_function(&s!("Read{}", &self.name), state) {
          Some(reader) => Some(Parameter {
            reader,
            optional: true,
            ..Parameter::default()
          }),
          None => match Parameter::check_reader_function(&s!("Read{}", basetype), state) {
            Some(reader) => Some(Parameter {
              reader,
              optional: true,
              novalue: true,
              ..Parameter::default()
            }),
            None => fatal!(Parameter, Init, s!("Can't initialize parameter {:?}, unknown?", self.name)),
          },
        },
      };
      descriptor.as_mut().unwrap().optional = true;
    } else if let Some(captures) = SKIP_REGEX.captures(&self.name) {
      let basetype = captures.get(1).map_or("", |m| m.as_str());
      descriptor = match state.lookup_mapping("PARAMETER_TYPES", basetype) {
        Some(&Stored::Parameter(ref d_lookup)) => Some(d_lookup.clone()),
        _ => match Parameter::check_reader_function(&self.name, state) {
          Some(reader) => Some(Parameter {
            reader,
            optional: true,
            novalue: true,
            ..Parameter::default()
          }),
          None => match Parameter::check_reader_function(&s!("Read{}", basetype), state) {
            Some(reader) => Some(Parameter {
              reader,
              optional: true,
              novalue: true,
              ..Parameter::default()
            }),
            None => None,
          },
        },
      };
      if let Some(ref mut desc) = descriptor {
        desc.novalue = true;
        desc.optional = true;
      }
    } else {
      descriptor = match Parameter::check_reader_function(&s!("Read{}", &self.name), state) {
        Some(reader) => Some(Parameter {
          reader,
          ..Parameter::default()
        }),
        None => None,
      };
    }

    match descriptor {
      Some(descriptor) => {
        // descriptor needs to get integrated into Self
        //  except `spec` and `name` which are always preserved!
        self.reader = descriptor.reader; // What else?
        self.novalue = descriptor.novalue;
        self.semiverbatim = descriptor.semiverbatim;
        self.optional = descriptor.optional;
        self.reversion = descriptor.reversion;
        self.before_digest = descriptor.before_digest;
        self.after_digest = descriptor.after_digest;
        self.reader_predigest = descriptor.reader_predigest;
      },
      None => fatal!(
        Parameter,
        Unknown,
        s!("Unrecognized parameter type with name {:?}, spec {:?}", self.name, self.spec)
      ),
    }
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

  pub fn read(&self, gullet: &mut Gullet, fordefn: &Definition, state: &mut State) -> Result<Tokens> {
    // For semiverbatim, I had messed with catcodes, but there are cases
    // (eg. \caption(...\label{badchars}}) where you really need to
    // cleanup after the fact!
    // Hmmm, seem to still need it...
    if self.semiverbatim {
      // Nasty Hack: If immediately followed by %, should discard the comment
      // EVEN if semiverbatim makes % into other!
      let peek = gullet.read_token(state);
      match peek {
        None => {},
        Some(tokens) => gullet.unread(Tokens!(tokens)),
      };
      state.begin_semiverbatim(None);
    }
    let closure = &self.reader;
    let mut value = closure(gullet, vec![], self.extra.clone(), state)?;
    // TODO:
    // $value = $value->neutralize if $$self{semiverbatim} && (ref $value)
    //   && $value->can('neutralize');
    if self.semiverbatim {
      state.end_semiverbatim()?;
    }

    if !self.optional && !self.novalue && (value.is_empty() && self.reader_predigest.is_none()) {
      // Deyan: Special exception, which may motivate switching the reader type to Option<Tokens> in the long-run
      //        Until *may* have a value, but it also may *not*, both OK. So... except it from the error message here
      if !self.name.starts_with("Until") {
        Error!("expected", self, gullet, state, s!("Missing argument {} for {}", self.stringify(), fordefn.stringify()));
        value = Tokens!(T_OTHER!("missing"));
      }
    }
    Ok(value)
  }

  pub fn digest(&self, stomach: &mut Stomach, mut value: Tokens, _fordefn: &Constructor, state: &mut State) -> Result<Option<Digested>> {
    // If semiverbatim, Expand (before digest), so tokens can be neutralized; BLECH!!!!
    let value_to_digest = value.clone();
    if self.semiverbatim {
      state.begin_semiverbatim(None);
    }

    if self.semiverbatim && !value.is_empty() {
      stomach.reading_from_mouth(Mouth::default(), state, move |stomach: &mut Stomach, state: &mut State| {
        let gullet = stomach.get_gullet_mut();
        gullet.unread(value.clone());
        let mut tokens = Vec::new();
        loop {
          match gullet.read_x_token(true, true, state) {
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
    }

    for pre in self.before_digest.iter() {
      // Done for effect only.
      pre(stomach, state)?; // maybe pass extras?
    }

    let digested_value = if let Some(ref closure) = &self.reader_predigest {
      closure(stomach, value_to_digest, state)?
    } else if !value_to_digest.is_empty() {
      Some(value_to_digest.be_digested(stomach, state)?)
    } else {
      None
    };
    for post in self.after_digest.iter() {
      // Done for effect only.
      let mut w = Whatsit::default();
      post(stomach, &mut w, state)?; // maybe pass extras?
    }
    if self.semiverbatim {
      state.end_semiverbatim()?; // Corner case?
    }

    Ok(digested_value)
  }

  pub fn revert(&self, value: Tokens, gullet: &mut Gullet, state: &mut State) -> Result<Tokens> {
    if let Some(ref reverter) = self.reversion {
      (reverter)(gullet, value.unlist(), self.extra.clone(), state)
    } else {
      Ok(Tokens::new(value.revert()))
    }
  }

  pub fn to_string(&self) -> String { self.spec.clone() }
}

#[derive(Clone, Debug)]
pub struct Parameters {
  pub params: Vec<Parameter>,
}

impl Parameters {
  pub fn get_num_args(&self) -> usize { self.params.iter().filter(|&p| !p.novalue).count() }
  pub fn get_parameters(&self) -> Vec<&Parameter> { self.params.iter().map(|p| p).collect() }
  pub fn revert_arguments(&self, args: Vec<Tokens>, gullet: &mut Gullet, state: &mut State) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    for (parameter, arg) in self.params.iter().zip(args.into_iter()) {
      if !parameter.novalue {
        tokens.append(&mut parameter.revert(arg, gullet, state)?.unlist());
      }
    }
    Ok(tokens)
  }

  pub fn read_arguments(&self, gullet: &mut Gullet, fordefn: &Definition, state: &mut State) -> Result<Vec<Tokens>> {
    let mut args = Vec::new();
    for parameter in &self.params {
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
    for parameter in &self.params {
      let value = parameter.read(&mut stomach.gullet, fordefn, state)?;
      if !parameter.novalue {
        let digested_value = parameter.digest(stomach, value, fordefn, state)?;
        args.push(digested_value);
      }
    }
    Ok(args)
  }

  pub fn reparse_argument(&self, _gullet: &mut Gullet, _value: Tokens, _state: &mut State) -> Tokens { Tokens!() }
  pub fn to_string(&self) -> String {
    let mut content = String::new();
    for parameter in &self.params {
      let param_content = parameter.to_string();
      if LAST_WCHAR_RE.is_match(&content) && FIRST_WCHAR_RE.is_match(&param_content) {
        content.push(' ');
      }
      content.push_str(&param_content);
    }
    content
  }
}
