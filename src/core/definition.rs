use std::sync::Arc;
use core::gullet::Gullet;
use core::stomach::Stomach;
use core::token::*;
use core::tbox::TBox;
use core::parameter::Parameter;
use common::object::Object;
use state::State;


pub trait Definition : Object {
  fn invoke(&self, gullet : &mut Gullet, state : &mut State) -> Vec<Token> {
    Vec::new()
  }
  fn invoke_primitive(&self, gullet : &mut Stomach) -> Vec<TBox> {
    Vec::new()
  }

  fn get_cs(&self) -> Token;
  fn get_cs_name(&self) -> String;

  fn is_expandable(&self) -> bool { false }
  fn is_protected(&self) -> bool { false }
  fn is_register(&self) -> bool { false }
  fn is_prefix(&self) -> bool { false }

  fn get_locator(&self) -> String;

  fn read_arguments(&self, gullet : &mut Gullet) -> Vec<Token> {
    // my ($self, $gullet) = @_;
    // my $params = $self->getParameters;
    // return ($params ? $params->readArguments($gullet, $self) : ()); 
    Vec::new()
  }

  // pub fn get_parameters(&self) ->  {
  //   my ($self) = @_;
  //   // Allow defering these until the Definition is actually used.
  //   if ((defined $$self{parameters}) && !ref $$self{parameters}) {
  //     require LaTeXML::Package;
  //     $$self{parameters} = LaTeXML::Package::parseParameters($$self{parameters}, $$self{cs}); }
  //   return $$self{parameters}; 
  // }

  //======================================================================
  // Overriding methods
  fn stringify(&self) -> String {
    unimplemented!()
  }

  fn to_string(&self) -> String {
    unimplemented!()
  }

  // Return the Tokens that would invoke the given definition with arguments.
  fn invocation(&mut self, args : Vec<Token>) -> Vec<Token> {
    
    let mut invocation_result = Vec::new();
    invocation_result.push(self.get_cs());

    for rev_param in self.get_parameters().into_iter() {
      invocation_result.push(rev_param)
    }
    invocation_result
  }

  fn get_parameters(&self) -> Vec<Token> {
    Vec::new() // ??? How do we handle these
  }
}

pub type ExpansionClosure = Arc<Box<Fn(&mut State) -> Vec<Token>>>;
#[derive(Clone)]
pub struct Expandable {
  pub is_protected : bool,
  pub alias : Option<String>,
  pub locator : String,
  pub cs : Token,
  pub paramlist : Vec<Parameter>,
  pub expansion : ExpansionClosure,
  pub trivial_expansion : Option<Vec<Token>>,
}
impl Default for Expandable {
  fn default() -> Self {
    Expandable {
      is_protected : false,
      trivial_expansion : None,
      alias : None,
      locator : String::new(),
      cs : T_CS("Expandable".to_string()),
      paramlist : Vec::new(),
      expansion : Arc::new(Box::new(|state| {Vec::new()}))
    }
  }
}
impl Object for Expandable {
  fn is_definition(&self) -> bool { true }
}
impl Definition for Expandable {
  fn is_expandable(&self) -> bool { true }
  fn is_protected(&self) -> bool { self.is_protected }
  
  fn get_cs(&self) -> Token {
    self.cs.clone()
  }

  fn get_cs_name(&self) -> String {
    match &self.alias {
      &Some(ref alias) => alias.clone(),
      &None => self.cs.get_cs_name()
    }
  }

  fn get_locator(&self) -> String {
    self.locator.clone()
  }

  fn invoke(&self, gullet : &mut Gullet, state : &mut State) -> Vec<Token> {
    // Expand the expandable control sequence. This should be carried out by the Gullet.
    if self.trivial_expansion.is_some() {
      match &self.trivial_expansion { 
        &Some(ref expansion) => expansion.clone(),
        &None => Vec::new()
      }
    } else {
      let args = self.read_arguments(gullet);
      self.do_invocation(gullet, args, state)
    }
  }
}

impl Expandable {
  fn do_invocation(&self, gullet : &mut Gullet, args : Vec<Token>, state : &mut State) -> Vec<Token> {
    let mut closure : &ExpansionClosure = &self.expansion;
    closure(state)
  }
}