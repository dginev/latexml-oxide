pub mod expandable;
pub mod constructor;
pub mod primitive;

use std::sync::Arc;
use std::collections::HashMap;

use Digested;
use gullet::Gullet;
use stomach::Stomach;
use token::Token;
use tokens::Tokens;
// use tbox::TBox;
use parameter::Parameters;
use document::Document;
use whatsit::Whatsit;
use state::{State, ObjectStore};

pub type ExpansionClosure = Arc<Fn(&mut Gullet, Vec<Tokens>, &mut State) -> Vec<Token>>;
pub type PrimitiveClosure = Arc<Fn(&mut Stomach, Vec<Tokens>, &mut State) -> Vec<Digested>>;
pub type BeforeDigestClosure = Arc<Fn(&mut Stomach, &mut State) -> Vec<Digested>>;
pub type DigestionClosure = Arc<Fn(&mut Stomach, &mut Whatsit, &mut State) -> Vec<Digested>>;
pub type ReplacementClosure = Arc<Fn(&mut Document,
                                     &Vec<Option<Digested>>,
                                     &HashMap<String, ObjectStore>,
                                     &mut State)
                                    >;
pub type ConstructionClosure = Arc<Fn(&mut Document, &Whatsit, &mut State)>;

pub trait Definition {
  fn invoke(&self, gullet: &mut Gullet, state: &mut State) -> Vec<Token>;
  fn invoke_primitive(&self, gullet: &mut Stomach, caller: Arc<Definition>, state: &mut State) -> Vec<Digested>;

  fn get_cs(&self) -> Token;
  fn get_cs_name(&self) -> String;

  fn is_protected(&self) -> bool {
    false
  }
  fn is_register(&self) -> bool {
    false
  }
  fn is_prefix(&self) -> bool {
    false
  }

  fn get_locator(&self) -> String;

  fn read_arguments(&self, gullet: &mut Gullet, state: &mut State) -> Vec<Tokens>
    where Self: Sized
  {
    match self.get_parameters() {
      &None => Vec::new(),
      &Some(ref params) => params.read_arguments(gullet, self, state),
    }
  }
  fn get_parameters(&self) -> &Option<Parameters>;

  // ======================================================================
  // Overriding methods
  fn stringify(&self) -> String {
    unimplemented!()
  }

  fn to_string(&self) -> String {
    unimplemented!()
  }

  // Return the Tokens that would invoke the given definition with arguments.
  fn invocation(&mut self, args: Vec<Token>, state: &mut State) -> Vec<Token> {

    let mut invocation_result = Vec::new();
    invocation_result.push(self.get_cs());

    match self.get_parameters() {
      &None => {}
      &Some(ref params) => {
        for result_token in params.revert_arguments(args, state) {
          invocation_result.push(result_token);
        }
      }
    }
    invocation_result
  }

  fn get_num_args(&self) -> usize {
    0
  }

  fn do_absorbtion(&self, _document: &mut Document, _whatsit: &Whatsit, _state: &mut State) {}
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> {None}
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> {None}
  fn after_digest_body(&self) -> Option<&Vec<DigestionClosure>> {None}
  fn capture_body(&self) -> bool;

  fn execute_before_digest(&self, stomach: &mut Stomach, state: &mut State) -> Vec<Digested> {
    state.unlocked = true;
    let mut before_digested = Vec::new();
    if let Some(pre_list) = self.before_digest() {
      for pre in pre_list.iter() {
        let before_digest_result = pre(stomach, state);
        before_digested.extend(before_digest_result);
      }
    }
    before_digested
  }
  fn execute_after_digest(&self, stomach: &mut Stomach, whatsit: &mut Whatsit, state: &mut State) -> Vec<Digested> {
    state.unlocked = true;
    let mut after_digested = Vec::new();
    if let Some(post_list) = self.after_digest() {
      for post in post_list.iter() {
        let after_digest_result = post(stomach, whatsit, state);
        after_digested.extend(after_digest_result);
      }
    }
    after_digested
  }

  fn execute_after_digest_body(&self, stomach: &mut Stomach, whatsit: &mut Whatsit, state: &mut State) -> Vec<Digested> {
    state.unlocked = true;
    let mut after_body_digested = Vec::new();
    if let Some(post_list) = self.after_digest_body() {
      // println_stderr!("Found {:?} after_digest_body closures, capture_body was: {:?}", post_list.len(), self.capture_body());
      for post in post_list {
        let after_body_digest_result = post(stomach, whatsit, state);
        after_body_digested.extend(after_body_digest_result);
      }
    }
    after_body_digested
  }
}
