use std::sync::Arc;
use core::token::Token;
use core::tbox::TBox;
use core::gullet::Gullet;
use core::stomach::Stomach;
use core::definition::constructor::Constructor;
use state::State;

pub type ReaderClosure = Arc<Box<Fn(&mut Gullet, Vec<Option<Parameters>>, &mut State) -> Vec<Token>>>;
#[derive(Clone)]
pub struct Parameter {
  pub novalue : bool,
  pub semiverbatim : bool,
  pub name : String,
  pub spec : String,
  pub extra : Vec<Option<Parameters>>,
  pub reader : ReaderClosure
}
impl Default for Parameter {
  fn default() -> Self {
    Parameter {
      novalue : false,
      semiverbatim : false,
      name : "parameter_default".to_string(),
      spec : String::new(),
      extra : Vec::new(),
      reader : Arc::new(Box::new(|_gullet, _args, _state| {Vec::new()}))
    }
  }
}

impl Parameter {
  pub fn init(self,state : &mut State) -> Self {
    // Create a parameter reading object for a specific type.
    // If either a declared entry or a function Read<Type> accessible from LaTeXML::Package::Pool
    // is defined.
    let optional_regex = regex!(r"^Optional(.+)$");
    let skip_regex = regex!(r"^Skip(.+)$");

    let descriptor = match state.lookup_mapping("PARAMETER_TYPES", &self.name) {
      Some(descriptor) => Some(descriptor),
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
      }
      else if skip_regex.is_match(&self.name) {
        let captures = skip_regex.captures(&self.name).unwrap();
        let basetype = captures.at(1).unwrap();

        let descriptor = match state.lookup_mapping("PARAMETER_TYPES", basetype) {
          Some(descriptor) => Some(descriptor),
          None => {
            let mut reader = Parameter::check_reader_function(self.name.clone());
            if reader.is_none() {
              reader = Parameter::check_reader_function("Read".to_string() +basetype)
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
      },
      None => {
        // Fatal('misdefined', $type, undef, "Unrecognized parameter type in \"$spec\"") unless $descriptor;
      }
    }
    return self;
  }

  /// Check whether a reader function is accessible within LaTeXML::Package::Pool
  pub fn check_reader_function(function : String) -> Option<ReaderClosure> {
    // if (defined $LaTeXML::Package::Pool::{$function}) {
    //   local *reader = $LaTeXML::Package::Pool::{$function};
    //   if (defined &reader) {
    //     return \&reader; } }
    None
  }

  pub fn read(&self, gullet: &mut Gullet, fordefn : &Constructor, state: &mut State) -> Vec<Token> {
    println!("-- Parameter read !");
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
        Some(token) => gullet.unread(vec![token])
      };
      state.begin_semiverbatim();
    }
    let closure : &ReaderClosure = &self.reader;
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
    return value;
  }
 pub fn digest(&self, stomach: &mut Stomach, value: Vec<Token>, fordefn : &Constructor, state: &mut State) -> Option<TBox> {
    println!("-- Parameter digest !");
    None
  }
}

#[derive(Clone)]
pub struct Parameters {
  pub params : Vec<Parameter>
}

impl Parameters {
  pub fn get_num_args(&self) -> usize {
    self.params.len()
  }

  pub fn revert_arguments(&self, args : Vec<Token>) -> Vec<Token> {
    Vec::new()
  }

  pub fn read_arguments_and_digest(&self, stomach: &mut Stomach, fordefn : &Constructor, state: &mut State) -> Vec<TBox> {
    let mut args = Vec::new();
    for parameter in self.params.iter() {
      let mut value = parameter.read(stomach.get_gullet(), fordefn, state);
      if ! parameter.novalue {
        match parameter.digest(stomach, value, fordefn, state) {
          None => {},
          Some(v) => args.push(v)
        }
      }
    }
    return args
  }
}