use log::debug;
use std::borrow::Cow;
use std::rc::Rc;

use crate::common::error::*;
use crate::common::font::Font;
use crate::common::object::Object;
use crate::state::{Scope, State};

use crate::definition::{BeforeDigestClosure, Definition, DigestionClosure, PrimitiveClosure};
use crate::document::Document;
use crate::gullet::Gullet;
use crate::parameter::Parameters;
use crate::stomach::Stomach;
use crate::token::*;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::Digested;

#[derive(Clone)]
pub struct PrimitiveOptions {
  pub bounded: bool,
  pub is_prefix: bool,
  pub require_math: bool,
  pub forbid_math: bool,
  pub locked: bool,
  pub nargs: Option<usize>,
  pub scope: Option<Scope>,
  pub font: Option<Font>,
  pub mode: Option<String>,
  pub alias: Option<String>,
  pub before_digest: Vec<BeforeDigestClosure>,
  pub after_digest: Vec<DigestionClosure>,
}
impl Default for PrimitiveOptions {
  fn default() -> Self {
    PrimitiveOptions {
      bounded: false,
      before_digest: Vec::new(),
      after_digest: Vec::new(),
      mode: None,
      font: None,
      is_prefix: false,
      scope: None,
      require_math: false,
      forbid_math: false,
      locked: false,
      alias: None,
      nargs: None,
    }
  }
}

#[derive(Clone)]
pub struct Primitive {
  pub cs: Token,
  pub paramlist: Option<Parameters>,
  pub replacement: Option<PrimitiveClosure>,
  pub options: PrimitiveOptions,
}
impl Default for Primitive {
  fn default() -> Self {
    Primitive {
      cs: T_CS!("Primitive"),
      paramlist: None,
      replacement: None,
      options: PrimitiveOptions::default(),
    }
  }
}
impl PartialEq for Primitive {
  fn eq(&self, other: &Primitive) -> bool { self.cs == other.cs }
}

impl Object for Primitive {}
impl Definition for Primitive {
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { Some(&self.options.before_digest) }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { Some(&self.options.after_digest) }

  fn invoke(&self, _gullet: &mut Gullet, _state: &mut State) -> Result<Tokens> { Ok(Tokens!()) }
  fn invoke_primitive(&self, stomach: &mut Stomach, _caller: Rc<Definition>, state: &mut State) -> Result<Vec<Digested>> {
    debug!(target:"primitive", "invoke for {:?}", self.cs);
    // my $profiled = $STATE->lookupValue('PROFILING') && ($LaTeXML::CURRENT_TOKEN || $$self{cs});
    // my $tracing = $STATE->lookupValue('TRACINGCOMMANDS');
    // LaTeXML::Core::Definition::startProfiling($profiled, 'digest') if $profiled;
    // print STDERR '{' . $self->tracingCSName . "}\n" if $tracing;
    let mut result: Vec<Digested> = self.execute_before_digest(stomach, state)?;
    let args = self.read_arguments(stomach.get_gullet_mut(), state)?;
    // print STDERR $self->tracingArgs(@args) . "\n" if $tracing && @args;
    let replacement_result = match self.replacement {
      None => Vec::new(),
      Some(ref closure) => closure(stomach, args, state)?,
    };
    result.extend(replacement_result);
    let mut w = Whatsit::default();
    let after_result = self.execute_after_digest(stomach, &mut w, state)?;
    result.extend(after_result);

    // LaTeXML::Core::Definition::stopProfiling($profiled, 'digest') if $profiled;
    Ok(result)
  }

  fn do_absorbtion(&self, _document: &mut Document, _whatsit: &Whatsit, _state: &mut State) -> Result<()> {
    fatal!(Definition, Unexpected, "do_absorbtion on Primitive should never be called!");
  }

  fn get_cs(&self) -> Cow<Token> { Cow::Borrowed(&self.cs) }
  fn get_cs_name(&self) -> Cow<str> { Cow::Borrowed(self.cs.get_cs_name()) }
  fn get_locator(&self) -> String { unimplemented!() }
  fn get_parameters(&self) -> &Option<Parameters> { &self.paramlist }
  fn get_num_args(&self) -> usize {
    match self.options.nargs {
      Some(n) => n,
      None => match self.paramlist {
        Some(ref params) => params.get_num_args(),
        None => 0,
      },
    }
    // TODO: Rethink the memoize in this immutable setting
    // self.nargs = Some(nargs);
  }
}
