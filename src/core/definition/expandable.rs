use std::sync::Arc;
use state::State;
use core::token::*;
use core::tbox::TBox;
use core::gullet::Gullet;
use core::stomach::Stomach;
use core::parameter::Parameters;
use common::object::Object;
use core::definition::Definition;

pub type ExpansionClosure = Arc<Box<Fn(&mut Gullet, Vec<Token>, &mut State) -> Vec<Token>>>;
#[derive(Clone)]
pub struct Expandable {
  pub is_protected: bool,
  pub alias: Option<String>,
  pub locator: String,
  pub cs: Token,
  pub paramlist: Option<Parameters>,
  pub expansion: ExpansionClosure,
  pub trivial_expansion: Option<Vec<Token>>,
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
      expansion: Arc::new(Box::new(|_gullet, _args, _state| Vec::new())),
    }
  }
}
impl Object for Expandable {
  fn is_definition(&self) -> bool {
    true
  }
  fn is_expandable(&self) -> bool {
    true
  }
}
impl Definition for Expandable {
  fn is_protected(&self) -> bool {
    self.is_protected
  }
  fn get_parameters(&self) -> &Option<Parameters> {
    &self.paramlist
  }
  fn get_cs(&self) -> Token {
    self.cs.clone()
  }

  fn get_cs_name(&self) -> String {
    match &self.alias {
      &Some(ref alias) => alias.clone(),
      &None => self.cs.get_cs_name(),
    }
  }

  fn get_locator(&self) -> String {
    self.locator.clone()
  }

  fn invoke(&self, gullet: &mut Gullet, state: &mut State) -> Vec<Token> {
    // Expand the expandable control sequence. This should be carried out by the Gullet.
    println!("-- expandable invoke for {:?}", self.get_cs());
    if self.trivial_expansion.is_some() {
      match &self.trivial_expansion { 
        &Some(ref expansion) => expansion.clone(),
        &None => Vec::new(),
      }
    } else {
      let args = self.read_arguments(gullet, state);
      self.do_invocation(gullet, args, state)
    }
  }

  fn invoke_primitive(&self, _gullet: &mut Stomach, _state: &mut State) -> Vec<TBox> {
    Vec::new()
  }
}

impl Expandable {
  fn do_invocation(&self, gullet: &mut Gullet, args: Vec<Token>, state: &mut State) -> Vec<Token> {
    let closure: &ExpansionClosure = &self.expansion;
    closure(gullet, args, state)
  }
}
