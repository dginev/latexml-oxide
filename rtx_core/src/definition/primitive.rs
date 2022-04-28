use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;

use crate::common::error::*;
use crate::common::font::Font;
use crate::common::object::Object;
use crate::state::{Scope, State};

use crate::definition::{BeforeDigestClosure, Definition, DigestionClosure, PrimitiveClosure, Reversion};
use crate::document::Document;
use crate::gullet::Gullet;
use crate::parameter::Parameters;
use crate::stomach::Stomach;
use crate::token::*;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::{Digested, Locator};

#[derive(Clone, Default)]
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
  pub reversion: Option<Reversion>
}

#[derive(Clone)]
pub struct Primitive {
  pub cs: Token,
  pub paramlist: Option<Parameters>,
  pub replacement: Option<PrimitiveClosure>,
  pub before_digest: Vec<BeforeDigestClosure>,
  pub after_digest: Vec<DigestionClosure>,
  pub alias: Option<String>,
  pub nargs: Option<usize>,
  pub reversion: Option<Reversion>,
  pub is_prefix: bool,
}
impl Default for Primitive {
  fn default() -> Self {
    Primitive {
      cs: T_CS!("Primitive"),
      paramlist: None,
      replacement: None,
      alias: None,
      before_digest: Vec::new(),
      after_digest: Vec::new(),
      nargs: None,
      reversion: None,
      is_prefix: false,
    }
  }
}
impl PartialEq for Primitive {
  fn eq(&self, other: &Primitive) -> bool { self.cs == other.cs }
}

impl fmt::Display for Primitive {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    unimplemented!();
  }
}
impl Object for Primitive {
  fn stringify(&self) -> String { <Self as Definition>::stringify_type(self, "Primitive") }
  fn get_locator(&self) -> Cow<Locator> { unimplemented!() }
}
impl Definition for Primitive {
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { Some(&self.before_digest) }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { Some(&self.after_digest) }
  fn is_prefix(&self) -> bool { self.is_prefix }

  fn invoke(&self, _gullet: &mut Gullet, _once_only: bool, _state: &mut State) -> Result<Tokens> { Ok(Tokens!()) }
  fn invoke_primitive(&self, stomach: &mut Stomach, _caller: Arc<dyn Definition>, state: &mut State) -> Result<Vec<Digested>> {
    Debug!("primitive invoke for {:?}", self.cs);
    // my $profiled = $STATE->lookupValue('PROFILING') && ($LaTeXML::CURRENT_TOKEN || $$self{cs});
    // my $tracing = $STATE->lookupValue('TRACINGCOMMANDS');
    // LaTeXML::Core::Definition::startProfiling($profiled, 'digest') if $profiled;
    // print STDERR '{' . $self->tracingCSName . "}\n" if $tracing;
    let mut invoked_boxes: Vec<Digested> = self.execute_before_digest(stomach, state)?;
    let args = self.read_arguments(stomach.get_gullet_mut(), state)?;
    // print STDERR $self->tracingArgs(@args) . "\n" if $tracing && @args;
    if let Some(ref closure) = self.replacement {
      invoked_boxes.extend(closure(stomach, args, state)?);
    }
    if !self.after_digest.is_empty() {
      // optimize to avoid needless generation of whatsits
      let mut w = Whatsit::default();
      let after_boxes = self.execute_after_digest(stomach, &mut w, state)?;
      invoked_boxes.extend(after_boxes);
    }

    // LaTeXML::Core::Definition::stopProfiling($profiled, 'digest') if $profiled;
    Ok(invoked_boxes)
  }

  fn do_absorbtion(&self, _document: &mut Document, _whatsit: &Whatsit, _state: &mut State) -> Result<()> {
    fatal!(Definition, Unexpected, "do_absorbtion on Primitive should never be called!");
  }

  fn get_cs(&self) -> Cow<Token> { Cow::Borrowed(&self.cs) }
  fn get_cs_name(&self) -> Cow<str> { Cow::Borrowed(self.cs.get_cs_name()) }
  fn get_alias(&self) -> Option<&String> { self.alias.as_ref() }
  fn get_parameters(&self) -> Option<&Parameters> { self.paramlist.as_ref() }

  fn get_num_args(&self) -> usize {
    match self.nargs {
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
