use std::fmt;
use {Digested, TexMode, BoxOps};
use state::State;
use document::Document;

/// Lists can contain any Digested items, such as boxes, whatsits or other lists
#[derive(Clone)]
pub struct List {
  pub boxes: Vec<Digested>,
  pub mode: TexMode
}

impl fmt::Debug for List {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    try!(write!(f, "\nList["));
    for tbox in self.boxes.iter() {
      try!(write!(f,"  {:?}\n", tbox));
    }
    write!(f,"]({:?})\n", self.mode)
  }
}

impl BoxOps for List {
  fn unlist(self) -> Vec<Digested> {
    self.boxes.into_iter().collect::<Vec<_>>()
  }

  fn to_string(&self) -> String {
    self.boxes
        .iter()
        .fold(String::new(), |joined, x| joined + &x.to_string())
  }

  /// NOTE: No longer used; Document->absorb bypasses this for stack efficiency.
  fn be_absorbed(self, document: &mut Document, state: &mut State) {
    for digested in self.unlist().into_iter() {
      document.absorb(digested, state);
    }
  }
}
