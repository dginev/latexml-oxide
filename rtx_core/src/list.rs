use {Digested, TexMode, BoxOps};
use state::State;
use document::Document;

/// Lists can contain any Digested items, such as boxes, whatsits or other lists
#[derive(Debug, Clone)]
pub struct List {
  // TODO
  pub boxes: Vec<Digested>,
  pub mode: TexMode
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
    self.unlist().into_iter().map(|digested| document.absorb(digested, state));
  }
}
