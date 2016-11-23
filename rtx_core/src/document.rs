extern crate regex;
extern crate libxml;

use std::collections::VecDeque;
// use common::model::Model;
use state::{ObjectStore, State};
use {Digested, BoxOps};
use libxml::tree::Document as XmlDoc;
use libxml::tree::{Node, NodeType};
use regex::Regex;

pub struct Document {
  // pub model : &'doc Model,
  pub document: XmlDoc,
  pub node: Node,
  pub debug: bool,
}

impl Document {
  pub fn new() -> Self {
    let mut doc_scaffold = XmlDoc::new().unwrap();
    let mut latexml_node = Node::new("document", None, &doc_scaffold).unwrap();
    doc_scaffold.set_root_element(&mut latexml_node);

    Document {
      document: doc_scaffold,
      node: latexml_node,
      debug: false,
    }
  }
  // **********************************************************************
  // This should be called before returning the final XML::LibXML::Document to the
  // outside world.  It resolves the fonts for each node relative to it's ancestors.
  // It removes the `helper' attributes that store fonts, source box, etc.
  pub fn finalize<'finalize>(&'finalize mut self, state: &'finalize mut State) {
    self.prune_XMDuals();
    let root = self.document.get_root_element().unwrap();
    // local $LaTeXML::FONT = LaTeXML::Common::Font->textDefault;
    self.finalize_rec(root);
    match state.lookup_value("RDFa_prefixes") {
      Some(&ObjectStore::StringStore(ref prefixes)) => self.set_RDFa_prefixes(Some(prefixes.clone())),
      _ => {}
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
  pub fn absorb(&mut self, object: Digested, state: &mut State) -> Vec<Node> {
    let mut results = Vec::new();
    let mut boxes = VecDeque::new();
    boxes.push_front(object);

    while !boxes.is_empty() {
      match boxes.pop_front().unwrap() {
        // Simply unwind Lists to avoid unneccessary recursion; This occurs quite frequently!
        Digested::ListObj(list) => {
          for tbox in list.unlist().into_iter().rev() {
            boxes.push_front(tbox);
          }
        }
        // A Proper Box or Whatsit? It will handle it.
        Digested::BoxObj(mut tbox) => tbox.be_absorbed(self, state),
        Digested::WhatsitObj(mut whatsit) => whatsit.be_absorbed(self, state),
      };
      //   // [ATTEMPT to] only record if we're running in NON-VOID context.
      //   // [but wantarray seems defined MUCH more than I would have expected!?]
      //   // if ($LaTeXML::RECORDING_CONSTRUCTION || defined wantarray) {
      //   //   my @n = ();
      //   //   { local $LaTeXML::RECORDING_CONSTRUCTION = 1;
      //   //     local @LaTeXML::CONSTRUCTED_NODES = ();
      //       $box->beAbsorbed($self);
      //       @n = @LaTeXML::CONSTRUCTED_NODES; }    // These were created just now
      //     map { $self->recordConstructedNode($_) } @n;    // record these for OUTER caller!
      //     push(@results, @n); }                           // but return only the most recent set.
      //   else {
      //     push(@results, $box->beAbsorbed($self)); } }
      // // Else, plain string in text mode.
      // elsif (!$props{isMath}) {
      //   push(@results, $self->openText($box, $props{font} || ($LaTeXML::BOX && $LaTeXML::BOX->getFont))); }
      // // Or plain string in math mode.
      // // Note text nodes can ONLY appear in <XMTok> or <text>!!!
      // // Have we already opened an XMTok? Then insert into it.
      // elsif ($$self{model}->getNodeQName($$self{node}) eq $MATH_TOKEN_NAME) {
      //   push(@results, $self->openMathText_internal($box)); }
      // // Else create the XMTok now.
      // else {
      //   // Odd case: constructors that work in math & text can insert raw strings in Math mode.
      //   push(@results, $self->insertMathToken($box, font => $props{font})); } }

      //   let mut box_node = self.node.add_child(None, "box").unwrap();
      //   box_node.set_content(&tbox.text);
    }
    results
  }

  /// Insert a ProcessingInstruction of the form <?op attr=value ...?>
  /// Does NOT move the current insertion point to the PI,
  /// but may move up past a text node.
  //  Rust note: attrib would have been best as Vec<(String,String)> but currently quote!() doesn't work out of the box on them
  pub fn insert_pi(&mut self, op: &str, attrib : Vec<&str>) {
    let pi_node = self.document.create_processing_instruction(op,"").unwrap();
    let mut key = "";
    for a in attrib {
      if key.is_empty() {
        key = a;
      } else {
        pi_node.set_attribute(&key, &a);
        key = "";
      }
    }
    // self.close_text_internal();  // Close any open text node
    // if ($$self{node}->nodeType == NodeType::DocumentNode) {
    //   push(@{ $$self{pending} }, $pi); }
    // else {
    println_stderr!("Trying to insert PI: {:?}",
                    self.document.node_to_string(&pi_node));
    println_stderr!("Into doc: {:?}", self.document.to_string());

    self.node.add_prev_sibling(pi_node);

    return;
  }
  pub fn to_string(&self) -> String {
    self.document.to_string()
  }

  pub fn set_node(&self, node: Node) {}

  // Internals
  fn set_RDFa_prefixes<'prefixes>(&'prefixes mut self, prefixes: Option<String>) {}

