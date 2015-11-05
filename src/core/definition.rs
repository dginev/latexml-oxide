use core::gullet::Gullet;
use core::stomach::Stomach;
use core::token::*;
use core::tbox::TBox;
use core::parameter::Parameter;
use common::object::Object;
use state::State;


pub trait Definition : Object {
  fn invoke(&mut self, gullet : &mut Gullet) -> Vec<Token> {
    Vec::new()
  }
  fn invoke_primitive(&mut self, gullet : &mut Stomach) -> Vec<TBox> {
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
    unimplemented!()
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

pub type ExpansionClosure = Box<FnMut(&mut State) -> Vec<Token>>;
pub struct Expandable {
  pub is_protected : bool,
  pub alias : Option<String>,
  pub locator : String,
  pub cs : Token,
  pub paramlist : Vec<Parameter>,
  pub expansion : ExpansionClosure
}
impl Default for Expandable {
  fn default() -> Self {
    Expandable {
      is_protected : false,
      alias : None,
      locator : String::new(),
      cs : T_CS("Expandable".to_string()),
      paramlist : Vec::new(),
      expansion : Box::new(|state| {Vec::new()})
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
}