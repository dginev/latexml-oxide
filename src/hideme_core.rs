use common::{InputFormat};
use core::token;
use state::{State};

pub struct Core {
  pub state : State,
}
pub struct Digested {
  pub stuff : String,
}

impl Core {
  pub fn digest(&self, source : String,
    preamble : Option<String>, postamble : Option<String>, mode : String, no_init : bool) 
    -> Option<Digested> {
    
    Some(Digested{ stuff: source})
  }

  pub fn digested_to_serialized(&self, format : OutputFormat, digested : Digested) -> Option<String> {
    let mut serialized = match format {
      OutputFormat::TeX => Some(core::token::untex(digested)),
      OutputFormat::Box => {
        if opts.verbosity > 0 {
          Some(digested.stringify())
        } else {
          Some(digested.toString())
        }
      },
      _ => { // Default is XML
         // $dom = $latexml->convertDocument($digested);
         None
      }
    };
    return serialized
  }
}