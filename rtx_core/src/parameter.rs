use std::sync::Arc;
use regex::Regex;
use token::Token;
use tokens::Tokens;
use tbox::TBox;
use gullet::Gullet;
use stomach::Stomach;
use definition::Definition;
use definition::constructor::{Constructor, DigestionClosure};
use definition::expandable::ExpansionClosure;
use state::State;
use mouth::Mouth;
use Digested;

pub type ReaderClosure = Arc<Fn(&mut Gullet, Vec<Option<Parameters>>, &mut State) -> Vec<Token>>;
pub type ReversionClosure = Arc<Fn(&mut Gullet, Vec<Token>, Vec<Option<Parameters>>, &mut State) -> Vec<Token>>;
#[derive(Clone)]
pub struct Parameter {
  pub novalue: bool,
  pub semiverbatim: bool,
  pub optional: bool,
  pub undigested : bool,
  pub name: String,
  pub spec: String,
  pub extra: Vec<Option<Parameters>>,
  pub reader: ReaderClosure,
  pub reversion: Option<ReversionClosure>,
  pub before_digest : Option<DigestionClosure>,
  pub after_digest : Option<DigestionClosure>,
}
impl Default for Parameter {
  fn default() -> Self {
    Parameter {
      novalue: false,
      semiverbatim: false,
      optional: false,
      undigested : false,
      name: "parameter_default".to_string(),
      spec: String::new(),
      extra: Vec::new(),
      reader: Arc::new(|_gullet, _args, _state| {
        println_stderr!("-- Warning: please define a real reader, this is a mock fallback!");
        Vec::new()
      }),
      reversion: None,
      before_digest : None,
      after_digest : None,
    }
  }
}

impl Parameter {
  pub fn init(mut self, state: &mut State) -> Self {
    // Create a parameter reading object for a specific type.
    // If either a declared entry or a function Read<Type> accessible from LaTeXML::Package::Pool
    // is defined.
    lazy_static!{
      static ref optional_regex : Regex = Regex::new(r"^Optional(.+)$").unwrap();
      static ref skip_regex : Regex = Regex::new(r"^Skip(.+)$").unwrap();
    }
    let looked_up_mapping = state.lookup_mapping("PARAMETER_TYPES", &self.name);
    let descriptor = match looked_up_mapping {
      Some(ref descriptor) => Some(descriptor.clone()),
      None => {
        if optional_regex.is_match(&self.name) {
          let captures = optional_regex.captures(&self.name).unwrap();
          let basetype = captures.at(1).unwrap();
          let descriptor = match state.lookup_mapping("PARAMETER_TYPES", basetype) {
            Some(descriptor) => Some(descriptor),
            None => {
              let mut reader = Parameter::check_reader_function("Read".to_string() + &self.name);
              if reader.is_none() {
                reader = Parameter::check_reader_function("Read".to_string() + basetype);
              }
              // descriptor = {reader: reader} // ???
              None
            }
          };
          // descriptor.optional = true // ???
          descriptor
        } else if skip_regex.is_match(&self.name) {
          let captures = skip_regex.captures(&self.name).unwrap();
          let basetype = captures.at(1).unwrap();

          let descriptor = match state.lookup_mapping("PARAMETER_TYPES", basetype) {
            Some(descriptor) => Some(descriptor),
            None => {
              let mut reader = Parameter::check_reader_function(self.name.clone());
              if reader.is_none() {
                reader = Parameter::check_reader_function("Read".to_string() + basetype)
              }
              // descriptor = { reader => $reader };
              None
            }
          };
          // $descriptor = { %$descriptor, novalue => 1, optional => 1 } if $descriptor; }
          descriptor
        } else {
          let reader = Parameter::check_reader_function("Read".to_string() + &self.name);
          if reader.is_some() {
            // descriptor = { reader => $reader } if $reader;
          }
          // descriptor
          None
        }
      }
    };
    match descriptor {
      Some(descriptor) => {
        // descriptor needs to get integrated into Self
        self.reader = descriptor.reader.clone(); // What else?
      }
      None => {
        // Fatal('misdefined', $type, undef, "Unrecognized parameter type in \"$spec\"") unless $descriptor;
      }
    }
    return self;
  }

