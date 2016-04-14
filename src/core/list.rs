use core::tbox::TBox;
use core::Digested;

/// Box is a Rust keyword, so we use "TBox" instead, as in "TeX Box"
#[derive(Debug)]
pub struct List {
  // TODO
  pub boxes: Vec<TBox>,
}

impl Digested for List {
  fn unlist(&self) -> Vec<&TBox> {
    self.boxes.iter().collect::<Vec<_>>()
  }

  fn to_string(&self) -> String {
    self.boxes
        .iter()
        .fold(String::new(), |joined, x| joined + &x.to_string())
  }
}
