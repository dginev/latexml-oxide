use std::collections::HashMap;
use state::{Scope, State, ObjectStore};
// use common::error::*;
use Digested;
use gullet::Gullet;
use token::{Token, Catcode};
use definition::Definition;
use tbox::*;

static MAXSTACK: usize = 200;
/// [CONSTANT]

pub struct Stomach {
  pub gullet: Gullet,
  token_stack: Vec<Token>,
  boxing: Vec<String>,
}

impl Default for Stomach {
  fn default() -> Self {
    Stomach {
      gullet: Gullet::default(),
      token_stack: Vec::new(),
      boxing: Vec::new(),
    }
  }
}

impl Stomach {
  pub fn initialize(&mut self) {}
  pub fn get_gullet_mut(&mut self) -> &mut Gullet {
    &mut self.gullet
  }
  pub fn get_gullet(&self) -> &Gullet {
    &self.gullet
  }

  // **********************************************************************
  // Digestion
  // **********************************************************************
  // NOTE: Worry about whether the $autoflush thing is right?
  // It puts a lot of cruft in Gullet; Should we just create a new Gullet?
  pub fn digest_next_body(&mut self, terminal: bool, state: &mut State) -> Vec<Digested> {
    let start_location = self.get_locator();
    let init_depth = self.boxing.len();
    let mut read_token: Option<Token>;
    let mut box_list: Vec<Digested> = Vec::new();

    loop {
      read_token = self.get_gullet_mut().read_x_token(true, true, state);
      match read_token {
        None => break,
        Some(token) => {
          // println_stderr!("Read in token: {:?}", token);
          for digested in self.invoke_token(token, state) {
            box_list.push(digested);
          }
          // TODO:
          // if terminal.is_some() && Equals(token, terminal.unwrap())
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
      box_list.push(Digested::BoxObj(TBox())); // Dummy `trailer' if none explicit.
    }
    return box_list;
  }

  fn get_locator(&self) -> String {
    "fake stomach locator".to_string()
  }

  /// Invoke a token;
  /// If it is a primitive or constructor, the definition will be invoked,
  /// possibly arguments will be parsed from the Gullet.
  /// Otherwise, the token is simply digested: turned into an appropriate box.
  /// Returns a list of boxes/whatsits.
  fn invoke_token(&mut self, input_token: Token, state: &mut State) -> Vec<Digested> {
    let mut maybe_token = Some(input_token);

    // Overly complex, but want to avoid recursion/stack
    let mut result: Vec<Digested> = Vec::new();
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
      state.assign_value("CURRENT_TOKEN",
                         ObjectStore::TokenStore(token.clone()),
                         &Some(Scope::Global));
      result = Vec::new();
      let mut looked_up_definition: Option<ObjectStore> = state.lookup_digestable_definition(&token);
      match looked_up_definition {
        None => {
          // Supposedly executable token, but no definition!
          result = self.invoke_token_undefined(token, state);
        }
        Some(store) => {
          match store {
            ObjectStore::TokenStore(meaning) => {
              // Common case
              result = self.invoke_token_simple(token, meaning, state);
            }
            ObjectStore::ExpandableStore(meaning) => {
              // A math-active character will (typically) be a macro,
              // but it isn't expanded in the gullet, but later when digesting, in math mode (? I think)
              let invoked_meaning = meaning.invoke(&mut self.gullet, state);
              self.gullet.unread(invoked_meaning);
              maybe_token = self.gullet.read_x_token(true, false, state); // replace the token by it's expansion!!!
              self.token_stack.pop();
              continue;
            }
            ObjectStore::ConstructorStore(meaning) => {
              // Otherwise, a normal primitive or constructor
              result = meaning.invoke_primitive(self, meaning.clone(), state);
              if !meaning.is_prefix() {
                state.clear_prefixes(); // Clear prefixes unless we just set one.
              }
            }
            _ => {
              // TODO:
              // Fatal('misdefined', $meaning, $self, "The object " . Stringify($meaning) . " should never reach Stomach!");
            }
          };
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
    // println!("Got Box: {:?}", result);
    // TBox()
    return result;
  }

  fn invoke_token_undefined(&mut self, token: Token, state: &mut State) -> Vec<Digested> {
    // println_stderr!("-- Undefined invoke {:?}", token);
    // TODO: Rework this carefully
    Vec::new()
  }
  fn invoke_token_simple(&mut self, token: Token, meaning: Token, state: &mut State) -> Vec<Digested> {
    // println_stderr!("-- Simple invoke {:?}", token);
    // let font = state.lookup_value("font");
    state.clear_prefixes();    // prefixes shouldn't apply here.

    if meaning.code == Catcode::SPACE {
      let in_math_lookup: Option<&ObjectStore> = state.lookup_value("IN_MATH");
      let in_math = match in_math_lookup {
        Some(&ObjectStore::BoolStore(x)) => x,
        _ => false,
      };
      let in_preamble_lookup: Option<&ObjectStore> = state.lookup_value("inPreamble");
      let in_preamble = match in_preamble_lookup {
        Some(&ObjectStore::BoolStore(x)) => x,
        _ => false,
      };
      if in_math || in_preamble {
        Vec::new()
      } else {
        vec![Digested::BoxObj(TBox {
               text: meaning.to_string(),
               font: String::new(),
               locator: self.gullet.get_locator(),
               tokens: vec![meaning],
               properties: HashMap::new(),
             })]
      }
    } else if meaning.code == Catcode::COMMENT {
      // Note: Comments need char decoding as well!
      //  let comment = LaTeXML::Package::FontDecodeString($meaning->getString, undef, 1);
      // // However, spaces normally would have be digested away as positioning...
      // my $badspace = pack('U', 0xA0) . "\x{0335}";    // This is at space's pos in OT1
      // $comment =~ s/\Q$badspace\E/ /g;
      // return LaTeXML::Comment->new($comment); }
      Vec::new()
    }
    // TODO
    // else if ($forbidden_cc[meaning.code]) {
    // Fatal('misdefined', $token, $self,
    //   "The token " . Stringify($token) . " should never reach Stomach!");
    // return; }
    else {
      vec![Digested::BoxObj(TBox {
             text: meaning.to_string(),
             font: String::new(),
             locator: String::new(),
             tokens: vec![meaning],
             properties: HashMap::new(),
           })]
    }
  }
}
