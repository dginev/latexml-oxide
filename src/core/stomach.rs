use core::gullet::{Gullet};
use core::token::{Token};
use common::object::Object;
use core::definition::{Definition,Expandable};
use core::tbox::*;
use state::{Scope,State};
use common::error::*;

static MAXSTACK : usize = 200;    /// [CONSTANT]

pub struct Stomach {
  pub gullet : Gullet,
  token_stack : Vec<Token>,
  boxing : Vec<String>
}

impl Default for Stomach {
  fn default() -> Self {
    Stomach {
      gullet : Gullet::default(),
      token_stack : Vec::new(),
      boxing : Vec::new()
    }
  }
}

impl Stomach {
  pub fn initialize(&mut self) {}
  pub fn get_gullet(&mut self) -> &mut Gullet {
    &mut self.gullet
  }

  //**********************************************************************
  // Digestion
  //**********************************************************************
  // NOTE: Worry about whether the $autoflush thing is right?
  // It puts a lot of cruft in Gullet; Should we just create a new Gullet?
  pub fn digest_next_body(&mut self, terminal : bool, state : &mut State) -> Vec<TBox> {
    let start_location = self.get_locator();
    let init_depth = self.boxing.len();
    let mut read_token : Option<Token>;
    let mut box_list : Vec<TBox> = Vec::new();

    loop {
      read_token = self.get_gullet().read_x_token(true, true, state); 
      match read_token {
        None => break,
        Some(token) => {
          for tbox in self.invoke_token(token, state) {
            box_list.push(tbox);
          }
          // TODO:
          //if terminal.is_some() && Equals(token, terminal.unwrap())
          if init_depth > self.boxing.len() {
            break;
          }
        }
      };
    }

    // Warn('expected', $terminal, $self, "body should have ended with '" . ToString($terminal) . "'",
    // "current body started at " . ToString($startloc))
    // if $terminal && !Equals($token, $terminal);
    if box_list.is_empty() {
      box_list.push(TBox()); // Dummy `trailer' if none explicit.
    }
    return box_list
  }

  fn get_locator(&self) -> String {
    "fake stomach locator".to_string()
  }

  /// Invoke a token;
  /// If it is a primitive or constructor, the definition will be invoked,
  /// possibly arguments will be parsed from the Gullet.
  /// Otherwise, the token is simply digested: turned into an appropriate box.
  /// Returns a list of boxes/whatsits. 
  fn invoke_token(&mut self, input_token : Token, state : &mut State) -> Vec<TBox> {
    let mut maybe_token = Some(input_token);
    println!("Invoking: {:?}", maybe_token);

    // Overly complex, but want to avoid recursion/stack
    let mut result : Vec<TBox> = Vec::new();
    // INVOKE:
    loop {
      if maybe_token.is_none() {
        break;
      }
      let token = maybe_token.unwrap();

      self.token_stack.push(token.clone());
      if self.token_stack.len() > MAXSTACK {
        // Fatal('internal', '<recursion>', $self,
        //   "Excessive recursion(?): ",
        //   "Tokens on stack: " . join(', ', map { ToString($_) } @{ $$self{token_stack} })); }
      }
      state.assign_value("CURRENT_TOKEN", Box::new(token.clone()), &Scope::Global);
      result = Vec::new();
      let looked_up_definition : Option<Box<Definition>> = state.lookup_digestable_definition(&token);
      match looked_up_definition {
        None => {// Supposedly executable token, but no definition!
         result = self.invoke_token_undefined(token, state); 
        },
        Some(mut meaning) => {
          if meaning.isa_token() { // Common case
            result = self.invoke_token_simple(token, meaning, state);
          } else if meaning.is_expandable() {
            // A math-active character will (typically) be a macro,
            // but it isn't expanded in the gullet, but later when digesting, in math mode (? I think)        

            let invoked_meaning = meaning.invoke(&mut self.gullet);
            self.gullet.unread(invoked_meaning);
            maybe_token = self.gullet.read_x_token(true, false, state); // replace the token by it's expansion!!!
            self.token_stack.pop();
            continue;
          } else if meaning.is_definition() { // Otherwise, a normal primitive or constructor
            result = meaning.invoke_primitive(self);
            if !meaning.is_prefix() {
              state.clear_prefixes(); // Clear prefixes unless we just set one.
            }
          } else {
            // TODO:
            // Fatal('misdefined', $meaning, $self, "The object " . Stringify($meaning) . " should never reach Stomach!");
          }
        }
      };
      break;
    }
    
    // TODO:
    // if grep { (!ref $_) || (!$_->isaBox) } @result {
    //   Fatal('misdefined', $token, $self,
    //   "Execution yielded non boxes",
    //   "Returned " . join(',', map { "'" . Stringify($_) . "'" }
    //       grep { (!ref $_) || (!$_->isaBox) } @result))
    // }
    
    self.token_stack.pop();
    println!("Got Box: {:?}", result);
    // TBox()
    return result
  }

  fn invoke_token_undefined(&mut self, mut token : Token, state : &mut State) -> Vec<TBox> {
    Vec::new()
  }
  fn invoke_token_simple(&mut self, mut token : Token, meaning : Box<Definition>, state : &mut State) -> Vec<TBox> {
    Vec::new()
  }

}