use std::rc::Rc;
use state::{Scope, State};
use Digested;
use token::*;
use tokens::Tokens;
use gullet::Gullet;
use stomach::Stomach;
use parameter::Parameters;
use common::object::Object;
use common::error::*;
use definition::{BeforeDigestClosure, Definition, DigestionClosure, ExpansionClosure};
use whatsit::Whatsit;
use document::Document;

#[derive(Clone)]
pub struct ExpandableOptions {
  pub locked: bool,
  pub scope: Option<Scope>,
}
impl Default for ExpandableOptions {
  fn default() -> Self {
    ExpandableOptions {
      locked: false,
      scope: None,
    }
  }
}

#[derive(Clone)]
pub struct Expandable {
  pub is_protected: bool,
  pub alias: Option<String>,
  pub locator: String,
  pub cs: Token,
  pub paramlist: Option<Parameters>,
  pub expansion: Option<ExpansionClosure>,
  pub trivial_expansion: Option<Tokens>,
  pub options: ExpandableOptions,
}
impl Default for Expandable {
  fn default() -> Self {
    Expandable {
      is_protected: false,
      trivial_expansion: None,
      alias: None,
      locator: String::new(),
      cs: T_CS!("Expandable".to_string()),
      paramlist: None,
      expansion: None,
      options: ExpandableOptions::default(),
    }
  }
}
impl PartialEq for Expandable {
  fn eq(&self, other: &Expandable) -> bool { self.cs == other.cs }
}

impl Object for Expandable {
  fn is_definition(&self) -> bool { true }
  fn is_expandable(&self) -> bool { true }
}

impl Definition for Expandable {
  fn is_protected(&self) -> bool { self.is_protected }
  fn get_parameters(&self) -> &Option<Parameters> { &self.paramlist }
  fn get_cs(&self) -> Token { self.cs.clone() }

  fn get_cs_name(&self) -> String {
    match self.alias {
      Some(ref alias) => alias.clone(),
      None => self.cs.get_cs_name(),
    }
  }

  fn get_locator(&self) -> String { self.locator.clone() }

  fn invoke(&self, gullet: &mut Gullet, state: &mut State) -> Result<Tokens> {
    // Expand the expandable control sequence. This should be carried out by the Gullet.
    // log!("-- expandable invoke for {:?}", self.get_cs());
    if let Some(ref trivial_expansion) = self.trivial_expansion {
      Ok(trivial_expansion.clone())
    } else {
      let args = try!(self.read_arguments(gullet, state));
      self.do_invocation(gullet, args, state)
    }
  }

  // Not implemented for expandable
  fn invoke_primitive(
    &self,
    _gullet: &mut Stomach,
    _caller: Rc<Definition>,
    _state: &mut State,
  ) -> Result<Vec<Digested>>
  {
    Ok(Vec::new())
  }
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { None }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { None }
  fn do_absorbtion(
    &self,
    _document: &mut Document,
    _whatsit: &Whatsit,
    _state: &mut State,
  ) -> Result<()>
  {
    fatal!(
      Definition,
      Unexpected,
      "do_absorbtion on Expandable should never be called!"
    );
  }
}

impl Expandable {
  fn do_invocation(
    &self,
    gullet: &mut Gullet,
    args: Vec<Tokens>,
    state: &mut State,
  ) -> Result<Tokens>
  {
    if let Some(ref closure) = self.expansion {
      closure(gullet, args.clone(), state)
    } else {
      // empty if no expansion
      Ok(Tokens!())
    }
  }
}

#[macro_export]
macro_rules! SimpleExpansion(($tokens:expr ) => ({
  use std::rc::Rc;
  Some(Rc::new(move |_gullet, _args, _state| Ok($tokens)))
}));
