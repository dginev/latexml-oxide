use core::mouth::{Mouth};
use core::token::{Token};
use std::collections::VecDeque;

#[derive(Clone)]
pub struct MouthRuntime {
    pub autoclose : bool,
    pub mouth : Mouth,
    pub pushback : VecDeque<Token>,
}

pub struct Gullet {
  pub mouth : Option<MouthRuntime>,
  pub mouthstack : VecDeque<MouthRuntime>
}

impl Default for Gullet {
  fn default() -> Self {
    Gullet {
      mouth : None,
      mouthstack : VecDeque::new()
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
}