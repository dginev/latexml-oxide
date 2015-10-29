use core::mouth::{Mouth};

pub struct Gullet {
  pub mouths : Vec<Mouth>
}

impl Default for Gullet {
  fn default() -> Self {
    Gullet {
      mouths : Vec::new()
    }
  }
}

impl Gullet {

  pub fn flush(&self) {
    // TODO
  }

  pub fn has_more_input(&self) -> bool {
    let current_mouth = self.mouths.last();
    match current_mouth {
      Some(m) => (*m).has_more_input(),
      None => false
    }
  }
  
}