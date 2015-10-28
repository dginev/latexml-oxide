extern crate libxml;

use common::model::Model;
use state::State;
use core::Digested;
use libxml::tree::Document as XmlDoc;
use libxml::tree::Node;

pub struct Document<'doc> {
  pub model : &'doc Model,
  pub document : XmlDoc
}

impl<'doc> Document<'doc> {
  pub fn new(model : &'doc mut Model) -> Self {
    Document {
      model : model,
      document : XmlDoc::new().unwrap()
    }
  }
  //**********************************************************************
  // This should be called before returning the final XML::LibXML::Document to the
  // outside world.  It resolves the fonts for each node relative to it's ancestors.
  // It removes the `helper' attributes that store fonts, source box, etc.
  pub fn finalize(&mut self, state : &mut State) {
    self.prune_XMDuals();
    let doc = self.document;
    let root = doc.get_root_element().unwrap();
    // local $LaTeXML::FONT = LaTeXML::Common::Font->textDefault;
    self.finalize_rec(root);
    match state.lookup_value("RDFa_prefixes") {
      None => {},
      Some(prefixes) =>Document::set_RDFa_prefixes(doc, *prefixes)
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
  fn set_RDFa_prefixes(doc : XmlDoc, prefixes : Option<String>) {

  }

  fn prune_XMDuals(&self) {}

  fn finalize_rec(&self,element : Node) {}


}