use log::info;
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

// DefMath Define a Mathematical symbol or function.
// There are two sets of cases:
//  (1) If the presentation appears to be TeX code, we create an XMDual,
// since the presentation may end up with structure, etc.
//  (2) But if the presentation is a simple string, or unicode,
// it is just the content of the symbol; even if the function takes arguments.
// ALSO
//  arrange that the operator token gets cs="$cs"
// ALSO
//  Possibly some trick with SUMOP/INTOP affecting limits ?
//  Well, not exactly, but....
// HMM.... Still fishy.
// When to make a dual ?
// If the $presentation seems to be TeX (ie. it involves #1... but not ONLY!)

// let simpletoken_options = {    # [CONSTANT]
//   name => 1,
//   meaning => 1,
//   omcd => 1,
//   role => 1,
//   mathstyle => 1,
//   font => 1,
//   scriptpos => 1,
//   scope => 1,
//   locked => 1 };

#[derive(Clone)]
pub struct MathPrimitiveOptions {
  pub bounded: bool,
  pub mode: Option<String>,
  pub before_digest: Vec<BeforeDigestClosure>,
  pub after_digest: Vec<DigestionClosure>,
  pub is_prefix: bool,
  pub scope: Option<Scope>,
  pub font: Option<Font>,
  pub require_math: bool,
  pub forbid_math: bool,
  pub locked: bool,
  pub alias: Option<String>,

  // Math specific
  pub name: Option<String>,
  pub meaning: Option<String>,
  pub omcd: Option<String>,
  pub reversion: bool,
  pub sizer: bool,
  pub role: Option<String>,
  pub operator_role: Option<String>,
  pub reorder: bool,
  pub dual: bool,
  pub mathstyle: Option<String>,
  pub scriptpos: Option<usize>,
  pub operator_scriptpos: Option<String>,
  pub stretchy: bool,
  pub operator_stretchy: bool,
  pub nogroup: bool,
  pub hide_content_reversion: bool,
}
impl Default for MathPrimitiveOptions {
  fn default() -> Self {
    MathPrimitiveOptions {
      bounded: false,
      before_digest: Vec::new(),
      after_digest: Vec::new(),
      mode: None,
      is_prefix: false,
      scope: None,
      require_math: false,
      forbid_math: false,
      locked: false,
      alias: None,
      font: None,

      // math-specific
      name: None,
      meaning: None,
      omcd: None,
      reversion: false,
      sizer: false,
      role: None,
      operator_role: None,
      reorder: false,
      dual: false,
      mathstyle: None,
      scriptpos: None,
      operator_scriptpos: None,
      stretchy: false,
      operator_stretchy: false,
      nogroup: true,
      hide_content_reversion: false,
    }
  }
}
impl PartialEq for MathPrimitiveOptions {
  fn eq(&self, other: &MathPrimitiveOptions) -> bool { self.name == other.name && self.meaning == other.meaning && self.role == other.role }
}

#[derive(Clone)]
pub struct MathPrimitive {
  pub cs: Token,
  pub paramlist: Option<Parameters>,
  pub nargs: Option<usize>,
  pub replacement: Option<PrimitiveClosure>,
  pub options: MathPrimitiveOptions,
  pub alias: Option<String>,
}
impl Default for MathPrimitive {
  fn default() -> Self {
    MathPrimitive {
      cs: T_CS!("MathPrimitive"),
      paramlist: None,
      nargs: None,
      replacement: None,
      options: MathPrimitiveOptions::default(),
      alias: None,
    }
  }
}
impl PartialEq for MathPrimitive {
  fn eq(&self, other: &MathPrimitive) -> bool { self.cs == other.cs }
}

impl Object for MathPrimitive {}
impl Definition for MathPrimitive {
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { Some(&self.options.before_digest) }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { Some(&self.options.after_digest) }

  fn invoke(&self, _gullet: &mut Gullet, _state: &mut State) -> Result<Tokens> { Ok(Tokens!()) }
  fn invoke_primitive(&self, stomach: &mut Stomach, _caller: Rc<Definition>, state: &mut State) -> Result<Vec<Digested>> {
    info!("-- Mathprimitive invoke for {:?}", self.cs);
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
    fatal!(Definition, Unexpected, "do_absorbtion on MathPrimitive should never be called!");
  }

  fn get_cs(&self) -> Cow<Token> { Cow::Borrowed(&self.cs) }
  fn get_cs_name(&self) -> Cow<str> { Cow::Borrowed(self.cs.get_cs_name()) }
  fn get_alias(&self) -> Option<String> { self.alias.clone() }
  fn get_locator(&self) -> String { unimplemented!() }
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
