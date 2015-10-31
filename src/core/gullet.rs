use core::mouth::{Mouth};
use core::token::{Token, Catcode};
use core::definition::{Definition};
use state::State;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct MouthRuntime {
    pub autoclose : bool,
    pub mouth : Mouth,
    pub pushback : VecDeque<Token>,
}

pub struct Gullet {
  pub mouth : Option<MouthRuntime>,
  pub mouthstack : VecDeque<MouthRuntime>,
  pub pending_comments : VecDeque<Token>
}

impl Default for Gullet {
  fn default() -> Self {
    Gullet {
      mouth : None,
      mouthstack : VecDeque::new(),
      pending_comments : VecDeque::new(),
    }
  }
}

impl Gullet {

  pub fn flush(&self) {
    // TODO
  }

  pub fn has_more_input(&self) -> bool {
    match self.mouth {
      Some(ref runtime) => runtime.mouth.has_more_input(),
      None => false
    }
  }

  pub fn open_mouth(&mut self, mouth : Mouth, autoclose : bool) {
    match self.mouth {
      Some(ref runtime) => {
        self.mouthstack.push_front(runtime.clone());
      },
      None => {}
     };
    self.mouth = Some(MouthRuntime {
      mouth : mouth,
      pushback : VecDeque::new(),
      autoclose : autoclose
    });
  }

  pub fn close_mouth<'close>(&'close mut self) {

  }

  pub fn read_token(&mut self, state : &mut State) -> Option<Token> {
    None
  }

  // Read the next non-expandable token (expanding tokens until there's a non-expandable one).
  // Note that most tokens pass through here, so be Fast & Clean! readToken is folded in.
  // `Toplevel' processing, (if $toplevel is true), used at the toplevel processing by Stomach,
  //  will step to the next input stream (Mouth) if one is available,
  // If $commentsok is true, will also pass comments.
  pub fn read_x_token(&mut self, toplevel : bool, commentsok : bool, state : &mut State) -> Option<Token> {
    // toplevel should be true by default
    if commentsok && !self.pending_comments.is_empty() {
      return self.pending_comments.pop_front()
    }

    loop {
      let mut read_token : Option<Token>;
      let mut cc : Catcode;
      let mut defn_next : Option<Definition> = None;
      let mut needs_close = false;
      let mut return_next = false;
      let mut expand_next = false;
      match self.mouth {
        None => {return None;},
        Some(ref mut runtime) => {    

          read_token = match runtime.pushback.is_empty() {
            false => runtime.pushback.pop_front(),
            true => runtime.mouth.read_token(state)
          };
          match read_token {
            None => {
              if !(runtime.autoclose && toplevel && !self.mouthstack.is_empty()) {
                return None;
              }
              needs_close = true; // Close mouth
            },
            Some(token) => {
              cc = token.code;
              match cc {
                Catcode::NOTEXPANDED => {    // NOTE: Inlined ->getCatcode
                  // Should only occur IMMEDIATELY after expanding \noexpand (by readXToken),
                  // so this token should never leak out through an EXTERNAL call to readToken.
                  return_next = true; //just return next token
                },
                Catcode::COMMENT => {
                  match commentsok {
                    true => { 
                      return Some(token);
                    },
                    false => {
                      self.pending_comments.push_back(token);
                    }    // What to do with comments???
                  }
                },
                // Catcode::MARKER => {
                //   LaTeXML::Core::Definition::stopProfiling($token, 'expand'); }        
                // }
                _ => {
                  match state.lookup_definition(&token) {
                    Some(defn) => {
                      if defn.is_expandable && (toplevel || !defn.is_protected) {
                        // is this the right logic here? don't expand unless digesting?
                        state.assign_value("current_token", Box::new(token));
                        defn_next = Some((*defn).clone()); 
                        expand_next = true;
                      } else {
                        return Some(token)
                      }
                    },
                    None => {
                      return Some(token)
                    }
                  };
                }
              };
            }
          }
        }
      };
      if needs_close {
        self.close_mouth(); // Next input stream.
      } else if return_next {
        return self.read_token(state);    // Just return the next token.
      } else if expand_next {
        // Do the check here, to be more forgiving and more informative
        let expansion = match defn_next {
          Some(mut defn) => defn.invoke(self),
          None => Vec::new()
        };
        // _ => Error("misdefined", token, undef,
        //         "Expected a Token in expansion of " . ToString($token),
        //         "got " . Stringify($_))

        // already checked tokens, so just push to be re-read (like ->unread(@expansion); )
        match self.mouth {
          None => {return None;},
          Some(ref mut runtime) => {    
            for expansion_token in expansion.into_iter() {
              runtime.pushback.push_front(expansion_token);
            }
          }
        };
      }
    }
  }
}