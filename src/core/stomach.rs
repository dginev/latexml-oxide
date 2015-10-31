use core::gullet::{Gullet};
use core::token::{Token};
use state::State;

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
  pub fn digest_next_body(&mut self, terminal : bool, state : &mut State) -> String {
    let start_location = self.get_locator();
    let init_depth = self.boxing.len();
    let mut read_token : Option<Token>;
    let token_list : Vec<Token> = Vec::new();
    let mut gullet = self.get_gullet();

    loop {
      read_token = gullet.read_x_token(true, true, state); 
      match read_token {
        None => break,
        Some(token) => {
          println!("Read in token: {:?}", token);
        }
      };
    }
    "next body?".to_string()
  }

  fn get_locator(&self) -> String {
    "fake stomach locator".to_string()
  }
}