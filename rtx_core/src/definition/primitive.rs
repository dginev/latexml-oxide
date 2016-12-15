use std::sync::Arc;
use state::{State, Scope};
use Digested;
use common::object::Object;
use token::*;
use gullet::Gullet;
use stomach::Stomach;
use whatsit::Whatsit;
use parameter::Parameters;
use document::Document;
use definition::{Definition, PrimitiveClosure, BeforeDigestClosure, DigestionClosure};

#[derive(Clone)]
pub struct PrimitiveOptions {
  pub bounded: bool,
  pub mode: String, // TODO
  pub before_digest: Vec<BeforeDigestClosure>,
  pub after_digest: Vec<DigestionClosure>,
  pub is_prefix: bool,
  pub scope: Option<Scope>,
  // font : TODO
  pub require_math: bool,
  pub forbid_math: bool,
  pub locked: bool,
  pub alias: Option<String>,
}
impl Default for PrimitiveOptions {
  fn default() -> Self {
    PrimitiveOptions {
      bounded: false,
      before_digest: Vec::new(),
      after_digest: Vec::new(),
      mode: String::new(),
      is_prefix: false,
      scope: None,
      require_math: false,
      forbid_math: false,
      locked: false,
      alias: None,
    }
  }
}

#[derive(Clone)]
pub struct Primitive {
  pub cs: Token,
  pub paramlist: Option<Parameters>,
  pub nargs: Option<usize>,
  pub replacement: Option<PrimitiveClosure>,
  pub options: PrimitiveOptions,
}
impl Default for Primitive {
  fn default() -> Self {
    Primitive {
      cs: T_CS!("Primitive".to_string()),
      paramlist: None,
      nargs: None,
      replacement: None,
      options: PrimitiveOptions::default(),
    }
  }
}

impl Object for Primitive {}
impl Definition for Primitive {
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> {
    Some(&self.options.before_digest)
  }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> {
    Some(&self.options.after_digest)
  }
  fn capture_body(&self) -> bool {false}

  fn invoke(&self, _gullet: &mut Gullet, _state: &mut State) -> Vec<Token> {
    Vec::new()
  }
  fn invoke_primitive(&self, stomach: &mut Stomach, _caller: Arc<Definition>, state: &mut State) -> Vec<Digested> {
    println_stderr!("-- primitive invoke for {:?}", self.cs);
    // my $profiled = $STATE->lookupValue('PROFILING') && ($LaTeXML::CURRENT_TOKEN || $$self{cs});
    // my $tracing = $STATE->lookupValue('TRACINGCOMMANDS');
    // LaTeXML::Core::Definition::startProfiling($profiled, 'digest') if $profiled;
    // print STDERR '{' . $self->tracingCSName . "}\n" if $tracing;
    let mut result : Vec<Digested> = self.execute_before_digest(stomach, state);
    let args   = self.read_arguments(stomach.get_gullet_mut(), state);
    // print STDERR $self->tracingArgs(@args) . "\n" if $tracing && @args;
    let replacement_result = match &self.replacement {
      &None => Vec::new(),
      &Some(ref closure) => closure(stomach, args, state)
    };
    result.extend(replacement_result);
    let mut w = Whatsit::default();
    let after_result = self.execute_after_digest(stomach, &mut w, state);
    result.extend(after_result);

    // LaTeXML::Core::Definition::stopProfiling($profiled, 'digest') if $profiled;
    result
  }

  fn do_absorbtion(&self, _document: &mut Document, _whatsit: &Whatsit, _state: &mut State) {
    panic!("do_absorbtion on Primitive should never be called!");
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
  fn get_parameters(&self) -> &Option<Parameters> {
    &self.paramlist
  }
  fn get_num_args(&self) -> usize {
    let nargs = match self.nargs {
      Some(n) => n,
      None => {
        match &self.paramlist {
          &Some(ref params) => params.get_num_args(),
          &None => 0,
        }
      }
    };
    // TODO: Rethink the memoize in this immutable setting
    // self.nargs = Some(nargs);
    nargs
  }
}