use std::sync::Arc;
use std::collections::HashMap;
use state::State;
use common::object::Object;

use core::token::*;
use core::tbox::TBox;
use core::gullet::Gullet;
use core::stomach::Stomach;
use core::whatsit::Whatsit;
use core::parameter::Parameters;
use core::definition::Definition;
use core::definition::expandable::ExpansionClosure;
use core::definition::compiler::*;
use core::document::Document;

#[derive(Clone)]
pub struct ConstructorOptions {
  pub bounded : bool,
  pub mode : String, // TODO
  pub before_digest : Vec<ExpansionClosure>,
  pub after_digest : Vec<DigestionClosure>,
  pub before_construct : Vec<ConstructionClosure>,
  pub after_construct : Vec<ConstructionClosure>,
}
impl Default for ConstructorOptions {
  fn default() -> Self {
    ConstructorOptions {
      bounded : false,
      before_digest : vec![],
      after_digest : vec![],
      before_construct : vec![],
      after_construct : vec![],
      mode : String::new()
    }
  }
}

pub type DigestionClosure = Arc<Box<Fn(&mut Stomach, &Whatsit, &mut State)>>;
pub type ReplacementClosure = Arc<Box<Fn(&mut Document, &Vec<TBox>, &HashMap<String, String>, &mut State)>>;
pub type ConstructionClosure = Arc<Box<Fn(&mut Document, &Whatsit, &mut State)>>;

#[derive(Clone)]
pub struct Constructor {
  pub cs : Token,
  pub paramlist : Option<Parameters>,
  pub nargs : Option<usize>,
  pub replacement : String,
  pub replacement_closure : Option<ReplacementClosure>,
  pub options : ConstructorOptions
}
impl Default for Constructor {
  fn default() -> Self {
    Constructor {
      cs : T_CS!("Constructor".to_string()),
      paramlist : None,
      nargs : None,
      replacement : String::new(),
      replacement_closure : None,
      options : ConstructorOptions::default()
    }
  }
}

impl Object for Constructor {}
impl Definition for Constructor {
  fn invoke(&self, _gullet : &mut Gullet, _state : &mut State) -> Vec<Token> {
    println_stderr!("-- constructor invoke for {:?}", self.get_cs());
    Vec::new()
  }
  /// Digest the constructor; This should occur in the Stomach to create a Whatsit.
  /// The whatsit which will be further processed to create the document.
  fn invoke_primitive(&self, stomach : &mut Stomach, state : &mut State) -> Vec<TBox> {
    println_stderr!("-- constructor/primitive invoke for {:?}", self.get_cs());
    // Call any `Before' code.
    // TODO: profiling / tracing
      // let profiled = state.lookup_value("PROFILING") && ($LaTeXML::CURRENT_TOKEN || $$self{cs});
      // let tracing = state.lookup_value("TRACINGCOMMANDS");
      // LaTeXML::Core::Definition::startProfiling($profiled, "digest") if $profiled;

    let pre = self.execute_before_digest(stomach, state);

    // println_stderr_stderr!("{" + $self->tracingCSName . "}\n" if $tracing;
  // Get some info before we process arguments...
  // let font   = state.lookup_value("font");
  // let ismath = state.lookup_value("IN_MATH");
  // Parse AND digest the arguments to the Constructor
  let mut args : Vec<TBox> = match self.get_parameters() {
    &None => Vec::new(),
    &Some(ref params) => params.read_arguments_and_digest(stomach, &self, state)
  };
  // println_stderr_stderr!($self->tracingArgs(@args) . "\n" if $tracing && @args;
  let nargs = self.get_num_args();
  args.truncate(nargs);

  // Compute any extra Whatsit properties (many end up as element attributes)
  // let properties = $$self{properties};
  // my %props = (!defined $properties ? ()
  //   : (ref $properties eq "CODE" ? &$properties($stomach, @args)
  //     : %$properties));
  // foreach let key (keys %props) {
  //   let value = $props{$key};
  //   if (ref $value eq 'CODE') {
  //     $props{$key} = &$value($stomach, @args); } }
  // $props{font}    = $font                                     unless defined $props{font};
  // $props{locator} = $stomach->getGullet->getMouth->getLocator unless defined $props{locator};
  // $props{isMath}  = $ismath                                   unless defined $props{isMath};
  // $props{level}   = $stomach->getBoxingLevel;

  // Now create the Whatsit, itself.
  // let whatsit = Whatsit { self, args, props};

  // Call any 'After' code.
  // let post = self.execute_after_digest(stomach, whatsit, state);
  // if (let cap = $$self{captureBody}) {
  //   $whatsit->setBody(@post, $stomach->digestNextBody((ref $cap ? $cap : undef))); @post = (); }

  // my @postpost = $self->executeAfterDigestBody($stomach, $whatsit);
  // LaTeXML::Core::Definition::stopProfiling($profiled, 'digest') if $profiled;
  // return (@pre, $whatsit, @post, @postpost);
    Vec::new()
  }

  fn get_cs(&self) -> Token {
    self.cs.clone()
  }
  fn get_cs_name(&self) -> String {
    self.cs.get_cs_name()
  }
  fn get_locator(&self) -> String {
    unimplemented!()
  }
  fn get_parameters(&self) -> &Option<Parameters> { &self.paramlist }
  fn get_num_args(&self) -> usize {
    let nargs = match self.nargs {
      Some(n) => n,
      None => {
        match &self.paramlist {
          &Some(ref params) => params.get_num_args(),
          &None => 0
        }
      }
    };
    // self.nargs = Some(nargs);
    nargs
  }

  fn do_absorbtion(&self, document: &mut Document, whatsit: &Whatsit, state: &mut State) {

    for pre_closure in &self.options.before_construct {
      pre_closure(document, whatsit, state);
    }

    match &self.replacement_closure {
      &None => {},
      &Some(ref main_closure) => main_closure(document, whatsit.get_args(), whatsit.get_properties(), state)
    };

    for post_closure in &self.options.after_construct {
     post_closure(document, whatsit, state);
    }
    return;
  }
}

impl Constructor {
  fn execute_before_digest(&self, _stomach: &mut Stomach, _state: &mut State) {}
  fn execute_after_digest(&self,_stomach: &mut Stomach, _state: &mut State) {}
  pub fn compile(&mut self) {
    self.replacement_closure = self.compile_replacement();
  }
}