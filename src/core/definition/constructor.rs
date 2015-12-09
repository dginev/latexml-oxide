use std::sync::Arc;
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
use core::document::Document;


pub struct ConstructorOptions {
  pub bounded : bool,
  pub mode : String, // TODO
  pub before_digest : Option<ExpansionClosure>,
  pub after_digest : Option<DigestionClosure>,
}
impl Default for ConstructorOptions {
  fn default() -> Self { 
    ConstructorOptions {
      bounded : false,
      before_digest : None,
      after_digest : None,
      mode : String::new()
    }
  }
}

pub type ConstructionClosure = Arc<Box<Fn(&mut Document, &mut State)>>;
pub type DigestionClosure = Arc<Box<Fn(&mut Stomach, &mut Whatsit, &mut State)>>;
#[derive(Clone)]
pub struct Constructor {
  pub cs : Token,
  pub paramlist : Option<Parameters>,
  pub nargs : Option<usize>,
  pub replacement : String
}
impl Default for Constructor {
  fn default() -> Self {
    Constructor {
      cs : T_CS!("Constructor".to_string()),
      paramlist : None,
      nargs : None,
      replacement : String::new()
    }
  }
}

impl Object for Constructor {}
impl Definition for Constructor {
  fn invoke(&self, _gullet : &mut Gullet, _state : &mut State) -> Vec<Token> {
    println!("-- constructor invoke for {:?}", self.get_cs());
    Vec::new()
  }
  /// Digest the constructor; This should occur in the Stomach to create a Whatsit.
  /// The whatsit which will be further processed to create the document.
  fn invoke_primitive(&self, stomach : &mut Stomach, state : &mut State) -> Vec<TBox> {
    println!("-- constructor/primitive invoke for {:?}", self.get_cs());
    // Call any `Before' code.
    // TODO: profiling / tracing 
      // let profiled = state.lookup_value("PROFILING") && ($LaTeXML::CURRENT_TOKEN || $$self{cs});
      // let tracing = state.lookup_value("TRACINGCOMMANDS");
      // LaTeXML::Core::Definition::startProfiling($profiled, "digest") if $profiled;

    let pre = self.execute_before_digest(stomach, state);

    // println_stderr!("{" + $self->tracingCSName . "}\n" if $tracing;
  // Get some info before we process arguments...
  // let font   = state.lookup_value("font");
  // let ismath = state.lookup_value("IN_MATH");
  // Parse AND digest the arguments to the Constructor
  let mut args : Vec<TBox> = match self.get_parameters() {
    &None => Vec::new(),
    &Some(ref params) => params.read_arguments_and_digest(stomach, &self, state)
  };
  // println_stderr!($self->tracingArgs(@args) . "\n" if $tracing && @args;
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
}

impl Constructor {
  fn execute_before_digest(&self, _stomach: &mut Stomach, _state: &mut State) {}
  fn execute_after_digest(&self,_stomach: &mut Stomach, _state: &mut State) {}

}