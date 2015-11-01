use core::gullet::{Gullet};
use core::token::{Token};
use core::tbox::*;
use state::State;
use common::error::*;

pub struct Stomach {
  pub gullet : Gullet,
  boxing : Vec<String>
}

impl Default for Stomach {
  fn default() -> Self {
    Stomach {
      gullet : Gullet::default(),
      boxing : Vec::new()
    }
  }
}

impl Stomach {
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
          println!("Read in token: {:?}", token);
          box_list.push(self.invoke_token(token));
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

  fn invoke_token(&mut self, token : Token) -> TBox {
    TBox()
  }
}