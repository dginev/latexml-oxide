use fmt;
use regex::Regex;
use std::rc::Rc;

use common::error::*;
use common::store::Stored;
use definition::constructor::Constructor;
use definition::{BeforeDigestClosure, Definition, DigestionClosure};
use gullet::Gullet;
use mouth::Mouth;
use state::State;
use stomach::Stomach;
use token::Token;
use tokens::Tokens;
use whatsit::Whatsit;
use Digested;

pub type ReaderClosure =
  Rc<Fn(&mut Gullet, Vec<Option<Parameters>>, Vec<ParameterExtra>, &mut State) -> Result<Tokens>>;
pub type ReversionClosure =
  Rc<Fn(&mut Gullet, Vec<Token>, Vec<ParameterExtra>, &mut State) -> Result<Tokens>>;

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

#[derive(Clone)]
pub struct Parameter {
  pub novalue: bool,
  pub semiverbatim: bool,
  pub optional: bool,
  pub undigested: bool,
  pub name: String,
  pub spec: String,
  pub extra: Vec<ParameterExtra>,
  pub reader: ReaderClosure,
  pub reversion: Option<ReversionClosure>,
  pub before_digest: Option<BeforeDigestClosure>,
  pub after_digest: Option<DigestionClosure>,
}
impl Default for Parameter {
  fn default() -> Self {
    Parameter {
      novalue: false,
      semiverbatim: false,
      optional: false,
      undigested: false,
      name: s!("parameter_default"),
      spec: String::new(),
      extra: Vec::new(),
      reader: Rc::new(|_gullet, _args, _extra, _state| {
        warn!("-- Warning: please define a real reader, this is a mock fallback!");
        Ok(Tokens!())
      }),
      reversion: None,
      before_digest: None,
      after_digest: None,
    }
  }
}
impl fmt::Debug for Parameter {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Parameter(\n\t name: {:?}, novalue:{:?}, semiverbatim:{:?},\n\t optional:{:?}, undigested:{:?}, spec:{:?}\n\t extra: {:?}\n\t reversion: {:?}, before_digest: {:?}, after_digest: {:?} )", 
    self.name,
    self.novalue,
    self.semiverbatim,
    self.optional,
    self.undigested,
    self.spec,
    self.extra,
    self.reversion.is_some(),
    self.before_digest.is_some(),
    self.after_digest.is_some())
  }
}
impl PartialEq for Parameter {
  fn eq(&self, other: &Parameter) -> bool { self.name == other.name }
}

lazy_static! {
  static ref OPTIONAL_REGEX: Regex = Regex::new(r"^Optional(.+)$").unwrap();
  static ref SKIP_REGEX: Regex = Regex::new(r"^Skip(.+)$").unwrap();
}

