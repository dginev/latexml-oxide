pub mod token;

use common::{OutputFormat, Error};
// use core::token;
use state::{State};

pub struct Core {
  pub state : State,
}
pub struct Digested {
  pub stuff : String,
}

impl Digested {
  pub fn to_string(&self) -> String {
    self.stuff.clone()
  }
  pub fn stringify(&self) -> String {
    self.stuff.clone()
  }
}

impl Default for Core {
  fn default() -> Self {
    Core {
      state : State {
        verbosity : 0,
        status_code: 0,
        map : Vec::new()
      }
    }
  }
}

impl Core {
  pub fn digest(&self, source : String,
    preamble : Option<String>, postamble : Option<String>, mode : String, no_init : bool) 
    -> Result<Digested, Error> {
    
    Ok(Digested{ stuff: source})
  }

  pub fn convert_document(&self, digested : Digested) -> Result<String, Error> {
    Ok(digested.stuff)
  }
}