  fn prune_XMDuals(&self) {}

  fn finalize_rec(&self, element: Node) {}

  pub fn insert_math_token(&self, text: &str) {}

  ///**********************************************************************
  /// Middle level, mostly public, API.
  /// Handlers for various construction operations.
  /// General naming: 'open' opens a node at current pos and sets it to current,
  /// 'close' closes current node(s), inserts opens & closes, ie. w/o moving current

  /// Tricky: Insert some text in a particular font.
  /// We need to find the current effective -- being the closest  _declared_ font,
  /// (ie. it will appear in the elements attributes).  We may also want
  /// to open/close some elements in such a way as to minimize the font switchiness.
  /// I guess we should only open/close "text" elements, though.
  /// [Actually, we'd like the user to _declare_ what element to use....
  ///  I don't like having "text" built in here!
  ///  AND, we've assumed that "font" names the relevant attribute!!!]

  pub fn open_text(&mut self, text: &str) -> Option<&Node> {
    // TODO: font arg
    lazy_static! {
      static ref leading_space_regex : Regex = Regex::new(r"^\s+$").unwrap();
    }
    let node_type = self.node.get_type();
    {
      // Ignore initial whitespace
      if leading_space_regex.is_match(text) && (node_type == Some(NodeType::DocumentNode) || (node_type == Some(NodeType::ElementNode) && self.can_contain(&self.node, "#PCDATA"))) {
        return None;
      }
    }
    // if font.get_family() == "nullfont" {
    //   return;
    // }
    // print STDERR "Insert text \"$text\" /" . Stringify($font) . " at " . Stringify($node) . "\n"
    //   if $LaTeXML::Core::Document::DEBUG;

    if node_type != Some(NodeType::DocumentNode) // If not at document begin
    && !((node_type == Some(NodeType::TextNode)) //&&    // And not appending text in same font.
      // ($font->distance($self->getNodeFont($node->parentNode)) == 0))
    ) {
      // then we'll need to do some open/close to get fonts matched.
      // node =
      self.close_text_internal();    // Close text node, if any.
      // let mut bestdiff = 99999;
      // let mut closeto = node;
      // let mut n = node;
      // while n.get_type() != NodeType::DocumentNode {
      // let d = $font->distance($self->getNodeFont($n));

      // if ($d < $bestdiff) {
      //   $bestdiff = $d;
      //   $closeto  = $n;
      //   last if ($d == 0); }
      // last if ($$self{model}->getNodeQName($n) ne $FONT_ELEMENT_NAME) || $n->getAttribute('_noautoclose');
      // $n = $n->parentNode; }

      // Move to best starting point for this text.
      // if closeto != node {
      //   self.close_to_node(closeto);
      // }
      // self.open_element($FONT_ELEMENT_NAME, font => $font, _fontswitch => 1) if $bestdiff > 0; // Open if needed.
    }
    // Finally, insert the darned text.
    self.open_text_internal(text);
    self.record_constructed_node(&self.node);
    return Some(&self.node);
  }


  pub fn can_contain(&self, node: &Node, spec: &str) -> bool {
    // TODO: Mock only
    true
  }
  pub fn close_text_internal(&mut self) -> &mut Node {
    // TODO: Mock only
    &mut self.node
  }
  pub fn close_to_node(&self, node: &Node) {} // TODO: Mock only

  pub fn open_text_internal(&mut self, text: &str) {
    lazy_static! {
      static ref has_nonspace : Regex = Regex::new(r"\S").unwrap();
    }
    if self.node.get_type() == Some(NodeType::TextNode) {
      // current node already is a text node.
      if self.debug {
        println_stderr!("Appending text \"{:?}\" to {:?}",
                        text,
                        self.document.node_to_string(&self.node));
      }
      self.node.append_text(text);
    } else if has_nonspace.is_match(text) || self.can_contain(&self.node, "//PCDATA") {
      // or text allowed here
      let mut point = self.find_insertion_point("//PCDATA");
      let mut node = Node::new_text(text, &self.document).unwrap();
      if self.debug {
        println_stderr!("Inserting text node for \"{:?}\" into {:?}",
                        text,
                        self.document.node_to_string(&point));
      }
      let added_node = point.add_child(node).unwrap();
      self.set_node(added_node);
    }
  }

  pub fn record_constructed_node(&self, node: &Node) {} // TODO: Mock only
  pub fn find_insertion_point(&self, target: &str) -> Node {
    // TODO: Mock only
    self.node.clone()
  }
}
