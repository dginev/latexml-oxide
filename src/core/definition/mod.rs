pub mod expandable;
pub mod constructor;
pub mod primitive;
pub mod compiler;

use core::gullet::Gullet;
use core::stomach::Stomach;
use core::token::*;
use core::tbox::TBox;
use core::parameter::Parameters;
use common::object::Object;
use state::State;

pub trait Definition : Object {
  fn invoke(&self, gullet : &mut Gullet, state : &mut State) -> Vec<Token>;
  fn invoke_primitive(&self, gullet : &mut Stomach, state : &mut State) -> Vec<TBox>;

  fn get_cs(&self) -> Token;
  fn get_cs_name(&self) -> String;

  fn is_protected(&self) -> bool { false }
  fn is_register(&self) -> bool { false }
  fn is_prefix(&self) -> bool { false }

  fn get_locator(&self) -> String;

  fn read_arguments(&self, gullet : &mut Gullet, state : &mut State) -> Vec<Token> where Self: Sized {
    match self.get_parameters() {
      &None => Vec::new(),
      &Some(ref params) => params.read_arguments(gullet, self, state)
    }
  }
  fn get_parameters(&self) -> &Option<Parameters>;

  //======================================================================
  // Overriding methods
  fn stringify(&self) -> String {
    unimplemented!()
  }

  fn to_string(&self) -> String {
    unimplemented!()
  }

  // Return the Tokens that would invoke the given definition with arguments.
  fn invocation(&mut self, args : Vec<Token>, state : &mut State) -> Vec<Token> {
    
    let mut invocation_result = Vec::new();
    invocation_result.push(self.get_cs());

    match self.get_parameters() {
      &None => {},
      &Some(ref params) => {
        for result_token in params.revert_arguments(args, state) {
          invocation_result.push(result_token);
        }
      }
    }
    invocation_result
  }

  fn get_num_args(&self) -> usize { 0 }
}

