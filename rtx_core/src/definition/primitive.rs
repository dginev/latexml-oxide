use libxml::tree::Node;
use std::borrow::Cow;
use rustc_hash::FxHashMap as HashMap;

use crate::common::error::*;
use crate::common::object::Object;
use crate::state::{Scope};
use crate::common::arena::EMPTY_SYM;
use crate::definition::{
  BeforeDigestClosure, Definition, DigestionClosure, FontDirective, PrimitiveBody, Reversion,
};
use crate::document::Document;
use crate::parameter::Parameters;
use crate::tbox::Tbox;
use crate::token::*;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::{Digested};

#[derive(Clone, Default)]
pub struct PrimitiveOptions {
  pub bounded: bool,
  pub is_prefix: bool,
  pub require_math: bool,
  pub forbid_math: bool,
  pub robust: bool,
  pub locked: bool,
  pub nargs: Option<usize>,
  pub scope: Option<Scope>,
  pub font: Option<FontDirective>,
  pub mode: Option<String>,
  pub alias: Option<String>,
  pub before_digest: Vec<BeforeDigestClosure>,
  pub after_digest: Vec<DigestionClosure>,
  pub reversion: Option<Reversion>,
}

#[derive(Clone)]
pub struct Primitive {
  pub cs: Token,
  pub paramlist: Option<Parameters>,
  // TODO: we have a case where the replacement is a simple string/character
  //       which gets auto-wrapped with a Tbox during invoke.
  pub replacement: Option<PrimitiveBody>,
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

// impl fmt::Display for Primitive {
//   fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
//     todo!();
//   }
// }
impl Object for Primitive {
  fn stringify(&self) -> String { <Self as Definition>::stringify_type(self, "Primitive") }

}
impl Definition for Primitive {
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { Some(&self.before_digest) }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { Some(&self.after_digest) }
  fn is_prefix(&self) -> bool { self.is_prefix }

  fn invoke(&self, _once_only: bool) -> Result<Tokens> {
    Ok(Tokens!())
  }
  fn invoke_primitive(&self) -> Result<Vec<Digested>> {
    Debug!("primitive invoke for {:?}", self.cs);
    // my $profiled = $state->lookupValue('PROFILING') && ($LaTeXML::CURRENT_TOKEN || $$self{cs});
    // my $tracing = $state->lookupValue('TRACINGCOMMANDS');
    // LaTeXML::Core::Definition::startProfiling($profiled, 'digest') if $profiled;
    // print STDERR '{' . $self->tracingCSName . "}\n" if $tracing;
    let mut invoked_boxes: Vec<Digested> = self.execute_before_digest()?;
    let args = self.read_arguments()?;
    // print STDERR $self->tracingArgs(@args) . "\n" if $tracing && @args;
    match self.replacement {
      Some(PrimitiveBody::Closure(ref closure)) => invoked_boxes.extend(closure(args)?),
      Some(PrimitiveBody::String(symbol)) => {
        let cs_token = self.alias.as_ref().map(|alias| T_CS!(alias)).unwrap_or(self.cs);
        let box_tokens = vec![cs_token];
        if let Some(ref _params) = self.paramlist {
          todo!(); // we need to generalize the revert functions to take ArgWrap-typed arguments
          // box_tokens.extend(params.revert_arguments(args)?);
        }
        let box_props = if symbol == *EMPTY_SYM {
          HashMap::default()
        } else {
          stored_map!("isEmpty" => true)
        };
        invoked_boxes.push(Digested::from(
          Tbox::new(symbol, None, None, Tokens::new(box_tokens),
          box_props))
        );
      },
      None => {}
    }
    if !self.after_digest.is_empty() {
      // optimize to avoid needless generation of whatsits
      let mut w = Whatsit::default();
      let after_boxes = self.execute_after_digest(&mut w)?;
      invoked_boxes.extend(after_boxes);
    }

    // LaTeXML::Core::Definition::stopProfiling($profiled, 'digest') if $profiled;
    Ok(invoked_boxes)
  }

  fn do_absorbtion(
    &self,
    _document: &mut Document,
    _whatsit: &Whatsit,
  ) -> Result<Vec<Node>> {
    fatal!(
      Definition,
      Unexpected,
      "do_absorbtion on Primitive should never be called!"
    );
  }

  fn get_cs(&self) -> Cow<Token> { Cow::Borrowed(&self.cs) }
  fn get_cs_name(&self) -> Cow<str> { Cow::Owned(self.cs.with_cs_name(ToString::to_string)) }
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
