extern crate libxml;

use common::model::Model;
use state::State;
use core::Digested;
use libxml::tree::Document as XmlDoc;
use libxml::tree::Node;

pub struct Document {
  // pub model : &'doc Model,
  pub document : XmlDoc
}

impl Document {
  pub fn new() -> Self {
    Document {
      document : XmlDoc::new().unwrap()
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


  pub fn absorb(&self, digested : Digested) -> String {
    "absorbed".to_string()
  }

  pub fn insert_pi(&self, which : &str, paths : Vec<String>) {
    // TODO
  }
  pub fn to_string(&self) -> String {
    "fake document to_string".to_string()
  }

  // Internals
  fn set_RDFa_prefixes<'prefixes>(&'prefixes mut self, prefixes : Option<String>) {

  }

  fn prune_XMDuals(&self) {}

  fn finalize_rec(&self,element : Node) {}


}