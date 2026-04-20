use libxml::tree::Node;
use std::borrow::Cow;

use crate::Digested;
use crate::common::arena::{SymHashMap};
use crate::common::store::Stored;
use crate::common::error::*;
use crate::common::object::Object;
use crate::definition::{
  BeforeDigestClosure, Definition, DigestionClosure, FontDirective, PrimitiveBody, Reversion,
};
use crate::document::Document;
use crate::parameter::Parameters;
use crate::state::Scope;
use crate::tbox::Tbox;
use crate::token::*;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::pin;

#[derive(Clone, Default)]
pub struct PrimitiveOptions {
  pub bounded:          bool,
  pub is_prefix:        bool,
  pub require_math:     bool,
  pub forbid_math:      bool,
  pub robust:           bool,
  pub locked:           bool,
  pub enter_horizontal: bool,
  pub leave_horizontal: bool,
  pub nargs:            Option<usize>,
  pub scope:            Option<Scope>,
  pub font:             Option<FontDirective>,
  pub mode:             Option<String>,
  pub alias:            Option<String>,
  pub before_digest:    Vec<BeforeDigestClosure>,
  pub after_digest:     Vec<DigestionClosure>,
  pub reversion:        Option<Reversion>,
}

#[derive(Clone)]
pub struct Primitive {
  pub cs:            Token,
  pub paramlist:     Option<Parameters>,
  // TODO: we have a case where the replacement is a simple string/character
  //       which gets auto-wrapped with a Tbox during invoke.
  pub replacement:   Option<PrimitiveBody>,
  pub before_digest: Vec<BeforeDigestClosure>,
  pub after_digest:  Vec<DigestionClosure>,
  pub alias:         Option<String>,
  pub nargs:         Option<usize>,
  pub reversion:     Option<Reversion>,
  pub is_prefix:     bool,
}
impl Default for Primitive {
  fn default() -> Self {
    Primitive {
      cs:            T_CS!("Primitive"),
      paramlist:     None,
      replacement:   None,
      alias:         None,
      before_digest: Vec::new(),
      after_digest:  Vec::new(),
      nargs:         None,
      reversion:     None,
      is_prefix:     false,
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

  fn invoke(&self, _once_only: bool) -> Result<Tokens> { Ok(Tokens!()) }
  fn invoke_primitive(&self) -> Result<Vec<Digested>> {
    Debug!("primitive invoke for {:?}", self.cs);
    // my $profiled = $state->lookupValue('PROFILING') && ($LaTeXML::CURRENT_TOKEN || $$self{cs});
    // my $tracing = $state->lookupValue('tracingcommands');
    // LaTeXML::Core::Definition::startProfiling($profiled, 'digest') if $profiled;
    // print STDERR '{' . $self->tracingCSName . "}\n" if $tracing;
    let mut invoked_boxes: Vec<Digested> = self.execute_before_digest()?;
    let args = self.read_arguments()?;
    // print STDERR $self->tracingArgs(@args) . "\n" if $tracing && @args;
    match self.replacement {
      Some(PrimitiveBody::Closure(ref closure)) => invoked_boxes.extend(closure(args)?),
      Some(PrimitiveBody::String(symbol)) => {
        // Perl L67: $stomach->enterHorizontal if defined $replacement
        crate::stomach::enter_horizontal();
        let cs_token = self
          .alias
          .as_ref()
          .map(|alias| Token::from(alias.as_str()))
          .unwrap_or(self.cs);
        let mut box_tokens = vec![cs_token];
        // Perl L69: append revertArguments for parameterized string primitives
        if let Some(ref params) = self.paramlist {
          for arg in &args {
            box_tokens.extend(arg.revert()?.unlist());
          }
          let _ = params; // acknowledge usage
        }
        let box_props = SymHashMap::default();
        invoked_boxes.push(Digested::from(Tbox::new(
          symbol,
          None,
          None,
          Tokens::new(box_tokens),
          box_props,
        )));
      },
      None => {
        // Perl: Box(undef, undef, undef, Tokens($self->getCSorAlias, ...), isEmpty => 1)
        // Even with no replacement, Perl creates a Box with the CS as reversion and isEmpty flag.
        // This is essential for font switches (\rm, \it, etc.) to appear in tex attributes.
        let cs_token = self
          .alias
          .as_ref()
          .map(|alias| Token::from(alias.as_str()))
          .unwrap_or(self.cs);
        let box_tokens = vec![cs_token];
        // TODO: add revert_arguments for ArgWrap type when needed
        let mut box_props = SymHashMap::default();
        box_props.insert("isEmpty", Stored::Bool(true));
        invoked_boxes.push(Digested::from(Tbox::new(
          pin!(""),
          None,
          None,
          Tokens::new(box_tokens),
          box_props,
        )));
      },
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

  fn do_absorption(&self, _document: &mut Document, _whatsit: &Whatsit) -> Result<Vec<Node>> {
    fatal!(
      Definition,
      Unexpected,
      "do_absorption on Primitive should never be called!"
    );
  }

  fn get_cs(&self) -> Cow<'_, Token> { Cow::Borrowed(&self.cs) }
  fn get_cs_name(&self) -> Cow<'_, str> { Cow::Owned(self.cs.with_cs_name(ToString::to_string)) }
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
