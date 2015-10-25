use core::mouth::{Mouth};

pub struct Gullet<'gullet> {
  pub mouths : Vec<Mouth<'gullet>>
}

impl<'gullet> Default for Gullet<'gullet> {
  fn default() -> Self {
    Gullet {
      mouths : Vec::new()
    }
  }
}

impl<'gullet> Gullet<'gullet> {

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