  /// Check whether a reader function is accessible within LaTeXML::Package::Pool
  pub fn check_reader_function(function: String) -> Option<ReaderClosure> {
    // if (defined $LaTeXML::Package::Pool::{$function}) {
    //   local *reader = $LaTeXML::Package::Pool::{$function};
    //   if (defined &reader) {
    //     return \&reader; } }
    None
  }

  pub fn read(&self, gullet: &mut Gullet, fordefn: &Definition, state: &mut State) -> Vec<Token> {
    // For semiverbatim, I had messed with catcodes, but there are cases
    // (eg. \caption(...\label{badchars}}) where you really need to
    // cleanup after the fact!
    // Hmmm, seem to still need it...
    if self.semiverbatim {
      // Nasty Hack: If immediately followed by %, should discard the comment
      // EVEN if semiverbatim makes % into other!
      let peek = gullet.read_token(state);
      match peek {
        None => {}
        Some(token) => gullet.unread(vec![token]),
      };
      state.begin_semiverbatim();
    }
    let closure: &ReaderClosure = &self.reader;
    let value = closure(gullet, self.extra.clone(), state);
    // TODO:
    // $value = $value->neutralize if $$self{semiverbatim} && (ref $value)
    //   && $value->can('neutralize');
    if self.semiverbatim {
      state.end_semiverbatim();
    }
    // TODO:
    // if ((!defined $value) && !$$self{optional}) {
    //   Error('expected', $self, $gullet,
    //     "Missing argument " . Stringify($self) . " for " . Stringify($fordefn),
    //     $gullet->showUnexpected);
    //   $value = T_OTHER('missing'); }

    value
  }

  pub fn digest(&self, stomach: &mut Stomach, value: Tokens, fordefn: &Constructor, state: &mut State) -> Option<Digested> {
    // If semiverbatim, Expand (before digest), so tokens can be neutralized; BLECH!!!!
    let mut value_to_digest = value.clone();
    if self.semiverbatim {
      state.begin_semiverbatim();
      stomach.reading_from_mouth(Mouth::default(), state, Box::new(move |stomach: &mut Stomach, state : &mut State| {
        let gullet = stomach.get_gullet_mut();
        gullet.unread(value.clone().unlist());
        let mut tokens = Vec::new();
        while let Some(token) = gullet.read_x_token(true, true, state) {
          tokens.push(token);
        }
        let evec = Vec::new();
        Tokens{tokens: tokens}.neutralize(&evec, state).unlist()
      }));
    }

    if let Some(ref pre) = self.before_digest { // Done for effect only.
      pre(stomach, None, state); // maybe pass extras?
    }

    let mut digested_value = None;
    if !value_to_digest.is_empty() && !self.undigested {
      digested_value = Some(value_to_digest.be_digested(stomach, state));
    }
    if let Some(ref post) = self.after_digest { // Done for effect only.
      post(stomach, None, state); // maybe pass extras?
    }
    if self.semiverbatim {
      state.end_semiverbatim() // Corner case?
    }

    digested_value
  }
}

#[derive(Clone)]
pub struct Parameters {
  pub params: Vec<Parameter>,
}

impl Parameters {
  pub fn get_num_args(&self) -> usize {
    self.params.len()
  }

  pub fn revert_arguments(&self, _args: Vec<Token>, _state: &mut State) -> Vec<Token> {
    Vec::new()
  }

  pub fn read_arguments(&self, gullet: &mut Gullet, fordefn: &Definition, state: &mut State) -> Vec<Token> {
    let mut args = Vec::new();
    for parameter in self.params.iter() {
      let values = parameter.read(gullet, fordefn, state);
      if !parameter.novalue {
        for value in values {
          args.push(value);
        }
      }
    }
    args
  }

  pub fn read_arguments_and_digest(&self, stomach: &mut Stomach, fordefn: &Constructor, state: &mut State) -> Vec<Digested> {
    let mut args = Vec::new();
    for parameter in self.params.iter() {
      let value = parameter.read(&mut stomach.gullet, fordefn, state);
      if !parameter.novalue {
        match parameter.digest(stomach, Tokens{tokens: value}, fordefn, state) {
          None => {}
          Some(v) => args.push(v),
        }
      }
    }
    return args;
  }

  pub fn reparse_argument(&self, _gullet: &mut Gullet, _value: Vec<Token>, _state: &mut State) -> Vec<Token> {
    Vec::new()
  }
}
