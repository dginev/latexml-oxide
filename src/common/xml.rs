pub struct XPath {
  foo: bool
}
impl Default for XPath {
  fn default() -> Self {
    XPath {
      foo: true
    }
  }
}

// pub type XPathClosure = Arc<Box<Fn(&mut Gullet, Vec<Token>, &mut State) -> bool>>;
impl XPath {
  // Any subroutine??? maybe not yet...
  // pub register_function(name : String, f : |Font,Font| -> bool
  pub fn register_ns(&mut self, codeprefix : String, namespace: String) {

  }
}