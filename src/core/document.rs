extern crate libxml;

// use common::model::Model;
use state::State;
use core::Digested;
use libxml::tree::Document as XmlDoc;
use libxml::tree::Node;

pub struct Document {
  // pub model : &'doc Model,
  pub document : XmlDoc,
  pub root : Node
}

impl Document {
  pub fn new() -> Self {
    let mut doc_scaffold = XmlDoc::new().unwrap();
    let mut latexml_node = Node::new("latexml", None, &doc_scaffold).unwrap();
    doc_scaffold.set_root_element(&mut latexml_node);

    Document {
      document : doc_scaffold,
      root : latexml_node
    }
  }
  //**********************************************************************
  // This should be called before returning the final XML::LibXML::Document to the
  // outside world.  It resolves the fonts for each node relative to it's ancestors.
  // It removes the `helper' attributes that store fonts, source box, etc.
  pub fn finalize<'finalize>(&'finalize mut self, state : &'finalize mut State) {
    self.prune_XMDuals();
    let root = self.document.get_root_element().unwrap();
    // local $LaTeXML::FONT = LaTeXML::Common::Font->textDefault;
    self.finalize_rec(root);
    match state.lookup_value("RDFa_prefixes") {
      None => {},
      Some(prefixes) => self.set_RDFa_prefixes(*prefixes)
    };
  }


  pub fn absorb(&mut self, digested : Digested) -> String {
    // TODO: Just a stub for now
    // println_stderr!("Will absorb: {:?}", digested.boxes);
    for tbox in digested.boxes.iter() {
      let mut box_node = self.root.add_child(None, "box").unwrap();
      box_node.set_content(&tbox.text);
    }
    String::new()
  }

  pub fn insert_pi(&mut self, which : &str, paths : Vec<String>) {
    // TODO
  }
  pub fn to_string(&self) -> String {
    self.document.to_string()
  }

  // Internals
  fn set_RDFa_prefixes<'prefixes>(&'prefixes mut self, prefixes : Option<String>) {

  }

  fn prune_XMDuals(&self) {}

  fn finalize_rec(&self,element : Node) {}


}