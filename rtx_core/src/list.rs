use {Digested, BoxOps};
use state::State;
use document::Document;

/// Lists can contain any Digested items, such as boxes, whatsits or other lists
#[derive(Debug)]
pub struct List {
  // TODO
  pub boxes: Vec<Digested>,
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

  fn be_absorbed(&mut self, document: &mut Document, state: &mut State) {}
}
