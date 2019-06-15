use std::borrow::Cow;
use std::fmt;
use std::rc::Rc;

use crate::common::error::*;
use crate::common::locator::Locator;
use crate::common::object::Object;
use crate::state::{Scope, State};

use crate::definition::{BeforeDigestClosure, Definition, DigestionClosure, ExpansionBody};
use crate::document::Document;
use crate::gullet::Gullet;
use crate::parameter::Parameters;
use crate::stomach::Stomach;
use crate::token::*;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::Digested;

#[derive(Clone)]
pub struct ExpandableOptions {
  pub locked: bool,
  pub protected: bool,
  pub outer: bool,
  pub long: bool,
  pub scope: Option<Scope>,
  pub alias: Option<String>,
}
impl Default for ExpandableOptions {
  fn default() -> Self {
    ExpandableOptions {
      locked: false,
      scope: None,
      protected: false,
      outer: false,
      long: false,
      alias: None,
    }
  }
}

#[derive(Clone)]
pub struct Expandable {
  pub is_protected: bool,
  pub is_long: bool,
  pub is_outer: bool,
  pub alias: Option<String>,
  pub locator: Locator,
  pub cs: Token,
  pub paramlist: Option<Parameters>,
  pub expansion: Option<ExpansionBody>,
  pub trivial_expansion: Option<Tokens>,
}
impl Default for Expandable {
  fn default() -> Self {
    Expandable {
      is_protected: false,
      is_long: false,
      is_outer: false,
      trivial_expansion: None,
      alias: None,
      locator: Locator::default(),
      cs: T_CS!("Expandable"),
      paramlist: None,
      expansion: None,
    }
  }
}
impl PartialEq for Expandable {
  fn eq(&self, other: &Expandable) -> bool { self.cs == other.cs }
}

impl fmt::Display for Expandable {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    unimplemented!();
  }
}
impl Object for Expandable {
  fn is_definition(&self) -> bool { true }
  fn is_expandable(&self) -> bool { true }
  fn get_locator(&self) -> Cow<Locator> { Cow::Borrowed(&self.locator) }
  fn stringify(&self) -> String { <Self as Definition>::stringify_type(&self, "Expandable") }
}
impl Definition for Expandable {
  fn is_protected(&self) -> bool { self.is_protected }
  fn get_parameters(&self) -> Option<&Parameters> { self.paramlist.as_ref() }
  fn get_cs(&self) -> Cow<Token> { Cow::Borrowed(&self.cs) }
  fn get_cs_name(&self) -> Cow<str> {
    Cow::Borrowed(match self.alias {
      Some(ref alias) => alias,
      None => self.cs.get_cs_name(),
    })
  }
  fn get_expansion(&self) -> Option<&ExpansionBody> { self.expansion.as_ref() }
  fn get_alias(&self) -> Option<&String> { self.alias.as_ref() }
  fn invoke(&self, gullet: &mut Gullet, state: &mut State) -> Result<Tokens> {
    // Expand the expandable control sequence. This should be carried out by the Gullet.
    // log!("-- expandable invoke for {:?}", self.get_cs());
    if let Some(ref trivial_expansion) = self.trivial_expansion {
      Ok(trivial_expansion.clone())
    } else {
      let args = self.read_arguments(gullet, state)?;
      self.do_invocation(gullet, args, state)
    }
  }

  // Not implemented for expandable
  fn invoke_primitive(&self, _gullet: &mut Stomach, _caller: Rc<dyn Definition>, _state: &mut State) -> Result<Vec<Digested>> { Ok(Vec::new()) }
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { None }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { None }
  fn do_absorbtion(&self, _document: &mut Document, _whatsit: &Whatsit, _state: &mut State) -> Result<()> {
    fatal!(Definition, Unexpected, "do_absorbtion on Expandable should never be called!");
  }
}

impl Expandable {
  pub fn new<T: Into<ExpansionBody>>(
    cs: Token,
    paramlist: Option<Parameters>,
    expansion: T,
    traits: Option<ExpandableOptions>,
    state: &State,
  ) -> Self
  {
    let expansion: ExpansionBody = expansion.into();
    let traits = traits.unwrap_or_else(ExpandableOptions::default);
    // let source = $STATE->getStomach->getGullet->getMouth;
    // if (ref $expansion eq 'LaTeXML::Core::Tokens') {
    //   Fatal('misdefined', $cs, $source, "Expansion of '" . ToString($cs) . "' has unbalanced {}",
    //     "Expansion is " . ToString($expansion)) unless $expansion->isBalanced;
    //   # If expansion is Tokens, and no arguments, we're a "trivial macro"
    let trivial_expansion = if let ExpansionBody::Tokens(ref tks) = expansion {
      if paramlist.is_none() && !tks.is_stub() {
        Some(tks.substitute_parameters(Vec::new()))
      } else {
        None
      }
    } else {
      None
    };

    Expandable {
      cs,
      paramlist,
      expansion: Some(expansion),
      trivial_expansion,
      // locator           => $source->getLocator,
      is_protected: traits.protected || state.get_prefix("protected"),
      is_outer: traits.outer || state.get_prefix("outer"),
      is_long: traits.long || state.get_prefix("long"),
      ..Expandable::default()
    }
  }

  fn do_invocation(&self, gullet: &mut Gullet, args: Vec<Tokens>, state: &mut State) -> Result<Tokens> {
    match self.expansion {
      Some(ExpansionBody::Closure(ref closure)) => closure(gullet, args.to_owned(), state),
      // but for tokens, make sure args are proper Tokens (lists)
      Some(ExpansionBody::Tokens(ref tks)) => {
        if !tks.is_stub() {
          Ok(tks.substitute_parameters(args))
        } else {
          Ok(Tokens!())
        }
      },
      // empty if no expansion
      None => Ok(Tokens!()),
    }
  }
}
