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
    let mut latexml_node = Node::new("document", None, &doc_scaffold).unwrap();
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


  ///%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  /// Document construction at the Current Insertion Point.
  ///%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  ///
  ///**********************************************************************
  /// absorb the given $box into the DOM (called from constructors).
  /// This will return a list of whatever nodes were created.
  /// Note that this may include nodes that are children of other nodes in the list
  /// or nodes that are no longer in the document.
  /// Also, note that when a text nodes is appended to, the complete text node is in the list,
  /// not just the portion that was added.
  /// [Note that recording the nodes being constructed isn't all that costly,
  /// but filtering them for parent/child relations IS, particularly since it usually isn't needed]
  ///
  /// A $box that is a Box, or List, or Whatsit, is responsible for carrying out
  /// its own insertion, but it should ultimately call methods of Document
  /// that will record the nodes that were created.
  /// $box can also be a plain string which will be inserted according to whatever
  /// font, mode, etc, are in %props.
  pub fn absorb(&mut self, object : Box<Digested>) -> String {
    for tbox in object.unlist().iter() {
      let mut box_node = self.root.add_child(None, "box").unwrap();
      box_node.set_content(&tbox.text);
    }
    String::new()
  }

  /// Insert a ProcessingInstruction of the form <?op attr=value ...?>
  /// Does NOT move the current insertion point to the PI,
  /// but may move up past a text node.
  pub fn insert_pi(&mut self, op: &str, kind: &str, content: &str, options : Option<String>) {
    // We'll just put these on the document itself.
    // Put these in an attractive order, main "operator" first
    // my @keys = ((map { ($attrib{$_} ? ($_) : ()) } qw(class package options)),
    //   (grep { $_ !~ /^(?:class|package|options)$/ } sort keys %attrib));
    // my $data = join(' ', map { $_ . "=\"" . ToString($attrib{$_}) . "\"" } @keys);
    let options_string = match options {
      Some(payload) => " options=".to_string() + &payload,
      None => String::new()
    };
    let mut data = kind.to_string() + "=" + content + &options_string;
    let pi = self.document.create_processing_instruction(op, &data).unwrap();

    // self.close_text_internal();  // Close any open text node
    // if ($$self{node}->nodeType == XML_DOCUMENT_NODE) {
    //   push(@{ $$self{pending} }, $pi); }
    // else {
    println_stderr!("Trying to insert PI: {:?}", self.document.node_to_string(&pi));
    println_stderr!("Into doc: {:?}", self.document.to_string());

    self.root.add_prev_sibling(pi);

    return;
  }
  pub fn to_string(&self) -> String {
    self.document.to_string()
  }

  pub fn set_node(&self, node: Node) {

  }

  // Internals
  fn set_RDFa_prefixes<'prefixes>(&'prefixes mut self, prefixes : Option<String>) {

  }

  fn prune_XMDuals(&self) {}

  fn finalize_rec(&self,element : Node) {}


}