impl Parameter {
  pub fn init(mut self, state: &mut State) -> Result<Self> {
    // Create a parameter reading object for a specific type.
    // If either a declared entry or a function Read<Type> accessible from LaTeXML::Package::Pool
    // is defined.
    let looked_up_mapping = state.lookup_mapping("PARAMETER_TYPES", &self.name);
    let mut descriptor: Option<Parameter>;
    if let Some(&Stored::Parameter(ref d_lookup)) = looked_up_mapping {
      descriptor = Some((*d_lookup).clone());
    } else if OPTIONAL_REGEX.is_match(&self.name) {
      let captures = OPTIONAL_REGEX.captures(&self.name).unwrap();
      let basetype = captures.get(1).map_or("", |m| m.as_str());
      descriptor = match state.lookup_mapping("PARAMETER_TYPES", basetype) {
        Some(&Stored::Parameter(ref d_lookup)) => Some(d_lookup.clone()),
        _ => match Parameter::check_reader_function(s!("Read{}", &self.name)) {
          Some(reader) => Some(Parameter {
            reader,
            optional: true,
            ..Parameter::default()
          }),
          None => match Parameter::check_reader_function(s!("Read{}", basetype)) {
            Some(reader) => Some(Parameter {
              reader,
              optional: true,
              novalue: true,
              ..Parameter::default()
            }),
            None => fatal!(
              Parameter,
              Init,
              s!("Can't initialize parameter {:?}, unknown?", self.name)
            ),
          },
        },
      };
      descriptor.as_mut().unwrap().optional = true;
    } else if SKIP_REGEX.is_match(&self.name) {
      let captures = SKIP_REGEX.captures(&self.name).unwrap();
      let basetype = captures.get(1).map_or("", |m| m.as_str());

      descriptor = match state.lookup_mapping("PARAMETER_TYPES", basetype) {
        Some(&Stored::Parameter(ref d_lookup)) => Some(d_lookup.clone()),
        _ => match Parameter::check_reader_function(self.name.clone()) {
          Some(reader) => Some(Parameter {
            reader,
            optional: true,
            novalue: true,
            ..Parameter::default()
          }),
          None => match Parameter::check_reader_function(s!("Read{}", basetype)) {
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
      descriptor = match Parameter::check_reader_function(s!("Read{}", &self.name)) {
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
        self.undigested = descriptor.undigested;
        self.reversion = descriptor.reversion;
        self.before_digest = descriptor.before_digest;
        self.after_digest = descriptor.after_digest;
      },
      None => panic!("Unrecognized parameter type in {:?}", self.spec),
    }
    Ok(self)
  }

  // TODO: This meta-programming approach won't fly in Rust, need an alternative.
  /// Check whether a reader function is accessible within LaTeXML::Package::Pool
  pub fn check_reader_function(_function: String) -> Option<ReaderClosure> {
    // if (defined $LaTeXML::Package::Pool::{$function}) {
    //   local *reader = $LaTeXML::Package::Pool::{$function};
    //   if (defined &reader) {
    //     return \&reader; } }
    None
  }

  pub fn read(
    &self,
    gullet: &mut Gullet,
    fordefn: &Definition,
    state: &mut State,
  ) -> Result<Tokens>
  {
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
        Some(tokens) => gullet.unread(&Tokens!(tokens)),
      };
      state.begin_semiverbatim(None);
    }
    let closure: &ReaderClosure = &self.reader;
    let mut value = closure(gullet, vec![], self.extra.clone(), state)?;
    // TODO:
    // $value = $value->neutralize if $$self{semiverbatim} && (ref $value)
    //   && $value->can('neutralize');
    if self.semiverbatim {
      state.end_semiverbatim();
    }

    if !self.optional && !self.novalue && value.is_empty() {
      error!(
        target: &s!("expected:{:?}", self),
        "Missing argument for TODO:fordefn"
      );
      //     $gullet->showUnexpected);
      value = Tokens!(T_OTHER!("missing"));
    }

    Ok(value)
  }

  pub fn digest(
    &self,
    stomach: &mut Stomach,
    mut value: Tokens,
    _fordefn: &Constructor,
    state: &mut State,
  ) -> Result<Option<Digested>>
  {
    // If semiverbatim, Expand (before digest), so tokens can be neutralized; BLECH!!!!
    let value_to_digest = value.clone();
    if self.semiverbatim {
      state.begin_semiverbatim(None);
      stomach.reading_from_mouth(
        Mouth::default(),
        state,
        Box::new(move |stomach: &mut Stomach, state: &mut State| {
          let gullet = stomach.get_gullet_mut();
          gullet.unread(&value);
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
        }),
      )?;
    }

    if let Some(ref pre) = self.before_digest {
      // Done for effect only.
      pre(stomach, state)?; // maybe pass extras?
    }

    let digested_value = if !value_to_digest.is_empty() {
      if !self.undigested {
        Some(value_to_digest.be_digested(stomach, state)?)
      } else {
        Some(Digested::Postponed(Rc::new(value_to_digest)))
      }
    } else {
      None
    };
    if let Some(ref post) = self.after_digest {
      // Done for effect only.
      let mut w = Whatsit::default();
      post(stomach, &mut w, state)?; // maybe pass extras?
    }
    if self.semiverbatim {
      state.end_semiverbatim() // Corner case?
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
}

#[derive(Clone, Debug)]
pub struct Parameters {
  pub params: Vec<Parameter>,
}

impl Parameters {
  pub fn get_num_args(&self) -> usize { self.params.iter().filter(|&p| !p.novalue).count() }

  pub fn revert_arguments(
    &self,
    args: Vec<Tokens>,
    gullet: &mut Gullet,
    state: &mut State,
  ) -> Result<Vec<Token>>
  {
    let mut tokens = Vec::new();
    for (parameter, arg) in self.params.iter().zip(args.into_iter()) {
      if !parameter.novalue {
        tokens.append(&mut parameter.revert(arg, gullet, state)?.unlist());
      }
    }
    Ok(tokens)
  }

  pub fn read_arguments(
    &self,
    gullet: &mut Gullet,
    fordefn: &Definition,
    state: &mut State,
  ) -> Result<Vec<Tokens>>
  {
    let mut args = Vec::new();
    for parameter in &self.params {
      let values = parameter.read(gullet, fordefn, state)?;
      if !parameter.novalue {
        args.push(values);
      }
    }
    Ok(args)
  }

  pub fn read_arguments_and_digest(
    &self,
    stomach: &mut Stomach,
    fordefn: &Constructor,
    state: &mut State,
  ) -> Result<Vec<Option<Digested>>>
  {
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

  pub fn reparse_argument(
    &self,
    _gullet: &mut Gullet,
    _value: Tokens,
    _state: &mut State,
  ) -> Tokens
  {
    Tokens!()
  }
}
